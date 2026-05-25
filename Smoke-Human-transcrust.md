# Human Smoke

## 1. Environment check

```sh
cargo run --offline -- --doctor
```

Expected:

- config path prints as `~/.config/transcrust/config.toml`
- the int8 model resolves from `~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int8`
- `wtype`, `dotool`, and `notify-send` show `yes`

## 2. Live daemon

Terminal A:

```sh
cargo run --offline -- --smoke
```

Terminal B:

```sh
tail -f logs/latest.log
```

## 3. What to do

- hold the configured hotkey
- speak a short phrase
- release the hotkey

## 4. Expected phases in `logs/latest.log`

- `startup`
- `recording`
- `transcription`
- `transcription.raw`
- `transcription.postprocess`
- `inject`

Optional quit path from a third terminal:

```sh
cargo run --offline -- --quit
```

## 5. What to report back

- the last phase that appears in the log
- whether you saw a desktop notification for `Recording started`, `Empty transcript`, `Injected: ...`, or an error
- whether text reached the clipboard, the active field, both, or neither
- whether `--quit` cleanly terminates the running process
