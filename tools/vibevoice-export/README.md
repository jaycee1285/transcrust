# VibeVoice ASR → ONNX  *(retained as a template)*

> **VibeVoice was evaluated and removed on 2026-09-06.** It lost to Parakeet on
> size (1.8 GB vs 391 MB), latency (RTF 1.1–1.6 vs 0.18) and showed no
> measurable quality advantage. The postmortem — weights inventory, timing
> tables, what went wrong — is at `~/syncthing/vibevoice-asr-15/`.
>
> This directory is kept because it is the repo's only working ONNX export
> pipeline, and `TASKBOARD-next.md` lists rebuilding the Granite
> export as blocked with this as the starting point. `quantize_onnx.py` and
> `common.py` are model-agnostic; the two `export_*.py` scripts are the worked
> example of tracing a torch model to a KV-cached graph.
>
> `install.sh` still runs and will still install VibeVoice. Nothing depends on
> it. It is 80 KB and one `rm -rf` from gone if you would rather it were.

Builds [`microsoft/VibeVoice-ASR-Streaming-1.5B`](https://huggingface.co/microsoft/VibeVoice-ASR-Streaming-1.5B)
into three ONNX graphs for the (since removed) `src/vibevoice.rs`, installed next to
the Parakeet and Granite models.

There is no published ONNX export of this checkpoint, which is why this exists
and why `--download-model` cannot fetch it.

```sh
./install.sh          # build + install, ~1.9 GB (int4)
./uninstall.sh        # remove the model
./uninstall.sh --all  # also drop the build tree, the venv and the HF cache
```

Everything is incremental; a rerun skips whatever is already on disk. First run
needs roughly **13 GB of RAM+swap**, **15 GB of scratch disk**, and downloads
**5.6 GB** from Hugging Face. It takes on the order of half an hour.

## Sizes, and why it is not smaller

"1.5B" names the language model only. The ASR path is 2.24 B parameters:

| component | params | int4 on disk |
|---|---|---|
| Qwen2 decoder layers | 1310 M | 836 MB |
| `lm_head` | 233 M | 131 MB |
| acoustic tokenizer encoder | 344 M | ~360 MB |
| semantic tokenizer encoder | 345 M | ~360 MB |
| embedding table (int8, per-row) | 233 M | 234 MB |
| connectors | 5 M | small |

so ~1.9 GB at int4 and ~2.6 GB at int8. There is no configuration that reaches
400–800 MB: the two conv encoders alone are larger than Parakeet.

Conv weights inside the encoders stay fp32 even in int4 mode. ORT's dynamic
path turns `Conv` into `ConvInteger`, which on CPU is routinely slower than the
fp32 kernel, and the convs are a minority of the encoder's parameters.

## The graphs

| file | shape contract |
|---|---|
| `speech_encoder.<q>.onnx` | `audio [1,1,83200] @ 24 kHz` → `speech_features [1,26,1536]` |
| `embed_tokens.int8.onnx` | `input_ids [1,S]` → `inputs_embeds [1,S,1536]` |
| `decoder.<q>.onnx` | `inputs_embeds`, 4-D additive `attention_mask`, `position_ids`, 28×2 `past.*` → `logits [1,1,151936]`, 28×2 `present.*` |

Three things about that contract are load-bearing:

* **The encoder's time axis is static.** The SConv1d stack reads `x.shape[-1]`
  to size its stride-alignment padding, so tracing freezes the trace-time
  length no matter what `dynamic_axes` claims — a longer input comes back with
  the traced number of frames and no error. Pinning it at the one window the
  runtime ever needs turns that into a loud ORT shape error instead.
* **The decoder takes `inputs_embeds`, not `input_ids`,** because speech frames
  are spliced into the embedding sequence. That is also why the embedding table
  has to be its own graph.
* **The attention mask is 4-D and additive.** Transformers passes a 4-D mask
  straight through instead of deriving one, which keeps every sequence-length
  decision in the caller and out of the frozen graph. A 2-D mask would bake the
  trace-time length in.

Only the last position's logits come back: greedy decode never reads the
others, and a full prefill output would be a 180 MB tensor.

## The protocol

This checkpoint is `VibeVoiceForASRStreamingTraining`. The batch `generate()`
path in `demo/vibevoice_asr_inference_from_file.py` — the one that asks for
speaker-attributed JSON — is the **wrong contract for it**, and produces
confident nonsense rather than an error. `streaming_generate` is the right one:

```
prompt → [ <|object_ref_start|> speech(26 frames) <|object_ref_end|> ]
         → text… <|text_chunk_end|>            (repeat per chunk)
```

over one KV cache for the whole utterance. Chunk is 22 frames, lookahead 4, so
consecutive 26-frame windows overlap and the window strides by 22. 26 × 3200 =
83200 samples exactly, which is what makes the static encoder shape work.

One deliberate deviation: `encode_speech` samples the acoustic latent
(`dist_type="gaussian"`), injecting per-utterance noise. That is a training
regulariser, and it makes the reference non-deterministic. These graphs emit
the distribution mean, so the same audio produces the same text.

## Files

| | |
|---|---|
| `export_speech_encoder.py` | acoustic + semantic encoders and their connectors |
| `export_decoder.py` | Qwen2 decoder + `lm_head`, KV-cached |
| `export_embed.py` | per-row int8 embedding lookup |
| `quantize_onnx.py` | `MatMulNBitsQuantizer` (int4) or `quantize_dynamic` (int8) |
| `run_onnx.py` | the reference implementation the removed `src/vibevoice.rs` transliterated |
| `reference_torch.py` | upstream PyTorch, for comparison |
| `validate_onnx.py` | runs both on one clip and reports word agreement |
| `VIBEVOICE_COMMIT` | the upstream commit the export is pinned to; `common.py` clones it into `build/vibevoice-src` on first use |
| `flake.nix` | fallback dev shell if `uv` is not already on PATH |

`run_onnx.py` was the spec the Rust engine was transliterated from. That engine
is gone; the file is kept as the worked example of the pattern:
prompt text, chunk arithmetic, mask convention and stop conditions are shared.

## Measured

20 s clip, 12-core CPU, int4:

```
onnx  19.6s   torch fp32  54.5s   word agreement 84.7%
```

The disagreements are ordinary ASR ambiguity on a noisy interview clip, not
quantisation damage — and part of the gap is the reference's own sampling
noise. Expect roughly 0.7–0.9× real time end to end.
