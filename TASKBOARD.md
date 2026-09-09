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
- **Granite as a command channel** — see `TASKBOARD-next.md`. The bet:
  Parakeet needs none of murmure's correction machinery for *dictation* because
  it emits grownup English natively; that machinery is worth pointing at
  commands, where the vocabulary is closed and output shape is irrelevant. It
  runs on Parakeet itself — the logits were always there, `parakeet-rs` just
  hid them. See `Parakeet-v3.md` for why this model fits on-device control.
- **Guard the dictionary against common English words.** Settled 2026-09-09 by a
  live collision: `Tauri` in `~/.config/transcrust/dictionary.txt` was claiming
  every `they're` — 7 hits and 0 surviving `they're` across one transcript, 100%
  and deterministic via `--fix`. `there` and `their` were unaffected, so only the
  contraction collides. `Tauri` has been removed from the local dictionary; the
  code guard is still open.
  - **Minimum entry length is the wrong axis** and this kills that idea: `Tauri`
    is 5 chars, `they're` is 7. The axis is *commonness* — the design assumes
    entries are rare jargon, so the guard belongs on the **source token**: skip
    the phonetic swap when the word the model emitted is already common English.
  - This is murmure's failure mode inverted — the corrector claiming a common
    word instead of rescuing a rare one. `murmure.md` covers the intended
    direction; this is the one it does not.
  - Whatever the guard, keep the dictionary file itself raw and hand-curated
    (documented in `dictionary.example.txt`).
- If a grammar/punctuation pass is wanted back, build a small purpose-built deterministic one (or the future small-LM toggle) rather than re-adding Harper.
- Add explicit first-load tray/icon feedback so users can see model warmup instead of only paying hidden latency on first transcription.
- Tighten startup/log ergonomics so steady-state smoke logs stay high-signal.
- Verify the `transcrust` `tauri.nix` entry in the config repo against the published release asset.

## Risks
- Pure Nix builds still need scrutiny because `ort` binary provisioning is touchy across environments.
- Tray/status behavior depends on SNI/DBus availability in the running desktop session.
- Feature brittleness remains around cross-session tray rendering and first-use model warmup UX because the repo does not control the status-notifier host theme path.
