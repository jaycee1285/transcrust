# Vela3-fast ONNX decision probe

```sh
nix develop -c bash tools/vela-probe/run.sh
```

Separate native Rust/ORT research binary. Downloads the pinned 452 MB ONNX
decision-only graph into `target/vela3-fast/`, then reuses it. No Python runs,
router/span adapters, audio replay, inference API, app backend or default change.
The paired baseline requires the current cached INT4 bundle at
`target/schema-probe/ultra-q4/`. Results go to a new `/tmp` directory unless a
first output-directory argument is supplied.

The default run checks both memory modes on 32 full-glossary and 32 stripped
phone decisions each. It then runs the fast stripped model on all 132 frozen
rows (32 phone plus 100 synthetic), and replays those exact states/questions
through the current native UltraBERT INT4 implementation. A second argument
`all` instead evaluates all 132 cases in every Vela mode/input combination.

## Native format and graph contract

Ported from pinned `vela_onnx.py` and `ub_d3_common.py`, with the existing repo's
ORT/tokenizers crates. A choice is encoded as separate tokenized text segments:

`BOS [Q] instructions [O] key: description … [SEP_SCHEMA] [SEG_user] state EOS`

IDs: BOS/EOS 0; Q 49152, O 49153, SEP_SCHEMA 49155, SEG_user 49156. Choice order
is explicitly keep/name/abstain. Yes/no public checks use no/yes. The state pool
starts **at** SEG_user and ends **before** EOS; question/option/pool offsets are
absolute within the packed token axis. Position IDs reset and segment IDs
advance per independent prompt. All seven graph inputs are int64, with the
three token-axis arrays shaped `[1,T]` and the four readout arrays shaped `[K]`.
Output logits are split per question, then softmaxed in f64 without a new floor.

The exact published packing policy is retained: prompts up to 129 tokens can
share roughly 128-token passes; longer prompts run alone. Packing multiple
prompts above that per-prompt bound is refused. Inputs above 32,768 tokens are
rejected rather than truncated. This export can therefore use several graph
calls for one utterance; this is not the live app's single packed Candle batch.
Full inputs run 7–9 graph calls; stripped phone batches use 6–9.

## Memory modes and verification

Four intra-op threads, one inter-op thread, CPU provider:

- `fast`: `session.disable_quant_qdq=1`; ORT folds stored INT8 projections to
  FP32 and uses fused float kernels.
- `low`: `session.qdq_matmulnbits_accuracy_level=0`; ORT can retain/repack the
  INT8 matrices using MatMulNBits with FP32 activations. It is not dynamic INT8
  activation quantization.

The file hash and tokenizer hash must match the publisher's pinned SHA256SUMS.
Before private text, each mode checks two **rounded published example
distributions** within .02 and checks two different billing/technical prompts
packed together against their isolated outputs within 1e-5. Different prompts
matter: duplicate prompts alone would not detect cross-prompt attention leakage.
These checks are partial validation, not a full upstream token/logit parity
suite. The public fixtures in the linked DuckDB repository were examined but
belong to an older DecisionTune model, and were **not** used as Vela goldens.

The public example's compact JSON state is stored literally, preserving the
published key order. Household/synthetic states are plain strings. Evaluation
requires a successful gate bound to the same mode, native source and artifacts.
The same frozen aliases, contexts, question/option meanings and reverse UTF-8
edits are used; the NLI hypotheses in the fixture are not sent to Vela.

## Evidence and references

[Measured comparison](../pair-router-probe/vela3-onnx-2026-11-01/README.md).
`paired.json` verifies equal cases, states and questions before listing choice
changes versus current UltraBERT. Synthetic execution blocks may interleave
context arms; batch labels show the first row's arm, while row-level metadata
and per-arm metrics remain authoritative. Initial/quoted-spelling diagnostics
are separate from the 92 ordinary/ambiguous synthetic rows.

- Export: `altertable/vela3-fast-onnx-int8`, revision
  `9afe1776af402e34ade77a62d05105d0ffe50e26`.
- Upstream: `vllm-sr/vela3-fast`, reviewed revision
  `fa60e3aa41132c60bf05a84cabfdced39cb6d4f7`.
- Model SHA-256: `a40ddd8c8662e05d9432f44d8f34ec11b0078375e1b709618914f251d6d33643`.
- Tokenizer SHA-256: `38734bb17ef19f84c67b5eb5692bea19801ffd02cb5c5ffc11429e76b0242e08`.

LICENSE/NOTICE cover the translated reference runner. Models remain separate
downloads. `ultra-baseline.rs` imports the actual shared app inference files;
it does not reimplement or approximate the existing decision model.
