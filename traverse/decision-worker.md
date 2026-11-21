---
id: decision-worker
kind: service
authority:
  - crates/decision/src/lib.rs
mutates:
  - resident-decision-model-availability
  - inference-cancellation-flag
  - transcript-vocabulary-edits
observes:
  - cleaned-transcript
  - selected-vocabulary-file
  - model-bound-candidates
  - caller-cancellation
persists_to: []
depends_on:
  - crates/core/src/correction.rs
  - crates/decision/src/ultra.rs
  - crates/decision/src/d1.rs
staleness_risks:
  - native-library-packaged-in-apk
  - disabled-resident-decision-instance
  - conflated-asr-and-decision-status
entrypoints:
  - crates/decision/src/lib.rs
  - crates/ffi/src/lib.rs
  - src/decision.rs
---

# Decision worker

## Known footgun — fixed 15-second deadline

`Decision::request` wraps the entire batch of model-bound occurrences in
`tokio::time::timeout(Duration::from_secs(15), task)`. The deadline includes
waiting for the worker mutex and model inference. Its budget stays fixed as
the number of occurrences and the length of the take grow.

On expiry, it terminates ORT work, sets the cooperative inference-stop flag,
and marks the decision instance disabled. Later requests on that instance
report model unavailability. Recreating the decision instance restores model
startup. Preloading the weights moves startup earlier in capture; inference
still has this deadline.

`adjudicate_with_cancel` retains deterministic edits and leaves unresolved
occurrences as heard. Its audit contains the deadline error. Hosts must keep
that text available for review and expose the warning.

November 19 phone evidence is retained at
`~/syncthing/mopvox/decision-test/results/f8f081d3-8da7-4517-aef8-a6eaa50c811a-nemotron.json`.
The 64.419876 s take streamed through 58 Nemotron chunks, 806 encoder frames
and 342 ASR tokens. Route inspection with the selected Pixel glossary found
25 vocabulary candidates, including 17 model-bound occurrences. The phone
reported 18.7 s wait and the audit warning
`15s decision budget exceeded; native inference terminated until restart`.

Treat the deadline as an open implementation defect for sustained dictation.
When implementing in-flight work, check how this policy affects each work
item, the complete parent utterance and recovery after a failed item. Verify
fallback text, model availability and explicit discard independently.

## Fused-name split (November 20)

`split.rs` runs first inside `adjudicate_with_cancel`, before candidate
matching. A capitalized token that is not a CMU headword and starts with a
single-word capitalized glossary name is split when the remainder is a
closed-class word (tried first), names or words (`Nelland` → `Nell and`).
It is deterministic, with no model and no score gate. The audit keeps `input` as
received, adds `split_text` and `splits`, and candidate offsets refer to
`split_text`. Hosts read no candidate offsets (checked in mopvox and src/).
Learned aliases are filed by `vocabulary::alias_field`: a non-word goes to `distinct:`,
a real word (or inflection) to `homophones:`, a phrase to `heard:`. Rule 0 in
`chain::decide` keeps mid-sentence capitals that match the dictionary.
The headword list is embedded (`crates/core/data/cmu-words.txt` via `transcrust_core::lexicon`, ~1 MB,
BSD-2). The LDC-derived 20k list is deliberately not shipped. Evidence:
`~/syncthing/transcrust-work/window-study/prefix-trial.md`. Pipeline tests: `nelland_take_*`.

## Authority and mutation

The decision crate owns candidate matching, current-sentence context,
deterministic routing, inference and vocabulary edits. Its worker owns the
loaded backend and availability state. Backend tensor/token limits also apply
to a batch and have their own error messages.

The FFI owns native handle lifetime and explicit cancellation. November 19
separated explicit `tc_cancel` from the worker's inference-stop flag. Before
that fix, a deadline set the response's `cancelled` flag and Android discarded
the retained transcript. A regression test checks fallback delivery, explicit
cancellation and reset on the next request.

The host owns microphone capture, parent take and field identity, recording
persistence, review and delivery. Desktop persists decision audits. Mopvox's
test IME writes raw/final text and warnings under its synced decision-test
results directory. The shared decision worker writes no durable audit itself.

## Scope of touch

Changes to deadlines, batching or backend failure recovery affect desktop,
Android JNI and future Apple callers. Preserve caller cancellation, retained
rule edits, UTF-8 edit offsets and complete review assembly. Check the shared
crate behavior and rebuild the native library packaged by the host.

Nemotron preserves its encoder and decoder state across a complete take.
Future 6–12-second scheduling windows should carry parent IDs and context
across work items. The proposed contract is in `crates/README.md`.

## Links

- [Runtime stack](runtime-stack.md)
