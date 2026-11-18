# NOTICE: licensing and provenance

**transcrust** is licensed under the **GNU Affero General Public License v3.0 or later** (see
`LICENSE`), as of 2026-11-17.

## Why AGPL

Two parts of transcrust follow **murmure** (© Kieirra and contributors, AGPL-3.0-or-later,
https://github.com/Kieirra/murmure, read at `8c7a07c`; notes in `murmure-reference.md`):
- `crates/core/src/dictionary.rs`: the Beider-Morse phonetic vocabulary correction (murmure's
  "leg 3"), ported with documented differences;
- `crates/core/src/boost.rs`: near-miss gates and alpha calibration, documented against murmure.

transcrust is public and contains that derived code, so it is AGPL for fairness to murmure's
authors.

## Planned replacement

Both legs are scheduled to be replaced by John's own design, `design-vocab-gate.md`:
- the command gate;
- vocabulary applied only in open slots;
- tiered entries;
- an uncertainty trigger;
- an explainable homophone chain.

See the TASKBOARD item "Retire the murmure-derived legs". The licence stays AGPL afterwards,
because earlier history contains the derived code.

## Libraries

- `rphonetic` (Beider-Morse rules, Apache-2.0).
- `parakeet-rs`.
- ONNX Runtime via `ort`.

Model weights (Parakeet-ultra, Nemotron streaming) keep their own licences; see `Models.md`.
