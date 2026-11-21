# SCOREBOARD

Does voice beat typing *for him*, per engine and build? Updated as evidence lands.
In-chat calibration, not a product metric. Newest evidence first in each table.

**His benchmark is Parakeet Ultra INT4 plus the dictionary, his only engine**
(decided 2026-11-20). Other engines' rows are context and John's own dictation; never
pool them into his numbers.

## The yardstick

| Constant | Value | Source |
|---|---:|---|
| His typing | 40 WPM = **1.5 s/word** | John, 2026-11-20 (adult-average) |
| Speaking | ~150 WPM = **0.4 s/word** | relaxed dictation |
| One needed correction | 6.5 s × 1.2 = **7.8 s** | JustCorrect 6.2–6.9 s (Android, complete repair), ×1.2 for his typing plus recognition delay (John) |
| Wait after stop | desktop ~0.3 s · phone ~1 s · Apple 0.1–0.23 s | history `wait_secs`, phone notes, `~/repos/swiftype/apple-metrics.md` |

**Needed-error rate (e)** = word errors after lowercasing and stripping punctuation
(`tools/wer` normalization, Mum≡Mom). "lightning McQueen and Mater" counts as right;
"…and maider" counts as wrong.

**Casing is produced but not graded** (John, 2026-11-20). The pipeline keeps getting
case right: rule 0 and canonical spellings. He should get correct results. But he may
not yet grok words by their casing, so a case-only difference costs him nothing and
doesn't count here. Revisit when casing starts to matter to him. A different *word*
still counts: gamble → Gambol is a real fix, because to anyone reading his text the
dog is named after frolicking, not craps. Chilli/chili and Mater/maider are the same.

**Seconds for an N-word utterance**

    typing = 1.5·N
    voice  = 0.4·N + wait + e·N·7.8
    break-even e* = (1.1 − wait/N) / 7.8   →  12.0% for a 6-word phone search, 13.5% on desktop, ~14% for long takes

A 6-word search ("Videos of Lightning McQueen and Mater") takes 9.0 s to type. One wrong
name costs 7.8 s, nearly the whole budget. **Any single wrong name loses.**

## Engines and builds: 6-word search, seconds (typing = 9.0)

| Surface · engine · build | Evidence | e | Voice s | Saved s |
|---|---|---:|---:|---:|
| Phone · Parakeet Ultra whole-take · 11-19 pipeline (before split/rule 0) | John's 148-word scripted take, delivered | 2.7% | 4.7 | **+4.3** |
| Phone · Parakeet Ultra whole-take · raw ASR, no pipeline | same take | 6.1% | 6.2 | +2.8 |
| Phone · Nemotron continuous · 11-19 pipeline | same take | 6.1% | 6.2 | +2.8 |
| Apple SpeechTranscriber, published, clean adult | LibriSpeech clean (M4 Pro), not our audio | 1.8% | 3.5 | +5.5 |
| Apple SpeechTranscriber, published, hard adult | Earnings-22, not our audio | 12.0% | 8.2 | +0.8 |
| Desktop · Parakeet · 11-20 (split + rule 0) | bloey cartoon clips, 4,946 words | 21.5% | 12.8 | −3.8 |
| Desktop · Nemotron · 11-20 | bloey | 24.5% | 14.2 | −5.2 |
| Desktop · Phonon 2.5 · 11-20 | bloey | 26.6% | 15.1 | −6.1 |

Reading it:
- **Apple has no custom vocabulary.** Its names can't be repaired by rules, so its
  name errors stay at the engine's rate. No child-speech WER or A14/A16 cold start is
  published.
- **Phonon on Apple Silicon/Bionic:** not measured. Fast is expected, accuracy is
  unknown. Desktop Phonon is the weakest engine on bloey.
- **The bloey rows are a stress test.** They're Australian cartoon voices, songs and
  sound effects, not his voice or yours. The cleanup pass costs ~1 point there by
  collapsing repeats and dropping fillers the subtitles keep.
- **The 11-19 take predates the splitter.** "Nelland" is one of its 4 delivered errors,
  and the splitter now fixes it.

## Names, the special-interest metric

Weighted by how often he says each name. Overall WER is secondary.

| Source | Names right after pipeline | Wrong changes (right → wrong) |
|---|---|---|
| John's phone takes, 11-20b | 2/2 (Gambol by rule 2, Rowan kept by rule 0) | 0 |
| bloey · Parakeet · after rule 0 | 43/99 subtitle names | **0** (was 13 across engines before rule 0) |
| bloey · Nemotron · after rule 0 | 42/99 | 0 |
| bloey · Phonon · after rule 0 | 32/99 | 0 |

**Sliding scale.** Value ≈ 7.8 s × uses per day × days the interest lasts. A Cars name
said 20 times a day for a year is about 7,300 uses. Fixing it once by a stated rule is
worth ~16 hours of his corrections a year. Leaving it broken costs the same.

## Refresh

```sh
# bloey corpus, all engines (needs a vocabulary file with show spellings)
XDG_CONFIG_HOME=<dir with config.toml pointing [vocabulary].path at it> \
  nix develop -c target/release/transcrust --wav ~/repos/bloey/corpus/<ep>/clips/*.wav \
  --mode parakeet --dec --output-dir <out>/<engine>/<ep>
```

The scorer is one-off (`score-bloey.ts`, Bun). Its reports live in
`~/syncthing/transcrust-work/` once John has reviewed them. Phone takes:
`~/syncthing/mopvox/decision-test/results/`. Script-take error counts:
`~/syncthing/transcrust-work/window-study/logs-20261119/script-comparison.json`.
