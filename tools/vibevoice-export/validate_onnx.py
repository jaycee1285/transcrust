#!/usr/bin/env python
"""Compare the int4 ONNX pipeline against the PyTorch reference on real audio.

Runs `streaming_generate` (fp32 CPU) and `run_onnx.VibeVoiceOnnx` over the same
clip and prints both transcripts plus a word-level agreement rate. This is the
check that says whether the export and the 4-bit quantisation cost anything
that matters, and it is the reason `run_onnx.py` is written as a spec rather
than a demo.
"""

import argparse
import difflib
import time
from pathlib import Path

import torch

from common import checkpoint_dir
from run_onnx import (
    CHUNK_SAMPLES,
    LOOKAHEAD_SAMPLES,
    SAMPLE_RATE,
    VibeVoiceOnnx,
    flatten_transcript,
    load_audio,
)


def torch_reference(audio, max_new_tokens: int) -> tuple[str, float]:
    from vibevoice.modular.modeling_vibevoice_asr import VibeVoiceASRForConditionalGeneration
    from vibevoice.processor.vibevoice_asr_processor import VibeVoiceASRProcessor

    ckpt = str(checkpoint_dir())
    processor = VibeVoiceASRProcessor.from_pretrained(ckpt)
    model = VibeVoiceASRForConditionalGeneration.from_pretrained(
        ckpt, torch_dtype=torch.float32, attn_implementation="sdpa", low_cpu_mem_usage=True
    ).eval()

    started = time.perf_counter()
    chunks = []
    for _, _, text in model.streaming_generate(
        audio_tensor=torch.from_numpy(audio),
        tokenizer=processor.tokenizer,
        chunk_duration=CHUNK_SAMPLES / SAMPLE_RATE,
        text_audio_delay=LOOKAHEAD_SAMPLES / SAMPLE_RATE,
        sample_rate=SAMPLE_RATE,
        max_new_tokens_per_chunk=max_new_tokens,
        temperature=0.0,
    ):
        chunks.append(text)
    return "".join(chunks), time.perf_counter() - started


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-dir", type=Path, required=True)
    parser.add_argument("--audio", type=Path, required=True)
    parser.add_argument("--seconds", type=float, default=None)
    parser.add_argument("--max-new-tokens", type=int, default=256)
    args = parser.parse_args()

    audio = load_audio(args.audio)
    if args.seconds:
        audio = audio[: int(args.seconds * SAMPLE_RATE)]
    print(f"audio: {len(audio) / SAMPLE_RATE:.2f}s")

    onnx_raw, timings = VibeVoiceOnnx(args.model_dir).transcribe(audio)
    onnx_text = flatten_transcript(onnx_raw)
    print(f"\nonnx  ({timings['total']:.1f}s): {onnx_text}")

    torch_raw, torch_seconds = torch_reference(audio, args.max_new_tokens)
    torch_text = flatten_transcript(torch_raw)
    print(f"\ntorch ({torch_seconds:.1f}s): {torch_text}")

    ratio = difflib.SequenceMatcher(
        None, torch_text.split(), onnx_text.split()
    ).ratio()
    print(f"\nword agreement: {ratio * 100:.1f}%")


if __name__ == "__main__":
    main()
