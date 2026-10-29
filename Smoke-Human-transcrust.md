# Human Smoke

## Recorded acceptance and observations (reconciled 2026-10-28)

| Date / path | Evidence | Interpretation |
|---|---|---|
| 2026-10-06 Linux review | John: “Smoke clears”; replacement/correction flow smooth; GTK FG/BG/accent also passed | GUI accepted; this is not vocabulary-quality or Apple acceptance |
| Following weighting smoke | Ordinary gamble became Gambol; later dog Gamble / son Rowan increased the error count | Bounded decoder weighting is implemented but failed that quality gate |
| 2026-10-10 live `--dec`, serial adapter | Parakeet and Nemotron delivered text; two dog-name occurrences repaired at P(name) .981/.961; single dog/son case repaired at .551; zero-candidate take added 32 ms | Initial live path was exercised; do not label all human testing “not done” |
| Same serial session latency | Two candidates added about 2.17s; one added about 1s | John rejected that latency; this prompted native Rust batched inference |
| 2026-10-10 native port | Batch/single/saved-Q8 parity and ORT cancellation/recovery passed; native two-name text probe took 1.52s | Machine evidence; no recorded post-port live acceptance yet |
| 2026-10-27/28 Android | Phonon/Parakeet comparison summaries in `~/syncthing/mopvox/phonon-trial/results/` | Separate ASR experiment, no phone D1 correction inference yet |
| 2026-10-29 native live feedback | John: corrections “really good”, delay “not too bad” but slower than desired; “really close” | Positive native dogfood evidence; preserve Q8 quality. Detailed preservation/reload checks and latency work remain open |
| 2026-10-29 acceptance | John: “We can accept the test as is for now”; mobile delay remains a concern | Desktop native Q8 accepted for current dogfood. Mobile readiness is a separate future gate; no further desktop smoke repetition is required now |

The live desktop evidence remains in ignored `logs/latest.log` and
`logs/decisions.jsonl`; the observations above survive log rotation. Log timestamps
and the old startup message identify the serial implementation. Do not credit
native batching with tests run before that port. No new live GUI smoke was run
during this documentation reconciliation.

## Contextual decisions — desktop native Q8 accepted for now; mobile latency open

Stop the old daemon through tray Exit, then:

```sh
nix develop -c cargo run -- --dec --smoke
```

Use the normal push-to-talk binding. Q8 stays loaded natively in Rust, all
candidate occurrences run in one batch, decoder boosting is off,
and the legacy vocabulary replacement pass is bypassed. Say the dog/casino,
Rowan/Nell, Cars and Bluey examples. Check `dec.choice` and `timing` in the terminal;
zero-candidate dictations should add no model inference. Correct mistakes through
the existing tray/chord interface and keep a new pair to check next-utterance
candidate reload. Edit meanings and aliases in `~/syncthing/john-vocab.md`
(or `[vocabulary].path`); deleting an alias there removes its candidates.
Full local audits: `logs/decisions.jsonl`; contract and machine checks: `design-dec.md`.

The working headless path still makes semantic errors with the compact starter;
report actual quality and tolerable delay. This is the experimental live path,
not accepted vocabulary accuracy or Apple interface acceptance.


## Correct last dictation — current slice (2026-10-06)

**Accepted by John (2026-10-06):** “Smoke clears”; correction flow worked
smoothly after the Mesa/glibc runtime fix. GTK palette also passed the next
human smoke. Foreground/background/accent come from
`$XDG_CONFIG_HOME/gtk-4.0/gtk.css` (normally `~/.config/gtk-4.0/gtk.css`), read
each time the correction window opens.

**Learning limitation found in follow-up:** `heard: gamble` under Gambol
unconditionally replaced the verb in “play poker and gamble with cards”.
The user requires vocabulary weighting rather than global substitution;
the interface smoke did not accept those learning semantics. Bounded weighting
is now wired in direct Parakeet and Nemotron, with forced learned replacements
removed. Weighting
acceptance must preserve both Gambol (the dog) and gamble (the verb) in one
dictation, alongside the successful Rowan, transcrust and Nell examples.

Stop any old daemon with its tray Exit first. From this repository:

```sh
nix develop -c cargo run -- --smoke --correct-chord C-A-c
```

Dictate into an ordinary editor using the existing push-to-talk binding.
Leave the caret after the transcript; press **Ctrl–Alt–C** (or right-click the
tray → **Correct last dictation…**). The latest delivered text is selected in
the review editor. Fix a name, confirm the caret is still after the dictation,
and click **Replace** or press **Ctrl+Enter**. For each short replacement,
choose **Keep** or **Drop**, then **Apply & replace**. Dictate the same name
again; the kept spelling is now a decoder preference, loaded without restarting.

For the weighting smoke, say: “I love my dog Gambol. I like to gamble and play
poker with cards. My son's name is Rowan. This project's Linux version is called
transcrust. My son Rowan's mother's name is Nell.” Gambol and gamble must both
survive. Test the mode you use; weighting currently supports direct Parakeet and
Nemotron. To compare without weighting, restart with
`TRANSCRUST_VOCAB_BOOST=0 nix develop -c cargo run -- --smoke --correct-chord C-A-c`.
The terminal reports armed token paths and changed near-miss token counts.

Also check:

- Enter inserts a newline; multiline replacement preserves the surrounding text.
- Escape/Cancel leaves the original editor and vocabulary untouched.
- Copy saves the correction and copies it without changing the original buffer.
- Reopening after a successful replacement replaces the current corrected span,
  rather than deleting the original character count again.
- After restarting, the saved correction is available for Copy; automatic
  replacement is disabled because the destination/caret is no longer known.
- A delivery failure reopens the correction with an error and the edited text.

Linux's replacement contract is deliberately narrow: closing the review returns
focus to the original editor, with its caret still at the end of the transcript.
Wayland placement belongs to the compositor; the window's app ID is
`transcrust-review`. Center it with a labwc window rule if its placement policy
does not already do that. This smoke does not certify the Apple interaction.

For a compositor binding instead of the daemon-side chord, execute
`/home/john/repos/transcrust/target/debug/transcrust --correct`; the daemon
retains the Nix environment needed by its review child. Vocabulary defaults to
`~/syncthing/john-vocab.md`; `[vocabulary].path` accepts another absolute path.
Only the last text record is saved, at
`~/.local/state/transcrust/last-dictation.json`; corpus audio stays opt-in.

Report whether focus returned to the right editor, the old span was replaced
exactly, and the next dictation used the kept correction.

Standing smoke for the app as it is now. Per-pass records live in
`Smoke-Human-<date>.md`; vocabulary-learning acceptance criteria live in
`Smoke-Human-vocabulary.md`.

## 1. Environment check

```sh
cd ~/repos/transcrust && nix develop -c cargo run --offline -- --doctor
```

Expected:

- config path prints as `~/.config/transcrust/config.toml`
- `Engine:` names whichever family resolved
- `Installed models (N):` lists every discovered model, `*` on the default.
  With both installed you should see Parakeet and Granite; Parakeet wins when
  nothing is pinned
- `Post-processing: shared by both engines`
- `Phonetic dictionary:` shows an entry count, not `absent`
- `wtype`, `dotool`, `notify-send` show `yes`

## 2. Post-processing without a microphone

```sh
nix develop -c cargo run --offline -- --fix "i uh deployed tori to the the kubernetis cluster on wayland period"
nix develop -c cargo run --offline -- --fix "I uh deployed Tori to the the Kubernetis cluster on Wayland."
```

Expected: **byte-identical output** from both —
`I deployed Tauri to the Kubernetes cluster on Wayland.`

The first line is Granite-shaped (bare lowercase CTC), the second
Parakeet-shaped (already cased and punctuated). Identical output is the engine
parity check, and it needs no audio.

## 3. Granite without a microphone

```sh
nix develop -c cargo run --offline -- --granite-smoke ~/.local/share/transcrust/models/granite-speech-5.0-470m-turboctc
```

Expected: `Granite smoke passed`, a short synthetic-audio transcript, and a
`post-processed:` line below it. On a 440Hz sine the transcript is usually
`"you"` → `"You"`; the content is meaningless, the point is that the frontend,
ONNX, CTC, tokenizer, and the shared post-processor all ran.

## 4. Live daemon

Terminal A:

```sh
cd ~/repos/transcrust && nix develop -c cargo run --offline -- --smoke
```

Terminal B:

```sh
tail -f logs/latest.log
```

Then: hold the hotkey, speak a short phrase, release.

Expected phases:

- `startup`, plus one `found <label>: <path>` line per installed model
- `recording`
- `transcription`
- `transcription.raw`
- `transcription.postprocess`
- `inject`

## 5. Tray

- icon changes through Idle → Recording → Transcribing → Complete → Idle.
  Complete is a **checkmark**; if it is a generic cog, the icon name is missing
  from the active theme and the pixmap fallback did not fire
- tooltip names the active engine
- with two or more models installed, right-click shows an **Engine** submenu.
  Switching moves the radio and it **stays** moved when you reopen the menu.
  A switch during transcription is refused with a "Busy" notification

Quit from a third terminal:

```sh
nix develop -c cargo run --offline -- --quit
```

## 6. What to report back

- the last phase that appears in the log
- whether §2 produced two identical lines
- whether the Complete icon was a checkmark or a cog
- whether the Engine radio stuck after switching, and whether the first
  utterance on a newly selected engine was slow (it pays the model load)
- whether text reached the clipboard, the active field, both, or neither
- whether `--quit` cleanly terminates a live instance
