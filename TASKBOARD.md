# TASKBOARD

> **Two boards, different jobs.** This one is the historical ledger — what has
> shipped, standing risks, and odds and ends with no phase. **`TASKBOARD-next.md`
> is the live plan**: sequenced Phases A–E with kill criteria and a Deferred
> table. Read that one to decide what to do next; read this one for what already
> happened.

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
- Guard the phonetic dictionary against common English words. `Tauri` was claiming every `they’re` — 7 hits, 0 survivors, deterministic via `--fix`. The guard sits on the source token (commonness, not length: `Tauri` is 5 chars and `they’re` is 7), and exact-match moved to its own pass so an earlier entry’s phonetic neighbourhood can no longer claim a word by line order. Shipped 2026-09-09.
- Replace the unfiltered linear-interpolation resampler with a band-limited polyphase decimator (`TASKBOARD-next.md` A.1/A.2). The aliasing was real and measured; fixing it did not measurably change recognition on codec-limited speech.

## Next
- **Granite as a command channel** — see `TASKBOARD-next.md`. The bet:
  Parakeet needs none of murmure's correction machinery for *dictation* because
  it emits grownup English natively; that machinery is worth pointing at
  commands, where the vocabulary is closed and output shape is irrelevant. It
  runs on Parakeet itself — the logits were always there, `parakeet-rs` just
  hid them. See `Parakeet-v3.md` for why this model fits on-device control.
- If a grammar/punctuation pass is wanted back, build a small purpose-built deterministic one (or the future small-LM toggle) rather than re-adding Harper.
- Add explicit first-load tray/icon feedback so users can see model warmup instead of only paying hidden latency on first transcription.
- Tighten startup/log ergonomics so steady-state smoke logs stay high-signal.
- Verify the `transcrust` `tauri.nix` entry in the config repo against the published release asset.

## Risks
- Pure Nix builds still need scrutiny because `ort` binary provisioning is touchy across environments.
- Tray/status behavior depends on SNI/DBus availability in the running desktop session.
- Feature brittleness remains around cross-session tray rendering and first-use model warmup UX because the repo does not control the status-notifier host theme path.
