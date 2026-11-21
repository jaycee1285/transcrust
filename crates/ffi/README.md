# Native boundary

The root package remains the CLI so `nix develop -c cargo run --release -- ...`
and existing research commands keep working. It depends on `transcrust-core`
and `transcrust-decision`; it does not compile duplicate copies of their code.
Header: `include/transcrust.h`. Mopvox uses this C API through a thin JNI
bridge in its ARM64 Decision Test APK. Apple linkage remains a separate gate.

`transcrust-core` contains resampling, cleanup, dictionary/pair persistence and
proposal diffs. Its optional `asr` feature adds direct Parakeet and Nemotron.
It has no platform capture, UI, paste, network/download, tokenizers or Candle.
The decision crate contains UltraBERT/D1 plus Candle/tokenizers/onig/ORT. Only
the CLI selects ORT's desktop `download-binaries` feature.

FFI defaults to text/correction operations only. `--features decision` adds
the resident decision worker; `--features asr,decision` also exposes PCM ASR.
Each embedding platform must supply its own matching ORT build/framework when
those features need it. Do not enable the desktop download feature for iOS.
The Android decision build cross-compiles onig and Candle and links against
the same ONNX Runtime Android AAR as Mopvox. On-device smoke and iOS linkage
remain separate gates.

`tools/build-android-ffi.sh OUTPUT_JNI_LIBS_DIR` builds the ARM64 library
with an Android Rust target installed. Set `ANDROID_NDK_HOME` and
`ORT_ANDROID_AAR` to the NDK and AAR used by the embedding app. Mopvox's
`tools/build-decision-apk.sh` builds the bridge and packages both libraries.

## Calls

- `tc_text`: stateless `{"op":"clean","text":"..."}` or
  `{"op":"proposals","before":"...","after":"..."}`. Proposals do not save.
  `{"op":"review","before","after","vocabulary_path"?}` lists every change with
  `heard`/`canonical`, `learnable`, the Learn `field` and `entry` (the corrected
  text is an existing `## Entry`). Mopvox's Correct screen uses it for desktop parity.
- `tc_create`: config `{"vocabulary_path":"/synced/john-vocab.md"}`;
  optionally `"decision_backend":"ultra"` (or `d1`) plus
  `"decision_model_path":"/synced/models/ultra-q4"`; optionally
  `"asr":"parakeet"` (or `nemotron`) plus `"asr_model_path":"..."`.
  Returns a handle or null plus allocated error JSON. Model load/warm is blocking.
  Model failure preserves rule edits and leaves unresolved spans as heard.
  Applications retain review on failure.
- `tc_process`: `{"op":"process","text":"raw ASR text"}` returns raw,
  cleaned, decision-adjusted `text`, audit if enabled, and `cancelled`.
  `{"op":"learn","pairs":[{"heard":"Gamble","canonical":"Gambol"}]}`
  atomically saves explicitly accepted pairs to this handle's glossary path.
  Keep/Drop is the caller's review action, never implicit inference feedback.
- `tc_transcribe`: mono float32 PCM and actual sample rate; returns the same
  text-stage result. Native ASR is unweighted, leaving contextual decisions in
  charge of names. Nemotron uses its stateful frontend and streaming driver;
  this call currently accepts a complete take.
- `tc_cancel`: request discard of in-flight work. Ultra cooperatively stops at
  layer boundaries. D1/native ASR may finish their current native call; the
  returned cancellation flag prevents delivery. A cancelled decision does not
  disable the model. The 15-second failure budget still disables until reopen.
  That deadline retains the transcript, rule edits and audit warning for review.
  Only an explicit `tc_cancel` sets the response's `cancelled` flag.
  **The fixed deadline is a known footgun.** It covers the complete candidate
  batch and can disable decisions on healthy long speech. See the
  [decision worker boundary](../../traverse/decision-worker.md).
- `tc_free` each returned string exactly once; `tc_destroy` only after active
  calls return. Input memory remains caller-owned throughout each call.

No call pastes, commits to an IME, opens a review window or changes focus. Native
shells must associate results with their own capture/session/field identity.
The library keeps no recordings and writes no audit files. The CLI owns file
logging. Decision glossary reads use the explicit per-handle path on each
request. The host resolves its own storage location. For example, Mopvox
combines a selected folder and profile `pixel` into `pixel-vocab.md` and passes
its absolute path. Apple can provide a path in its own storage container.

The library does not protect against Syncthing conflict files or promise a
transaction across multiple processes editing one vocabulary simultaneously.
Existing same-directory rename/merge behavior is preserved.
