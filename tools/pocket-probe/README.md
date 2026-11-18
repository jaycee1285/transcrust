# MNN-Pocket-0.3B-Dec decision probe (2026-11-18)

First native correctness/quality probe of Pocket as a keep / supplied-term / abstain
decision model. **Verdict: not competitive with UltraBERT INT4 or D1; do not promote.**
Research only: nothing here touches the daemon, `--dec`, `~/syncthing`, or any commit.

```sh
bash tools/pocket-probe/run.sh          # evidence lands in tools/pocket-probe/results/
```

Downloads only the pinned model (~754 MB, SHA-256 checked against `SHA256SUMS`) into
`~/.cache/transcrust/pocket-dec`, builds the standalone crate into
`~/.cache/transcrust/pocket-target`, runs five configurations (about 4 minutes on 4 CPU threads),
then the parity check and report. The parity step needs `nix develop .#pair-probe` (numpy + tokenizers).

## Model

- `taobao-mnn/MNN-Pocket-0.3B-Dec`, revision **`dfbe0aed3abd4caed175b883a4b462ed35184fad`** (lastModified 2026-11-04).
- **Licence: Apache-2.0** (card + `LICENSE.txt`). Base `MNN-Pocket-0.3B` is a Qwen3 post-train.
- Qwen3 backbone (28 layers, hidden 896, GQA 14/2, RoPE theta 1e6), unmerged LoRA r16/alpha32,
  FP32 256-wide pointer head, calibrated temperature 1.1112 (probabilities are not comparable with other models').

## Route

Route 1 (Rust/Candle). No ONNX of the Dec model exists and torch is not in the flake, so
`main.rs` ports the pinned `inference.py` onto `candle-core` 0.9.2 (same crate/lock as the app;
standalone Cargo package, empty `[workspace]`, own target dir):

- Serializer: `[state] state [question] instructions ([option] "key: description" [close])... [decide]`, control ids 65526/65527/65515/65516/65528 verified against the tokenizer at load, `<|X|>` escaping reproduced.
- LoRA merged into F32 weights at load (W + 2BA); embeddings stay BF16 and are gathered per row; lm_head is not loaded.
- One question per request, so the reference's branch mask and position reset reduce to plain causal / contiguous positions. Option order is keep, name, abstain with the same descriptions the other models got.
- Pointer logit = q(decide) . k(option_end) / 16; argmax picks the answer (temperature affects only the reported probability).
- **Parity:** `parity.py` is an independent numpy forward (unmerged LoRA, as the reference runs it) over 6 rows: logits agree to 2.8e-5 max, same argmax, and Python `tokenizers` reproduces the state tokens (`results/parity.txt`). This is a second reading of `inference.py`, not a run of the PyTorch reference (no torch available); it does not exclude a shared misreading of the reference.

Inputs are the frozen rows from `tools/pair-router-probe/vela3-onnx-2026-11-01/fast/{expanded,full}.json`
(same 132 states, questions, labels, candidates, batch grouping, edit application as the Ultra/Vela runs; scored against
`ultra-baseline.json` from the same directory). Headline setup is the stripped glossary, like current Ultra.

## Results (stripped glossary, 92 ordinary synthetic rows; 8 initial/quoted diagnostics kept apart)

Repairs / unwanted edits / abstentions (abstentions count every row answered "abstain", including rows labelled abstain).
Ultra and Vela are the frozen numbers in `decision-model-results.md`.

| Model | Sentence only (46) | Neighboring context (46) | Both (92) | Diagnostics changed (single, two) |
|---|---|---|---|---|
| Ultra INT4 | 8/8, 4/38, 2 | 20/20, 8/26, 0 | **28/28, 12/64**, 2 | 1/4, 3/4 |
| Vela fast | 8/8, 3/38, 2 | 17/20, 5/26, 0 | 25/28, 8/64, 2 | 3/4, 3/4 |
| **Pocket** | 6/8, 10/38, 18 | 13/20, 11/26, 14 | **19/28, 21/64**, 32 | 2/4, 3/4 |

D1 Q8 has no result on this 92-row surface; its own 23-scenario context study (neighbors: repairs 2 to 5/8, unwanted 5 to 7/15) is not comparable. Pocket has about 1.8x Ultra's unwanted edits while repairing fewer, and answers abstain on 32 of 92 rows.

### Household phone replay (79-word script; errors single / two-sentence context)

| Model | Parakeet | Phonon |
|---|---|---|
| Raw phone output | 3 / 3 | 5 / 5 |
| D1 Q8, full glossary | 2 / 3 | 4 / 4 |
| Ultra INT4, stripped | 2 / 3 | 4 / 5 |
| **Pocket, stripped** | 3 / 3 | 5 / 5 |
| **Pocket, full glossary** | 3 / 3 | 4 / 4 |

Pocket stripped equals raw phone output: net zero improvement.

Known cases (choices in `results/report.txt`):

- **"to gamble" infinitive (should keep):** wrongly changed to Gambol in **all four** arms, both glossary modes. Ultra/D1 get it right single-sentence; Ultra/D1 only miss it with context, Vela keeps it with context.
- **Dog Gamble to Gambol:** repaired in all four arms.
- **Flo repair:** stripped, Pocket abstains in every arm (misses it, like Ultra and Vela). With the full glossary it repairs the Phonon Flo ("meter and flow") in both arms. Water flow is kept/abstained throughout.
- **Controls (wrong name edits):** stripped, gambling verb in all four arms plus robber "bandit" to Bandit in the two-sentence arms (D1/Ultra: 0 single, 1 with context); full glossary, gambling verb plus robber "bandit" in all four arms (2 per transcript; D1 full: 0/1). Chili and bingo are kept or abstained.
- **Synthetic regressions vs Ultra** (stripped): unwanted edits on dog/son/project neighbor-name, Bluey bingo/chili/bandit common-word and project/son abstain-labelled rows (full list in `results/report.txt`); missed repairs including Lightning, Flo, Bingo, son/project/habit two-sentence rows and the explicit Chili/Bandit rows.

## Extra arm: local window and name-or-word question

Requested by the coordinator (John's hypothesis: name-vs-word is syntax within a few words; extra sentences add noise).
Same rows and labels. **Window** = up to 3 whitespace words each side of the marked span, never crossing a sentence boundary.
**Frame** = the question string replaced by `Is "<span>" used here as a name or as the ordinary word?` (Pocket's format takes any question string; option descriptions unchanged, no format change). Frame is out-of-distribution for the trained question text.
Single-sentence labels are context-independent; **two-sentence labels that were derived from neighbours cannot be reached by a window**, so judge the window rows mainly on the single-sentence column.

| Pocket input | Single: repairs, unwanted, abstain | Two (labels need neighbours): | Both | Phone errors P/Ph (single) |
|---|---|---|---|---|
| Sentence / context, generic question | 6/8, 10/38, 18 | 13/20, 11/26, 14 | 19/28, 21/64, 32 | 3 / 5 |
| Window +-3, generic question | 3/8, 14/38, 18 | 7/20, 10/26, 18 | 10/28, 24/64, 36 | 3 / 5 |
| Window +-3, name-or-word question | 1/8, 6/38, 28 | 3/20, 4/26, 28 | 4/28, 10/64, 56 | 4 / 6 |
| Sentence / context, name-or-word question | 3/8, 8/38, 27 | 7/20, 11/26, 19 | 10/28, 19/64, 46 | 4 / 6 |

Result: **hypothesis not supported for Pocket.** The window alone lost repairs and added unwanted edits (single 14/38); the framed question mostly makes it abstain (56/92) rather than separate names from words, and the phone "gamble" verb still changes. It is not a reliable local syntax signal for this model. One model, one prompt wording; it says nothing about D1/Ultra under the same window.

## Resources (4 CPU threads, x86, no ASR resident; `results/*-process.json`)

| Measure | Pocket (F32 merged, not quantized) | Ultra INT4 | D1 Q8 |
|---|---|---|---|
| Peak RSS | **1,727,232 KiB = 1.65 GiB** (all runs 1.65 GiB) | 299-304 MiB | ~701 MiB |
| Load (warm page cache) | 2.1-2.8 s | 0.3 s | n/a here |
| Warm phone batch, stripped (7-9 cands) | 1.86-2.96 s | 3.21-5.41 s | 10.2-14.5 s |
| Warm phone batch, full glossary | 7.0-9.5 s | n/a | n/a |
| 8-row synthetic batch (stripped, median) | about 2.5 s | n/a | n/a |

Candidates run one at a time (no packing); the first row is a discarded warm-up. Latency is fast for its size but RSS is ~5.5x Ultra, and no INT4 build of the full decision checkpoint exists (the 173 MB MNN INT4 is the base model without the adapter/head; see the 2026-11-08 review).

## Verdict

Pocket runs natively and the port checks out numerically, but on our surfaces it repairs less than Ultra (19/28 vs 28/28), makes more unwanted edits (21/64 vs 12/64), abstains heavily, fails the "to gamble" preservation case in every arm and wrongly edits the robber "bandit", and does not improve phone WER over raw output. Windowing or a name-or-word question does not rescue it. It needs 5.5x Ultra's RAM at F32. Recommendation: park; a quantized variant is not worth building unless a new reason appears. Caveats: one run, 4 threads, uncontrolled load; synthetic rows are constructed and correlated; the model is trained on its own question distribution, and our fixed keep/name/abstain descriptions may be a poor fit.

## Files

`main.rs` (probe), `Cargo.toml`/`Cargo.lock` (copied from the app lock; standalone), `rss.py` (peak RSS wrapper, no GNU time on this host), `parity.py`, `report.py`, `run.sh`, `SHA256SUMS`, `results/` (rows with token ids, raw logits, per-run process/stderr, phone transcripts and WER tsv, `report.txt`, `parity.txt`).
Environment knobs: `POCKET_WINDOW=N`, `POCKET_FRAME=1` (extra arm only).
