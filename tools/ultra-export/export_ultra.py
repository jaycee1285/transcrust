#!/usr/bin/env python3
"""Export moondream/parakeet-ultra to the onnx-asr layout transcrust, mopvox and
dayshade load, then quantize the encoder to int4.

    encoder-model.onnx(.data)   fp32 encoder, mel [B,128,T] -> frames [B,1024,T/8]
    encoder-model.int4.onnx     the same, MatMul -> MatMulNBits 4-bit, block 128
    decoder_joint-model.onnx    projector + LSTM prediction net + TDT joint, one step
    decoder_joint-model.int8.onnx  the same, dynamic int8 (the name the phone apps load)
    install/parakeet-ultra-0.6b-int4/  the four files the apps load, with nemo128.onnx
    preprocessor.onnx           Photon's mel recipe (reference only; the apps use nemo128.onnx)
    vad-model.onnx              the checkpoint's speech head (not used by the apps)
    vocab.txt, config.json

Adapted from eschmidbauer/parakeet-redux-onnx's export_onnx.py (CC BY 4.0),
which is ternary-only: ultra is an fp16 checkpoint with no ternary.json, so it
goes through kestrel's own fp loader instead, and the int4 step is ORT's
generic MatMulNBitsQuantizer (lossy, unlike redux's exact repack).
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import types
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F
from torch import nn

SOURCE_MODEL = "moondream/parakeet-ultra"
OPSET = 18
BLOCK = 128


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def load_fp32():
    import kestrel.models.parakeet_tdt.model as parakeet_model
    from kestrel.models.parakeet_tdt.weights import load_parakeet_tdt
    from kestrel_kernels.conformer_ops import torch_conformer_runtime
    from kestrel_kernels.tdt_ops import torch_tdt_runtime

    # Photon picks fused kernels per device; tracing needs the plain torch references.
    reference = types.SimpleNamespace(conformer=torch_conformer_runtime("conv1d"), tdt=torch_tdt_runtime())
    parakeet_model.get_runtime = lambda device=None: reference
    loaded = load_parakeet_tdt(SOURCE_MODEL, dtype=torch.float32)
    return loaded.model.eval(), loaded.tokenizer


class EncoderExport(nn.Module):
    def __init__(self, model):
        super().__init__()
        self.encoder = model.encoder

    def forward(self, audio_signal, length):
        features = audio_signal.transpose(1, 2)
        mask = torch.arange(features.shape[1], device=features.device)[None] < length[:, None]
        encoded, valid = self.encoder(features, mask)
        return encoded.transpose(1, 2), valid.sum(-1)


class DecoderJointExport(nn.Module):
    def __init__(self, model):
        super().__init__()
        self.encoder_projector = model.encoder_projector
        self.decoder = model.decoder
        self.joint = model.joint

    def forward(self, encoder_outputs, targets, target_length, input_states_1, input_states_2):
        frame = self.encoder_projector(encoder_outputs.transpose(1, 2))
        pred, (hidden, cell) = self.decoder(targets.to(torch.int64), (input_states_1, input_states_2))
        logits = self.joint(frame, pred)
        return logits.unsqueeze(2), target_length, hidden, cell


class VadExport(nn.Module):
    def __init__(self, model):
        super().__init__()
        self.subsampling = model.encoder.subsampling
        self.vad_head = model.vad_head

    def forward(self, audio_signal, length):
        features = audio_signal.transpose(1, 2)
        mask = torch.arange(features.shape[1], device=features.device)[None] < length[:, None]
        hidden, valid = self.subsampling(features, mask)
        return torch.sigmoid(self.vad_head(hidden)), valid.sum(-1)


class Preprocessor(nn.Module):
    """Photon's features, verbatim from the redux exporter."""

    def __init__(self):
        super().__init__()
        from kestrel.models.asr.features import mel_filters

        n_fft, win, hop = 512, 400, 160
        window = F.pad(torch.hann_window(win, periodic=False), ((n_fft - win) // 2, (n_fft - win) // 2))
        k = torch.arange(n_fft // 2 + 1, dtype=torch.float64)[:, None]
        n = torch.arange(n_fft, dtype=torch.float64)[None]
        angle = 2 * torch.pi * k * n / n_fft
        basis = torch.cat([torch.cos(angle), -torch.sin(angle)]) * window.double()[None]
        self.register_buffer("basis", basis.float()[:, None, :])
        self.register_buffer("mel", mel_filters(n_fft, 128, 16_000).clone())
        self.hop = hop
        self.pad = n_fft // 2

    def forward(self, waveforms, waveforms_lens):
        x = waveforms
        emphasised = torch.cat((x[:, :1], x[:, 1:] - 0.97 * x[:, :-1]), dim=1)
        inside = torch.arange(x.shape[1], device=x.device)[None] < waveforms_lens[:, None]
        emphasised = torch.where(inside, emphasised, torch.zeros((), dtype=x.dtype))
        spectrum = F.conv1d(F.pad(emphasised, (self.pad, self.pad))[:, None], self.basis, stride=self.hop)
        bins = self.basis.shape[0] // 2
        power = spectrum[:, :bins] ** 2 + spectrum[:, bins:] ** 2
        logmel = torch.log(self.mel @ power + 2.0 ** -24)
        valid = waveforms_lens // self.hop
        frame_mask = (torch.arange(logmel.shape[2], device=x.device)[None] < valid[:, None])[:, None].to(logmel.dtype)
        count = valid.to(logmel.dtype)[:, None, None]
        mean = (logmel * frame_mask).sum(2, keepdim=True) / count
        variance = (((logmel - mean) * frame_mask) ** 2).sum(2, keepdim=True) / (count - 1)
        features = ((logmel - mean) / (torch.sqrt(variance) + 1e-5)) * frame_mask
        return features, valid


def export(module, args, names, outputs, dynamic, path: Path, external: bool = False):
    from torch.export import Dim

    dims: dict = {}

    def dim(name):
        if name not in dims:
            dims[name] = Dim(name, min=16) if name in ("time", "samples") else Dim(name)
        return dims[name]

    shapes = {name: {axis: dim(label) for axis, label in spec.items()} for name, spec in dynamic.items()}
    program = torch.onnx.export(module, args, dynamo=True, opset_version=OPSET, input_names=names,
                                output_names=outputs, dynamic_shapes=shapes, external_data=external)
    program.save(str(path), external_data=external)


def quantize_int4(src: Path, dst: Path) -> None:
    import collections

    import onnx
    from onnxruntime.quantization.matmul_nbits_quantizer import MatMulNBitsQuantizer

    model = onnx.load(str(src))
    quant = MatMulNBitsQuantizer(model, bits=4, block_size=BLOCK, is_symmetric=False)
    quant.process()
    quant.model.save_model_to_file(str(dst), use_external_data_format=False)
    ops = collections.Counter(n.op_type for n in onnx.load(str(dst), load_external_data=False).graph.node)
    log(f"  int4 encoder: {dst.stat().st_size / 1e6:.0f} MB, MatMulNBits {ops.get('MatMulNBits', 0)}, "
        f"MatMul left {ops.get('MatMul', 0)}, Conv {ops.get('Conv', 0)}")


def quantize_joint_int8(src: Path, dst: Path) -> None:
    """mopvox and dayshade load `decoder_joint-model.int8.onnx` by name, so the
    joint has to be int8 like v3's. The dynamo export carries value_info that
    ORT's shape re-inference contradicts (1280 vs 2560 on the unrolled LSTM),
    so it is stripped first."""
    import onnx
    from onnxruntime.quantization import QuantType, quantize_dynamic

    model = onnx.load(str(src))
    del model.graph.value_info[:]
    stripped = src.with_suffix(".noshape.onnx")
    onnx.save(model, str(stripped))
    quantize_dynamic(str(stripped), str(dst), weight_type=QuantType.QInt8)
    stripped.unlink()
    log(f"  int8 joint: {dst.stat().st_size / 1e6:.0f} MB")


def assemble(out: Path, nemo128: Path) -> Path:
    """The four files the apps load, under a name transcrust discovers."""
    import shutil

    target = out / "install" / "parakeet-ultra-0.6b-int4"
    target.mkdir(parents=True, exist_ok=True)
    for name in ("encoder-model.int4.onnx", "decoder_joint-model.int8.onnx", "vocab.txt"):
        shutil.copyfile(out / name, target / name)
    shutil.copyfile(nemo128, target / "nemo128.onnx")
    return target


def write_vocab(tokenizer_json: Path, out: Path, vocab_size: int, blank_id: int) -> None:
    tokenizer = json.loads(tokenizer_json.read_text(encoding="utf-8"))
    vocab = tokenizer["model"]["vocab"]
    pieces = {i: piece for piece, i in vocab.items()} if isinstance(vocab, dict) else {
        i: piece for i, (piece, *_rest) in enumerate(vocab)}
    for added in tokenizer.get("added_tokens", ()):
        pieces[added["id"]] = added["content"]
    lines = []
    for i in range(vocab_size):
        piece = "<blk>" if i == blank_id else pieces.get(i, f"<unused{i}>")
        if " " in piece or "\n" in piece:
            raise RuntimeError(f"token {i!r} contains whitespace")
        lines.append(f"{piece} {i}")
    (out / "vocab.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--out", default="build", help="output directory")
    parser.add_argument("--skip-quantize", action="store_true")
    parser.add_argument("--nemo128", default="~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int4/nemo128.onnx",
                        help="the istupakov feature extractor the apps share")
    args = parser.parse_args()
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)

    started = time.perf_counter()
    model, _tokenizer = load_fp32()
    config = model.config
    log(f"loaded {SOURCE_MODEL}: {sum(p.numel() for p in model.parameters()) / 1e6:.0f}M params, "
        f"vad head {'yes' if model.vad_head is not None else 'no'}, {time.perf_counter() - started:.1f}s")

    sample = (torch.randn(2, config.encoder.num_mel_bins, 1100), torch.tensor([1100, 550]))
    with torch.inference_mode():
        if not (out / "encoder-model.onnx").exists():
            started = time.perf_counter()
            export(EncoderExport(model), sample, ["audio_signal", "length"], ["outputs", "encoded_lengths"],
                   {"audio_signal": {0: "batch", 2: "time"}, "length": {0: "batch"}},
                   out / "encoder-model.onnx", external=True)
            log(f"encoder exported (fp32) in {time.perf_counter() - started:.1f}s")
        batch = 2
        export(DecoderJointExport(model),
               (torch.randn(batch, config.encoder.hidden_size, 1),
                torch.tensor([[config.blank_token_id], [17]], dtype=torch.int32), torch.ones(batch, dtype=torch.int32),
                torch.zeros(config.num_decoder_layers, batch, config.decoder_hidden_size),
                torch.zeros(config.num_decoder_layers, batch, config.decoder_hidden_size)),
               ["encoder_outputs", "targets", "target_length", "input_states_1", "input_states_2"],
               ["outputs", "prednet_lengths", "output_states_1", "output_states_2"],
               {"encoder_outputs": {0: "batch"}, "targets": {0: "batch"}, "target_length": {0: "batch"},
                "input_states_1": {1: "batch"}, "input_states_2": {1: "batch"}},
               out / "decoder_joint-model.onnx")
        export(Preprocessor().eval(), (torch.randn(2, 16_000), torch.tensor([16_000, 9_000])),
               ["waveforms", "waveforms_lens"], ["features", "features_lens"],
               {"waveforms": {0: "batch", 1: "samples"}, "waveforms_lens": {0: "batch"}}, out / "preprocessor.onnx")
        if model.vad_head is not None:
            export(VadExport(model), sample, ["audio_signal", "length"], ["probabilities", "encoded_lengths"],
                   {"audio_signal": {0: "batch", 2: "time"}, "length": {0: "batch"}}, out / "vad-model.onnx")
        log("decoder_joint, preprocessor, vad exported")

    from huggingface_hub import hf_hub_download

    write_vocab(Path(hf_hub_download(SOURCE_MODEL, "tokenizer.json")), out, config.vocab_size, config.blank_token_id)
    (out / "config.json").write_text(json.dumps({
        "model_type": "nemo-conformer-tdt",
        "features_size": config.encoder.num_mel_bins,
        "subsampling_factor": config.encoder.subsampling_factor,
        "max_tokens_per_step": config.max_symbols_per_step,
        "vocab_size": config.vocab_size,
        "blank_token_id": config.blank_token_id,
        "durations": list(config.durations),
        "sample_rate": 16_000,
        "source": SOURCE_MODEL,
        "encoder_weights": f"MatMulNBits 4-bit asymmetric, block {BLOCK} (encoder-model.int4.onnx)",
    }, indent=2) + "\n")

    if not args.skip_quantize:
        started = time.perf_counter()
        quantize_int4(out / "encoder-model.onnx", out / "encoder-model.int4.onnx")
        quantize_joint_int8(out / "decoder_joint-model.onnx", out / "decoder_joint-model.int8.onnx")
        log(f"quantized in {time.perf_counter() - started:.1f}s")
        # Photon's own mel (preprocessor.onnx) matches nemo128 except the first
        # two frames and gave identical clean transcripts, so the apps' shared
        # nemo128.onnx is used as-is.
        nemo128 = Path(args.nemo128).expanduser()
        if nemo128.is_file():
            log(f"assembled {assemble(out, nemo128)}")
        else:
            log(f"no nemo128.onnx at {nemo128}; skipping the install directory")

    for path in sorted(out.iterdir()):
        log(f"  {path.stat().st_size / 1e6:8.1f} MB  {path.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
