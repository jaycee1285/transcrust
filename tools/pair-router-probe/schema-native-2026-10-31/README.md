# UltraBERT / EmbeddingGemma native correction tests — 2026-10-31

**Keep D1 in the app. UltraBERT is worth retaining as a research candidate;
EmbeddingGemma did not meet preservation requirements in either input form.**
Both were tested in Rust using their native decision formulations. The
comparison includes 32 frozen phone decisions per model and a second 32-row
glossary-removal ablation. No production prompts, shared vocabulary, aliases,
models or phone code changed.

```sh
nix develop -c bash tools/schema-probe/run.sh
```

[Implementation, formatting, pinned artifacts and method](../../schema-probe/README.md).
The cards supplied in the repository root are the starting references. UltraBERT
is the **curation specialist**, not an untested general-decision checkpoint.
EG2 uses the pinned published Q8 export of the requested typed-decision model.

## Independent implementation checks

| Check | Result |
|---|---|
| EG2 four encoder vectors | Cosines ≥ .99999815; largest coordinate drift .00005099 |
| EG2 supported public tokenizer fixtures | Exact IDs for state, question and options |
| EG2 full public decisions | **54/54** top answers; max probability drift **.00037165** |
| UltraBERT author's public scored pages | **9/9** top answers; max top-probability drift **.00554778** |

UltraBERT checks cover three pages × topic/education/reasoning with the exact
author instructions/labels and 3,000-character cutoff. Only top labels and top
probabilities are published: this is partial output validation, not full-logit
parity. The reference is BF16 GPU; native computation is FP32 CPU. EG2 uses
published encoder vectors and full Q8 decision distributions. Its unsupported
state is skipped as in the reference; yes/no output order is adapted explicitly.

Both downloaded weight hashes match pinned Hugging Face LFS objects. Each
correction run requires a successful public gate bound to identical native
sources and artifact hashes. The initial EG2 fixture reader encountered a null,
explicitly unsupported state; that was fixed before private evaluation. No
failed reader/model run is counted as a quality result.

## Original glossary/context comparison

Same 17-entry frozen glossary, candidates, question, keep/name/abstain meanings
and argmax edit policy as D1/d3. Seven Parakeet and nine Phonon occurrences,
each with target sentence alone and original neighboring sentence. No fitted
confidence floor. Initial O is not an expansion candidate; `meter` and
`Chilean` have no new aliases. Each output is scored against the intended
79-word script, not independently verified audio ground truth.

| Model / input | Parakeet word errors: sentence / context | Phonon word errors: sentence / context |
|---|---:|---:|
| Raw phone text | 3 / 3 | 5 / 5 |
| Accepted D1 Q8, same-day context test | **2 / 3** | **4 / 4** |
| UltraBERT FP32, full glossary | 2 / 3 | 4 / 5 |
| EG2 Q8, full glossary | 4 / 4 | 5 / 5 |
| UltraBERT FP32, glossary removed | 2 / 3 | 4 / 5 |
| EG2 Q8, glossary removed | 2 / 3 | 4 / 5 |

**WER hides incorrect name capitalization.** It ignores case/punctuation, so
ordinary food chili → Chili, game bingo → Bingo and robber bandit → Bandit do
not increase these counts. Preservation tells a more useful story:

| Model / input | Wrong ordinary-word edits per transcript: sentence / context |
|---|---:|
| D1 Q8, full glossary | **0/5 / 1/5** |
| UltraBERT, full glossary | 3/5 / 4/5 |
| EG2, full glossary | 5/5 / 5/5 |
| UltraBERT, glossary removed | **0/5 / 1/5** |
| EG2, glossary removed | 2/5 / 4/5 |

The five controls are gambling verb, ordinary water flow, food chili, game
bingo and robber bandit. Both phone transcripts contain all five.

- Both models repair dog Gamble → Gambol and project Transcrust → transcrust
  in both scopes and input forms.
- UltraBERT with the glossary preserves water flow, but misclassifies food,
  game and robber words. Neighbor context also flips “to gamble” to Gambol.
  Removing the glossary fixes the three ordinary nouns without losing the dog
  repair, but **still misses character flow → Flo**. It also loses the optional
  character bandit → Bandit casing repair in the stripped input.
- EG2 with the glossary chooses **name for all 32 decisions**, including every
  ordinary-word control. Without the glossary, sentence-only preserves gamble,
  water flow and bingo, but still changes food chili and robber bandit. Context
  also changes gamble and bingo; character Flo is now missed in both scopes.
- Neither model abstains in either input form. Probabilities retain their own
  calibration; e.g. EG2's all-name result does not mean all P(name) exceeded .5.

The glossary-removal pass was selected after seeing the original failures.
It is a controlled diagnostic, not a fresh held-out evaluation. It shows that
this prompt/model combination is sensitive to unrelated proper-name entries;
it does not establish the internal cause. Native question/option descriptions
still supply the specific canonical term and meaning.

## Measured CPU resources

Four Rayon threads, separate sequential processes. UltraBERT uses original
FP32 weights; EG2 retains packed Q8 matrices and expands one operator into
FP32 scratch. All arithmetic is FP32, including EG2: its card explicitly warns
against FP16 inference. On-demand embedding row reads avoid a full expanded
embedding table for EG2. These are native reference ports, not optimized
integer/GPU kernels. Machine load is uncontrolled.

| Model / input | Weight file | Peak correction RSS | Four correction batches |
|---|---:|---:|---:|
| UltraBERT, full glossary | 1,529,215,712 B | 1,892,256 KiB ≈ **1.80 GiB** | **18.42–25.23 s** |
| EG2, full glossary | 292,851,712 B | 486,428 KiB ≈ **475 MiB** | **3.70–16.04 s** |
| UltraBERT, glossary removed | Same FP32 | 1,662,572 KiB ≈ **1.59 GiB** | **3.75–7.15 s** |
| EG2, glossary removed | Same Q8 | 365,992 KiB ≈ **357 MiB** | **1.02–5.37 s** |

EG2 caches shared question/options and **identical state texts across engine
outputs/scopes**. Its first Parakeet batch costs 16.04 s / 5.37 s respectively;
the Phonon sentence batch needs only two new text encodings. Its range is not
guaranteed new-utterance latency. UltraBERT reruns every occurrence in one
packed forward batch per utterance/scope. Model loading after hash verification
was .894 / 1.265 s for EG2 full/stripped and 1.178 / 1.739 s for UltraBERT.
These are individual observations, not stable startup benchmarks. Process wall
times include verification and writes.
File-cache memory outside RSS is excluded; no ASR model is loaded here.

EG2's memory is encouraging relative to D1's ~712 MiB context-test peak, but
its correction errors disqualify this setup. UltraBERT's FP32 memory is not a
fair estimate of a potential Q8 deployment. No Android/iOS RAM or latency was
measured, and no ONNX/export work was needed for these tests.

## Disposition / evidence

The broader synthetic surface and mobile work are deferred. A justified next
UltraBERT experiment would validate a quantized version and run the stripped
formulation on broader frozen/fresh cases, including character-name repairs.
It has not beaten D1 here; do not switch the daemon or shared glossary.

`eg2/` and `ultra/` retain public gates, all full/stripped states and probability
vectors, delivered text, scores, separate process RAM/time records and source
hashes. `no-glossary-cases.json` records the exact ablation. Model/audio files
remain outside Git. The code is a separate optional research binary; all
private inference ran locally.
