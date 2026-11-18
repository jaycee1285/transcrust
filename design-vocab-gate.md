# Vocabulary gate and homophone chain (design, 2026-11-17)

Status: **design only, not built.** Worked out by John and Claude on 2026-11-17, starting from
Dragon Professional v16's command card
(`ct-dragon-professional-v16-and-legal-v16-command-cheat-sheet-en-us.pdf`). It complements
`design-dictation-as-control.md`. It doesn't change the engines: Parakeet-ultra int4 for takes
under 20 s, Nemotron streaming int4 beyond that, both ONNX, laptop and Pixel 9a.

**Why:** no engine will ever be trained on John, his son or the boy's mother. State of the art for
this family is the deterministic layer after the ASR. Boosting the whole decode toward
`john-vocab.md` failed the Gambol/gamble gate, because it biases every utterance.

## 1. What Dragon's card teaches (the evidence)

- **Commands are verb-first, and silence on both sides marks one** ("pause before and after
  commands but not within them").
- **The slots are bounded:** `<n>` is a small number grammar; `<xyz>` must be text already on
  screen; open free text appears only at the end of a command.
- **Fixed words carry fixed meanings:** "that" always means the last utterance or selection;
  `un-` undoes; `on|off` switches a mode.
- **Exact strings are spelled out:** spell mode with NATO letters and spoken formatting words
  (`cap b a hyphen 5`).
- **Collisions are admitted:** the `**` commands (delete/cut/copy <text>) can be disabled.
- **The phonetics resist accents:**
  - each command is told apart by its opening consonant, and consonants don't vary across American
    accents, while vowels do (cot–caught, Mary–marry–merry, pin–pen before nasals, Northern Cities
    and Southern shifts);
  - most commands are two syllables, or one syllable hardened with plosives;
  - the stressed syllable usually ends in a plosive (select, correct, delete, quote, cut, start),
    and the microphone commands end in /p/.
  - The command matcher only needs `s·ɛ·kt`, `k·ɹɛ·kt` and so on, not whole words.

## 2. Two separate ideas

**A. Control vs dictation (the command gate).**
1. Take the first word from Nemotron's streaming partial results, or roughly the first 400 ms.
2. Match it by phoneme n-grams against the command set: the opening consonant cluster, plus the
   stressed vowel after accent mergers, plus the closing plosive.
3. Use pause timestamps (silence before and after) as the boundary.
4. **On a hit:** parse the command's slots (number, last-take text, free text at the end).
   **On a miss:** it's dictation, with no vocabulary bias at all.

**B. Vocabulary only inside open slots.** `john-vocab.md` applies only inside a slot the gate
opened, or after a cue word ("dog …", "task …"). Never as a global boost.

## 3. Scoring each vocabulary entry when it's added (deterministic)

1. **Phonemes:** CMUdict first; `espeak-ng --ipa` (rule-based) for words CMUdict lacks; a
   manual override in `john-vocab.md` wins: `Gambol | G AE1 M B AH0 L | tier 0 | prefer: kid`.
2. **Collapse accent mergers:** cot=caught, Mary=marry=merry, pin=pen before nasals,
   father=bother, flapped t/d, dropped final r.
3. **Distance to the nearest common words** (CMUdict filtered by a word-frequency list):
   feature-weighted phoneme edit distance (Kondrak's ALINE, or PanPhon features) or Kondrak's
   n-gram distance (2005). Substitution weights can be calibrated from John's own
   `corrections.md` and `pairs.md`.
4. **Tier:** **0** homophone (Gambol/gamble, Flo/flow, Lightning) · **1** near (Mater/matter) ·
   **2** distinct (McQueen, transcrust). Only tier 2 may be substituted on sound alone.
5. **Quality check for commands:** no collision after mergers; a distinguishing opening
   consonant; at least one plosive, ideally closing the stressed syllable; two syllables or a
   hardened single syllable.
6. **Index:** a map from n-gram to entries, rebuilt on reload (an insert, no retrain, microseconds).
   Built on the laptop and shipped as `john-vocab.index.json` so mopvox and swiftype load
   identical data.

## 4. When the chain runs: uncertainty, not presence (John, 2026-11-17)

**Phrase boosting is presence-triggered.** Aho-Corasick / GPU-PB, as in murmure and transcrust's
Nemotron boosting, fires whenever a phrase's tokens appear, with no context. It's only safe for
phrases that couldn't occur otherwise, so it can never resolve a homophone.

**This design is uncertainty-triggered.** From the **unboosted** decode, take each word's logit
margin (top-1 minus top-2) or its token entropy:
- **Confident:** leave the word alone, whatever the dictionary says.
- **Unsure, *and* the span's phoneme n-grams hit a dictionary entry:** run the chain below on that
  span only.

The model is never steered. It reports where it's guessing, and the rules decide only there.
murmure's fixed 0.45 confidence cutoff is the crude version of this. Calibrate the margin
threshold on John's takes.

## 5. The homophone chain (tier 0 and 1; each step can decide)

**Every decision records which step fired and why,** e.g. `kept "flow": verb slot after "to"` or
`named "Flo": phrase "Flo and Mater"`. The correct-last UI shows the reason. For the person this
is built for, an explainable wrong answer beats an unexplained right one: he can learn the rule
and say it differently.

1. **Inflection:** *gambled, gambling, flowing* can't be a name.
2. **Grammar slots within ±2 words:** *to / will / can ___*, *___ on/away/into* mean a verb;
   *walk ___*, *___'s* and a pause-marked call ("Gambol… come here") mean a name.
3. **A bare singular countable noun can't be a subject:** "gamble barked" is ungrammatical,
   "Gambol barked" isn't.
4. **Animacy:** verbs that need a living subject (said, barked, won, ate, likes) rule out
   flow and lightning.
5. **Fixed phrases:** "Lightning McQueen", "Flo and Mater"; the characters co-occur.
6. **Per-speaker prior:** on the kid's devices, Cars names win close calls; in John's dictation,
   the common word wins.
7. **Only then, the decision model** (0.3–0.8B, keep / name / abstain), defaulting to abstain.

Optional leftovers parser: Link Grammar (rule-based) or UDPipe (small), about 1 ms per sentence,
run only on sentences containing a tier 0 or 1 hit. Rules 1–4 take microseconds.

## 6. Next steps (in order)

1. **The command list as Markdown,** with sound markers per command: opening consonant, plosives,
   syllables, merger-safe or not. Generated from the scorer, so the card and the grammar are always
   the same list. Seeds: `~/syncthing/dayshade/commands.md` and Dragon's card.
2. **Build the scorer** (sections 3.1–3.5) and run it over the current `john-vocab.md`. Expect it to
   flag Gambol, Flo, Lightning and Mater.
3. **Build the trigger and chain** (sections 4–5), rules first, and replay the Gambol/gamble probe from the
   2026-11-13 migration report.
4. **The command gate** on Nemotron partials, with pause gating.

## 7. Prior art (read 2026-11-17)

Neither Handy nor murmure has a command grammar, phoneme tiers or any context chain. Sections 2–4
go beyond both.

- **`~/repos/dz/handy`** (MIT, about 39k stars; code may be reused):
  - Custom words are a plain list. They're sent to transcribe-cpp's `vocabulary` prompt, or
    post-replaced by `apply_custom_words` (`src-tauri/src/audio_toolkit/text.rs:151`): 1–3-word
    n-grams scored by Levenshtein/length, ×0.3 on a Soundex match, under a length-difference cap.
  - **Worth taking:**
    - n-gram joining ("Charge B" → "ChargeBee", `text.rs:10-135`);
    - `preserve_case_pattern` and `extract_punctuation`;
    - `echoes_vocabulary` (`managers/transcription.rs:1627`), which catches a prompted model reading
      the word list back during silence.
- **`~/repos/dz/murmure`** (AGPL; ideas only).
  - **John's analysis:** `~/syncthing/learning/Programming/transcrust-model/murmure.md`. It covers
    the "head" drop (duration logits ignored, decoding frame by frame, which keeps the phonetic trace
    for correction) and why the decode and the correction are co-designed.
  - **murmure has moved since that June read:**
    - it now boosts with a weighted Aho-Corasick tree over SentencePiece ids (from NeMo GPU-PB),
      only for tokens already in the top 5, with a guard that reverts to the unboosted decode if
      output diverges;
    - post-correction is Levenshtein-capped, not Beider-Morse; Beider-Morse now lives only in
      transcrust's port.
  - **One idea worth adding here:** a **per-word confidence gate** (above 0.45, never
    fuzzy-correct), as a free step 0 before the homophone chain. Confidence comes from the
    unboosted logits.
