# Native paired-encoder correction test

```sh
nix develop -c bash tools/cross-encoder-probe/run.sh
```

Run from transcrust. Downloads pinned ONNX/tokenizer/config files into ignored
`target/pair-probe/`, builds the Rust `pair-probe` binary, freezes cases, and runs
DeBERTa followed by BGE through the existing ORT/tokenizers dependencies. Prints
held-out metrics and the new `/tmp/transcrust-cross-encoder.*` evidence path.
Optional first argument: output directory. Inference is offline; no interpreter,
ASR invocation, injection, shared-vocabulary write or production backend change.

Initial results: [2026-10-30 report](../pair-router-probe/cross-encoder-q8/README.md).
The first formulation was strongly biased toward glossary names; its conservative
development policy abstained on everything. This is a reproducible research
baseline, not a working replacement for D1.

## Inputs and paired formulation

`prepare` snapshots the two phone audits and the saved expanded-glossary suite:

- 32 phone decisions: Parakeet's seven and Phonon's nine occurrences, each with
  a target-sentence-only and original neighboring-sentence arm.
- 100 synthetic decisions: the 50 existing single/two-sentence scenarios.
- Two paired rows per decision: **264 paired rows per model**, plus validation.

Both candidate interpretations receive the same premise/query: the original
glossary plus the selected dictation context. Only the target occurrence is
marked. Each interpretation explicitly names that occurrence and quotes the
unmarked target sentence:

> The occurrence of "gamble" in "I like to gamble and play poker with cards."
> refers to ordinary English about gambling or betting money.

The competing interpretation ends with the candidate's supplied name meaning,
such as `Gambol, pet dog`. No expected answers, categories or split labels enter
the model. These are metalinguistic hypotheses; successful general NLI/retrieval
does not establish that a model can evaluate them. No wording was tuned after
the correction results were inspected.

Phone candidate discovery and UTF-8 replacement spans come directly from frozen
production audits. Unknown aliases remain unoffered. The sentence-focus helper
is a fixture-only punctuation splitter, not a replacement for the app's parser.
Synthetic single/two contexts and their scope-specific labels remain as supplied.
Glossaries are constant between context arms within each surface; synthetic
cases retain their historical glossary, while phone cases retain their snapshot.

## Native scoring

- DeBERTa: published `cross-encoder/nli-deberta-v3-small`, revision
  `fa2804872c3b4bd748f38c0185cc85775361e735`,
  `onnx/model_quint8_avx2.onnx` (172,503,005 bytes). Each pair returns
  contradiction/entailment/neutral logits in the checked config order.
  Softmax is applied **within each pair**; forced ranking compares the two
  entailment probabilities, even if both are weak.
- BGE: Xenova's conversion of `BAAI/bge-reranker-base`, revision
  `280bcc27a84e0b898c251e06fddb25171bd9b101`,
  `onnx/model_quantized.onnx` (279,301,077 bytes). Compare raw relevance logits;
  these are not correction probabilities.

Both use each published tokenizer's native paired special-token template,
right padding and attention masks. Segment IDs are passed only if the graph
advertises that input (the tested graphs both use IDs and attention mask only).
Truncation is disabled; inputs beyond 512 tokens and malformed/non-finite logits
are rejected. Artifact hashes are checked before loading. CPU threads: four
intra, one inter. Each phone utterance/context arm submits all its candidates in
one call (14 or 18 paired rows). Synthetic chunks contain up to eight occurrences
(16 paired rows); batch IDs and token counts are saved.

## Split and edit policy

Normalize each scenario's original single-sentence target (remove markers,
lowercase, normalize spaces), SHA-256 it, and put it in development when byte
zero modulo five is zero. All other ordinary scenarios go to evaluation.
Repeated targets and both context arms stay together, including the paired
ambiguous examples whose neighbors establish opposing meanings.

Result: **13 dev / 33 eval / 4 diagnostic cases per arm**. Initial-expansion and
quoted-spelling controls are always diagnostics, excluded from fitting and
primary metrics. Phone text is never used for threshold fitting. These are
existing development fixtures with partly contrived expectations, not a new
independent production distribution; the split only holds cases out of the
new threshold fitting.

Report forced ranking separately from a policy selected using dev labels only.
The policy minimizes unwanted dev edits, then maximizes dev repairs; an explicit
all-abstain candidate guarantees zero dev edits. NLI grids: absolute probability
margin 0/.05/.15/.3/.6 and entailment floor 0/.5/.7/.9; entailment must also be
the top native NLI class for the chosen interpretation. BGE grids: raw relevance
margin 0/.5/1/2/4/8, with no absolute relevance floor. Sub-threshold cases abstain.
No thresholds were added after inspecting held-out or phone results.

All-abstain is displayed as zero repairs, not counted as useful correction.
Raw logits/probabilities permit later analysis without hiding failed ranking
behind thresholds. Reconstructed phone text is scored against the read-aloud
reference with the existing standalone Rust word-edit-distance helper.
Case-only mistakes must also be inspected: WER ignores capitalization.

## Validation and measurement

Two unit tests check target occurrence selection and that evaluation labels
cannot fit the policy. Strong entailment and contradiction examples gate the NLI
run; a third neutral example is retained as an accuracy diagnostic. It was
initially a gate and stopped the first attempt. The published FP32 graph makes
the same error; it was not silently discarded. BGE's Mars-versus-fruit relevance
check gates its run. Original and repeated identical first-batch logits must
match within 0.00001. An isolated occurrence pair is also measured: dynamic
quantization can change logits with batch composition, so equality to a
different batch is not asserted. These checks are not a full upstream parity
suite or a quantization-accuracy study.

GNU time measures each model in a separate process, including initialization,
canaries, validation repeats and both evaluation surfaces. Batch graph times
exclude tokenization; wall time includes it. Peak RSS is a full-run high-water
mark, not model-only or phone RAM. Do not sum shared batch time once per row.
Machine load is uncontrolled; no timing is an iPhone/Android measurement.

The optional FP32 canary diagnostic uses the same pinned repo's
`onnx/model.onnx`, SHA-256
`59fd8dd78926e15907ab419303179e4196bdedd710e888773e2f0143fe430897`.
With that graph and identical tokenizer/config in `target/pair-probe/deberta-reference`:

```sh
nix develop -c target/release/pair-probe run deberta-reference target/pair-probe/deberta-reference tools/pair-router-probe/cross-encoder-q8/cases.json /tmp/deberta-reference-canaries.json
```

Functional checks:

```sh
nix develop -c cargo test --bin pair-probe
```

Models and recordings stay out of Git. Saved household text remains local,
with no external API calls or public-CI fixture promotion.
