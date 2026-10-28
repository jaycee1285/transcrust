# d1 text Q8 and Needle 3 — offline first shot, 2026-10-08

The deployment comparison is local d1 versus local Needle alongside NVIDIA ASR.
The family app must work offline and keep speech/text on the device. The earlier
explicitly authorized API measurements are a quality baseline only; there is
no proposed API runtime or fallback.

## Result

d1 is the more useful untuned candidate in this adapter test. On the main suite
it selects 16/20 repairs, edits 1/20 preservation cases and 7/20 ambiguous cases.
On Cars it selects 5/6 repairs, preserves every ordinary-word control and makes
no edits on the six ambiguous cases. It misses flow -> Flo in the V8 Cafe
sentence. Both deliberate privacy initials remain unchanged in both prompt arms.

Needle 3's default gating selects 4/20 main repairs and makes two preservation
edits, with no ambiguous edits. Forced classification selects 17/20 repairs,
but edits 17/20 preservation cases and all 20 ambiguous cases, and produces two
invalid multiple-call results. Forced means it must dispatch a classification;
KEEP and abstain remain available. Invalid results are never treated as edits.

These are the existing full-depth, 20-layer, 121M local Needle 3 weights (35.3 MB),
not newly downloaded weights. Its native tool interface is adapted as one
classification function with a three-value enum. This is not a verdict on
fine-tuned Needle or its best possible task-specific representation.

d1's remaining preservation error changes the number six to Cigs in "Set the
Cigs habit target to six cigarettes", with P(name)=0.919. Four underspecified
sigs sentences are changed to Cigs at probabilities above 0.93. Confidence alone
therefore does not establish safe automatic correction. The explicit editing
question worsens both candidates; d1 gives 12/20 repairs, ten preservation edits
and sixteen ambiguous edits in that arm. No production behavior was changed.

## What ran

- d1: 157 local inferences over all saved baseline arms; CPUExecutionProvider,
  ONNX Runtime 1.24.4, two threads, one warm-up.
- Needle: the same 157 inputs in both default and forced-call mode, 314 CLI
  calls; two threads, full depth, input-overflow rejection, max 128 new tokens.
  All CLI requests completed without runtime errors. Native multiple calls are
  retained as invalid classifications rather than picking the first one.
- Both reuse the saved sentence scope, question and option meanings. No gold
  answer is given to either model. Native encodings differ by model.
- Needle's question/alternatives are tool metadata, with only the sentence in
  user input. NEEDLE_TELEMETRY=0 and DO_NOT_TRACK=1 are set explicitly.
- d1 uses the inspected upstream `prompt.py`; the only formatting parameter
  change is per_option=64, retaining the full saved option descriptions. All
  states, questions, options and marker positions are checked: zero truncation.
  The native three-choice temperature is 1.3998981714248657.
- d1's two stock billing/technical control questions both produce the expected
  answer above 0.99. This supports the adapter contract, not spelling accuracy.

## Size, RAM and latency

The pinned text bundle is 411.1 MB including tokenizer/config/references. Only
`model_quantized.onnx` and its weight sidecar were downloaded, plus small
support files. No audio or vision graph is present or loaded.

d1 loaded in 0.93 s; end process RSS was 522 MiB and recorded process peak was
521 MiB (Linux counters have slightly different sampling). Median inference
was 1,065 ms: 880 ms for the main suite, 919 ms for Cars and 2,030 ms for the
longer original probe. Competing Rust builds were active on the i5-1345U laptop.
These timings are not isolated, and no mobile timing or combined ASR+adjudicator
memory measurement has been made. RAM looks workable; quality remains the gate.

Needle's own peak-RAM counter is retained in its receipts. An independent
Linux process measurement on one long-input CLI case peaked at 78.7 MiB; see
`../needle3/memory-smoke.json`. This is one representative case, not the maximum
over the entire suite. CLI timings include loading the model each time.

## Inspect or reproduce

[Offline comparison](../offline-comparison.md) is the primary table;
[d1 details](report.md) and [Needle details](../needle3/report.md) retain every
sentence and decision. d1 results include the graph/tokenizer/prompt hashes,
raw logits, calibrated probabilities and input IDs; Needle receipts retain
raw/suppressed calls, native confidence, schema, system prompt and binary/model
hashes. Thresholds are not calibrated to this correction task.

Read saved results, entirely offline:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/offline-report.py
```

Rerun local d1 inference with the downloaded bundle:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py
```

If the temporary model cache has been removed, `python3
tools/pair-router-probe/download-d1.py` fetches the pinned text-only bundle again.
The results manifest records ONNX revision
`4ebc1b97bf1477485371c79d1cf8d5e4e8eebfa0` and original prompt revision
`02b55d7076f15129e59ab3f94783f32c4b088674`. The LFM license is retained in the bundle.
