# Native UltraBERT / EmbeddingGemma decision probes

```sh
nix develop -c bash tools/schema-probe/run.sh
```

Optional first argument `eg2` or `ultra` selects one model; optional second
argument chooses the evidence directory (default: a new directory in `/tmp`).
First run downloads about 1.9 GB. UltraBERT references are cached under
`~/.cache/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d`.
`TRANSCRUST_ULTRA_REFERENCE` overrides that location. EG2 uses a revision-named
directory under `~/.cache/transcrust/model-trials/`. Evidence and the generated
scorer stay in the selected output directory.
The runner includes a separate controlled replay without the glossary table.
A Cargo feature builds a separate Rust/Candle CPU research binary. The original
experiments did not change live defaults. On 2026-11-01 John explicitly requested
UltraBERT INT4 as the primary `--dec` backend: the shared inference implementation
now lives in `crates/decision/src/ultra.rs` and `decision_tensor.rs`. D1 remains selectable
with `--dec-backend d1`; the shared glossary, ASR and phone app are unchanged.

### UltraBERT INT4 experiment

```sh
nix develop -c bash tools/schema-probe/int4.sh
```

Requires the cached pinned FP32 UltraBERT model from the original runner.
The native Rust exporter streams one source tensor at a time into
`~/.local/share/transcrust/models/ultrabert-380m-curation-q4/` (or
`TRANSCRUST_DEC_MODEL`): encoder matrices and embeddings use GGML Q4_0
(32 weights in 18 bytes, including the scale); norms, biases and the decision
projectors/head retain FP32. The artifact is weight-only INT4 with FP32
activations/arithmetic. Runtime keeps matrices packed and dequantizes one
operator into scratch. It reads and decodes only requested embedding rows,
deduplicated within the request, rather than expanding the embedding table.

The source Safetensors hash is checked before export; output size/hash and all
tensor descriptors are recorded in `ultra-q4.json`. Runtime verifies the packed
hash/size. A numeric unit test checks packed row slicing/decoding against the
quantizer's own full-tensor decode and bounds quantization error on known data.

The runner first reruns the FP32 independent public checks with the current
native source. Q4 public output recording requires that successful FP32 gate,
matching source/tokenizer, and all nine finite outputs. Its `passed` field
authorizes a quantization experiment from that validated port; it **does not
declare quantization quality acceptable**. `public_quality_pass`, `agrees`,
and every distribution separately record the original stricter public-output
criteria. This allows measurement of real INT4 degradation without pretending
quantized probabilities should match FP32 within .02.

Both precisions then run the identical 32 full-glossary and 32 stripped phone
inputs. Side-by-side distributions and decision changes are saved in
`{full,stripped}-comparison.json`; each process has its own peak RSS/wall time.
The runner also measures the unchanged D1 driver on the original 32 phone cases.
The experiment itself does not select a backend or deploy to mobile. Live
selection is documented in `design-dec.md`; this runner and the daemon share
the inference code. [Measured Q4 results](../pair-router-probe/ultra-int4-2026-11-01/README.md).

## Model-specific input and arithmetic

- **UltraBERT:** original FP32 Safetensors, 24 bidirectional layers. Every third
  layer attends globally; other layers have a ±128-token window. Label-marker
  hidden states and the first token feed the released projectors and MLP.
  Exactly `BOS <<LABEL>>description…<<SEP>>instructionsSTATE`, without inserted
  whitespace. Softmax over released logits. All occurrence requests from one
  utterance run on one packed token axis, with independent attention ranges.
- **EmbeddingGemma 2:** released decision.js Q8 bundle, 24 bidirectional layers
  including per-layer embeddings and the original typed decision head. State,
  question and each option are independent texts. Each is prefixed with
  `task: classification | query: ` and wrapped in BOS/EOS. Mean pooling,
  projection and normalization produce 768-dimensional vectors. The native
  head combines state/question/option interactions, cosine prior and the stored
  temperature. Question and option texts are cached across occurrences; unseen
  texts run in packs of up to eight, each with independent attention ranges.
  The model card forbids FP16 inference: **all arithmetic here is FP32**;
  published FP16 scale/auxiliary storage is converted to FP32.

Packed matrices stay Q8 while resident. A single operator is expanded to FP32
scratch for Candle CPU matmul; embedding rows are read from the packed file on
demand. This is a correctness-first CPU port, not an optimized integer/GPU
implementation or a phone timing estimate. UltraBERT is FP32 and EG2 is Q8;
their memory measurements are not an equal-precision comparison.

## Independent checks before private text

EG2 checks four published encoder vectors, exact token IDs for the supported
public state/question/option strings, and all 54 supported public decisions.
Noul reference service outputs `[yes,no]` are reversed to native `[false,true]`.
The gate requires encoder cosine ≥ .999, all 54 top answers, and largest
probability difference < .02. The explicitly unsupported state is skipped.

UltraBERT checks nine public classifications (topic, education and reasoning
on three pages) against the author's scored `fineweb-curated-1m` rows. Text is
cut to 3,000 Unicode characters, with the exact label lists and instructions
from `curate.py`. The gate requires all nine top answers and top-probability
drift < .02. These are partial published-output checks, not full-logit parity;
the reference used BF16 GPU math while the probe uses FP32 CPU math.

Correction evaluation refuses a missing/failed gate or a different native
source/artifact signature. Original weight SHA-256 values are checked against
the pinned Hugging Face LFS objects. No interpreter is launched and no private
text/audio is sent to an inference service.

## Frozen correction surface

The same 32 phone occurrence decisions used for D1/d3: seven Parakeet and nine
Phonon candidates, each with sentence-only and original neighboring context.
Same glossary, marked spans, keep/name/abstain descriptions, argmax policy and
reverse-span edit application. No new aliases, prompt tuning or fitted floor.
Phone candidates' saved questions supply meanings; NLI hypotheses are not
passed to these typed models. UTF-8 occurrence spans are checked before edits.

WER uses the existing 79-word intended script and ignores casing. Inspect
ordinary chili/bingo/bandit separately: incorrect capitalization can be a real
semantic mistake that WER does not count. Initial O and absent aliases remain
untouched by candidate selection. The broader synthetic surface is a separate
escalation after this household preservation gate.

The second pass removes only `Vocabulary glossary: … Dictation:` from each
state, leaving the marked sentence/context, question and answer descriptions
unchanged. `no-glossary-cases.json` records this derived input. This is an
explicit ablation selected after the first results, not a blind held-out win or
a proposed live prompt change. EG2 caches all encoded texts across the four
batches, including identical states in the two engine outputs; its faster
later batches can reuse those states as well as the question/options. Do not
report that range as guaranteed cold-input latency.

## Pinned references

- UltraBERT model/scripts: `davanstrien/ultrabert-380m-curation`, revision
  `c6a1fa055d36aeb3349b9a54537a5bc3f585d77d`.
- EG2 original model: `mrm8488/embeddinggemma-2-ft-typed-decision`, revision
  `8403ff7fddc75b6994130e581a832f875fb5bd0e`.
- EG2 packed bundle: `Jacqkues/embeddinggemma-2-typed-decision-decisionjs`,
  revision `e7a9e1fe811e00be92484ae463f78909b0981c2c`.
- decision.js native math/format fixtures: commit
  `4782abfd297fe067d965ab7f587a449b7b0cad0a`, `src/eg2-{engine,kernels,head,prompt}.ts`
  and `test/fixtures/{eg2-prompts,prompts}.json`.
- UltraBERT public fixture: first three `scored/train` rows of
  `davanstrien/fineweb-curated-1m`, frozen here with their published outputs.

See LICENSE/NOTICE for the translated runtime reference. Model weights are
separate downloads with their own upstream terms.
