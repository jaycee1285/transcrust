#!/usr/bin/env python
"""Reference implementation of the VibeVoice ASR *streaming* ONNX pipeline.

This is the spec `src/vibevoice.rs` transliterates. Keep the two in step: the
prompt text, the chunk arithmetic, the per-chunk token frame and the stop
conditions are shared, and `validate_onnx.py` compares this against PyTorch.

The checkpoint is `VibeVoiceForASRStreamingTraining`, so the batch
`generate()` path in `vibevoice_asr_inference_from_file.py` is the *wrong*
contract for it — that one asks for speaker-attributed JSON and this model
answers with streaming-chunk text regardless. The protocol implemented here is
`VibeVoiceASRForConditionalGeneration.streaming_generate`:

    prompt  ->  [ <|object_ref_start|> speech(26) <|object_ref_end|> ]
                 -> text ... <|text_chunk_end|>   (repeat per chunk)

with one shared KV cache running the length of the utterance.
"""

import argparse
import math
import re
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort

SAMPLE_RATE = 24000
COMPRESS_RATIO = 3200
HIDDEN = 1536
NUM_LAYERS = 28
KV_HEADS = 2
HEAD_DIM = 128

# From the checkpoint's preprocessor_config.json: chunk_frames 22, lookahead 4.
# 26 frames x 3200 samples lands on exactly 83200, so every encoder call gets an
# identically shaped, alignment-clean window and the last one is zero-padded up
# to it.
CHUNK_SAMPLES = 70400
LOOKAHEAD_SAMPLES = 12800
WINDOW_SAMPLES = CHUNK_SAMPLES + LOOKAHEAD_SAMPLES
WINDOW_TOKENS = WINDOW_SAMPLES // COMPRESS_RATIO

SPEECH_START_ID = 151646  # <|object_ref_start|>
SPEECH_END_ID = 151647  # <|object_ref_end|>
TEXT_CHUNK_END_ID = 151665  # <|text_chunk_end|>
EOS_ID = 151643  # <|endoftext|>

PROMPT = (
    "You are a helpful assistant that transcribes audio input into text output. "
    "Please transcribe the following audios streamingly with these keys: speaker, content\n"
)

MAX_NEW_TOKENS_PER_CHUNK = 256


def causal_mask(sequence: int, past: int) -> np.ndarray:
    total = past + sequence
    rows = np.arange(sequence).reshape(-1, 1) + past
    columns = np.arange(total).reshape(1, -1)
    mask = np.where(columns <= rows, 0.0, np.finfo(np.float32).min)
    return mask.astype(np.float32).reshape(1, 1, sequence, total)


def session(path: Path, threads: int) -> ort.InferenceSession:
    options = ort.SessionOptions()
    options.intra_op_num_threads = threads
    options.inter_op_num_threads = 1
    options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    return ort.InferenceSession(str(path), options, providers=["CPUExecutionProvider"])


def chunk_windows(audio: np.ndarray) -> list[np.ndarray]:
    """Split into overlapping encoder windows, exactly as `streaming_generate`
    does in its default `split_then_encode` mode: each window carries its chunk
    plus a lookahead tail, and the stride is the chunk alone."""
    windows = []
    start = 0
    total = len(audio)
    while start < total:
        segment = audio[start : min(start + WINDOW_SAMPLES, total)]
        if len(segment) < WINDOW_SAMPLES:
            segment = np.pad(segment, (0, WINDOW_SAMPLES - len(segment)))
        windows.append(segment)
        start += CHUNK_SAMPLES
    return windows


class VibeVoiceOnnx:
    def __init__(self, model_dir: Path, threads: int = 0):
        from tokenizers import Tokenizer

        self.encoder = session(model_dir / "speech_encoder.onnx", threads)
        self.embed = session(model_dir / "embed_tokens.onnx", threads)
        self.decoder = session(model_dir / "decoder.onnx", threads)
        self.tokenizer = Tokenizer.from_file(str(model_dir / "tokenizer.json"))
        self.present_names = [
            f"present.{layer}.{kind}"
            for layer in range(NUM_LAYERS)
            for kind in ("key", "value")
        ]
        self.output_names = ["logits"] + self.present_names
        self.reset()

    def reset(self) -> None:
        self.past = {
            f"past.{layer}.{kind}": np.zeros((1, KV_HEADS, 0, HEAD_DIM), dtype=np.float32)
            for layer in range(NUM_LAYERS)
            for kind in ("key", "value")
        }
        self.position = 0

    def embed_ids(self, ids) -> np.ndarray:
        array = np.asarray(ids, dtype=np.int64).reshape(1, -1)
        return self.embed.run(["inputs_embeds"], {"input_ids": array})[0]

    def step(self, embeds: np.ndarray) -> np.ndarray:
        """Advance the shared cache by `embeds` and return the final logits."""
        sequence = embeds.shape[1]
        feeds = {
            "inputs_embeds": embeds,
            "attention_mask": causal_mask(sequence, self.position),
            "position_ids": np.arange(
                self.position, self.position + sequence, dtype=np.int64
            ).reshape(1, -1),
            **self.past,
        }
        outputs = self.decoder.run(self.output_names, feeds)
        for index, name in enumerate(self.present_names):
            self.past[name.replace("present.", "past.")] = outputs[index + 1]
        self.position += sequence
        return outputs[0]

    def transcribe(self, audio: np.ndarray, verbose: bool = False):
        timings = {"encode": 0.0, "decode": 0.0, "generated_tokens": 0}
        started = time.perf_counter()
        self.reset()

        prompt_ids = self.tokenizer.encode(PROMPT, add_special_tokens=False).ids
        self.step(self.embed_ids(prompt_ids))

        start_embed = self.embed_ids([SPEECH_START_ID])
        end_embed = self.embed_ids([SPEECH_END_ID])
        chunk_end_embed = self.embed_ids([TEXT_CHUNK_END_ID])

        chunks = []
        for index, window in enumerate(chunk_windows(audio)):
            encode_started = time.perf_counter()
            features = self.encoder.run(
                ["speech_features"], {"audio": window.reshape(1, 1, -1).astype(np.float32)}
            )[0]
            timings["encode"] += time.perf_counter() - encode_started
            assert features.shape[1] == WINDOW_TOKENS, features.shape

            logits = self.step(np.concatenate([start_embed, features, end_embed], axis=1))

            tokens = []
            for _ in range(MAX_NEW_TOKENS_PER_CHUNK):
                token = int(np.argmax(logits[0, -1]))
                if token in (TEXT_CHUNK_END_ID, EOS_ID):
                    break
                tokens.append(token)
                logits = self.step(self.embed_ids([token]))
            # The chunk boundary token is fed even when the model stopped on its
            # own, so the cache carries the same frame the model was trained on.
            self.step(chunk_end_embed)

            timings["generated_tokens"] += len(tokens)
            text = self.tokenizer.decode(tokens, skip_special_tokens=True)
            chunks.append(text)
            if verbose:
                print(f"  [{index + 1}] {text!r}", flush=True)

        timings["total"] = time.perf_counter() - started
        timings["decode"] = timings["total"] - timings["encode"]
        timings["chunks"] = len(chunks)
        return "".join(chunks), timings


SPEAKER_PREFIX = re.compile(r"(?:^|(?<=[\s.,!?]))[Ss]peaker\s*\d+\s*:\s*")


def flatten_transcript(raw: str) -> str:
    """Strip the speaker labels the streaming format interleaves with the words.

    The model answers with `speaker, content` pairs — "Speaker 0:" markers
    inline with the text. transcrust injects dictation, not a diarised
    transcript, so the labels come out and the words stay.
    """
    text = SPEAKER_PREFIX.sub("", raw)
    for special in (
        "<|text_chunk_end|>",
        "<|object_ref_start|>",
        "<|object_ref_end|>",
        "<|box_start|>",
    ):
        text = text.replace(special, "")
    return re.sub(r"\s+", " ", text).strip()


def load_audio(path: Path) -> np.ndarray:
    import librosa

    audio, _ = librosa.load(str(path), sr=SAMPLE_RATE, mono=True)
    return audio.astype(np.float32)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-dir", type=Path, required=True)
    parser.add_argument("--audio", type=Path, required=True)
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument("--seconds", type=float, default=None, help="trim the clip")
    args = parser.parse_args()

    audio = load_audio(args.audio)
    if args.seconds:
        audio = audio[: int(args.seconds * SAMPLE_RATE)]
    duration = len(audio) / SAMPLE_RATE
    print(f"audio: {duration:.2f}s")

    engine = VibeVoiceOnnx(args.model_dir, args.threads)
    raw, timings = engine.transcribe(audio, verbose=True)
    print(f"raw       : {raw!r}")
    print(f"flattened : {flatten_transcript(raw)!r}")
    print(
        "timing    : encode {encode:.2f}s decode {decode:.2f}s total {total:.2f}s "
        "({chunks} chunks, {generated_tokens} tokens)".format(**timings)
    )
    print(f"            RTF {timings['total'] / duration:.2f}")


if __name__ == "__main__":
    main()
