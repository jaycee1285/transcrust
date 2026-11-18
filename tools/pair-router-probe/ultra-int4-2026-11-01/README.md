# UltraBERT weight-only INT4 — 2026-11-01

**The RAM saving is real. With the glossary removed, INT4 matches all 32 FP32
phone decisions and peaks at 299 MiB, versus 701 MiB for a fresh D1 Q8 run.**
This preserves UltraBERT's existing behavior; it does not make that behavior
better than D1 or integrate a new backend into the app.

```sh
nix develop -c bash tools/schema-probe/int4.sh
```

The runner reuses cached pinned FP32 weights, exports the INT4 artifact if
absent, validates the current FP32 port, and runs both precisions over full and
stripped inputs. It also measures unchanged production D1 on the 32 original
phone cases. [Native implementation and gate semantics](../../schema-probe/README.md).

## Weights and implementation

- Native Rust exporter: one source tensor at a time; original source SHA-256
  checked before conversion. No Python, training or calibration corpus.
- Encoder matrices and embeddings: GGML **Q4_0**, 32 weights / 18 bytes including
  the scale. Norms, biases and the small projectors/decision head retain FP32.
  Activations and computation remain FP32: this is weight-only INT4.
- Weight file: **231,562,952 bytes**, about 232 MB / 221 MiB, down from
  1,529,215,712 bytes. SHA-256:
  `75ffd6cdf6c147ecaceab1878e388701d4feadcf69bfc8bd88a59cbd69ab8869`.
- Runtime retains packed matrices, expands one operator into scratch, and
  decodes only requested embedding rows with within-request deduplication.
  It never expands the complete encoder or embedding table into FP32.
- Conversion took 4.29 s, peak 429,676 KiB. Export-time memory is separate from
  inference. Deployment needs only the packed artifact, manifest and tokenizer.
- Numeric unit test passes: packed row slicing/decoding equals the quantizer's
  full-tensor decode, with a bounded error check against a known FP32 input.

## Quality against FP32

Same 17-entry frozen glossary, marked occurrences, question, descriptions,
candidate spans, context arms and argmax policy. The stripped pass removes only
the global glossary table; answer descriptions still identify the specific
canonical term. No shared vocabulary or aliases changed.

| Surface | Same top decisions | Largest probability drift versus native FP32 |
|---|---:|---:|
| Public page checks | **9/9** | .073159 |
| Full-glossary phone cases | **27/32** | .430412 |
| Stripped phone cases | **32/32** | .140899 |

FP32 again matched all nine author's public top answers and passed the original
.02 top-probability tolerance. Q4 retained those answers but failed that strict
probability tolerance; this is **not numerical probability parity**. Q4 private
replay was authorized by the current verified FP32 implementation, matching
tokenizer/source and finite quantized outputs. Its `passed` field means that
experimental replay is authorized; `public_quality_pass: false` records the
failed strict probability criterion explicitly.

All five full-glossary changes occur with neighboring context: Q4 loses the
dog Gamble → Gambol repair on both engine outputs, correctly keeps the gambling
verb on both, and correctly keeps ordinary bingo on Parakeet. Those are mixed
changes, not five regressions or a model improvement. Total word errors happen
to stay unchanged because missed dog repairs trade against preserved verbs.

With stripped input, **all delivered text and all 32 decisions match FP32**.
The dog repair survives both scopes. Existing limitations remain: character
flow → Flo and character bandit → Bandit are missed; neighboring context still
causes the wrong “to gamble” → Gambol edit. Sentence-only preserves all five
ordinary controls; neighboring context wrongly edits one of five, per engine.
No Q4 threshold or prompt was fitted after these results.

| Model / input | Parakeet word errors: sentence / context | Phonon word errors: sentence / context |
|---|---:|---:|
| D1 Q8, original glossary | 2 / 3 | 4 / 4 |
| UltraBERT FP32, either input form | 2 / 3 | 4 / 5 |
| UltraBERT Q4, either input form | 2 / 3 | 4 / 5 |

Scores use the intended 79-word script, not independently verified spoken
ground truth. WER ignores casing and hides several proper/common-noun mistakes.
The 32 rows include shared texts across two engines and two context scopes;
they are not 32 independent recordings. This is quantization evidence on the
existing small surface, not broad validation for the family apps.

## Peak RAM and latency

Separate sequential native CPU processes, four threads, no ASR loaded. RSS
excludes file-cache memory outside the process. D1 uses the original glossary;
both Ultra input forms are shown so the input-length benefit is explicit.

| Configuration | Peak RSS | Per-utterance batch time, 7–9 occurrences |
|---|---:|---:|
| D1 Q8, original glossary | **717,568 KiB ≈ 701 MiB** | 10.21–14.49 s |
| Ultra FP32, full glossary | 1,892,424 KiB ≈ 1.80 GiB | 17.27–22.99 s |
| Ultra Q4, full glossary | **529,212 KiB ≈ 517 MiB** | 17.72–23.13 s |
| Ultra FP32, stripped | 1,662,764 KiB ≈ 1.59 GiB | 3.12–4.55 s |
| Ultra Q4, stripped | **306,132 KiB ≈ 299 MiB** | 3.21–5.41 s |

Q4 uses **57.3% less peak RSS than D1 with stripped input**, and **26.2% less
even with the full glossary**. The latter is not the preferred quality setup.
Removing the glossary reduces computation; quantization itself did not speed
inference relative to FP32 with the same input. Packed integer/GPU kernels
were not implemented. These are individual desktop observations under
uncontrolled background load, not phone latency or RAM measurements.

## Disposition and evidence

Keep the Q4 stripped formulation for broader/fresh tests before considering
desktop dogfood or device integration. D1 remains the live backend. Probability
drift and the full-glossary dog regression remain visible rather than being
hidden behind equal WER.

`fp32/`, `q4/`, and `d1/` contain raw distributions, contexts, delivered text,
scores and independent process resource records. `*-comparison.json` lists
every quantization decision change; `resources.json` calculates the RSS
reductions. `quantization.json` pins the packed artifact, while
`provenance.sha256` pins the Rust source, harness, fixtures and Cargo lockfile.
Model/audio files stay outside Git. No inference text was sent to a service.
