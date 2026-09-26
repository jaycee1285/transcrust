# parakeet-ultra → int4 (onnx-asr layout)

Builds [moondream/parakeet-ultra](https://huggingface.co/moondream/parakeet-ultra)
into the same four files transcrust, mopvox and dayshade already load for
Parakeet v3. Built and measured 2026-09-25.

```sh
cd tools/ultra-export
nix develop -c bash -c 'uv venv --python 3.12 .venv && uv pip install --python .venv/bin/python --index-strategy unsafe-best-match -r requirements.txt'
nix develop -c .venv/bin/python export_ultra.py --out build     # ~2 min after a 1.26 GB download
# -> build/install/parakeet-ultra-0.6b-int4/  (350 MB)
../../target/release/transcrust --parakeet-direct build/install/parakeet-ultra-0.6b-int4 some.wav
```

| file | size | v3 int4 equivalent |
|---|---|---|
| `encoder-model.int4.onnx` | 331 MB | 373 MB |
| `decoder_joint-model.int8.onnx` | 18 MB | 18 MB |
| `nemo128.onnx`, `vocab.txt` | copied from v3 | identical |

## Footguns hit

1. **The published ONNX exports are the wrong layout.** `mldecode/parakeet-ultra-onnx-int8`
   is a sherpa split (encoder/decoder/joiner, 640-dim frames). Re-quantizing it would
   stack two losses. The route that works: the HF checkpoint goes through kestrel's
   own fp loader (`pip install moondream`), then the tracing wrappers from
   `eschmidbauer/parakeet-redux-onnx`'s `export_onnx.py`. That script is
   ternary-only as published, so this is an adaptation.
2. **The checkpoint is fp16 on disk** (1.26 GB) even though `config.json` says float32.
   Harmless: it's upcast to fp32 before export.
3. **The phone apps hardcode `decoder_joint-model.int8.onnx`** (`mopvox/core/.../Parakeet.kt:155`).
   transcrust would take an fp32 joint, but the phones won't.
4. **ORT's int8 quantizer rejects the dynamo-exported joint** (`Inferred shape and existing
   shape differ: 1280 vs 2560`). Fix: strip `graph.value_info` first. The LSTM comes out
   unrolled, not as `DynamicQuantizeLSTM` like v3's, at the same 18 MB.
5. **onnx refuses a `.onnx.data` with multiple hard links** ("potential hardlink attack").
   Don't `ln` build outputs into test directories; copy them.
6. **Mel mismatch was a non-issue.** Photon's `preprocessor.onnx` and v3's `nemo128.onnx`
   agree except in the first two frames (a start-padding difference). Clean transcripts
   were identical either way.
7. **Number formatting flips.** Both models emit "zero"/"0" or "eighteen"/"18"
   inconsistently, even between a clean and a noisy copy of the same clip. It
   inflates every word-diff below. Ultra leans towards digits.

## Measured on Record-2 (10 × 15 s clips, 44.1 kHz, John's voice)

No ground truth, so this is **drift from each model's own clean transcript** once
noise is mixed in (lower = more robust). Pink noise is broadband, the stand-in for
road/HVAC noise. "Rumble" is brown noise low-passed at 300 Hz plus 30 Hz hum, the
idle-engine proxy.

| condition | v3 int4 | ultra int4 |
|---|---|---|
| rumble 5 dB / 0 dB | 7.5% / 4.5% | 3.6% / 1.6% |
| pink 10 dB | 13.1% | 5.7% |
| pink 5 dB | 20.6% | 8.8% |
| pink 0 dB | 37.2% | 23.2% |
| clean, v3 vs ultra | 12.1% (mostly number formatting; ultra fixed "job as" → "job ads", and v3 said "have" where ultra said "had") | |

- **int4 cost:** ultra fp32 vs int4 differs by 1% clean and 4% at pink 5 dB.
- **Speed (desktop, 10 clips incl. 10 model loads):** v3 int4 39 s, ultra int4 28 s,
  ultra fp32 63 s.
- **ORT 1.24.3 (the phones' version) loads and runs all three graphs** on desktop CPU.
  Graphs are IR 10 / opset 18 vs v3's IR 8 / opset 17.
- **Not measured:** phone speed and RAM,
  and real carline audio.
