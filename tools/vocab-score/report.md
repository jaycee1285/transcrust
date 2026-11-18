# vocab-score: first run (2026-11-18)

Step 2 of `design-vocab-gate.md`. Deterministic, standard library only, no model.
- **Run:** `nix shell nixpkgs#espeak-ng --command nix develop ~/repos/transcrust -c cargo run
  --release -- ~/syncthing/john-vocab.md --out out/vocab-tiers --commands
  ~/syncthing/dayshade/commands.md`
- **Per-entry tables** are in `out/` (gitignored: they contain family names, and this repo is
  public).
- **Tests:** `cargo test` (4 passing). The mergers collapse cot/caught, merry/marry/Mary,
  pin/pen and latter/ladder, and keep ship/sheep, cut/cat and pit/pet apart. Mater/matter is near
  but not equal. espeak's IPA for "gambol" maps cleanly.

## Method

- **Phonemes:** a `phonemes:` override in the vocab entry (proposed field, none exist yet), then
  CMUdict (every listed variant), then `espeak-ng --ipa` mapped to ARPAbet through an explicit
  table. Unmapped IPA symbols are reported, never silently dropped.
- **Mergers:** `data/mergers.txt` (us-general): cot=caught, Mary=marry=merry before R, pin=pen
  before nasals, t/d flapping between vowels.
- **Distance:** a feature-weighted Levenshtein (voicing, place and manner; vowel height,
  backness, rounding, diphthong, tense/lax) plus a phone-bigram Dice distance, against the 20k most
  frequent English words.
- **Tiers:** NEAR = 1.0, one phoneme of change, **fixed before the first run and not tuned
  afterwards**. When a word has several pronunciations, the scorer judges the one closest to a
  common word (the worst case).

## Canon results (expected without hints: Gambol/Flo/Lightning 0, Mater 1, McQueen 2)

| Entry | Got | Expected | Why |
|---|---|---|---|
| Flo | **0** | 0 | Identical to "flow". |
| Mater | **0** | 1 | CMUdict's third variant `M AE1 T ER0` *is* "matter". Under the worst-case rule that's correct. The family says "MAY-ter" (`M EY1 T ER0`), which would be tier 1. **A `phonemes:` override fixes it; the scorer is right to warn.** |
| Gambol | **1** (0.35 from "gamble") | 0 | It isn't in CMUdict. espeak spells it `ɡˈæmbɑːl` with a full vowel in the unstressed syllable ("gam-bahl"), but natural speech reduces that vowel to schwa, making it identical to "gamble". Cause: **no vowel-reduction rule**. Not patched (see Q1). |
| Lightning | n/a | 0 | The vocabulary has only the phrase "Lightning McQueen", which scores 2: phrases aren't scored word by word yet. Alone, "lightning" would score 0 as itself a frequent word. |
| McQueen | **1** (0.67 from "mclean") | 2 | W vs L in `K W IY N` vs `K L IY N` is a small step. Plausibly a real confusion, not a scorer bug. |

**Verdict:** Flo is right, and Mater is right by its own rule. Gambol misses for one nameable
reason (no vowel reduction). Lightning wasn't actually tested. McQueen is a surprise worth
believing.

## Surprises (the interesting part)

- **`and`, `in`, `it` (and `outfits`) are vocabulary entries,** all `kind: term`, with
  `heard: in`, `heard: and` and `heard: I`. **They are rewrite rules between function words,
  mutually** (and↔in). They score tier 0 because they *are* common words. By the design these
  must never be sound-swapped; only context may touch them. **This is probably the source of
  some live mis-corrections.**
- **Bingo and Chili (Bluey characters) are tier 0:** they're ordinary words. It matters for bloey
  and swiftype: like Flo, they need context or fixed phrases, never sound alone.
- **Void and Rowan** are tier 0 as ordinary words.
- **Nell** is tier 0 via "nel", which is noise in the web frequency list. The real collision is
  "knell" (already in the entry's `ordinary:`), so the conclusion stands for the wrong reason.
- **Cigs** is 0.11 from "figs", and close to "six". That matches the vocab file's own note ("`Cigs`
  would claim every `six`").
- **Syncthing / sinking** sits exactly at the 1.00 threshold, and **Wayland / warrant** is 0.96:
  both are borderline tier 1.
- **Kubernetes, Svelte, transcrust** are tier 2. Those are what Beider-Morse fixes today, so tier-2
  parity is plausible.
- **dayshade command verbs** (`out/vocab-tiers-commands.md`): `cut`, `set`, `count`, `task(s)`
  and `list` have a plosive closing the stressed syllable. `plus` has a plosive onset cluster.
  Dragon's shape holds.

## Licences

- **CMUdict** (`data/cmudict.dict`): BSD-2-Clause-style (© CMU), fine with AGPL. Committed.
  sha256 `81917843c7f44ce2b094ac63873c2c7a4cf802040792c455ba3ca406891c3d22`.
- **20k frequency list** (first20hours/google-10000-english): derived from the LDC Google Web 1T
  corpus via Norvig. "Educational and personal/research use permitted", not for commercial use.
  **Gitignored, fetched by `fetch-data.sh`.** sha256
  `4ed6e5336d7760d281f7e72df31827da880c861363e820d8c65666b0f10d9ac0`. A freely licensed list
  (e.g. wordfreq's data, CC-BY-SA) would remove that caveat if this ever ships.
- **espeak-ng:** GPL-3.0, called as a subprocess at scoring time only, never linked.
- **The feature table** is hand-written from standard phonetics; PanPhon wasn't used.

## Questions for John

1. **Add a vowel-reduction rule** (unstressed full vowels → AH) to the mergers? It's a genuine
   rule of connected speech, not tuning, and it would make Gambol tier 0. *I recommend yes, as its
   own rule with a test, recorded as added after the first run.*
2. **Add `phonemes:` overrides** for Mater (`M EY1 T ER0`) and Gambol (`G AE1 M B AH0 L`) in
   `john-vocab.md`? *I recommend yes for Mater: it's what the family says. For Gambol, rule 1
   should be enough.*
3. **The function-word entries** (`and`, `in`, `it`, `outfits`): keep them, or move them to a
   context-only list? *I recommend context-only. Under this design a sound-based swap between
   "and" and "in" can never be right.*
