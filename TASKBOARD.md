# TASKBOARD

## Done
- Stabilize ONNX Runtime startup by aligning the app and vendored `parakeet-rs` to `ort 2.0.0-rc.10`.
- Strip the vendored `parakeet-rs` copy down to the TDT path actually used by Transcrust.
- Restore working transcription with Parakeet and int8 model resolution from `~/.local/share/transcrust/models`.
- Add `--smoke`, tray status, and `--quit`.
- Promote the working app to the repo root, build the release tarball, and wire the matching `tauri.nix` fetch entry.
- Remove inactive investigation/reference dirs (`parakeetvox`, `ortprobe10`, `silentkeys`, `rustvox`).
- Human smoke confirmed the repo-root app, local-share int8 model resolution, tray icons, and live quit flow.
- Move Parakeet worker/model load to first use so idle daemon startup stays under `40 MB` RSS.
- Simplify the tray menu to a single `Exit` entry.
- Port murmure's phonetic vocabulary corrector (leg 3): `rphonetic` + embedded Beider-Morse rules, English-only, as the last post-processing step. Wired, tested, signed off 2026-06-08.
- Pull Harper out of the post-processing loop (it re-ranked tokens toward general English before the dictionary could claim them). Added `--fix <text>` debug surface and a `--doctor` dictionary report.

## Next
- Decide on a minimum-entry-length guard for the dictionary, or keep it raw and curate the dictionary file by hand (current choice: raw + documented in `dictionary.example.txt`).
- If a grammar/punctuation pass is wanted back, build a small purpose-built deterministic one (or the future small-LM toggle) rather than re-adding Harper.
- Add explicit first-load tray/icon feedback so users can see model warmup instead of only paying hidden latency on first transcription.
- Tighten startup/log ergonomics so steady-state smoke logs stay high-signal.
- Verify the `transcrust` `tauri.nix` entry in the config repo against the published release asset.

## Risks
- Pure Nix builds still need scrutiny because `ort` binary provisioning is touchy across environments.
- Tray/status behavior depends on SNI/DBus availability in the running desktop session.
- Feature brittleness remains around cross-session tray rendering and first-use model warmup UX because the repo does not control the status-notifier host theme path.
