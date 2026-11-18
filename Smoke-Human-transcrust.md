# Human Smoke

## Family pipeline — November14, new human check pending

Installed f14e3a110e8afb41cacc3cb767af069e1a80f234ab4853aa66f0be79246a4ae2.
Nemotron selected; Print toggles; Ctrl+Alt+C. New review shows all changed spans,
optional unchecked learning, durable per-take/revision history and corpus links.
Nemotron starts during capture. Nine frozen recordings replayed through both
engines; same-engine raw parity8/8, dog repair retained, legacy grammar/casing
edits removed. Paced red-racer tail0.111s + DEC0.003s; not a microphone smoke.
75 core tests, history/audio/decision regressions and release build pass.

Observed earlier replacement log caught a one-grapheme shortfall and verified
the full137-grapheme saved span before typing. No fixed extra deletion used.
Human confirmation of full review, Replace/Copy, retention after another take,
new glossary learning and live latency remains open. [Report and replay command](pipeline-2026-11-14.md).

## Correction replacement prefix — November 13

**Follow-up:** John still sees a leading character after the focus-settle
patch, including `MMy son...`; the earlier timing hypothesis did not resolve
the bug. Latest patch uses saved `buffer_text` as the replacement contract:
copy the destination selection back, require exact text including final
punctuation, extend only when the copied selection is an exact shorter suffix,
and refuse replacement on unrelated/empty selection. Corrected clipboard text
is restored on success/failure. Engine state advances only after verified
selection and successful typing. `correct.selection` logs verification and
missing-grapheme counts. Regression tests cover exact/suffix/mismatched/empty
and Unicode selections. Human replacement acceptance remains pending.

John reports the same four-character gap with both engines. Submitted result
starts `We'rWe're gonna...`: the full correction was typed, but four original
characters remained. Persisted corrected/buffer text has the full correct
prefix, so this is destination selection/delivery, not ASR truncation.
Replace now allows 150ms for focus restoration before selecting backward and
uses a separate literal-text wtype invocation after successful selection.
No count offset or extra deletion is introduced. Live validation remains pending.

Nemotron take `2026-11-14T00-18-51Z` is 35.67s, ASR11.44s + DEC2.24s,
13.68s to typing. DEC kept `in` and abstained on `flow` (P(name).334).
Root starts the worker only after recording stops; the “encoding while
recording” worker log overstates actual overlap on this route. Fix remains open.

## Zed first dictation/correction — November 13 local, rough human result

John tried both and reports “That was brutal.” Banked take:
`2026-11-14T00-12-36Z.wav` + JSON, 41.69s. ASR 4.24s, decisions 1.54s,
5.79s before typing begins. Raw has transcest, news ed, O'Ryan, flower and
“in a bath”; decisions considered only lightning McQueen/Mater and changed
those two. User corrected the text; log records successful replacement.
This is not a quality acceptance. Audio and last-dictation corrected text are
retained for replay; do not require a rerecord. Which part of the experience
was most painful still needs clarification.

Correction also persisted broad learned proposals including `and: heard in`
and `outfits: heard outfit`, plus `Flo or: heard flower`. These are present in
the shared glossary after the edit, not approved general linguistic rules.
No glossary entries were removed or prompts/models retuned during inspection.

## Zed AMD migration — 2026-11-13, human result pending

Ctrl+Alt+C is now persistent in `[hotkey].correct_chord = "C-A-c"` in the
user config. The live daemon was restarted with this binding; no terminal is
needed to open correct-last. Human shortcut/replacement check remains pending.

Fresh locked app-only build matches November 11 binary and inference output.
Parakeet/Nemotron/Granite ASR and UltraBERT/D1 loaded on Zed. Keyboard, audio
discovery and empty wtype request succeeded. Live `transcrust-zed-smoke.service`
has default Parakeet, --dec --smoke, shared glossary and audio banking; tray
registration verified. Synthetic timed correction window logged no EGL/glutin
startup error. This is machine verification, not visual/human acceptance.

Hold Print, speak into an editor, release; check text, wait and injection.
Use tray correct-last or the explicit local executable with `--correct`;
check GTK colors and Replace/Copy. [Restart commands and migration findings](migration-2026-11-13.md).

## Installed app after build/model cleanup — 2026-11-11

Fresh app-only release built offline; installed executable matches its build
hash. CLI Parakeet WAV transcript and UltraBERT Gamble→Gambol correction match
the preserved pre-cleanup executable. Models load from their installed homes
outside target. These are machine checks, not human acceptance.

Startup, installed Ultra loading and tray registration verified. Live launcher:
`~/.local/libexec/transcrust/launch-smoke`, running through
`transcrust-cleanup-smoke.service`. Default Parakeet, UltraBERT `--dec`, audio
banking enabled by `--smoke`. Hold Print, dictate into an editor, release;
check delivered text/delay, then tray correct-last and Replace/Copy. Human
result pending. Stop the unit before launching a second instance manually.

## Twelve-second phoneme read — 2026-11-06

John stopped after four paragraphs because the tongue-twister script was too
hard to read naturally. Captured122.24s, intended excerpt200 words. Same audio:
6s gives28 windows,12s gives12; both match C on all windows. Script disagreement
22→14 (or16→10 with recognized paragraph markers removed), not verified WER.
Twelve improves several phrases and regresses others. Live ASR tail1.605s,
zero DEC candidates;1.62s to starting injection, additional time to finish it.
Keep12 for ordinary dogfood; no need to finish this script. Human quality
acceptance remains open. [Full comparison](tools/phonon-probe/script-2026-11-06.md).

## Saved Phonon take — 2026-11-06

35.84-second capture banked as `2026-11-06T19-11-20Z.wav` + JSON in the corpus.
All seven original windows reproduce the live raw transcript and match C text
exactly. Two larger 20-second windows also match C but introduce other name
errors; no window-size change is promoted. Decisions made only Gamble→Gambol.
Original post-release wait was 11.20 seconds: 9.52 ASR + 1.67 decisions.

Root dispatch now starts Phonon at microphone press and tees exact corpus audio
alongside inference. Original decoding/glossary remain unchanged. Forwarding
and banking unit tests pass. Paced same-audio replay preserves the exact raw
transcript and original audio, and reduces post-release ASR wait from 9.517s to
1.490s (decision/delivery excluded). Live human acceptance remains pending.
[Evidence and replay](tools/phonon-probe/smoke-2026-11-06.md).

## Native Phonon 2.5 — failed first human smoke (2026-11-04)

John recorded four takes and reports it is markedly worse. Raw outputs already
contain the bad wording; all four decision passes had zero candidates. Audio
banking had been left disabled, so these takes cannot be replayed. That was a
smoke setup failure. Original logs preserved at
`/tmp/transcrust-phonon-smoke-2026-11-04.log` and
`/tmp/transcrust-decisions-before-audio-banking.jsonl`.

Corpus is now enabled in John's config; `--smoke` also enables it automatically
and logs its destination on startup. Next capture must leave WAV + JSON before
claiming replay evidence. Quality cause is unresolved. Logs also show the root
pipeline invokes the worker after capture stops, so the adapter's window work
was not hidden under recording; earlier during-capture claims were incorrect.

Native Rust desktop test runs as `transcrust-native-phonon.service`, with
Phonon 2.5 Toggle selected and UltraBERT `--dec` enabled. Tap Print to start,
dictate into an editor, tap Print again to stop. Check text, post-stop delay and
correct-last. Tray registration and startup were verified; that is not human
acceptance. [Implementation, measurements and launch command](crates/phonon/README.md).

Mopvox's separate live Phonon 2.5 IME was accepted by John as working and
remarkably fast. This does not establish desktop or shared mobile-decision acceptance.

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
| 2026-11-01 default selection | John requested UltraBERT INT4 integration and “make it the primary for now” | Explicit desktop dogfood default change; new UltraBERT live human smoke is pending |

The live desktop evidence remains in ignored `logs/latest.log` and
`logs/decisions.jsonl`; the observations above survive log rotation. Log timestamps
and the old startup message identify the serial implementation. Do not credit
native batching with tests run before that port. No new live GUI smoke was run
during this documentation reconciliation.

## Contextual decisions — UltraBERT INT4 primary dogfood (2026-11-01)

John requested UltraBERT as primary for now. This changes the default backend
for `--dec`; it does not record human acceptance of this new backend. D1's
earlier acceptance remains in the ledger above. Live dictation/injection and
correction-window smoke for UltraBERT are **pending John's test**.

Stop the old daemon through tray Exit, then:

```sh
nix develop -c cargo run --release -- --dec --smoke
```

Confirm startup says `UltraBERT INT4 ready`. Use the normal push-to-talk binding.
The INT4 model stays loaded natively in Rust, all
candidate occurrences run in one batch, decoder boosting is off,
and the legacy vocabulary replacement pass is bypassed. Say the dog/casino,
Rowan/Nell, Cars and Bluey examples. Check `dec.choice` and `timing` in the terminal;
zero-candidate dictations should add no model inference. Correct mistakes through
the existing tray/chord interface and keep a new pair to check next-utterance
candidate reload. Edit meanings and aliases in `~/syncthing/john-vocab.md`
(or `[vocabulary].path`); deleting an alias there removes its candidates.
Full local audits now include backend identity: `logs/decisions.jsonl`;
contract and machine checks: `design-dec.md`. Ultra receives marked sentence
context and the relevant term's descriptions; the global glossary is omitted.
For the D1 baseline, restart with `--dec-backend d1` instead of `--dec`.

Machine checks for this promotion: optimized suite 151 pass / three ignored;
the explicit real-model test separately passes all 32 saved Q4 distributions
and pre-start/in-flight cancellation. Debug functional tests pass apart from
the unchanged resampler timing assertion; the release timing test passes.
These checks do not replace the live human dictation/injection smoke.
The built release binary's `--fix --dec` also loaded UltraBERT and repaired
Gamble → Gambol using the actual shared file; explicit D1 kept ordinary gamble.
Missing-bundle input was preserved and an invalid selector was rejected. The
running tray daemon was left for John's explicit live launch below.

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
