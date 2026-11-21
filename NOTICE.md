# NOTICE: licensing and provenance

**transcrust** is licensed under the **GNU Affero General Public License v3.0 or later** (see
`LICENSE`), as of 2026-11-17.

## History

Revisions before November 2026 contained `crates/core/src/dictionary.rs` and
`crates/core/src/boost.rs`, derived from **murmure** (© Kieirra and contributors,
AGPL-3.0-or-later, https://github.com/Kieirra/murmure). Both were removed in
November 2026 and remain AGPL in this repository's history. The current tree
contains no murmure-derived code (checked 2026-11-20, including the direct
Parakeet decode). `murmure-reference.md` keeps research notes only.

## Libraries

- `parakeet-rs`.
- ONNX Runtime via `ort`.

Model weights (Parakeet-ultra, Nemotron streaming) keep their own licences; see `Models.md`.
