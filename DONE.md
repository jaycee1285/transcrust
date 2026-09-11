# Done
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
- Guard the phonetic dictionary against common English words. `Tauri` was claiming every `they’re` — 7 hits, 0 survivors, deterministic via `--fix`. The guard sits on the source token (commonness, not length: `Tauri` is 5 chars and `they’re` is 7), and exact-match moved to its own pass so an earlier entry’s phonetic neighbourhood can no longer claim a word by line order. Shipped 2026-09-09.
- Replace the unfiltered linear-interpolation resampler with a band-limited polyphase decimator (`TASKBOARD.md` A.1/A.2). The aliasing was real and measured; fixing it did not measurably change recognition on codec-limited speech.
- Route the live Parakeet path through the direct ORT driver, with a crate fallback when `nemo128.onnx` is absent (`TASKBOARD.md` B.3). RTF 0.11 against 0.17, and per-word confidence now reaches the live path.
- **The input/output path had no validation, and a human smoke found two bugs no test could have.** Neither is a logic error; both are environment and integration, which is exactly the class a unit test cannot see:
  - `Space + LeftAlt` typed into the focused window for the whole hold, because transcrust reads evdev passively and cannot suppress a printable key. Default is now modifier-only; `--doctor` warns; `hotkey.grab` is an opt-in partial mitigation; `--keys` shows the live stream so a chord stops being guesswork.
  - Transcription succeeded and nothing was typed, silently: `inject_text` returned `Ok(())` whenever *any* method worked and discarded the rest, and `clipboard` (an in-process call that always succeeds) masked a missing `wtype`/`dotool`. Those tools live only in the nix devShell. Failures are now reported, and `--doctor` says what the missing command implies.
- Accept labwc-style `chord = "A-space"` in config so a binding moves between `rc.xml` and `config.toml` unchanged, and never needs a rebuild.

---

Live work is `TASKBOARD.md`. This file is the ledger: what shipped, in the
order it shipped. Nothing here is a decision still open — settled findings and
the reasons things were deferred both live on the board, because they are
evidence for the next decision rather than history.
