#!/usr/bin/env python
"""Run the upstream PyTorch pipeline, for validating the ONNX port against."""

import argparse
import time
from pathlib import Path

import numpy as np
import torch

from common import checkpoint_dir


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--audio", type=Path, required=True)
    parser.add_argument("--seconds", type=float, default=None)
    parser.add_argument("--max-new-tokens", type=int, default=512)
    parser.add_argument("--dtype", default="bfloat16")
    args = parser.parse_args()

    from vibevoice.modular.modeling_vibevoice_asr import VibeVoiceASRForConditionalGeneration
    from vibevoice.processor.vibevoice_asr_processor import VibeVoiceASRProcessor
    import librosa

    ckpt = str(checkpoint_dir())
    processor = VibeVoiceASRProcessor.from_pretrained(ckpt)
    model = VibeVoiceASRForConditionalGeneration.from_pretrained(
        ckpt,
        torch_dtype=getattr(torch, args.dtype),
        attn_implementation="sdpa",
        low_cpu_mem_usage=True,
    ).eval()

    audio, _ = librosa.load(str(args.audio), sr=24000, mono=True)
    if args.seconds:
        audio = audio[: int(args.seconds * 24000)]
    print(f"audio: {len(audio) / 24000:.2f}s")

    inputs = processor(
        audio=[audio.astype(np.float32)], sampling_rate=24000, return_tensors="pt", padding=True
    )
    started = time.perf_counter()
    with torch.no_grad():
        out = model.generate(
            **inputs,
            max_new_tokens=args.max_new_tokens,
            do_sample=False,
            pad_token_id=processor.pad_id,
            eos_token_id=processor.tokenizer.eos_token_id,
        )
    elapsed = time.perf_counter() - started
    generated = out[0, inputs["input_ids"].shape[1]:]
    text = processor.decode(generated, skip_special_tokens=True)
    print(f"raw       : {text!r}")
    print(f"timing    : {elapsed:.1f}s for {len(generated)} tokens")


if __name__ == "__main__":
    main()
