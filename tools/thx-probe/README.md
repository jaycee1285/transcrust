# THX-01 native choice probe

Optional research binary in the transcrust codebase. The daemon and `--dec`
remain on D1. Run from the repository:

```sh
nix develop -c bash tools/thx-probe/run.sh
```

The script installs pinned test weights under ignored `target/thx-01`, builds
the optional release binary, and prints the evidence directory and corrected
phone transcripts. First run needs downloads; inference itself is offline.
`THX_MODEL_DIR` overrides the weights directory; the first script argument is
an output directory. `RAYON_NUM_THREADS` defaults to 4.

## Contract and scope

Upstream [doofz/THX-01](https://huggingface.co/doofz/THX-01), revision
`b323afcef149f9288a9d0daef4badd67b9b87c61`, publishes Safetensors but no ONNX
graph. This probe loads those weights with native Rust `candle-core` and
implements its ModernBERT encoder and choice head, following:

- `thx01/common.py`: `build_sequence`, `DecisionModel.forward`.
- `thx01/agent.py`: automatic head budget and `choice:3-5` temperature.
- Transformers ModernBERT: pre-normalization, alternating full/local attention,
  half-rotation RoPE, GeGLU and final norm. Local attention includes positions
  within 64 tokens on either side; full attention every third layer.

Weights, tokenizer and configs are SHA-256 checked before loading. The published
F16 weights become F32 on CPU; **this is not a quantized memory or speed test**.
The optional dependency already existed transitively in the lockfile; default
desktop builds do not enable this feature. No interpreter, export, remote
inference or media encoder is involved.

The 22-layer encoder is followed by choice-type embedding and two pre-norm
TransformerEncoderLayers (ReLU feed-forward), then LayerNorm/Linear/GELU/Linear
scoring at option markers. The ancillary action-confidence head does not affect
choice logits and is not evaluated. Probabilities use temperature 2.55, then
argmax in keep/name/abstain order. No confidence floor is added.

Tokenizer CLS is `<bos>` token 2; the encoder config's `cls_token_id=1` is not
the formatter authority. Both THX and D1 include option labels, using their
different native sequence layouts. Both receive the same semantic question,
descriptions and text. Any instruction, option or state truncation is rejected.
The full upstream library's numbers/excerpts/citations/tournaments are out of
scope; this probe supports exactly the three choices used by correction.

## Evaluation

The two frozen native D1 household audits supply the exact production candidate
spans, glossary, two-sentence states and questions. One batch per utterance;
reverse UTF-8 replacement uses the original spans. Missing aliases remain
missing, and shared vocabulary is never modified. A first-row standalone run
warms the model and checks padded-batch probability parity (tolerance 0.0002).

The 100 saved expanded-glossary D1 rows supply single/two-sentence synthetic
states and expected choices, using bounded batches of four. Those use their
historical glossary, not the newer household snapshot. Keep initial-expansion
and quoted-spelling stress diagnostics separate from ordinary-use comparisons.
These are correlated and partly contrived development fixtures, not a held-out
production accuracy estimate.

Results retain each model input, encoded IDs/markers, probabilities, expected
labels where available, D1 choices and graph-batch time. GNU time records
standalone maximum RSS and total wall time, including loading, warmup and both
evaluation surfaces, with no ASR loaded. Shared batch times must not be summed
once per row. Saved D1 timings came from different runtimes/builds and machine
load; they are not a controlled speed comparison.

Validation includes an attention test with explicit expected values for packed
QKV layout, masking and independent batch rows. The installed model additionally
checks single versus padded-batch probabilities. **Neither substitutes for
independent numerical parity against upstream logits**, which remains a gate
before promoting this custom native adapter to production or treating close
quality differences as conclusive. This pass does not run upstream Python.

Functional check:

```sh
nix develop -c cargo test --features thx-probe --bin thx-probe
```

License: upstream weights/code Apache-2.0; Candle MIT/Apache-2.0. License and
NOTICE accompany the downloaded local weights. Household text stays local;
no audio or weights belong in Git or public CI.
