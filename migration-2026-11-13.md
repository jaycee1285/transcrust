# Backup laptop migration — Zed, November 13

Reference host: AMD Ryzen 7 PRO 6850U, x86_64, NixOS, labwc/Wayland.
The previous Intel executable runs on this CPU; models require no conversion.
The app-only release rebuild passed and matches the migrated executable.
No model weights were downloaded, quantized or changed.

## Migration findings

- Parakeet Ultra encoder, decoder/joint and nemo128 hashes match Models.md.
- Installed UltraBERT Q4 hash matches
  `75ffd6cdf6c147ecaceab1878e388701d4feadcf69bfc8bd88a59cbd69ab8869`.
- Shared glossary `~/syncthing/john-vocab.md`, app config, GTK palette, corpus,
  D1 Q8, Granite, Nemotron and Phonon directories are present. Model symlinks
  point into the migrated Syncthing tree.
- UltraBERT FP32 reference cache did not migrate. It is optional research data;
  live decisions load the installed Q4 bundle. Recreate only when needed using
  the pinned revision and commands in Models.md.
- Cargo sources and build outputs did not migrate. The initial offline build
  failed resolving cpal. The replacement uses the existing lockfile:
  `nix develop -c cargo build --release --locked -p transcrust --bin transcrust`.
  Future builds can use `--offline` once dependencies are cached.
- The real desktop user belongs to input; `/dev/input/event0` is the internal
  AT Translated Set 2 keyboard. No old device path is pinned. Print is freed
  in labwc. Empty `wtype` request succeeds without typing text; no uinput
  permission change is required for the normal Wayland injection path.
- PipeWire selects the AMD digital microphone; app discovery reports default
  44100 Hz stereo and keeps its existing channel-0/resample policy. Old Intel
  configured-default identifiers remain in WirePlumber, but current fallback
  selects the AMD source. Microphone intelligibility needs the human smoke.
- The profile `transcrust` executable is older than the retained November 11
  executable. Autostart and Super+/ now select the verified local install via
  `~/.local/libexec/transcrust/launch` (`--dec` enabled). Original labwc files
  are retained beside them with `.before-transcrust-20261113` suffixes.

## Baseline inference

The migrated November 11 executable transcribes `tools/bench-clips/chat-05s.wav`
as “I feel like my second home some fans here. We love you feel the love”,
matching the November 11 baseline. D1 loads its manifest-verified Q8 bundle
and repairs the isolated dog-name sentence.

UltraBERT loads and repairs the isolated dog name, but a separate two-sentence
probe, “My dog Gamble loves her frisbee. We went to the casino to gamble.”,
changes both occurrences to Gambol (P(name) .891/.612). This is an observed
preservation failure, not evidence of AMD incompatibility. Keep it as a quality
case; do not retune prompts/models while establishing migration parity.

Build and runtime logs are under ignored `logs/migration-20261113/`. Permanent
results belong here and in Smoke-Human-transcrust.md; human acceptance remains
pending until John tries microphone, injection and correct-last on Zed.

## Completed verification and live handoff

Latest saved-selection replacement build, superseding both binaries below:
SHA256 `0b04cb180b82056f82f1c606f37a2bf617559220c39a745acb1fc56ef9686fb1`.
Core correction regression tests passed (4/4); app release build passed.
Installed and restarted. Replacement copies the active editor selection back,
compares it with saved destination state, extends verified missing prefixes
and requires exact text before typing. Human acceptance remains pending.

Follow-up correction-delivery build: SHA256
`8640e4785b7924c905c468440d6064c8820b35cacf2ba068ad1f5823df1e6c80`.
This supersedes the initial migration binary below. Adds 150ms focus settle
before backward selection and separates literal replacement typing. Release
build passed; installed and restarted for Nemotron human smoke. Prefix fix
acceptance is pending. See Smoke-Human-transcrust.md and TASKBOARD regressions.

Correction shortcut: Ctrl+Alt+C, persisted in the user config under
`[hotkey] correct_chord = "C-A-c"`. Daemon restarted with the binding enabled.

- Fresh app-only locked build passed in 7m45s; no runtime code changes.
  Build and installed executable SHA256 both match the November 11 executable:
  `2ae7008cb62b03decdeb8def1d0f1bede3bc99989971935c78fe598e4a14aedc`.
- Fresh Parakeet transcript and both UltraBERT decisions match the migrated
  executable exactly. Nemotron and Granite also transcribed the same clip.
  The mixed casino sentence failure reproduces with identical probabilities.
- Stable installation remains outside Cargo output. Original pre-cleanup
  executable is retained. Repository after rebuilding is about 1.31 GiB;
  no research weights were placed in target.
- Live `transcrust-zed-smoke.service` is active, default Parakeet, `--dec
  --smoke`, with the shared glossary and corpus enabled. Tray registration
  verified through StatusNotifierWatcher. No prior live daemon was killed.
- A synthetic read-only correction window ran briefly under timeout using
  the pinned Mesa EGL driver; no EGL/glutin startup errors were logged.
  Visual appearance, real microphone/injection and actual replacement still
  require John's smoke. GTK palette remains the migrated gtk.css.

To restart in a terminal for the human check:

```sh
cd /home/john/repos/transcrust
systemctl --user stop transcrust-zed-smoke.service
nix develop -c /home/john/.local/libexec/transcrust/transcrust --dec --smoke
```

Hold Print, speak into an editor, release. Check text and latency. In another
terminal, from this repo:

```sh
nix develop -c /home/john/.local/libexec/transcrust/transcrust --correct
```

Check the window/palette, edit, and Replace with the editor caret at the end of
the previous dictation. `--pick` against the same executable opens the mode
picker. The general profile command `transcrust` is still the older Nix package;
updating that package remains a separate packaging task.
