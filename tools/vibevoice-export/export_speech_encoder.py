#!/usr/bin/env python
"""Export the VibeVoice ASR audio front end to a single ONNX graph.

    audio [1, 1, 83200 @ 24 kHz]  ->  speech_features [1, 26, 1536]

The time axis is **fixed, not dynamic**. Two reasons, and the first alone
settles it: the SConv1d stack reads `x.shape[-1]` to size its stride-alignment
padding, so tracing freezes the trace-time length into the graph no matter what
`dynamic_axes` claims — a longer input silently comes back with the traced
number of frames. The second is that it costs nothing: `streaming_generate`
encodes one fixed 26-frame window at a time (22 chunk frames + 4 lookahead) and
zero-pads the last one up to it, so 83200 is the only length the runtime ever
needs. A static graph also makes a wrong length a loud ORT shape error instead
of a quiet wrong answer.

Deviation from the PyTorch reference, deliberate: `encode_speech` samples the
acoustic latent (`dist_type="gaussian"`), which injects a per-utterance
`randn * fix_std/0.8` scale plus per-element noise. That is a training-time
regulariser; a dictation app wants the same audio to produce the same text, so
this graph emits the distribution mean. `validate_onnx.py` measures the gap.
"""

import argparse
from pathlib import Path

import torch
import torch.nn as nn

from common import SPEECH_PREFIXES, checkpoint_dir, load_config, load_tensors, report, strip_prefix

COMPRESS_RATIO = 3200
TARGET_SAMPLE_RATE = 24000
# preprocessor_config.json: chunk_frames 22 + lookahead_frames 4.
WINDOW_FRAMES = 26
WINDOW_SAMPLES = WINDOW_FRAMES * COMPRESS_RATIO


class SpeechFrontEnd(nn.Module):
    def __init__(self, acoustic_encoder, semantic_encoder, acoustic_connector, semantic_connector):
        super().__init__()
        self.acoustic_encoder = acoustic_encoder
        self.semantic_encoder = semantic_encoder
        self.acoustic_connector = acoustic_connector
        self.semantic_connector = semantic_connector

    def forward(self, audio: torch.Tensor) -> torch.Tensor:
        acoustic = self.acoustic_encoder(audio).permute(0, 2, 1)
        semantic = self.semantic_encoder(audio).permute(0, 2, 1)
        return self.acoustic_connector(acoustic) + self.semantic_connector(semantic)


def build(ckpt: Path) -> SpeechFrontEnd:
    from vibevoice.modular.configuration_vibevoice import (
        VibeVoiceAcousticTokenizerConfig,
        VibeVoiceSemanticTokenizerConfig,
    )
    from vibevoice.modular.modular_vibevoice_tokenizer import (
        VibeVoiceAcousticTokenizerModel,
        VibeVoiceSemanticTokenizerModel,
    )
    from vibevoice.modular.modeling_vibevoice import SpeechConnector

    config = load_config(ckpt)
    acoustic_config = VibeVoiceAcousticTokenizerConfig(**config["acoustic_tokenizer_config"])
    semantic_config = VibeVoiceSemanticTokenizerConfig(**config["semantic_tokenizer_config"])

    # These two encoders are ~100 MB together, so they are built for real
    # rather than on `meta`: the tokenizer stack calls `.item()` during
    # construction, which meta tensors do not support.
    acoustic = VibeVoiceAcousticTokenizerModel(acoustic_config)
    semantic = VibeVoiceSemanticTokenizerModel(semantic_config)
    acoustic_encoder = acoustic.encoder
    semantic_encoder = semantic.encoder
    del acoustic, semantic

    hidden = config["decoder_config"]["hidden_size"]
    acoustic_connector = SpeechConnector(config["acoustic_vae_dim"], hidden)
    semantic_connector = SpeechConnector(config["semantic_vae_dim"], hidden)

    state = load_tensors(ckpt, lambda name: name.startswith(SPEECH_PREFIXES))
    acoustic_encoder.load_state_dict(
        strip_prefix(state, "model.acoustic_tokenizer.encoder."), assign=True
    )
    semantic_encoder.load_state_dict(
        strip_prefix(state, "model.semantic_tokenizer.encoder."), assign=True
    )
    acoustic_connector.load_state_dict(strip_prefix(state, "model.acoustic_connector."), assign=True)
    semantic_connector.load_state_dict(strip_prefix(state, "model.semantic_connector."), assign=True)

    model = SpeechFrontEnd(
        acoustic_encoder, semantic_encoder, acoustic_connector, semantic_connector
    ).eval()
    for parameter in model.parameters():
        parameter.requires_grad_(False)
    return model


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--opset", type=int, default=17)
    args = parser.parse_args()

    ckpt = checkpoint_dir()
    print(f"checkpoint: {ckpt}")
    model = build(ckpt)

    example = torch.zeros(1, 1, WINDOW_SAMPLES, dtype=torch.float32)
    with torch.no_grad():
        probe = model(example)
    assert probe.shape[1] == WINDOW_FRAMES, f"expected {WINDOW_FRAMES} frames, got {probe.shape[1]}"
    print(f"traced shape check ok: {tuple(probe.shape)}")

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with torch.no_grad():
        torch.onnx.export(
            model,
            (example,),
            str(args.out),
            input_names=["audio"],
            output_names=["speech_features"],
            opset_version=args.opset,
            do_constant_folding=True,
            dynamo=False,
        )
    report(args.out)


if __name__ == "__main__":
    main()
