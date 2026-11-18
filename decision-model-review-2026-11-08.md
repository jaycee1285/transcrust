# Dictionary decision candidates — 2026-11-08

Five READMEs fetched through Git, without weight downloads or upstream code
execution. This is a source/viability review, **not new inference results**.
John accepted the proposed triage with “Okay” on November10; it is now recorded
in TASKBOARD. GLiDE was subsequently queued alongside Pocket after John
confirmed the private non-commercial family scope on November 10.

## Inventory and proposed order

| Candidate | Fit for keep / supplied term / abstain | Proposed disposition |
|---|---|---|
| MNN-Pocket-0.3B-Dec | Small, trained single-forward pointer classifier; independent question branches | First native correctness/quality probe |
| hs-decide-1-flash | Trained single-forward typed model, 0.8B hybrid Qwen3.5 | Second; larger/new backbone work |
| joey-1 | Trained shared-prefix multi-question decisions, 2B multimodal Qwen3.5 | Later larger quality baseline |
| EmbeddingGemma 2 typed decision | Already tested; current weights and inference code unchanged | Duplicate; retain preservation failure |
| Minico M1 1105 | 350M generative LFM2, no supplied typed decision head | Separate bounded-scoring experiment, conditional on license/scope discussion |

Current competition is native UltraBERT INT4, approximately 299 MiB measured
process RSS and 232 MB packed weights, with D1 Q8 as the accepted alternative.
Preserving ordinary gamble/flow/chili/bingo/bandit matters as much as repairing
names. API results remain historical references; runtime remains offline.

## Pocket: closest small-model fit

[Saved README](tools/model-card-review-2026-11-08/cards/MNN-Pocket-0.3B-Dec.md)
· [Pinned reference loader](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Dec/blob/dfbe0aed3abd4caed175b883a4b462ed35184fad/inference.py).

Standard Qwen3 backbone: 28 layers, hidden896, GQA, RoPE, SwiGLU; trained
LoRA plus an FP32 256-wide pointer head. It runs one forward over state and
question branches. Source confirms causal branch isolation, per-branch position
resets, candidate-end gathering and temperature-scaled pointer logits. Options
within a question are not isolated from preceding options.

Published weight payload is **753,698,200 bytes total**: 723,137,248 backbone,
28,723,600 adapter and 1,837,352 head. These are file sizes, not process RAM.
The separate 173 MB MNN INT4 link is a base-model build, not this complete
decision checkpoint. Preserve the trained adapter/head when building a packed
variant and verify quantized decisions; do not substitute the base model.

The reference calls the backbone without computing vocabulary-wide LM logits.
Tied input embeddings still need storage. The MNN name does not require a C++
MNN runtime: its published Transformers/PEFT implementation can be translated
into Rust. Native Qwen3, LoRA and branch-mask support are work still to do.
There is no measured latency or preservation result here.

## Hindsight: promising trained alternative, different backbone

[Saved README](tools/model-card-review-2026-11-08/cards/hs-decide-1-flash.md)
· [Pinned readout source](https://huggingface.co/hindsight-ai/hs-decide-1-flash/blob/b9a782058dbc81ad59d9397bf2a9fed946778cb3/serving/hindsight_decision/typed.py).

Qwen3.5 text-only hybrid: 24 layers, eighteen linear-attention and six full
attention. The supplied vLLM plugin classifies answer-code LM rows at specific
colon positions, in one forward. It handles token spelling variants by taking
their maximum logit. This is not free text generation or a generic chat prompt.
The rendering and slot mapping are part of the trained model.

Published payload: **1,504,827,608 bytes**. Quantized RSS and CPU latency are
unknown. Our UltraBERT/D1 backends do not directly load this backbone; a native
hybrid path and exact formatter/readout need validation. The GPU-serving example
is a reference implementation, not evidence that CPU inference is impossible.
Multiple questions share the prompt; measure order/additional-question effects
rather than assuming Pocket-style isolation. Apache-2.0 card.

## Joey: useful larger baseline, expensive first port

[Saved README](tools/model-card-review-2026-11-08/cards/joey-1.md)
· [Pinned attention/readout configuration](https://huggingface.co/baselabs/joey-1/blob/9efbe485efbed9598dfca5294cbbe8da00e21908/config.json).

The card reports 63.30 on Jev Decision Index 0.4's public split. That does not
establish improvement on our corrections. It is trained for shared-prefix
multi-question classification, with explicit uncertainty training.

Published payload: **5,443,677,680 bytes**, including the multimodal model.
Text-only loading could omit vision tensors, but exact savings and packed RSS
are unmeasured. It has the same hybrid-attention family as Hindsight, plus
block-bidirectional full-attention masks and a slot-LM readout. The serving
patch checks prefix/cache boundaries; ordinary causal chat inference would
not reproduce its trained behavior. Its documented OneJev recipe uses CUDA;
a Rust CPU implementation is possible work, not supplied deployment parity.
Propose after smaller probes, rather than treating it as a drop-in model.

## EmbeddingGemma: confirmed duplicate

[Saved README](tools/model-card-review-2026-11-08/cards/embeddinggemma-2-ft-typed-decision.md)
· [Existing measured results](tools/pair-router-probe/schema-native-2026-10-31/README.md).

Previously tested original revision `8403ff7fddc75b6994130e581a832f875fb5bd0e`;
current revision `3d0069b4c56d7a7f505feb53bee90694b2c07845`. Git tree comparison
shows only an added evaluation-results YAML. Weights, renderer, head code and
config are unchanged. Both model LFS-pointer blobs have Git ID
`d9a0daa6a4a08b725805d87b41441057c021e7f2`.

Our native Q8 port passed 54/54 public decision-top checks, but the full-glossary
form chose the name on all32 phone cases. Removing the glossary still changed
ordinary words. Repeating the same checkpoint is not a fresh candidate. Its
FP16 overflow warning also still applies; existing native FP32-compute results
already respected that constraint.

## Minico: small, but a different experiment and license

[Saved README](tools/model-card-review-2026-11-08/cards/Minico-M1-1105.md)
· [Pinned license](https://huggingface.co/Mincofficial/Minico-M1-1105/blob/822419b44aed2111dabdbf685a21769449127107/LICENSE).

Generative LFM2.5-350M fine-tune; payload **708,984,403 bytes**. Card advertises
a roughly229 MB Q4_K_M artifact and371 tokens/s on a MacBook Air. Neither is
measured resident RAM or dictionary-decision latency on our device.

The release reports zero loops over80 mixed prompts, versus18.8% for Preview.
That is an author-reported limited test, not our validation or a guarantee.
Thinking can add40–78 median reasoning tokens and sometimes lowers benchmark
accuracy. A relevant probe would set effort explicitly to `off` and score a
fixed choice set without generating explanations. That needs a tested scoring
formulation; no trained dictionary/typed-choice head is advertised.

LFM Open License v1.0 has a commercial revenue restriction, unlike permissive
MIT/Apache terms. Personal offline research is not the flagged commercial case,
but this differs from the project's FOSS default and needs John's scope choice.

## Shared gates before integration

1. Freeze the current glossary, candidate occurrences, two-sentence scopes,
   keep/name/abstain meanings and application policy. Initials stay excluded.
2. Validate each model's native tokenization, positions, masks and readout using
   independent reference outputs before claiming port parity. Do not execute
   downloaded setup scripts or add Python to the application.
3. Reuse the32 phone decisions and92 non-diagnostic synthetic rows, scoring
   repairs and unwanted edits separately. Keep questioned initial/spelling
   diagnostics separate, and do not tune on the held-out rows.
4. Test repeated words in different roles in one take: dog Gamble versus “to
   gamble,” character flow versus water flow, food chili versus Chili. Also
   repeat identical requests, reorder options and reorder/duplicate independent
   questions. Map labels back and inspect changes; match isolation expectations
   to the model's actual contract. Duplicate aliases must not duplicate edits.
5. Measure cold load, warm one/many-occurrence batches, peak RSS with ASR
   resident, and release-to-delivery overhead. Quantize only after a valid
   reference exists; packed file sizes are not RAM measurements.

No new correction quality, repeatability, quantization or memory benchmark has
been run in this review. The order above is recorded as future work, not tested
performance or permission to promote a backend without its gates.

## Download provenance

All five exact README snapshots and hashes:
[cards and manifest](tools/model-card-review-2026-11-08/manifest.json),
[SHA256 checks](tools/model-card-review-2026-11-08/SHA256SUMS).
Git used shallow, partial, no-checkout clones in
`/tmp/transcrust-modelcards-20261108/`. Small configs/reference source were read;
weights were inspected only as Git LFS pointers. No weights, APIs, inference
servers, model setup scripts or Python execution were used.
