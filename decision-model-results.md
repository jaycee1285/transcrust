# Decision models: inventory and results

Compiled **2026-11-02** from the saved experiments, measured **2026-10-07 through
2026-11-01**. This is the cross-model results index; [TASKBOARD.md](TASKBOARD.md)
remains the work/acceptance tracker. No models were rerun to write this report.

**Later card-only reviews:** [November8 candidate triage](decision-model-review-2026-11-08.md)
and [November10 GLiDE review](decision-model-review-2026-11-10-glide.md). No new
inference scores are added to this results inventory by those reviews.

Rejected/parked model download caches were removed on 2026-11-02; saved evidence
and runners remain. See [cleanup record](decision-model-cleanup.md). Their
reproduction commands download the pinned artifacts again when needed.

## Models

| Model | Versions / approach examined | Work completed | Current disposition |
|---|---|---|---|
| [UltraBERT-380M-curation](https://huggingface.co/davanstrien/ultrabert-380m-curation) | Original FP32; native weight-only Q4_0 | Public checks, phone replay, glossary ablation, quantization comparison, broader synthetic comparison, app integration | **Primary `--dec` backend**, INT4 with stripped context; new live human smoke pending |
| [LiquidAI d1-omni-600M](https://huggingface.co/LiquidAI/d1-omni-600M) | Text-only Q8 and Q4; later native Rust/ORT Q8 | Early sentence/Cars suites, context/glossary/Bluey tests, phone replay, native parity/cancellation, live dogfood | **Selectable baseline**, `--dec-backend d1`; desktop Q8 accepted by John on October 29 |
| [Vela3-fast](https://huggingface.co/vllm-sr/vela3-fast) / [ONNX INT8 export](https://huggingface.co/altertable/vela3-fast-onnx-int8) | Decision backbone only; FP32-expanded fast mode and retained-INT8 low-memory mode | Native Rust/ORT, public example/packing checks, both phone scopes, broader paired comparison against current INT4 | Preservation/repair tradeoff; larger RAM; no live integration |
| Verdict / OpenJev, ModernBERT 151M | Existing local FP16 ONNX bundle, `~/repos/dz/jev-options/rlcd-151m` | Record-2 selected spans, main/Cars suites, sentence/context and glossary comparisons | Fast, but context/glossary contamination prevents promotion |
| [Needle 3](https://github.com/cactus-compute/needle) | Existing full-depth 20-layer, 121M local bundle; default gating and forced classification | Same early 157 inputs in both tool-call modes | Tiny footprint; untuned tool adapter too conservative by default, too aggressive when forced |
| [THX-01](https://huggingface.co/doofz/THX-01) | 321.9M; published F16 weights, native CPU FP32 execution | 16 phone decisions and 100 synthetic rows | Provisional port; no household improvement; independent upstream numerical parity still open |
| [DeBERTa v3 Small NLI](https://huggingface.co/cross-encoder/nli-deberta-v3-small) | Published AVX2 Q8 ONNX; FP32 canary follow-up | Paired ordinary/name hypotheses, 132 decisions, conservative policy fitting | Tested formulation failed; not a deployed correction model |
| [BGE Reranker Base](https://huggingface.co/BAAI/bge-reranker-base) | Xenova Q8 ONNX export | Same paired-hypothesis surface and policy fitting | Name-biased ranking and batch-sensitive results; not promoted |
| [EmbeddingGemma 2 typed decision](https://huggingface.co/mrm8488/embeddinggemma-2-ft-typed-decision) | Published decision.js Q8 bundle; FP32 computation | Encoder/head reference checks, 32 phone decisions with and without global glossary | Smaller memory, but ordinary-word preservation fails |
| [d3-lite](https://huggingface.co/vllm-sr/d3-lite) | Published decision.js Q8 bundle; native Rust CPU port | Full supported public reference checks and 32 phone decisions | Ordinary-word preservation fails; broader synthetic escalation deferred |
| `typesafe/jev-1.13` | OpenRouter API, resolved snapshot recorded in receipts | Early 157-case quality baseline | Strong reference; **not an offline deployment option** |
| `cloudflare/clef-flash` | OpenRouter API, routed providers recorded in receipts | Same early 157-case quality baseline | Quality reference only; distinct from local Clef-text |
| [Laya](https://huggingface.co/convaiinnovations/laya) | Card/reference discussion | No saved correction inference benchmark found in this repo | Reviewed candidate; no measured score assigned here |
| [Clef-text-0.6b](https://huggingface.co/FluidInference/clef-text-0.6b) | Model-card/deployment-path review | Execution remains an open taskboard item | Pending local typed-head and Apple viability tests |
| [Needle 2](https://huggingface.co/Cactus-Compute/needle2) | Earlier proposed bounded-decision path | Research/taskboard reference; the actual comparison used **Needle 3** | Do not attribute Needle 3 results to Needle 2 |

## Overall result

**UltraBERT INT4 is the current practical choice for desktop testing.** It has
the smallest measured process RSS among our tested native three-way classifiers
that reached the live app: about **299 MiB** on the phone replay, with **232 MB
of weights**. Removing the global glossary table was necessary for preservation.
INT4 retained all 32 FP32 decisions in that stripped setup. This is not a claim
that it is universally the most accurate model.

**D1 Q8 remains the accepted desktop baseline.** It has stronger native-reference
validation and actual human dogfood acceptance, at roughly **701 MiB** in the
fresh phone-only measurement. Neighboring context sometimes enables a useful
name repair and sometimes corrupts an ordinary verb. D1 Q4 was tested too; it
was smaller, with mixed quality changes, rather than a universally superior
replacement for Q8.

**Vela is a credible alternative, not a clear upgrade.** It fixes the real
contextual “to gamble” error and reduces unwanted edits on the broader fixtures,
but loses some Lightning/Flo/Chili repairs that current UltraBERT makes. Its
lower-memory ONNX mode still uses roughly three times our current model's RSS.

The other local trials have not established a better deployment choice. Some
were fast or repaired more names, but also changed too many ordinary words.
The API models supply a useful quality reference; the family app's offline and
privacy requirements exclude an API runtime or fallback.

## What the numbers measure

The decision stage chooses **keep, supplied canonical name, or abstain** for a
marked occurrence. Keep and abstain both leave the text unchanged. It does not
freely rewrite a transcript. Candidate discovery, supplied aliases, descriptions,
scope, model-native formatting and the final edit policy all affect results.

There are several different test surfaces:

| Surface | Contents | Appropriate interpretation |
|---|---|---|
| Early selected-span probe | Eight chosen Record-2 repair spans plus typed examples/controls | Supplied-candidate viability, not full recording WER |
| Main 60 + Cars 18 | Repair, preservation and deliberately ambiguous cases; separate explicit-correction prompt arm | Early cross-model behavioral comparison |
| Neighbor/glossary/Bluey studies | Paired single/two-sentence scenarios and changing glossary tables | Context helps and contamination costs; repeated scenarios are correlated |
| Frozen phone replay | Seven Parakeet and nine Phonon candidates, each in two context scopes: 32 decisions | Actual phone-produced text against an intended 79-word script |
| Later frozen synthetic set | 100 rows: 50 per context arm; per arm 13 dev, 33 eval, four diagnostic | Controlled development comparison; not representative production accuracy |

The headline later synthetic comparison excludes the eight initial/quoted-word
diagnostics, leaving **92 rows: 28 wanted repairs and 64 preservation cases**.
Desired labels can change when context is supplied, so repair denominators differ
between context arms. Compare models within matching inputs and labels.

WER ignores capitalization: food chili → Chili, game bingo → Bingo and robber
bandit → Bandit can be semantic mistakes without increasing word-error counts.
The 79-word reference is the intended script, not independently checked spoken
ground truth. Missing aliases such as `meter` and `Chilean` remain unoffered;
no decision model gets credit or blame for repairing candidates it never sees.

Initial-expansion and quoted-spelling probes remain separately inspectable.
Some expectations were challenged by John; they are not automatically valid
editing-policy tests for a referent classifier. In particular, forced one-letter
initial tests bypass the actual app's candidate filter. Actual phone O survives
because the app does not offer it as a candidate.

## Early comparison: OpenJev, Needle, D1 and API references

Same main 60-case referent-question surface, no confidence floor. The columns
are separate: ambiguous name selections are not ordinary preservation errors.

| Model / policy | Wanted repairs / 20 | Edits on preservation / 20 | Edits on ambiguous / 20 |
|---|---:|---:|---:|
| OpenJev / Verdict FP16 | 20 | 6 | 8 |
| Needle 3, default gating | 4 | 2 | 0 |
| Needle 3, forced classification | 17 | 17 | 20 |
| D1 Q8 | 16 | 1 | 7 |
| D1 Q4 | 18 | 1 | 6 |
| Jev 1.13 API | 20 | 0 | 7 |
| Clef Flash API | 19 | 2 | 3 |

Needle's forced mode still allowed keep/abstain; it did not force an edit.
Default mode returned no call on 52/60 cases. Forced mode produced two invalid
multiple-call results in this arm; those were retained as invalid, not guessed
edits. These are untuned tool-schema adapters, not a verdict on fine-tuned Needle.

Jev repaired all six Cars positives with no ordinary-control edits; Clef repaired
all six with one control edit. Both API models preserved the initial controls
in these early arms. The 314 successful paid calls cost **$0.003404616** total.
Their roughly 308/325 ms median client round trips include network/provider time
and are not comparable with local inference-only timings. No new API requests
were made for this report.

OpenJev's initial sentence scope repaired 6/8 selected recording spans and
preserved 7/7 typed prose controls at about 85 ms median inference. Its later
two-sentence test repaired 8/8 names, but selected names on **8/16** ordinary/
other-person controls without the glossary and **12/16** with it. Fast inference
did not solve target isolation. D1's glossary also improved some contextual
repairs while introducing unsupported single-sentence edits.

Sources: [early offline comparison](tools/pair-router-probe/offline-comparison.md),
[D1 Q4/Q8](tools/pair-router-probe/d1-q4/SUMMARY.md),
[API reference](tools/pair-router-probe/openrouter-baseline/SUMMARY.md),
[OpenJev glossary study](tools/pair-router-probe/openjev-glossary-report.md),
[Bluey expansion](tools/pair-router-probe/bluey-report.md).

## Later phone replay: corrections and preservation

Each cell is **sentence-only / neighboring-context** word errors out of 79.
These results span different dated runs; the linked reports preserve exact
builds, inputs, quantizations and timings.

| Model / formulation | Parakeet errors | Phonon errors |
|---|---:|---:|
| Raw phone output | 3 / 3 | 5 / 5 |
| Native D1 Q8, full glossary | 2 / 3 | 4 / 4 |
| Current UltraBERT INT4, stripped | 2 / 3 | 4 / 5 |
| Vela, stripped, either ONNX memory mode | **2 / 2** | **4 / 4** |
| Vela, full glossary | 2 / 3 | 4 / 5 |
| EmbeddingGemma Q8, full glossary | 4 / 4 | 5 / 5 |
| EmbeddingGemma Q8, stripped | 2 / 3 | 4 / 5 |
| d3-lite Q8, full glossary | 3 / 4 | 4 / 5 |
| DeBERTa, forced paired ranking | 4 / 4 | 5 / 5 |
| BGE, forced paired ranking | 3 / 4 | 5 / 5 |

THX tested the neighboring-context arm only: **3 Parakeet / 5 Phonon errors**,
versus the saved D1 comparison's 3 / 4. Its numerical port remains provisional.

The five ordinary controls per transcript are gambling verb, water flow, food
chili, game bingo and robber bandit. **Wrong name edits**, per transcript:

| Model / formulation | Sentence only / five | With context / five |
|---|---:|---:|
| D1 Q8, full glossary | 0 | 1 |
| UltraBERT FP32 or INT4, stripped | 0 | 1 |
| UltraBERT FP32, full glossary | 3 | 4 |
| Vela, stripped | **0** | **0** |
| Vela, full glossary | 1 | 2 |
| EmbeddingGemma Q8, full glossary | 5 | 5 |
| EmbeddingGemma Q8, stripped | 2 | 4 |
| d3-lite Q8, full glossary | 4 | 5 |

The best practical prompt change was **omitting the global glossary while
retaining the relevant term's answer description**. This is the current
UltraBERT setup. All occurrences still use their marked sentence and a neighbor;
there is no cross-dictation history. Candidate discovery continues to reload
the sole shared `~/syncthing/john-vocab.md` each utterance.

Vela keeps the gambling verb with neighboring dog-name context; current
UltraBERT and D1 wrongly change it. D1's neighbor also enables the phone Flo
repair that its sentence-only arm misses. Both stripped UltraBERT and Vela miss
that phone Flo repair. Thus a lower total WER can hide different repair and
preservation behavior; globally dropping context is not a settled solution.

Sources: [native D1 context study](tools/pair-router-probe/d1-native-context/README.md),
[UltraBERT / EmbeddingGemma](tools/pair-router-probe/schema-native-2026-10-31/README.md),
[INT4 comparison](tools/pair-router-probe/ultra-int4-2026-11-01/README.md),
[Vela](tools/pair-router-probe/vela3-onnx-2026-11-01/README.md),
[d3-lite](tools/pair-router-probe/d3-native-q8/README.md).

## Broader synthetic results

The most direct current-model comparison feeds **identical stripped states,
questions and option meanings** to each model's native formatter. The Ultra
baseline imports the actual app inference code. Excluding diagnostics:

| Context / cases | Ultra INT4 repairs | Vela repairs | Ultra unwanted edits | Vela unwanted edits |
|---|---:|---:|---:|---:|
| Sentence only, 46 rows | 8/8 | 8/8 | 4/38 | **3/38** |
| Neighboring context, 46 rows | **20/20** | 17/20 | 8/26 | **5/26** |
| Both arms, 92 rows | **28/28** | 25/28 | 12/64 | **8/64** |

Vela loses contextual Lightning McQueen, Flo and Chili repairs that Ultra makes.
It also avoids some unwanted mother/project/Bluey edits, while adding Mater
errors. It is a coverage/preservation tradeoff, not a clean win or loss.

On the 33 held-out `eval` rows **with context**, Ultra repairs **16/16** and
Vela **13/16**, both with **3/17** unwanted edits. Development rows account for
some of Vela's overall preservation advantage; no new policy was fitted here.

Other later trials used the full glossary or different scoring formulations:

| Model / formulation | Context eval repairs / 16 | Unwanted edits / 17 | Qualification |
|---|---:|---:|---|
| Native D1 Q8, full glossary | 10 | 3 | Native keep/name/abstain; no fitted floor |
| DeBERTa Q8, forced higher entailment | 13 | 12 | Paired hypotheses; no native correction abstention |
| BGE Q8, forced higher relevance | 16 | 15 | Relevance is not correction confidence; batch-sensitive |

The dev-fitted conservative policy for both DeBERTa and BGE selected
**all-abstain**: zero unwanted edits, but also zero repairs. That is not a
working correction replacement. BGE also flipped the dog decision on identical
inputs across different batch compositions; export/batch parity remains open.
DeBERTa's neutral canary failed in the publisher's FP32 graph as well as Q8.

THX's full-glossary, 92 non-diagnostic rows repaired **7/8** sentence-only and
**16/20** with context, but made **20/38** and **10/26** unwanted edits. More
repairs came with more false edits; independent upstream port parity is still
needed before treating close differences as model-quality evidence.

EmbeddingGemma and d3-lite were not escalated to the broader synthetic surface
after phone preservation failures. This absence is not a zero score.

Sources: [Vela/current-Ultra paired report](tools/pair-router-probe/vela3-onnx-2026-11-01/README.md),
[paired NLI/reranker report](tools/pair-router-probe/cross-encoder-q8/README.md),
[THX report](tools/pair-router-probe/thx-01-native/README.md).

## RAM, weights and time to typing

These are **observed desktop CPU measurements**, not a controlled universal
leaderboard. Precision, drivers, prompts, batching, allocator and machine load
differ. Most later figures are peak process RSS with no ASR resident; file-cache
memory outside RSS is excluded. Neither weight-file size nor a model's parameter
count specifies its runtime footprint. No phone RAM/latency claim follows.

| Model / tested configuration | Observed memory | Observed latency and measurement unit |
|---|---:|---|
| UltraBERT INT4, stripped | **299 MiB** phone-only; **304 MiB** fresh 132-row baseline; weights **232 MB** | **3.21–5.41 s per 7–9-occurrence phone batch** |
| UltraBERT FP32, stripped | **1.59 GiB** peak; weights 1.53 GB | 3.12–4.55 s per phone batch in paired INT4 run |
| Native D1 Q8 | **701 MiB** fresh phone-only; ~712 MiB in 132-row context study | 10.21–14.49 s per phone batch in fresh comparison |
| Historical D1 Q4 | **~333 MiB** process peak; graph/weights 244.7 MB | ~330 ms median per early candidate, not a later phone batch |
| Vela fast, stripped | **2.06 GiB** peak; ONNX file 452 MB | **1.83–2.89 s per phone batch** |
| Vela low-memory, stripped | **~861 MiB** peak | **3.11–3.90 s per phone batch** |
| OpenJev FP16, historical glossary run | **649–653 MiB** end RSS; **849 MiB** startup/process peak | 94 ms no-glossary / 169 ms glossary median per two-sentence candidate |
| Needle 3, historical CLI | **78.7 MB runtime-reported counter**, not comparable process RSS; model 35.3 MB | ~783/821 ms median default/forced CLI round trip, including reload each call |
| THX CPU FP32 | **2.25 GiB** full-run peak; published F16 weights 643.8 MB | 9.23–12.34 s per phone batch |
| DeBERTa Q8 AVX2 | **1.15 GiB** full-run peak; graph 172.5 MB | 1.28–1.97 s per phone batch, two hypothesis rows per candidate |
| BGE Q8 | **1.03 GiB** full-run peak; graph 279.3 MB | 2.48–3.49 s per phone batch, two hypothesis rows per candidate |
| EmbeddingGemma Q8, stripped | **357 MiB** peak; packed weights 292.9 MB | 1.02–5.37 s per phone batch, with shared and identical-state caches |
| d3-lite Q8, native CPU reference | **1.07 GiB** phone peak; complete bundle 802.8 MB | 33.7–44.1 s per phone batch; correctness-first CPU implementation |

Important qualifications:

- Ultra's INT4 weights stay packed; one operator expands into FP32 scratch,
  and embedding rows are decoded on demand. Norms/head and activations stay
  FP32. INT4 reduced memory, **not compute time** versus the same FP32 prompt.
  Its 57% RSS reduction versus D1 also uses the stripped prompt configuration.
- Vela's default INT8 file expands to FP32 at load. The lower-memory mode
  retains INT8 projections with float activations. Both modes gave the same
  64 phone decisions; fast/low probability drift was at most about 1.25e-6.
- EmbeddingGemma caches questions/options **and identical states across engine
  outputs/scopes**. Its fastest later batch is not guaranteed new-utterance
  latency. Its card forbids FP16 inference; computation here stayed FP32.
- Early D1/OpenJev measurements include their historical research processes;
  those scripts are not app runtime dependencies. Later probes and both live
  backends are native Rust. No inference script was executed for this write-up.
- Adding ASR, capture, injection and UI changes the end-to-end footprint and
  wait. The measured model time is only one part of time to typing.

## Validation strength and deployment status

| Model / path | Evidence obtained | Remaining qualification |
|---|---|---|
| D1 native Q8 | Token/marker and saved/single/batched probability comparisons; timeout/recovery; human desktop acceptance | Preservation and mobile measurements remain open |
| Ultra FP32 / INT4 | 9/9 public FP32 answers; packed numeric tests; stripped INT4 32/32 FP32 choices; promoted app matches all 32 saved Q4 distributions; in-flight cancellation and release/CLI checks | Quantized probabilities drift; full-glossary Q4 changes five choices; new live human smoke pending |
| Vela ONNX | Artifact checksums; two rounded published example distributions; distinct packed/isolated prompts agree to ~1e-9; both memory modes compared | Partial validation, not a complete independent upstream parity suite |
| EmbeddingGemma | Four encoder vectors, exact supported tokenizer fixtures, **54/54** public decision tops | Good implementation agreement did not yield good correction preservation |
| d3-lite | **56/56** supported public tops; max probability drift .001142; wrong water-flow choice reproduced in isolation | Good reference agreement did not solve task accuracy |
| THX | Hashes, layout/masking checks and first-row batch consistency | **Independent upstream numerical parity still missing**; provisional results |
| DeBERTa / BGE | Native published tokenizers, artifact checks, finite outputs, repeats/canaries | No complete FP32/export parity; BGE batch rank flip unresolved |
| Needle / API historical arms | Native tool/API contracts and full saved receipts | Untuned adapters; invalid calls/no-calls tracked; confidence is not interchangeable across models |

The app now uses **UltraBERT INT4 with `--dec`**, at John's explicit request.
It loads and warms at startup, keeps candidate/edit/reload/audit behavior shared,
and retains D1 through `--dec-backend d1`. No API fallback is planned. The Apple
review-before-paste contract is unchanged, and mopvox has no decision stage
integrated by these experiments.

The next useful evidence is fresh, representative dictation and the new live
Ultra smoke, followed by device RAM/latency. Further Vela quantization would
need a new paired test: the export author's reported 94.6% four-bit answer
agreement is not proof of safe correction fidelity. Laya, local Clef-text and
Needle 2 have no correction scores to rank here.

Current launch/smoke contract: [design-dec.md](design-dec.md) and
[Smoke-Human-transcrust.md](Smoke-Human-transcrust.md). Earlier chronology:
[decision-experiments.md](decision-experiments.md). Detailed reports linked
above retain model revisions, source/artifact hashes, raw inputs/outputs and
resource logs. Model weights and source audio stay outside Git.
