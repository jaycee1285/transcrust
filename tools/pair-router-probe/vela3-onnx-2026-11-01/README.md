# Vela3-fast ONNX trial — 2026-11-01

**Mixed result: Vela fixes the real gambling-verb mistake and is faster here,
but needs substantially more RAM and misses some wanted name repairs that the
current INT4 model makes. Keep the current live default.**

```sh
nix develop -c bash tools/vela-probe/run.sh
```

[Native runner, format, gates, pinned artifacts and reproduction](../../vela-probe/README.md).
All inference ran locally in Rust. No Python, API inference, live switch,
shared-vocabulary edits or model-adapter downloads.

## Validation

Both modes match the two rounded published billing/urgent distributions:
largest difference .000448, below the predeclared .02 canary tolerance. Two
distinct billing/technical prompts agree packed versus isolated to about
2.0e-9 in fast mode and 1.0e-9 in low-memory mode. Source/model/tokenizer hashes
bind the public checks to private evaluation. A bounded readout unit test passes.
This is partial published-output validation, **not full PyTorch/logit parity**.

The final replay uses the strengthened different-prompt packing check. The
initial duplicate-only check was insufficient to detect cross-prompt mixing,
so it was replaced before the recorded final replay. Unrelated old DecisionTune
fixtures were not treated as Vela references. The ONNX graph itself is the
publisher's unmodified, checksum-verified file.

## Actual phone text

Seven Parakeet and nine Phonon candidate occurrences, each with sentence-only
and original neighboring context. Both memory modes ran full and stripped
states: 128 phone decisions across the four configurations. The stripped
setup removes the global glossary table while retaining each candidate's term
description. No floor or new prompt was fitted to these results.

| Model / input | Parakeet word errors: sentence / context | Phonon word errors: sentence / context |
|---|---:|---:|
| Raw phone output | 3 / 3 | 5 / 5 |
| Current UltraBERT INT4, stripped | 2 / 3 | 4 / 5 |
| Vela, full glossary, either memory mode | 2 / 3 | 4 / 5 |
| Vela, stripped, either memory mode | **2 / 2** | **4 / 4** |

Reference is the intended 79-word script, not independently verified audio.
WER ignores casing, so semantic preservation is checked separately.

- Stripped Vela preserves **all 5/5 ordinary controls in both scopes** for
  both transcripts: gambling verb, water flow, food chili, game bingo and
  robber bandit. Current Ultra preserves 5/5 alone and 4/5 with context.
- It repairs dog Gamble → Gambol and project casing. It still misses character
  flow → Flo; character bandit is kept or abstained, so its casing is not fixed.
- The full glossary restores the wrong context gambling edit and incorrectly
  capitalizes robber bandit: one ordinary error alone, two with context.
- Intentional O remains unchanged in the phone output because it is not a
  candidate. Missing aliases `meter` / `Chilean` remain unoffered.

Fast and low-memory modes agree on **all 64 phone choices** across input forms.
Largest probability drift is 1.2414e-6 full / 3.6242e-7 stripped. The expanded
fast run repeats the same phone decisions while adding the synthetic surface.

## Broader paired comparison

Fast stripped Vela ran all 132 frozen rows: 32 phone plus 100 synthetic. The
actual current native UltraBERT INT4 code then ran exactly those same states,
questions and meanings. Its 32 phone choices and probabilities match the prior
INT4 benchmark **exactly**, despite the changed execution block grouping.

Exclude the eight initial/quoted-spelling diagnostics from headline synthetic
results. The remaining **92 frozen synthetic rows** (dev + held-out evaluation)
show a repair/preservation tradeoff:

| Context arm | Vela wanted repairs | Current Ultra wanted repairs | Vela unwanted edits | Current Ultra unwanted edits |
|---|---:|---:|---:|---:|
| Sentence only | 8/8 | 8/8 | **3/38** | 4/38 |
| Neighboring context | 17/20 | **20/20** | **5/26** | 8/26 |
| Both arms | 25/28 | **28/28** | **8/64** | 12/64 |

Held-out `eval` alone, 33 rows per arm:

| Context arm | Vela repairs | Ultra repairs | Vela unwanted edits | Ultra unwanted edits |
|---|---:|---:|---:|---:|
| Sentence only | 6/6 | 6/6 | **3/27** | 4/27 |
| Neighboring context | 13/16 | **16/16** | 3/17 | 3/17 |

The repair denominator changes between arms because context changes the desired
answer on deliberately ambiguous scenarios. Compare models within the same arm,
not raw repair counts between different arms. No dev fitting was performed.

Three context repairs lost relative to Ultra:

| Marked sentence and preceding context | Vela | Current Ultra |
|---|---|---|
| “He gave the red race car eyes and the number 95. My son drew «lightning».” | Keep | Lightning McQueen |
| “She was serving the other Cars characters at the V8 Cafe. My son drew «flow».” | Keep | Flo |
| “He meant the mom of Bluey and Bingo. My son asked about «chili».” | Keep | Chili |

Vela also avoids several unwanted mother/project/Bluey-context edits, while
adding some Mater errors; the complete changed rows are in `paired.json`.
These are difficult constructed cases, not a large representative corpus.
Diagnostics remain separate: Vela changes 3/4 per arm, Ultra 1/4 alone and
3/4 with context. The forced one-character initial probes bypass the real
candidate filter, so they do not demonstrate an actual app privacy regression.

## Measured x86 CPU RAM and latency

Separate sequential processes, four CPU threads, no ASR, uncontrolled background
load. Fast mode expands weights to FP32; low-memory mode retains INT8 projections
with FP32 activations. The file is 452 MB; **download size is not runtime RAM**.

| Configuration | Peak process RSS | Phone batch time, 7–9 candidates |
|---|---:|---:|
| Vela fast, full glossary | 2,164,060 KiB ≈ **2.06 GiB** | 5.63–13.53 s |
| Vela fast, stripped | 2,164,044 KiB ≈ **2.06 GiB** | **1.83–2.89 s** |
| Vela low-memory, full glossary | 882,632 KiB ≈ **862 MiB** | 10.56–13.63 s |
| Vela low-memory, stripped | 882,128 KiB ≈ **861 MiB** | **3.11–3.90 s** |
| Current UltraBERT INT4, prior phone-only run | **299 MiB** | 3.21–5.41 s |

The fresh current-Ultra baseline across all 132 rows peaks at **311,232 KiB ≈
304 MiB** and takes 61.33 s total, using native packed blocks of eight. Vela's
expanded fast pass takes 43.25 s, peak 2,164,060 KiB. Its published packing policy
uses 6–9 graph calls for phone batches, versus the app's one packed native
forward call. Individual synthetic batches may mix context arms; row metrics
remain separated correctly. Different batching and uncontrolled load prevent
turning these observations into stable speedup guarantees.

Fast model loading was about 1.8–1.9 s; low-memory about .8–.9 s. Process wall
times include verification, loading and serialization. File-cache RAM outside
RSS is excluded. No Android/iOS/Apple latency or process memory was measured.

## Disposition / evidence

Vela is a credible preservation-oriented alternative, but not a clear overall
upgrade on these cases. Even its lower-memory mode uses roughly three times
our current model's RSS. Leave UltraBERT INT4 primary. A future native INT4 Vela
trial would need its own quantization validation: the export author reported
94.6% answer agreement for their attempted 4-bit conversion, not parity.

`fast/` and `low/` preserve raw states, questions, IDs/offsets, logits-derived
probabilities, choices, delivered text and per-process resource logs.
`ultra-baseline.json`, `paired.json` and `baseline-phone-check.json` preserve the
same-input current-model comparison. `provenance.sha256` records native source,
harness, fixtures and lockfile. Models stay in ignored `target/`; no audio or
weights enter Git, and no production backend/default was changed.
