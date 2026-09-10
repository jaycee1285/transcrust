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
- Route the live Parakeet path through the direct ORT driver, with a crate fallback when `nemo128.onnx` is absent (`TASKBOARD-next.md` B.3). RTF 0.11 against 0.17, and per-word confidence now reaches the live path.
- **The input/output path had no validation, and a human smoke found two bugs no test could have.** Neither is a logic error; both are environment and integration, which is exactly the class a unit test cannot see:
  - `Space + LeftAlt` typed into the focused window for the whole hold, because transcrust reads evdev passively and cannot suppress a printable key. Default is now modifier-only; `--doctor` warns; `hotkey.grab` is an opt-in partial mitigation; `--keys` shows the live stream so a chord stops being guesswork.
  - Transcription succeeded and nothing was typed, silently: `inject_text` returned `Ok(())` whenever *any* method worked and discarded the rest, and `clipboard` (an in-process call that always succeeds) masked a missing `wtype`/`dotool`. Those tools live only in the nix devShell. Failures are now reported, and `--doctor` says what the missing command implies.
- Accept labwc-style `chord = "A-space"` in config so a binding moves between `rc.xml` and `config.toml` unchanged, and never needs a rebuild.

## Next
- **The command channel moved.** It is now `~/repos/dayshade/spec.md` — a Quick
  Settings tile on Android that writes DayLight Markdown directly. What stays in
  this repo is `TASKBOARD-next.md` **E.1**, the decode-time boost tree, which was
  never really about commands: it is vocabulary biasing, and it is the answer to
  the proper-noun errors this repo actually has.
- **Ship what is built.** Five commits are local-only and the published release
  asset is now seven behind: `./release.sh`, then
  `git push -u origin engines-and-corpus`, then check the `tauri.nix` entry in
  the config repo against the new asset.
- **Turn on `observe.corpus`.** It is `false`, so nothing is banked and the
  corpus is empty. C.2, C.4, the A.2 re-run against real microphone audio, and
  Dragon mechanism 1 all wait behind one boolean.
- **Bump yt-dlp.** The system binary is `2025.12.08` against nixpkgs
  `2026.08.19`, and the stale one 403s on every download — so anything shelling
  out to it is broken, not just the new skills.
- If a grammar/punctuation pass is wanted back, build a small purpose-built deterministic one (or the future small-LM toggle) rather than re-adding Harper.
- Add explicit first-load tray/icon feedback so users can see model warmup instead of only paying hidden latency on first transcription.
- Tighten startup/log ergonomics so steady-state smoke logs stay high-signal.
- **Log path is cwd-relative.** `observe.rs` builds `./logs/latest.log` from
  `current_dir()`, so a tray- or systemd-launched instance writes somewhere
  arbitrary and `--doctor` does not say where. `~/.local/state/transcrust/` is
  the XDG-correct home; two lines plus a `--doctor` line.

## Risks
- Pure Nix builds still need scrutiny because `ort` binary provisioning is touchy across environments.
- Tray/status behavior depends on SNI/DBus availability in the running desktop session.
- Feature brittleness remains around cross-session tray rendering and first-use model warmup UX because the repo does not control the status-notifier host theme path.
