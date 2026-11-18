# Actual phone transcripts → native D1 Q8

Run on 2026-10-29, using the unchanged desktop `--fix TEXT --dec` path at
`65d2f8d4f45438dc6669f16bcc05dfce2411bf9b`. No API calls, interpreter, new
aliases, model/prompt tuning, injection or phone APK changes.

## Result

| Phone ASR | Raw errors / 79 words | After Q8 | Added decision time | Candidates / graph calls | Standalone peak RSS |
|---|---:|---:|---:|---:|---:|
| Parakeet | 3 (3.80%) | 3 (3.80%) | 5.405 s | 7 / 1 | 648.9 MiB |
| Phonon | 5 (6.33%) | 4 (5.06%) | 8.904 s | 9 / 1 | 699.8 MiB |

Both corrected the dog name, but both also introduced a gambling error. Phonon
additionally repaired Flo. That is one net word-error improvement for Phonon
and no net improvement for Parakeet on this clip. This does not reopen John's
accepted desktop dogfood or establish general ASR accuracy.

| Script line / intended meaning | Phone input | Q8 output / disposition |
|---|---|---|
| 1 — pet dog | Both: “My dog Gamble is an Australian shepherd.” | Both: “My dog Gambol is an Australian shepherd.” Correct repair; P(name) .652. |
| 2 — ordinary gambling | Both: “I like to gamble and play poker with cards.” | Both: “I like to Gambol and play poker with cards.” **Introduced error**; P(name) .756. |
| 3 — intentional initial | “My son O and I watched cars.” (Parakeet); “… Cars.” (Phonon) | Unchanged; O is not a candidate. |
| 4 — Cars characters | Parakeet: “Lightning McQueen raced past Mater and Flo.” | Unchanged; names already canonical. |
| 4 — Cars characters | Phonon: “Lightning McQueen raced past meter and flow.” | “… meter and Flo.” Flo repaired, P(name) .445 (largest of three choices, not a .5 threshold). `meter` never reaches D1: no Mater alias. |
| 5 — ordinary weather / water | Both: “Lightning lit the sky, and water began to flow.” | Unchanged; ordinary flow kept. Standalone “Lightning” is not an alias of Lightning McQueen. |
| 6 — Bluey characters | “… played with Chilean Bandit.” (Parakeet); “… Chilean bandit.” (Phonon) | Unchanged. `Chilean` never reaches D1: no Chili alias. Lowercase character bandit abstained; Parakeet's Bandit already canonical. |
| 7 — ordinary food / game | Both: “We ate chili and played bingo after the show.” | Unchanged; both kept. |
| 8 — ordinary robber | Both: “I said a bandit stole the cards.” | Unchanged; kept. |
| 9 — project casing | Both: “This Linux software project is called Transcrust.” | Both: “… transcrust.” Casing repair, not counted as a word-error improvement. |
| 10 — Nell | Both: “Nell and I talked about the next school day.” | Unchanged; canonical. |

The erroneous gamble row contains the preceding dog sentence and marks **gamble**
in the gambling sentence. Its dog interpretation is a model decision despite
the marker and ordinary-meaning description; it is not an unconditional alias
replacement. Both identical rows produced identical probabilities. Do not fix
this by blindly increasing name weights or adding context-specific string rules.

## Measurement boundaries

### Unchanged second pass (2026-10-29)

Both transcripts were replayed again from fresh processes at John's request.
The shared vocabulary was byte-identical. Inputs, candidates, delivered text,
choices and every probability were exactly equal to the first pass. In
particular, “to gamble” again became “to Gambol”, with P(name)
0.7555611826983235. The infinitive cue did not prevent the wrong interpretation.
This path uses argmax over scores, with no sampling; repetition is a
reproducibility check, not another stochastic attempt at correction.

Added decision times were 6.124s (Parakeet) and 10.611s (Phonon), versus
5.405s / 8.904s initially. Peak RSS was 664256 / 712752 KiB. Error counts
remained 3→3 and 5→4. Full second-pass audits, process measurements and scores
are in `evidence/repeat/`. No prompts, aliases or runtime code changed.

Inputs are the Pixel 9a's completed October 28 JSON reports, not fresh desktop
ASR. Both phone passes emitted identical text per engine, so each unique
transcript was replayed once. Both used the same 49.719875-second recording:
`3e7fdd89c054dfb7d41113cb19fd8347764662cbc554fd1b74ec62527f8bfe46`.

The numbered read-aloud script is the expected reference, not independently
verified spoken ground truth. Rust word edit distance ignores case/punctuation;
the missing “Chili and” → “Chilean” costs two edits. Casing and initial
preservation are reported separately. Full raw, cleaned and delivered text,
per-occurrence immutable contexts, probabilities and shared batch times are in
`evidence/*-decision.json`. Deterministic cleanup removed trailing whitespace.

These are single desktop observations using the debug binary, not mobile timing
or a controlled performance benchmark. GNU time's maximum RSS includes native
model loading, startup warmup and inference, with **no ASR model resident**.
Model-ready times were 6.76 / 6.57 seconds; total process wall times were
12.29 / 15.60 seconds. Daemon startup pays loading once; the table's added
decision time excludes loading. Graph times were 5.344 / 8.834 seconds; all
rows run in one padded call (283–302 tokens per row), not serial calls. Larger
batches consume more working memory than the earlier small-batch probes.
Do not add these desktop figures to phone ASR time to predict mobile performance.

The unchanged shared glossary snapshot has SHA-256
`9aaeeeefd08458761005a5798245b11d5e995074e27d1d8f5d9d1a567a1c4c51`.
It includes the later “Cars theme” entry. Installed Q8 graph/data and upstream
revisions are pinned by `evidence/model-manifest.json`; loading verifies hashes.
Original report hashes and extracted phone scores are retained. Audio and model
weights remain outside this repository. These household fixtures stay local;
they are not public CI data.

## Replay

From the repository:

```sh
nix develop -c bash tools/pair-router-probe/phone-household-q8/replay.sh
```

Builds the app, replays both actual phone inputs through native Q8, validates
successful audits, measures process RSS/wall time and scores raw/output text
with the small Rust scorer. Prints a fresh `/tmp/transcrust-phone-q8.*`
evidence directory. Uses the current shared glossary/config and installed
model, so later vocabulary changes may change results; the saved snapshot is
the authority for this run. Runtime audits append normally; no last-dictation
state or daemon `latest.log` is changed. Optional arguments: reports directory,
output directory (use a fresh directory).

Before mobile integration: retain these inputs as regressions, distinguish
reviewed alias coverage (`meter`, `Chilean`) from model judgment, and measure
candidate-rich latency/RAM on the actual device. No vocabulary changes were
made as part of this test.
