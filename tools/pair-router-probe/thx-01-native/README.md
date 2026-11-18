# THX-01 native viability — 2026-10-29

**This initial native probe did not improve the household corrections over D1.**
It fixes Gambol, but repeats the “to gamble” error and abstains on the Cars Flo
repair. Native CPU F32 execution is working; the inference implementation still
needs independent upstream numerical parity before production promotion. Do not
treat these preliminary results as a definitive ranking of the original models.

Reproduce:

```sh
nix develop -c bash tools/thx-probe/run.sh
```

Implementation, pinned artifacts, formatting and limitations:
[native probe](../../thx-probe/README.md). No Python execution or export;
the graph operations run through Rust Candle. D1 remains the desktop backend.
No aliases, prompts, shared vocabulary or phone APKs changed.

## Actual phone inputs

One call per frozen production utterance, with the same candidates, glossary,
two-sentence states, question and option meanings as the saved D1 audit. Native
THX option encoding differs as required by its published formatter. The initial
standalone first-row warmup and padded-batch probabilities matched exactly.

| Source | Raw word errors / 79 | D1 output errors | THX output errors | THX warmed batch |
|---|---:|---:|---:|---:|
| Parakeet | 3 | 3 | 3 | 7 candidates, 9.229 s |
| Phonon | 5 | 4 | 5 | 9 candidates, 12.342 s |

The read-aloud script is the expected reference, not independently verified
spoken ground truth. Scores ignore case/punctuation. Both engines preserve O
because candidate discovery never proposes initial expansion.

| Occurrence | THX | D1 |
|---|---|---|
| “My dog Gamble…” | Gambol; P(name) .436 | Gambol; P(name) .652 |
| “I like to gamble and play poker…” | **Wrongly Gambol**; P(name) .498 | **Wrongly Gambol**; P(name) .756 |
| Phonon's “raced past meter and flow” | flow unchanged, abstain | Flo repaired |
| Ordinary water flow | Kept | Kept |
| Ordinary food chili / game bingo | Abstained, preserved | Kept |
| Ordinary robber bandit | Kept | Kept |
| Phonon's character bandit | Kept lowercase | Abstained, preserved lowercase |
| Transcrust project spelling | transcrust | transcrust |
| meter / Chilean | Not candidates | Not candidates |

THX's name probability on the erroneous infinitive is again higher than on the
actual dog-name repair. Simply increasing a confidence floor is not evidence
of a better discriminator here.

## Synthetic glossary comparison

All 100 saved expanded-glossary rows were evaluated. The table excludes the
eight privacy-initial/explicit-spelling diagnostic rows (four per scope), as
previously requested. Remaining cases still include contrived “other Rowen”
controls and hand-authored labels; this is a development diagnostic, not a
representative dictation accuracy estimate. Single/two inputs are correlated.

| Scope | Expected repairs | D1 repairs | THX repairs | Expected preservation | D1 unwanted edits | THX unwanted edits |
|---|---:|---:|---:|---:|---:|---:|
| Single sentence (46 rows) | 8 | 2 | 7 | 38 | 9 | 20 |
| Two sentences (46 rows) | 20 | 12 | 16 | 26 | 7 | 10 |

THX repairs more synthetic cases while also editing more cases that should
remain untouched. Exact three-way label agreement was D1 10/46 vs THX 19/46
for single sentences, and D1 31/46 vs THX 30/46 for two. This distinction matters:
keep and abstain have identical preservation behavior despite different labels.
The eight excluded diagnostics remain in `results.json` for inspection.

## Runtime and memory

- Published F16 file: 643,835,524 bytes, 321,908,998 total parameters. Ancillary
  action-head parameters are not used to calculate our choice logits.
- Native execution: CPU F32, release binary, Rayon threads 4, no ASR resident.
- Load/hash/weight conversion: 2.451 s; first-row warmup is separate.
- Synthetic batches of four: 2.677–3.373 s. Batch time belongs to the whole
  batch, not each row.
- Full process wall: 100.90 s for loading, validation warmup, 16 household
  decisions and 100 synthetic decisions.
- Full-run maximum RSS: 2,354,968 KiB (**2.25 GiB**). Includes conversion,
  allocator and attention working memory; this is not steady model-only RAM.

This is **not** a controlled runtime contest with saved D1 Q8 observations:
precision, kernels, build settings and machine load differ. It is not mobile
evidence. Quantization may change both quality and memory; neither was measured
for THX here. There is no deployable ONNX artifact in the inspected pinned repo.

## Validation and remaining gates

Attention layout/masking/batch-independence test passes; release build passes;
hash-verified weights load; first row is identical in standalone and padded
batch; no input truncation or non-finite probabilities occurred. All 116 rows
completed, and the native household output was scored with the existing Rust
word-edit-distance helper. Default `cargo run` still targets transcrust.

Before expanding the adapter or attributing close differences to model quality:
obtain independent upstream token/logit goldens, verify this native port, then
consider a quantized artifact if quality justifies the work. Do not tune prompts
or thresholds against this one clip and call it held-out validation. DeBERTa,
Clef and BGE experiments remain on the main taskboard.

`results.json` retains all states, option meanings, IDs/markers, choices,
probabilities and batch times. `inputs.sha256` pins the frozen comparison files;
`SHA256SUMS` pins downloaded artifacts under ignored model storage. No audio or
weights are included in this evidence directory, and nothing was sent to an API.
