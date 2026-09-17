# Keyword boost — handoff PRD

Decode-time biasing toward John's own vocabulary, for transcrust's Parakeet and
Nemotron decoders. This is `TASKBOARD.md` **E.1**; read that entry and
`murmure-reference.md` first. This document adds the evidence, the vocabulary
source and the acceptance tests.

## Why

The errors that cost a re-read are names, and no engine swap fixes them.
Measured 2026-09-13 on `~/syncthing/Record-2.wav` (John's voice listing
DayLight habits and tasks, recorded 2026-09-10). Fixtures and reports are in
`fixtures/record-2-2026-09-12/margins/`.

| Compared | Median p1, agree | Median p1, disagree | AUC (p1) | Permutation p |
|---|---|---|---|---|
| Nemotron int8-dynamic vs int8-static, 560 ms | 0.88 / 0.86 | 0.46 / 0.37 | 0.90 / 0.91 | 0.0011 / 0.0008 |
| Nemotron 560 ms vs 1120 ms | 0.88 / 0.83 | 0.47 / 0.54 | 0.82 / 0.81 | 0.0045 / 0.0024 |

- **The words that flip between runs are the model's least confident, and they
  are his task names.** `Terakeet` came out as `terrake` (Parakeet),
  `parakeet` (int8-dynamic) and `tarakeet` (int8-static): every engine and
  export missed it. Others: `Apptrack start` → `apt track` / `at track start`,
  `Incog clear` → `incogre`.
- **It's significant on this recording.** One-sided permutation p is
  0.0008–0.0045 over 20,000 shuffles. The four comparisons share one recording;
  multiplied by 4, every p is still below 0.02.
- **Low p1 means a close runner-up**, which is E.1's premise: *"in greedy
  decoding only near-misses are recoverable."*
- **A confidence gate alone is not the fix.** p1 < 0.75 catches the
  disagreements but flags 37–45% of all words.
- **What it does not show:** whether this holds on other recordings, or how
  often boosting misfires on prose. Only acceptance test 2 answers the second.

### On E.1's "do not start before C.2" gate

The gate exists so the kill criterion (false positives on prose) can be
evaluated, and the corpus is still empty. Record-2 can't measure false
positives either. It does establish, significantly, that the target errors
exist and are near-misses. Agent's assessment: that is enough to build, but
not to ship. Acceptance test 2 on prose is the shipping gate, and it doesn't
need C.2 if the prose clip checks out. Update the E.1 entry if John accepts
that.

## The vocabulary John already maintains

There is no step between dictation and injection where John could approve or
correct text, so corrections cannot flow back through the UI. The boost list
has to come from files he already edits, and reload without a restart.
`dictionary.txt` today loads once per process (`LazyLock`).

**Sources, merged at load:**

1. **DayLight: every file with an entry in the last 7 days.**
   - Vault: `~/syncthing/DayLight/Tasks/*.md`. The filename is the title; there
     is no title field (see the daylight skill).
   - An entry is a `timeEntries` date (`  - date: YYYY-MM-DD`) or a
     `habit_entries` date (`  YYYY-MM-DD: n`) inside the window.
   - Skip `*.sync-conflict-*` and `*.bak*`. Strip a trailing ` (1)` / ` 2`
     duplicate suffix (`Homework (1)` → `Homework`).
   - Measured 2026-09-13: **46 titles** in 7 days (all 4 habits included), alpha
     2.54. Fallback if that proves noisy: last 3 days, **22 titles**, alpha 2.86.
     All 1,063 files would drop alpha to ~1.17 and arm far more first tokens.
   - Record-2's spoken tasks are almost all in the 7-day set (`CHB`,
     `Incog clear`, `Outfit and snack`, `GSC Check`, `Terakeet 1p phone 30m`,
     `Apptrack start`, `Orion Laundry`, `Grocery list update`,
     `DMV 28 Munch Road…`), so the fixture tests the real list.
2. **`~/.config/transcrust/dictionary.txt`**, the existing hand list, one entry
   per line. Reuse it rather than adding a file. C.5 already wants one list
   feeding dictionary, boost and router.

**Titles carry digits and punctuation** (`Bath and Candies 8, BR reminder`,
`DMV 28 Munch Road, beginning 830 a.m.`, `dinner 6`). Nemotron spells numbers
out and Parakeet mixes both forms, so boost the **words**. Drop digit tokens
and punctuation from each phrase.

**Reload:** rebuild at the start of each capture when any source mtime changed
or the date rolled over, since the 7-day window moves. No daemon restart.
Apply the same fix to `dictionary.rs` while there.

**Tokenise per engine:** Parakeet `vocab.txt` has 8,193 pieces and Nemotron
`tokens.txt` has 1,024, so they split a phrase differently. Build one trie per
active engine.

## Where it plugs in

- **Parakeet:** `parakeet_ort.rs::decode_greedy`. `split_at(self.vocab_width)`
  separates vocab from duration logits; add the boost to vocab logits before
  `argmax`. The duration head is untouched.
- **Nemotron:** `nemotron.rs::decode_chunk`, before `argmax`. The trie position
  must carry across chunks in `StreamState`, like the decoder state, or a
  phrase split by a chunk boundary loses its boost. `TokenScore` (p1, p2) is
  already recorded per token there.
- Never boost blank.

## Algorithm

Implement from the design, **not murmure's source**. murmure is
AGPL-3.0-or-later and transcrust has no licence; see `murmure-reference.md`.
Weighted Aho-Corasick over subword tokens, per NeMo GPU-PB. Constants as
recorded there:

| Constant | Value |
|---|---|
| Top-K gate at phrase start | 5 |
| Relaxed gate once engaged | 20, after depth 3 |
| Depth scaling | 2.0 |
| Alpha | `(3.5 − log10(n / 5)).clamp(1.0, 3.5)`, n = phrase count |
| Backoff | negative; reimburses boost from abandoned partials; completed phrases keep theirs |
| Divergence guard (E.2) | 0.35 normalised token edit distance; gated, because on a transducer it doubles joint calls |

## Acceptance

1. **Recovery on `Record-2.wav`.** Run the `margin_study` streaming path with
   and without boost. Report each target: `Terakeet`, `Apptrack start`,
   `Incog clear`, `Orion Laundry`, `CHB`, `GSC Check`, `Job ads`.
2. **False positives on prose.** E.1's kill criterion is more than ~2%.
   `tools/bench-clips/chat-60s.wav` is an interview clip (per its README), so
   it should be prose with little of his vocabulary. That hasn't been checked:
   transcribe it first and confirm. Boosted vs unboosted divergence there is
   then the false-positive rate.
3. **Tests:** trie match and backoff; title normalisation (duplicate suffix,
   digits, punctuation); the 7-day window; a reload test that edits a source
   mid-process.
4. **Human smoke:** John dictates habit and task names through the daemon.
   Tests passing is not acceptance.

## Out of scope

The phone (dayshade, outspoke), the router (C.6/C.7), any correction or
approval step before injection, and automatic vocabulary learning.
