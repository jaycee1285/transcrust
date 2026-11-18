# DeBERTa / BGE correction viability — 2026-10-30

Both native Rust Q8 tests completed. **This first paired-hypothesis formulation
is not a useful automatic-correction replacement for D1.** Forced ranking
favored glossary names too often; the development-fitted conservative policies
both selected all-abstain. That yields zero repairs, not a successful correction
system. These results concern this formulation and these artifacts, not every
possible use or fine-tuning of NLI/rerankers.

```sh
nix develop -c bash tools/cross-encoder-probe/run.sh
```

[Runner, pinned models and exact methodology](../../cross-encoder-probe/README.md).
No app backend, glossary, aliases or phone APK changed. No Python, exported graph
construction, paid API or audio replay was used.

## Held out from threshold fitting

Each model evaluated 132 decisions / 264 paired rows: 32 phone decisions across
both context arms and 100 existing synthetic decisions. The synthetic split is
13 dev / 33 eval / 4 diagnostic cases **per arm**, grouped by normalized target
sentence. Initial-expansion and quoted-spelling diagnostics are excluded here.
These existing, partly contrived development fixtures are not new production
ground truth. Scope-specific labels and paired scenarios are correlated.

| Model / forced ranking | Single: repairs / 6 | Single: unwanted / 27 | Two sentences: repairs / 16 | Two sentences: unwanted / 17 |
|---|---:|---:|---:|---:|
| Saved D1 three-way argmax reference | 2 | 8 | 10 | 3 |
| DeBERTa Q8, higher entailment | 6 | 18 | 13 | 12 |
| BGE Q8, higher relevance | 6 | 26 | 16 | 15 |

D1 can natively abstain; the forced two-way baselines cannot, so the rows are
not equivalent abstention policies. Raw scores and the conservative-policy
comparison remain separate. With the supplied margin/floor grids, **neither
model found a policy with both a repair and zero unwanted dev edits**. The
explicit all-abstain candidate won for both, yielding zero repairs and zero
unwanted edits on all held-out cases. No held-out or phone labels fitted those
thresholds, and no grid or hypothesis wording was changed after those results.

The marked occurrence, its quoted sentence and candidate meaning are paired
with the same glossary/context for both interpretations. This asks a
metalinguistic question of models trained for entailment or retrieval; high
relevance/entailment is not automatically evidence that a spelling should change.
The strong name bias is evidence that **this adapter wording fails to make the
required distinction**, not proof that the model families cannot be adapted.

## Phone text

Reference: 79-word household read-aloud script, not independently verified spoken
ground truth. Word scores ignore capitalization; case-only semantic mistakes
must also be inspected in the full delivered text.

| Mode | Parakeet errors, single / two | Phonon errors, single / two |
|---|---:|---:|
| Raw phone output | 3 / 3 | 5 / 5 |
| DeBERTa forced | 4 / 4 | 5 / 5 |
| BGE forced | 3 / 4 | 5 / 5 |
| Either fitted policy | 3 / 3 | 5 / 5 |

- DeBERTa forced changed every offered phone occurrence to the name spelling,
  including ordinary gamble, water flow, food chili, game bingo and robber
  bandit. Some capitalization mistakes do not increase WER.
- BGE forced **kept “to gamble” in both sentence-only inputs**, but changed it to
  Gambol in both neighboring-sentence inputs. This is interesting target-focus
  evidence, but not a reliable full pipeline: its two-sentence Parakeet result
  left the actual dog as Gamble while changing the gambling verb to Gambol.
- `meter` and `Chilean` still never reach either model because they are not
  offered aliases. O survives because candidate discovery never expands it.
- The fitted all-abstain policies preserve cleaned phone text without repairs.

## Batch sensitivity and checks

Identical first-batch repeats were bit-identical at the logits for both models.
Changing batch composition did change logits: first occurrence isolated-pair
maximum drift was **0.607** for DeBERTa and **1.482** for BGE.

There is an actual BGE rank flip on **identical** two-sentence dog input and
hypotheses in the two phone batches:

| Batch | Ordinary relevance | Name relevance | Forced choice |
|---|---:|---:|---|
| Parakeet (14 paired rows) | 2.373 | .957 | Keep |
| Phonon (18 paired rows) | 1.052 | 2.729 | Name |

This export's batch-conditioned scores matter for deployment. Do not attribute
all cross-arm differences solely to added context or treat a raw relevance
margin as a stable calibrated probability. FP32 BGE comparison and quantization
parity across realistic batches remain open before promotion.

DeBERTa's initial neutral canary (pizza premise / dog-ownership hypothesis) was
classified as contradiction. The same pinned publisher's FP32 graph makes the
same error (95.3% contradiction), so it was retained as a visible model accuracy
diagnostic. The first attempt stopped on it; the continuation gated on the
strong entailment/contradiction checks and retained the neutral miss in results.
No correction wording was changed. See `deberta-reference-canaries.json`.

The two runner unit tests pass, model/tokenizer/config hashes verify, both
published paired tokenizers are used, and all logits are finite. Maximum pair
lengths were **270 tokens (DeBERTa)** and **364 (BGE)**; no truncation occurred.
Labels/category/split fields never enter model inputs. The FP32 canary check is
not a complete export-parity or quantization-quality evaluation.

## Resources

Native ORT CPU, four intra threads and one inter thread, release binary. Each
phone/context arm runs all occurrences in one graph call (two rows per
occurrence); synthetic batches contain up to eight occurrences.

| Model | Graph file | Load + hash check | Parakeet single / two batch | Phonon single / two batch | Full-run peak RSS |
|---|---:|---:|---:|---:|---:|
| DeBERTa Q8 AVX2 | 172.5 MB | .765 s | 1.293 / 1.279 s | 1.693 / 1.967 s | 1,206,532 KiB ≈ **1.15 GiB** |
| BGE Q8 | 279.3 MB | 1.121 s | 2.475 / 2.774 s | 3.337 / 3.485 s | 1,076,316 KiB ≈ **1.03 GiB** |

Full wall time was 21.17 / 49.93 seconds, including canaries and repeat/isolated
validation. RSS covers loading, initialization, arena/workspace and every batch,
with no ASR resident; it is neither steady model-only memory nor device memory.
BGE synthetic batches varied from 1.566 to 6.238 seconds, especially late in
the run. Machine load was uncontrolled. Saved D1 measurements used other builds
and runs; no controlled speed ranking or phone-latency claim is made here.

## Evidence and next gate

`cases.json` is the pre-inference fixture including split and exact hypotheses.
Each model's results retain all paired inputs, raw logits, NLI probabilities or
BGE relevance scores, forced/policy decisions, batch IDs, timings and delivered
phone text. Scoring TSVs, process measurements and artifact/input hashes are
retained alongside the matched D1 reference metrics. Models/audio remain outside
Git and household text stays local.

Keep D1 as-is. A next experiment would need a justified input formulation and
fresh evaluation cases, with batch/quantization validation, before claims about
NLI/reranker correction quality or mobile integration. d3-lite's custom Q8
runtime reference is recorded separately on the main taskboard; no d3 port was
started in this paired-encoder test.
