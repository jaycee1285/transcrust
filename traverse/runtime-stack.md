# Transcrust Runtime Stack

## Scope
- Root Rust CLI app in `src/`
- Vendored TDT-only runtime in `parakeet-rs/`
- User model root at `~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int8`

## Authority
- `src/main.rs`: CLI entrypoints, smoke mode, tray startup, quit flow
- `src/model.rs`: model resolution and expected on-disk layout
- `src/parakeet.rs`: service boundary, worker startup, direct ORT preflight
- `parakeet-rs/src/model_tdt.rs`: final encoder/decoder selection order

## Observations
- The original failure surface was not “bad model files”; it was an ORT/runtime mismatch.
- `ort 2.0.0-rc.12` plus the Nix-provided runtime wedged before model open.
- A matched `ort 2.0.0-rc.10` stack loaded `nemo128`, Whisper, and the Parakeet encoder/decoder normally.
- The custom ORT logger callback also caused probe crashes and is intentionally not used now.

## Mutation Notes
- The vendored `parakeet-rs` copy is intentionally narrowed to the TDT path to keep the runtime surface small.
- Int8 is preferred everywhere for normal use; mixed fp32 directories are no longer the target operator path.
- `--smoke` is the permanent high-observability path and should stay available even after UX cleanup.
- The authoritative operator-facing app now lives at the repo root; the earlier investigation subprojects were removed after the working runtime was promoted.
- Local-share model resolution accepts both canonical int8 filenames and the older `encoder-int8` / `decoder_joint-int8` layout so the user model cache remains valid without renaming.
