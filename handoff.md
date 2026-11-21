# Transcrust handoff — active work, 2026-11-06

This note records continuation hazards, not another taskboard. Paths below are
relative to the transcrust repository unless stated otherwise.

## Immediate scope

**November 14 pipeline BUILT/INSTALLED:** read `pipeline-2026-11-14.md` for source
changes, replay results and the pending human acceptance. John
explicitly asked for a complete real correction pipeline using recent takes;
his son is trying it with him. Full edit review, durable linked correction
history, true Nemotron capture-time dispatch and actual --wav --dec replay are
implemented and tested. Installed SHA256 f14e3a110e8afb41cacc3cb767af069e1a80f234ab4853aa66f0be79246a4ae2,
live unit `transcrust-zed-smoke`, Nemotron restored, C-A-c enabled. Nine WAVs
replayed with both engines; paced Nemotron tail0.111s. Core75 tests + history,
audio tee/failure and decision guard tests pass. Ordinary function-word aliases
and casing-only lowercase-term rows are now excluded from name decisions;
file entries and weights remain unchanged. Human full-review/history/latency
acceptance is pending. Previous binary retained outside target for rollback.

**November 13 live smoke follow-up:** user reports rough dictation and correction
on both engines; do not call migration machine checks human acceptance. New
TASKBOARD top section has prefix replacement, Nemotron late dispatch and real
decision coverage gaps. Saved take/corrected text should be replayed, not
re-recorded. Prefix example `We'rWe're...` means four original characters
remained; it is not missing correction text. Replacement patch adds a short
focus-settle before selection and types literal text separately; human check
pending. Keep Nemotron selected for the next smoke and Ctrl+Alt+C enabled.

**Prefix fix update:** focus-settle alone FAILED human smoke (one leading M
remained). Latest replacement code copies selection back to validate against
persisted `buffer_text`, extends only a verified short suffix, and refuses to
type unless exact text including punctuation is selected. This supersedes the
earlier timing-only patch; requires human acceptance. Do not revert to a fixed
extra character count. `correct.selection` logs the verification evidence.

**2026-11-13 latest — migrated to Zed AMD:** see `migration-2026-11-13.md`.
App-only rebuild passed and matches November 11 executable SHA256; installed
under the same libexec path. Models/glossary present and tested; optional FP32
reference cache absent. Labwc autostart/Super+/ use new `libexec/transcrust/launch`
with --dec; profile `transcrust` remains older. Active live unit is now
`transcrust-zed-smoke.service`, superseding November 11 unit references below.
Parakeet hold-Print smoke, injection and correct-last await John. No model or
runtime code changes. Mixed casino-verb overcorrection recorded in report.

**2026-11-11 cleanup completed:** John's five steps executed. Q4 is installed
at `~/.local/share/transcrust/models/ultrabert-380m-curation-q4`; FP32 source is
in the revision-named external reference cache. Hashes verified, callers fixed,
old build trees removed, fresh app-only release built and installed at
`~/.local/libexec/transcrust/transcrust`. Original executable retained beside it
as `transcrust-before-cleanup-20261111`. CLI ASR/correction outputs match.
Repository 13.14→1.31 GiB; see `decision-model-cleanup.md` for byte counts and
retained external assets. `transcrust-cleanup-smoke.service` launches the fresh
app with default Parakeet and `--dec --smoke`, using `launch-smoke` beside the
binary. Human microphone/injection/correct-last acceptance remains pending.
Hold Print and release for Parakeet; old Phonon toggle instructions below are
historical. Temporary `11-11.md` is deleted as requested. No package/platform
acceptance, commits or publication were performed in this cleanup.

**2026-11-11 model inventory:** `Models.md` is now the canonical Hugging Face
registry, linked prominently and in the read order in `AGENTS.md` and from
TASKBOARD. Maintain new references, versions, storage, trial status and update
watches there. GLiDE/Laya HEAD baselines were checked November 11. Installed
Parakeet is CONFIRMED our local moondream/parakeet-ultra export despite its
legacy v3 directory name: ONNX embeds export_ultra.py and Kestrel stack traces.
Installed encoder/joint/frontend hashes and downloader distinction are recorded
in Models.md. Do not infer installed model identity from directory names.

**GLiDE scope resolved, November 10:** John says single-family use is
non-commercial. Record private/offline family use as intended scope; do not
park this trial over commercial-use terms or re-ask the same license question.
GLiDE is now queued alongside Pocket, before Hindsight/Joey. No weights or
inference have been downloaded/run during these card reviews. Earlier pending
license language below is historical and superseded by this clarification.

**2026-11-10 latest request:** GLiDE-0.4B card/source review complete at
`6bda7ab91aeb36dcad6d27828a8871e548c7fc71`. See
`decision-model-review-2026-11-10-glide.md` and saved Git cards/config/license
under `tools/model-card-review-2026-11-10/`. No weight downloads or inference.
ModernBERT/Ettin 28-layer encoder, independent question rows batched, final
prompt-token 255-way readout, two-tokenizer rendering. THX ModernBERT research
code is a starting point, not a compatible loader. Raw weight payload790.1MB;
CPU quantized RSS/latency unmeasured. FCL commercial attribution/$10M/derivative
terms are the material issue. Async question pending: add research-only trial
beside Pocket or exclude for FOSS preference. GLiDE execution is NOT queued.

Prior five-card triage was accepted by John's “Okay” on November10 and is now
in TASKBOARD: Pocket first, Hindsight second, Joey later, Minico research-only,
EG2 duplicate rejected (only evaluation YAML changed, weights/code identical).
Phonon desktop tuning is discontinued per November8 discussion; crate/evidence
retained for Android/Apple, source/weights still present. No runtime removal or
daemon switch was performed during card reviews. Do not treat old Phonon script
experiment below as current next work.

**Four-paragraph trial completed:** corpus `2026-11-06T22-28-33Z.wav` is122.24s,
first200 script words plus paragraph commands; user stopped because the script
was too difficult. Same-audio6/12 replay:28vs12 windows, Rust/C text exact on
all40,12 replay equals live raw. Written-script disagreements22vs14, or16vs10
after stripping recognized paragraph markers; NOT verified WER or ground truth.
12 improves “threaded”, “chewy”, “cluttered her”, “ribbons” but regresses
thistles→fistles and sells→cells. Six begins “Clennard Hert” at109.845s whereas
12 keeps phrase inside105.075–114.285. Live ASR tail1.605s, zero DEC candidates,
1.62s to starting injection, several more seconds to finish typing. Leave12
selected for ordinary dogfood; do not ask John to finish/rerecord this script.
Report: `tools/phonon-probe/script-2026-11-06.md`; private evidence in ignored
`logs/phonon-script-20261106/`. Separate22-26-17Z take is instruction header.

**Latest requested experiment:** 500-word phoneme stress read at a 12-second
Phonon cap. Script delivered to `~/syncthing/phonon-12s-phoneme-smoke.md`,
source `tools/phonon-probe/phoneme-script-500.md`; marked body is exactly 500
whitespace words. New `TRANSCRUST_PHONON_WINDOW_SECS=12` selects cap; default 6
retained. Same min-pause 6s/lookback 3s. Native lossless/bounded tests cover 6/12;
root check passes. Old 35.84s take: four 12s-cap windows vs seven baseline, all
four match C; name spellings change (Rowan→Rowen, Chloe→Clo). Replay example
has optional `live|twelve|twenty` plan. Root release build/restart for 12s trial
must finish before human reads; check live unit env/logs. Next submitted WAV
should be scored raw and delivered against script, and replayed at both 6/12
from the same full audio. Do not retune models/glossary alongside comparison.

12s release build and relaunch are DONE: `transcrust-native-phonon.service`
active/running, PID1954959 at launch, Environment explicitly contains
`TRANSCRUST_PHONON_WINDOW_SECS=12`. Tray, native model selection, UltraBERT and
audio banking startup verified. Default without env is still6. Glossary before
read saved in ignored `logs/phonon-smoke-20261106/twelve/vocabulary-before-read.md`.
Waiting for John's 500-word read; no new script recording/results exist yet.

**2026-11-06 latest:** corpus `2026-11-06T19-11-20Z.wav` + JSON exists, 35.84s
at 44.1k. Native replay matches raw live output and C on all seven windows.
Two fixed20s windows match C too but introduce O'Reilly/Maider; do not promote
larger windows. C default long-form also has Clo/Maider. Phone uses ARM int8w
and native16k VOICE_RECOGNITION; x86 C uses exactfloat, so phone parity is not
established. Unweighted Parakeet comparison and raw diagnostics are saved in
ignored `logs/phonon-smoke-20261106/`; report and rerun at
`tools/phonon-probe/smoke-2026-11-06.md`, native example `replay`.

Confirmed delay fault FIXED: root now calls PhononService::start_capture on
press, immediately forwarding chunks to inference while retaining an exact
corpus copy. Release waits for pending result before decisions/delivery.
Other engines keep their existing dispatch. Two unit tests pass (forwarding
before close, banking/failed consumer); paced actual-model ignored test passes
with exact raw text+audio and 1.490s ASR tail vs original9.517s. Original DEC
added1.67s; paced test excludes DEC/injection. Build/restart and final human
acceptance are not implied by machine checks; confirm current unit/logs.

Release build completed and supervised unit restarted with the fix; active/
running, tray, Phonon selection and banking verified. PID1943239 at launch.
Next human take uses Print → speak → Print; quality acceptance remains open.
Identical resampled float32 WAV is staged in Syncthing mopvox trial audio as
`desktop-smoke-20261106-16k.wav`; existing phone Setup → Phonon comparison →
Test Phonon 2.5 English needs no new APK. Compare synced results to exact x86
live windows to separate ARM int8w behavior from capture/audio differences.

**2026-11-04 smoke failure overrides acceptance-pending notes below:** John says
native desktop Phonon is markedly worse after four takes. All four have zero
DEC candidates; poor wording exists in raw ASR. We failed to enable corpus, so
there is no new audio to replay. Original logs saved in
`/tmp/transcrust-phonon-smoke-2026-11-04.log` and
`/tmp/transcrust-decisions-before-audio-banking.jsonl`. Config corpus is now
enabled, and `--smoke` automatically enables WAV/JSON banking with a startup
destination log. Root main invokes transcription after release: the worker's
claimed during-capture processing did not actually happen. Fix that separately;
do not attribute quality failures to a cause without the next banked takes.

**Active authorized task:** finish native Rust Phonon 2.5 English on NixOS,
then work on packing/building a new Nemotron checkpoint. An asynchronous text
question asks for that checkpoint URL/path; no answer yet. Keep working on
Phonon meanwhile. Do not stop at a plan or replace Phonon with a Parakeet graph.
John explicitly wants actual 2.5 packed weights/arithmetic and a Rust runtime;
C is an oracle/benchmark, not the deployed runtime. Nemotron's resident,
bounded long-running behavior is the target, not invented trained caches for
Phonon. Swiftype PRD still owns the cross-platform product direction.

Mopvox now runs real Phonon 2.5 English in the IME, replacing Whistle. John
said “Okay, that worked” and “remarkably fast.” Delivered as
`~/syncthing/mopvox/mopvox.apk` and `mopvox-phonon25-ime.apk`. Existing trial
screen has Phonon 2/2.5/Parakeet separately. Source changes, smoke sheet and
board are saved in mopvox. IME default still has legacy correction, no decision
engine. C/Needle target removed from APK; historical Whistle research retained.

## Extraction already implemented in this active slice

Root package remains CLI to preserve commands. Shared files moved (not copied)
to `crates/core/src/` and `crates/decision/src/`. Core includes direct Parakeet,
Nemotron inference, resampler, graph-layout selection, correction/dictionary/
boost/cleanup; minimal core has no ORT/Candle/tokenizers/platform dependencies.
`asr` feature adds ORT/ndarray/realfft. Desktop Nemotron service, microphone,
notifications/paste/tray and unused Harper reference remain in root `src/`.
Decision depends on core's tiny Events trait, not desktop Observer. Explicit
per-instance model/glossary paths and external cancellation are implemented.
Ultra cancels at layer boundaries and can recover next take. Default desktop
policy and target/schema-probe/ultra-q4 fallback survive. Tools import the
shared crate and provenance scripts follow moved paths.

`crates/ffi` has seven C functions (header/README): explicit-path sessions,
clean/propose/process/save kept pairs, optional ASR/decisions, cancel/free/
destroy. Pure defaults; native builds still need target ORT. Linux link smoke:
`cargo build --release -p transcrust -p transcrust-ffi --bin transcrust --lib
--features transcrust-ffi/asr,transcrust-ffi/decision` unifies root's desktop
ORT provider. Standalone FFI ASR build without a supplied ORT may leave a
missing-link marker; standalone decision tests need ORT too. Do not treat a
successful staticlib build as platform linkage acceptance.

Release workspace/all-features tests passed; ignored 32-case Ultra numeric/
cancel test passed after move, and new cancelled-utterance/retry test passed.
Pre/post CLI bench on chat-05s matched all Parakeet/Nemotron/Granite/direct
transcripts. Actual C client `/tmp/transcrust-ffi-client` loaded the FFI shared
library and corrected Gamble→Gambol. `--fix --dec` also works. Apple/Android
Rust link acceptance remains pending.

## Native Phonon work in progress

**Latest state overrides initial notes below:** user explicitly rejected
guessing memory fixes and said to learn the C implementation. Traced
eschmidbauer reference `weights.c` / `blas.c` / `encoder.c` completely through
the relevant paths. The reference itself expands decoder tables to FP32
(`expand_q8_rows`, 69,813,760 bytes). Removing those is NOT the instructed port.
Its encoder expands small panels into reusable per-thread scratch, not a full
operator matrix; its conv bias/ReLU update in place. Those are now translated
in Rust. `kernel.rs` is exact MR4/NR16 AVX2/FMA with <=256KiB per-thread reused
scratch; original container is still mapped/packed. Subsampling uses the C
allocation lifetimes/in-place activation shape. Large integer encoder matrices
are expanded by panels, not cached FP32. Decoder remains reference-compatible.

Latest `/tmp/phonon25/rust-c-layout.jsonl/log` / `rust-c-layout-trace/`:
6.015s public clip -> 1.336/1.309s exact transcript, process VmHWM266504/280968KiB
(~274MiB peak after two passes). C same clip peak329472KiB (~322MiB), 1.11s;
Rust-before-panel port was ~312MiB. Final encoder max_abs .00000322, relative
L2 .00000112. Second public clip5639-40744-0013 also matches C text exactly.
Do not claim 158MB disk download equals process RAM or that Rust must eliminate
FP32 decoder allocation to mirror the C reference. Earlier user-facing guess
about decoder expansion being the wrong thing was corrected with measurements.

Speculative int8w/VNNI experiment was slower and changed Count→count; it is
**parked** at tools/phonon-probe/int8w-research.rs, not part of live inference.
PHONON_MATMUL is no longer used. Default/runtime is reference-checked exact.

Root model/mode/transcription/PhononService wiring now exists (Phonon Toggle,
raw profile, own six-second segmenter, resident worker, idle unload, encode
windows during recording). Source model installed at
`~/syncthing/models/phonon-2.5-english-native/` and symlinked into local-share.
Original upstream NOTICE copied accurately (NOT converted Android NOTICE).
Native and root release binaries built and checked. Root five-second bench
matches existing-engine transcripts, with Phonon at 1.21–1.25s. A 49.72s local
household WAV ran through the resident six-second window service in ~14s;
this was ASR-only, not a decision or human quality check. Third public clip
4077-13754-0014 matches C on both passes too.

Live daemon launched as `transcrust-native-phonon.service`, PID782342 at launch,
Phonon Toggle selected and UltraBERT `--dec` ready. Active/running and tray bus
registration verified. No previous daemon was running. Tap Print to start/stop;
human native-desktop acceptance is pending. Do not run offline bench/WAV while
the daemon is running: those commands truncate the same logs/latest.log.
Crate README, board, runtime notes and smoke ledger now record the current seam.

All workspace release tests passed before the final panel/spatial edits (root50,
core90 +1ignored, decision11 +3ignored, FFI1, research7 etc.). Current native
panel/tail-arithmetic and bounded-segment unit tests both pass. Root all-target
check passed with the Phonon wiring. Need final appropriate checks after current
changes; do not rerun every old model experiment. C FFI shared-library client
is still proven, but Phonon has not yet been added to that C ABI.

Initial implementation history (superseded where noted above): new
`crates/phonon` (`transcrust-phonon`) is in workspace. Binary
`phonon-native MODEL_DIR WAV [TRACE_DIR]`:
actual pinned model.fermion, read-only mmap, packed five-value encoder with
one-operator float scratch, cached small/dense/decoder tensors; pure Rust
frontend + Candle CPU encoder + native SIMD LSTM/TDT decoder. No C/ONNX/Python.
This is the actual 2.5 trained graph, not a Parakeet checkpoint substitution.

Source release rev `da200358d638886bc96c758c672ad5568828b119`, unpacked at
`/tmp/phonon25/source` (not yet copied to permanent native model dir).
Container SHA c57119c0f04756932acc1d60d90506346e3708d69f00e654084acc7fe507fe27;
archive config SHA 9e5dfcc4e49a6c169d53540f07f4115e591007759b835132c849166b235dac95.
Default exact Rust run on public 5105-28240-0004.wav gives the identical
reference sentence; ~305 MiB peak RSS; initially ~6–10 s per 6 s audio, **too
slow**. Tensor trace vs old C exact oracle: final encoder max abs .00000244,
relative L2 .00000123; frontend .00003517 max. All 24 block traces match
numerically closely. Strong correctness baseline, not speed/mobile acceptance.

`weights.rs` now has lookup trit reconstruction and cached-per-row magnitudes
(avoids repeated fp16 conversion per element). A native signed-int8 AVX2 dot
kernel and optional `PHONON_MATMUL=int8w` path were just added, matching the
phone C engine's per-row activation quantization / low≈round(127*lo/hi), hi127
arithmetic. It remains opt-in pending tests/quality. Default is exact. Latest
build session **77035** is running (`cargo build --release -p
transcrust-phonon --bin phonon-native`). Need poll; then compare exact vs int8w
quality/performance, run kernel and segment tests. No production C dependence.
`segment.rs` ports mopvox's 6 s/last3 s quiet-gap policy, not yet integrated.

Working data under `/tmp/phonon25/`: original2 and2.5 archives/sources; native
C engine clone at pinned c4b860f6ad745369d6f21599d6e23d381c1fcc7a; host-build;
C reference weights converted by new native Rust converter now saved in
mopvox/tools/phonon-weights. Converter reproduces original C weights/tokens/
tensor layout **byte-for-byte**. Phone C bundle in synced
models/phonon-2.5-english-c (~244MiB), NOT native runtime's direct container.
C engine source has trace-only modifications driven by /tmp/phonon25/trace-c.pl.
Reference and Rust traces: c-trace/, rust-trace/; results rust-first.jsonl,
rust-profile.jsonl/log. `/tmp/phonon25/compare` built from
tools/phonon-probe/compare.rs; compile with --edition2021. No Python executed.

Official serving investigation: fermionresearch/phonon rev
b528461fcc86c10f27fe06e4a4755b711bcc1418 cloned at official/; wheel0.2.15
unpacked at wheel/ with 7z (no installation). Native C encoder/TDT sharedlibs
are Apache-licensed according NOTICE; kernel3 AVX2 loads on this i5-1345U.
Source wrappers expose ABI, but C kernel sources not found. Small C ABI/matrix
probe source is tools/phonon-probe/abi.c; result kernel-probe.txt. It is only
research, not the Rust implementation. Keep old C and optional newer binary
as oracles. Do not bring their Python server into app.

## Working-tree state matters

HEAD is `65d2f8d`. Substantial completed work remains **uncommitted**, including
the UltraBERT runtime integration and the model research/tools/reports. Numerous
new source files and evidence directories are untracked. This is accumulated
session work, not disposable scratch or unexplained user changes to revert.
No new commit or push was made during these turns. Inspect `git status` and
preserve the existing tree before making a new slice.

The root `decision-model-results.md` is the new cross-model results index.
`decision-experiments.md` is older chronology, not the current default/acceptance
authority. `decision-model-cleanup.md` records the explicitly approved and
completed cache deletion. Do not mistake absent rejected-model weights for an
incomplete feature or automatically download them again.

## Refactoring pitfalls not obvious from the board

- Running `rustfmt src/main.rs` follows child modules and reformats unrelated
  files. That happened here; all incidental unrelated formatting was reversed.
  Use targeted formatting with `--config skip_children=true` when appropriate.
- The native tensor/model code was promoted from the schema research directory,
  then moved to `crates/decision/src/decision_tensor.rs` and `ultra.rs`.
  Research imports the shared crate. Older source hashes describe earlier code;
  a mismatch after promotion does not imply a changed historical measurement.
  Do not rewrite old provenance merely to make a checksum command pass.
- `QStorage::from_data` deliberately receives **borrowed** buffers kept alive
  until its copy completes. A Candle 0.9.2 owned-buffer lifetime problem was
  encountered and fixed during the d3 port. See `tools/d3-probe/README.md` around
  the loader notes; do not simplify this back to the problematic owned path.
- The small-ASR-core rule in the Apple PRD predates the new decision runtime.
  Settle the decision component boundary before moving Candle/tokenizers/onig
  into the future iOS static library. This remains an architectural decision,
  not permission to reinterpret the Apple review-before-paste behavior.

## What not to infer from the last turns

The post-cleanup checks were no-microphone/no-injection `--fix` probes. The
running tray daemon was not restarted, and no new live human acceptance was
reported. Functional state, exact commands and machine-test results belong in
`design-dec.md` and `Smoke-Human-transcrust.md`; consult them instead of treating
cleanup approval as acceptance of the new model's dictation quality.

Model weights and voice recordings remain outside Git. Do not export private
fixtures to public CI; the PRD's parity fixture policy still applies.

## Suggested skills

- `session-start`: if the user asks to begin the next session under that workflow.
- `tray-app-restart`: only if actually restarting/testing the live tray daemon;
  read its environment/bus/permission guidance before killing a process.
- `grill-for-unknowns` or `grill-with-docs`: if the user wants to stress-test the
  crate/FFI boundary against the PRD before implementation; not an automatic
  requirement to stop and ask more questions.
- `ponytail-clean`: only for further requested cleanup. The approved cache cut
  is already complete; it is not authorization to remove research source or
  shared model files in other repositories.

No keys, personal voice contents or family details are included in this note.
