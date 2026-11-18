# GLiDE 0.4B — dictionary decision viability

Reviewed 2026-11-10 at Git revision
`6bda7ab91aeb36dcad6d27828a8871e548c7fc71`. README, configuration, encoder,
prompt/readout source and license inspected. **No weights downloaded, Python
executed, hosted API used or inference benchmark run.**

## Recommendation

Worth a native correction probe alongside Pocket, before the larger Hindsight
and Joey ports. **John confirmed the private family use is non-commercial on
November 10; the trial is now queued.** This is a genuinely
new candidate in our records. It is a trained 400M encoder decision model,
without image weights or autoregressive text generation. Neither the benchmark
index nor parameter count establishes better correction quality than current
UltraBERT INT4.

The Fastino Community License terms are recorded below. They are not a reason
to park the intended private offline family trial.

## What actually runs

[Saved README](tools/model-card-review-2026-11-10/cards/GLiDE-0.4B.md)
· [Pinned encoder reference](https://huggingface.co/fastino/GLiDE-0.4B/blob/6bda7ab91aeb36dcad6d27828a8871e548c7fc71/glide/encoder.py).

Ettin/ModernBERT encoder: 28 layers, hidden1024, 16 heads, intermediate2624,
local attention128 and a global layer every three layers; noncausal attention.
Its readout is a bias-free 255×1024 matrix. Every question gets an independent
state/question/options prompt; rows from all questions/requests are batched.
There is no shared-state-prefix saving across rows. This directly supports
our all-occurrences batch, but its cost still grows with occurrence count.

There is an existing Rust ModernBERT research implementation in
`tools/thx-probe/main.rs`, plus shared packed tensor/attention code. Those are
useful starting points, not an already compatible or independently validated
GLiDE backend. The current UltraBERT loader is a different model and cannot
simply load these weights.

Important implementation details:

- Two tokenizers: render the chat-format prompt using `render_tokenizer/`, then
  encode that text using the encoder tokenizer with CLS/SEP wrapping.
- Trained readout uses the final prompt token before SEP, the `(` in
  `Answer: (`. With left padding it is hidden row `-2`. Do not substitute mean
  pooling from generic classifier metadata or pool SEP itself.
- Softmax only the first N readout slots for the N offered options, then map
  those positions back to the supplied keys. The render tokenizer's code IDs
  are not encoder token IDs or direct head indices.
- Returned confidence is the margin between the top two probabilities, not
  the probability of the selected answer. Preserve the whole distribution.
- The published wrapper shortens overlong prompts by retaining head and tail.
  Our bounded adapter should refuse such input and preserve text rather than
  silently omit context. Normal two-sentence checks are far below7999 tokens.

## Memory and time

Git LFS pointers report **789,581,344 bytes encoder +522,320 bytes readout**,
or **790,103,664 bytes total** before tokenizers/config. The card specifies BF16.
Raw four-bit encoder storage would be roughly 197 MB before quantization
metadata, unquantized tensors and runtime scratch. That is an estimate, not a
released Q4 artifact or measured RSS. No claim of lower memory than UltraBERT
is justified yet; permanent FP32 expansion would erase the hoped-for saving.

The card's 6.9ms median and 1.77GB peak GPU memory were measured on an RTX PRO
6000 Blackwell, not an x86 CPU or phone. Its 51.10 Decision Index 0.3.1 score
cannot be ranked directly against Joey's reported 0.4 index score. Our frozen
dictionary cases and release-to-delivery timings decide viability.

## Repetition and batching

No generated-token loop exists in this encoder path. The card reports
deterministic isolated requests on an unchanged GPU/software stack, with small
probability changes possible when padding/batch shapes change. That is not a
native CPU repeatability test.

Use the existing 32 phone decisions and 92 ordinary synthetic cases. Check
keep/name/abstain, ordinary-word preservation, duplicate occurrences with
different meanings, single-versus-batch parity, padding-length changes and
option-order permutations. Obtain native prompt/token/readout parity against
published examples before trusting private scores. Then evaluate quantization
against that reference, retaining packed weights and measuring real CPU RSS.
No new glossary aliases or decision thresholds are part of this comparison.

## License and confirmed scope

[Pinned license](https://huggingface.co/fastino/GLiDE-0.4B/blob/6bda7ab91aeb36dcad6d27828a8871e548c7fc71/LICENSE)
· [Saved terms](tools/model-card-review-2026-11-10/LICENSE.fastino).

The Ettin base remains MIT; Fastino's contributions use its Community License.
The terms require a separate agreement for commercial production above the
stated $10M revenue threshold, commercial product attribution, and continued
license conditions on derivatives. Quantized/pruned/distilled models are
explicitly included. Evaluation/research is excluded from its production
definition, so this is a production/distribution scope issue, not a claimed
ban on our local probe. It differs from the project's FOSS default.

John's scope clarification resolves the trial decision: private, offline,
single-family use. GLiDE is queued alongside Pocket. No inference benchmark or
runtime backend change has occurred during this review.

## Provenance

Shallow partial no-checkout Git clone at
`/tmp/transcrust-modelcards-20261110/GLiDE-0.4B`. Actual weight payloads were
not fetched: only their small Git LFS pointer records were read.

- Encoder SHA256: `4540aa6d168fd15b06f7d6a61c34dd45523b5a0a1a72f98f1b2a81cf60774f34`.
- Readout SHA256: `0cb6da049081b05dc60dcf0c9f95dc4309c8849ab921f61f67f4658172b57f2a`.
- Local [snapshot checksums](tools/model-card-review-2026-11-10/SHA256SUMS).
