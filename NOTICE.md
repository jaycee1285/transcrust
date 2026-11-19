# NOTICE: licensing and provenance

**transcrust** is licensed under the **GNU Affero General Public License v3.0 or later** (see
`LICENSE`), as of 2026-11-17.

## Why AGPL

Earlier transcrust revisions carried two parts derived from **murmure**
(© Kieirra and contributors, AGPL-3.0-or-later,
https://github.com/Kieirra/murmure, read at `8c7a07c`).
The old `crates/core/src/dictionary.rs` used Beider-Morse phonetic vocabulary
correction. The old `crates/core/src/boost.rs` used near-miss gates and alpha
calibration documented against murmure.

The contextual implementation in [design-vocab-gate.md](design-vocab-gate.md)
replaced those two runtime mechanisms in November 2026. The project remains
AGPL and the earlier implementation remains in its history.

## Libraries

- `parakeet-rs`.
- ONNX Runtime via `ort`.

Model weights (Parakeet-ultra, Nemotron streaming) keep their own licences; see `Models.md`.
