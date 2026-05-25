# Human Smoke 2026-03-18

## What changed in this pass

- Promoted the working Transcrust app from `parakeetvox/` into the repo root.
- Kept the app on the matched `ort 2.0.0-rc.10` stack with the vendored TDT-only `parakeet-rs` path.
- Restored tray state, `--smoke`, and `--quit`.
- Fixed model resolution so the app accepts the local int8 layout in `~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int8`.

## Commands John actually ran

```sh
cargo run --offline -- --doctor
cargo run --offline -- --smoke
cargo run --offline -- --quit
```

## What John observed

- `--doctor` resolved the model from:
  `~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int8`
- `--smoke` ran with the int8 path and the process sat around `1.3 GB` resident, which was considered acceptable relative to prior fp32 behavior.
- Tray icons worked through the expected state changes.
- Transcription worked across GUI and terminal apps.
- `--quit` successfully terminated a live smoke instance once the pid-file path handling was fixed.

## Passed

- Local-share int8 model resolution
- Working transcription with the Parakeet int8 model
- Tray icon/status transitions
- Smoke-mode operator loop
- Quit path against a live process

## Still worth watching

- Pure Nix packaging still needs scrutiny around runtime distribution and wrapper behavior.
- Tray behavior still depends on the running desktop session's SNI/DBus support.
- The vendored `parakeet-rs` fork is intentionally narrow and should be treated as an owned runtime surface.
