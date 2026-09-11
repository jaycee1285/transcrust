# Human Smoke

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
