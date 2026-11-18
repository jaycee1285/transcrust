# Native packed-Q8 d3-lite reference probe

```sh
nix develop -c bash tools/d3-probe/run.sh
```

From transcrust: downloads the pinned public bundle/fixtures into ignored
`target/d3-lite`, builds the optional native Rust binary, checks all supported
public goldens, then evaluates both context arms of the two frozen phone
transcripts. Prints a new `/tmp/transcrust-d3.*` evidence directory. No daemon
backend, live prompt policy, shared glossary, audio, injection or APK changes.
Inference requires no network, interpreter, browser or API.

Default correction scope is `household` (32 decisions: 16 occurrences × two
context arms). Use an explicit output directory and second argument `all` to
also run the existing 100 synthetic decisions. The reference CPU path can take
several minutes; it prioritizes correctness over optimized deployment kernels.

## Reference and native implementation

- Bundle: [Jacqkues/d3-lite-decisionjs](https://huggingface.co/Jacqkues/d3-lite-decisionjs),
  revision `d4d54cab63bf0b1493cb23d0fd838fd253766251`.
- Runtime reference: [decision.js](https://github.com/Jacqkues/decision.js), commit
  `4782abfd297fe067d965ab7f587a449b7b0cad0a`; specifically `d3-engine.ts`,
  `d3-kernels.ts`, `d3-prompt.ts`, and the packed-weight/normalization kernels.
- Original model: [vllm-sr/d3-lite](https://huggingface.co/vllm-sr/d3-lite).
  This is the text-only Q8 conversion, with no vision weights.

The Rust model transliterates the reference's 24-layer hybrid backbone: 18
causal Gated DeltaNet layers and six bidirectional gated full-attention layers,
per-token RMSNorm, SiLU MLP and code readout. Norm weights already contain the
folded `1 + w`. The recurrent state is F32 and activations reproduce F16 storage
rounding. Full-attention RoPE rotates the first 64 of 256 head dimensions; GQA
maps eight query heads to two key/value heads. No autoregressive generation,
KV reuse or approximate shared-prefix mode is used.

Each request/question remains an independent sequence. Correction calls pack
all occurrences on a ragged token axis; matrix operations process the whole
axis, while recurrent states and attention ranges stay isolated per sequence.
There is one `Model::infer` call per utterance/context arm. Zeroed recurrent
state is never retained across dictations.

## Weight storage and compute precision

The pinned 802,817,536-byte bundle is verified against its published LFS hash
`47993e5bb658cef6b339e6aa4615c6988beb398179d0c81a61b918e5953592e9`.
Manifest and tokenizer hashes are also checked.

- Linear matrices stay resident as packed **Q8**, with F16 scales per 32 values.
  The file stores offset-u8 (zero at 128); the GGML Q8_0 storage used by Candle
  stores signed codes. Conversion changes that byte representation without
  requantizing or expanding the weights. Persistent matrix storage is
  **530,595,840 bytes**.
- The large embedding table stays in the packed file. Only requested rows are
  read and decoded; this saves eagerly materializing unused embedding rows.
  The OS may retain file pages in its shared cache outside process RSS.
- **One operator's weights are temporarily dequantized**, rounded to F16 as in
  the portable reference kernels, and multiplied with F32 CPU accumulation.
  The scratch dies before the next operator. There is no retained expanded
  trunk. This is not a fully fused integer or native GPU deployment benchmark.
- Auxiliary weights retain their supplied F16/F32 values. Code logits and their
  softmax use the shipped temperature 1.0; correction chooses keep/name/abstain
  by argmax, with no added confidence floor.

This path is useful for native correctness and resource investigation. Phone
GPU performance and memory require separate device implementation/measurement;
neither raw bundle size nor private RSS represents total device RAM use.

## Agreement gate

The public request fixture supplies upstream token IDs; the bundle supplies
Q8 reference probabilities. Of 63 request/question combinations, seven are
unsupported upstream, leaving **56 supported goldens**. The first string-state
formatter is also checked token-for-token (147 tokens). Real correction prompts
use the exact native system/user wrapper and fixed keep/name/abstain order.

Native readout order for yes/no is false/true, while the service goldens expose
yes/no probabilities; the probe reverses those reference vectors before
comparison. Choice and ordinal score options retain native code order. Structured
public states use their supplied token IDs, so this is not a claim that all
structured-state serialization paths are independently implemented.

Before evaluating private correction text, the **current compiled source** must
match all 56 reference top answers with maximum absolute probability drift
≤ 0.02. The actual measured drift is retained; the gate is not bit-exact parity.
Any kernel or formatter source change invalidates the saved gate signature.
Independent first-sequence versus packed-batch agreement is checked again on
the first correction batch (same top answer and drift ≤ 0.02).

Intermediate first/last-token states are retained for debugging. Their separate
reference is the original model's FP32 hidden states, so those errors include
Q8/F16 differences; they are not the Q8 probability agreement metric.

The prototype caught two porting errors before correction evaluation: the raw
loader's owned-buffer lifetime path (fixed by keeping borrowed bytes live until
the copy completes), and offset-u8 versus signed Q8 coding. An early checksum
was also taken before an asynchronous download completed; the corrected pin
matches the published LFS object. No correction result used those failed builds.

Tests cover packed signed-code conversion/scale preservation and a known
recurrent state update/decay:

```sh
nix develop -c cargo test --features d3-probe --bin d3-probe
```

## Correction fixture and evidence

Correction candidates, states and spans come from the frozen context fixture
already used for DeBERTa/BGE and the native D1 comparison. Hypotheses added for
NLI are **not** sent to d3; it receives the original typed decision question and
option meanings. D3 never sees expected answers or split/category labels.
Unknown aliases remain unoffered, initial O remains unexpanded, and reverse
UTF-8 edits use the immutable original transcript spans.

The script records artifact/source hashes, public probabilities, native tokens,
choices, batch times, reconstructed phone text and word-error scores. Each
phase runs in its own process; GNU time peak RSS includes initialization and
scratch allocations, excludes ASR, and does not account for all OS file cache.
Machine load is uncontrolled. Public validation canaries and a full correction
batch have different latency/memory profiles; report them separately.

The numbered read-aloud script is the expected reference, not independently
verified spoken ground truth. WER ignores case and punctuation; inspect ordinary
word capitalization separately. Audio/models stay out of Git. Private text
stays local and is not approved for public CI by this experiment.

Apache-2.0 original model/reference; the downloaded bundle carries LICENSE and
NOTICE. The native CPU probe remains optional research code, not a new app engine.
