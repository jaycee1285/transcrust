# Transcrust Runtime Stack

## Scope
- Root Rust CLI app in `src/`
- TDT runtime from the crates.io `parakeet-rs` 0.3.5 crate (used as-shipped, not vendored).
  Functionality is layered on top of the crate, not forked into it.
- User model root at `~/.local/share/transcrust/models/` (int8 preferred, int4 supported)

## Authority
- `src/main.rs`: CLI entrypoints, smoke mode, tray startup, quit flow
- `src/model.rs`: model resolution and expected on-disk layout (int8/int4)
- `src/parakeet.rs`: service boundary, worker startup, direct ORT preflight
- `src/dictionary.rs` + `src/postprocess.rs`: post-transcription pipeline (see contract below)
- TDT greedy decode (incl. duration-head frame-skip) lives in the `parakeet-rs` crate's
  `model_tdt.rs`; transcrust uses it as-is — see Mutation Notes for the decode decision.

## Observations
- The original failure surface was not “bad model files”; it was an ORT/runtime mismatch.
- `ort 2.0.0-rc.12` plus the Nix-provided runtime wedged before model open.
- A matched `ort 2.0.0-rc.10` stack loaded `nemo128`, Whisper, and the Parakeet encoder/decoder normally.
- The custom ORT logger callback also caused probe crashes and is intentionally not used now.

## Post-Processing Contract
- The post-transcription pipeline lives in `src/postprocess.rs::fix_transcription`,
  called once from `main.rs` after the worker returns text.
- Order: course-correction → repetition cleaning → filler removal → spoken
  punctuation → **phonetic dictionary** (`src/dictionary.rs`, last).
- The dictionary is the ported murmure leg: `rphonetic` with the `embedded_bm`
  feature (Beider-Morse rules compiled into the binary, no resource dir). It runs
  English-only via plain `encode()` — no language set, no French. It does a
  whole-token phonetic swap against `~/.config/transcrust/dictionary.txt`, and is
  a no-op when that file is absent.
- **Harper is intentionally not in this pipeline.** `harper-core` is still a
  dependency and its known-good config survives as dead code in `postprocess.rs`,
  but it is not called. Reason: Harper is a probabilistic grammar layer that
  re-ranks tokens toward general English *before* the dictionary can claim them —
  it was splitting `tori` into `tor i`, starving the phonetic corrector. The app's
  job is exact non-standard vocabulary; a general-English re-ranker is the wrong
  shape at this stage, and murmure (the source pipeline) never put grammar here
  either. Removing Harper unblocked `tori -> Tauri`.
- Consequence: no capitalization/punctuation grammar. Casing is left as the model
  emitted it. If that becomes a problem, the answer is a small purpose-built
  deterministic pass (or a toggled local post-processor), not re-adding Harper.

## Mutation Notes
- `parakeet-rs` is consumed from crates.io (0.3.5), not vendored. The earlier vendored
  TDT-only copy has been replaced by the published crate; the intent is to build on the
  crate and add functionality outside it rather than fork it.
- The crate's TDT decode keeps the duration head and frame-skips (`model_tdt.rs`). murmure's
  no-skip decode (leg 2) was reviewed and deferred — it would require forking the crate, and
  the payoff is second-order for short dictation. See `murmure.md` and `Smoke-Human-2026-06-08.md`.
- Int8 is preferred for normal use; int4 is supported (working local layout). Mixed fp32
  directories are no longer the target operator path.
- `--smoke` is the permanent high-observability path and should stay available even after UX cleanup.
- The authoritative operator-facing app now lives at the repo root; the earlier investigation subprojects were removed after the working runtime was promoted.
- Local-share model resolution accepts both canonical int8 filenames and the older `encoder-int8` / `decoder_joint-int8` layout so the user model cache remains valid without renaming.
