# transcrust — start here

Push-to-talk voice input for a keyboard-driven Wayland desktop. One user, one
machine, no network. Written by John (content strategist, not a career
engineer); assume the architecture notes are load-bearing and the constraints
are deliberate.

**Read [Models.md](Models.md) before handling models.** It is the canonical
inventory of model sources, installed identities, revisions, local paths,
experiment status and upstream update watches. Add new model references there
before downloading weights. Refresh the GLiDE and Laya watch entries during
model reviews and the session's model sweep.

## Known footgun — decision deadline (fixed 2026-11-21)

`crates/decision/src/lib.rs`, `Decision::request`, used a fixed 15-second
wall-clock deadline on the whole model-bound batch, and expiry disabled the
instance. November 19 phone evidence: 64.4 s of Nemotron speech, 25 candidates,
17 model-bound, killed decisions for the rest of the session.

Since 2026-11-21 the deadline is `decision_budget`: 10 s plus 1.5 s per
model-bound occurrence, capped at 60 s. Expiry stops inference, clears the stop
flag and returns the error (`decision budget of Ns exceeded; unresolved spans
left as heard`); rule edits and as-heard text survive and the instance stays
usable. Only genuine worker failures set `disabled`. Explicit cancellation is
unchanged. Surface the warning and model availability in diagnosis. See
[the decision worker boundary](traverse/decision-worker.md).

## Household data lives outside the repo (John, 2026-11-20)

Anything privacy-tipped never goes in the repo tree, not even gitignored: family
names and their mishearings, recordings, transcripts of household speech, personal
vocabularies, phone takes and the logs or fixtures derived from them.

1. Stage new results in `/tmp/` for John's review.
2. After review, write them to `~/syncthing/transcrust-work/<study>/`. Do this even if the
   inputs started elsewhere, e.g. `~/syncthing/mopvox/decision-test/`.
3. Tracked code and tests use the substitute name **Rowan** for the household
   child, as the 2026-11-18 scrub did.

The point is that a later agent asked to `git add`/commit cannot sweep household data
in. Do not depend on remembering `.gitignore` entries. The 11-19 window study now lives at
`~/syncthing/transcrust-work/window-study/` (logs in `logs-20261119/`). The older
gitignored `logs/`, `fixtures/` and `tools/rules-probe/private/` were moved to
`~/syncthing/transcrust-work/repo-leftovers-2026-11-23/` on 2026-11-23.

## Read in this order

| | |
|---|---|
| `traverse/runtime-stack.md` | how the thing is built and why. The authority on architecture. |
| `TASKBOARD.md` | what to do next, in order, with kill criteria |
| `SCOREBOARD.md` | does voice beat typing for him: needed-error rate × 7.8 s per correction vs 40 WPM, per engine/build, plus name accuracy |
| `Models.md` | every known Hugging Face model reference, lineage, status, storage and update watchlist |
| `Dragon-Mechanisms.md` | the research north star — what pre-neural dictation solved and which of it is still load-bearing |
| `Parakeet-v3.md` | why the default engine is the right one for on-device control. Reference doc, not a task list. |
| `murmure-reference.md` | notes from an AGPL reference app that was read and removed — calibrated constants, and the licence caveat before copying any of it |
| `design-dictation-as-control.md` | the unbuilt half: dictation as a command channel |
| `design-long-form-routes.md` | the four long-form dictation routes, their acceptance gates, and the fuzzel picker |

## State reconciled 2026-11-11

**2026-11-03 crate extraction:** root package is still the CLI (existing launch
commands preserved). Shared Rust source is now in `crates/core/src/` (including
direct Parakeet/Nemotron, resampler and correction/glossary) and
`crates/decision/src/` (UltraBERT/D1). `crates/ffi` has the C ownership boundary;
Android JNI and Apple target builds remain pending. Do not recreate deleted
`src/ultra.rs`, `src/d1.rs`, etc. Native Rust Phonon 2.5 was implemented and
compared with C. Desktop evaluation is retired; retain the crate/evidence for
Android/Apple work. Pocket was parked on 2026-11-18 (Models.md); GLiDE remains a queued dictionary-decision trial.
New Nemotron packing still needs the checkpoint link. `Models.md` records
model identities; `TASKBOARD.md` owns the current work order.

The installed default **Parakeet TDT** is our local **moondream/parakeet-ultra
INT4 export**, housed in the legacy `parakeet-tdt-0.6b-v3-int4` directory.
November 11 ONNX stack-trace inspection confirms the local Ultra exporter and
Kestrel implementation. The downloader still points at community v3 weights.
Keep those identities separate. See `Models.md` for hashes and evidence.

Other retained desktop engines are **Granite Speech 5 470m TurboCTC** (int8)
and **Nemotron Speech Streaming EN**
(int4, toggle capture). Modes carry model, profile and capture policy.
`Granite — Long` uses the same directory with a repair profile.

VibeVoice was evaluated and removed. Export research remains under
`tools/vibevoice-export/`. The previously recorded external postmortem path
`~/syncthing/vibevoice-asr-15/` was absent at the November 11 inventory. Its
location needs recovery if that evidence is required.

Recently built and worth knowing about:

- November14 family pipeline: complete change review, separate explicit Learn
  controls, stable take IDs and retained correction history, capture-time
  Nemotron dispatch, and `--wav --dec --paced --output-dir` replay. Read
  [pipeline-2026-11-14.md](pipeline-2026-11-14.md) for measured results and pending
  human acceptance. Legacy ordinary-word rules are excluded from name decisions;
  model weights and the shared glossary file were not replaced.

- `--bench <WAV...>` — times every mode over the same clips, through the live
  path. Fixtures in `tools/bench-clips/`.
- `observe.corpus = true` — banks every dictation as WAV + JSON under
  `~/.local/share/transcrust/corpus/`. That directory has its own README.
- `crates/core/src/parakeet_ort.rs` — Parakeet driven directly through `ort`, recovering the
  per-token confidence `parakeet-rs` discards. **Wired into the live path**,
  with a crate fallback if the direct graph set is absent or broken.
- `--pick` opens fuzzel on a running daemon; startup does not open the picker.
- Correct last dictation: `--correct`, optional `--correct-chord C-A-c`, and a
  tray entry open an egui editor. Replace/Copy and Keep/Drop vocabulary proposals
  are wired; **the GUI and GTK palette passed John's 2026-10-06 smoke**.
  Learned vocabulary accuracy is a separate, still-open gate. See `design-correct-last.md` and
  `Smoke-Human-transcrust.md`. Apple's review-before-paste contract remains in
  `~/repos/swiftype/PRD.md`; Linux crate extraction is built, native mobile/Apple
  linkage and parity gates remain pending.
- `--dec` is implemented in native Rust. **UltraBERT INT4 is primary as of
  2026-11-01 at John's request**, with marked sentence context and candidate
  descriptions, no global glossary table. `crates/decision/src/ultra.rs` / `decision_tensor.rs`
  share the tested packed inference with the research harness. D1 Q8 remains
  selectable via `--dec-backend d1`, retaining its full-glossary padded ORT batch.
  Run optimized: `nix develop -c cargo run --release -- --dec --smoke`.
  No runtime Python. John accepted desktop D1 Q8 as-is on 2026-10-29; UltraBERT's
  live human smoke passed on 2026-11-19. Mobile latency/RAM remain separate gates.
  Both descriptions and learned aliases
  belong in `~/syncthing/john-vocab.md` (or `[vocabulary].path`).
- Phonon 2.5 is live in mopvox's Android IME. Its native Rust crate and desktop
  adapter remain present after the decision to retire desktop evaluation.
  Phone findings and remaining correction/Apple work are at the top of `TASKBOARD.md`.
  Its current-state table and ordered next steps outrank historical phase prose.

## Three facts that are easy to get wrong

1. **Parakeet exposes confidence.** The claim that it does not describes the
   `parakeet-rs` crate boundary, not the model. The joint's vocab logits are in
   `decoder_joint-model.onnx`. This error cost a night and sent a plan toward
   the wrong engine.
2. **A traced ONNX graph can declare a dynamic axis it does not honour.** Bit us
   twice — the VibeVoice encoder returned the traced frame count for any input,
   and Granite's pad-to-512 is a hard constraint despite a dynamic declaration.
   Verify shapes by running the graph, not by reading its metadata.
3. **The resampler was fixed, not proven to improve accuracy.** `audio.rs` uses
   a band-limited polyphase decimator. A.2 did not demonstrate an accuracy win
   on codec-limited speech; real microphone replay remains open.

## House style

FOSS, minimal abstraction, local files plus Syncthing, no SQLite unless asked.
Touch only what the seam requires. Do not pin the app to a reference's
dependencies just because the reference used them. Hand work back as one
copy-pasteable `nix develop -c ...` command and wait for the human.

## Sweep up after a model trial

Give each download a home before fetching it. Use the model directory the app
already discovers for installed weights, and a named temporary or cache
directory for trial weights. Keep model payloads out of Cargo's build tree.

At the end of a trial, check which paths the app and rerun commands still use.
Keep the revision, checksums, license, rerun command, fixtures, and result in
the repo. Record the exact paths and sizes of leftover weights and build
outputs in `decision-model-cleanup.md`, with what each removal would cost to
recreate. Decide with John what to keep, relocate, or delete while the paths
and results are still known. Verify any relocated live bundle through the
normal loader before clearing its old location, then hand back the live smoke.
