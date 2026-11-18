# rules-probe: rules-first ablation of the dictionary decision chain

Deterministic, Python 3 std-only, no model, no network. Implements steps 1-5 of the
homophone chain in `design-vocab-gate.md` section 5 (inflection, grammar slots,
bare singular count noun, animacy, fixed phrases / co-occurrence). Steps 6 (speaker prior)
and 7 (model) are not implemented: anything unsettled is **pass**, the residue.
Each rule logs its reason as a sentence; the per-case log is printed after the tables.

## Run

```sh
python3 -I tools/rules-probe/rules_probe.py            # tables + per-case log
python3 -I tools/rules-probe/rules_probe.py --no-log   # tables only
```

## Inputs and scoring (reused, not redefined)

- Cases: `tools/pair-router-probe/cross-encoder-q8/cases.json`, the 132 frozen decisions
  of the "Native D1 context ablation": 32 household rows (16 phone occurrences x single/two
  arms) and 100 synthetic rows (50 scenarios x 2 arms). Rows with `candidate: null` take their canonical
  from `bluey-q8-expanded-glossary/results.json`; descriptions come from the glossary table in each row's state.
- Labels: synthetic rows use `want` (keep / name / abstain). **Household rows have `want: null` in the fixture;**
  gold for them is `gold-household.json`, derived by me from the read-aloud script
  (`phone-household-q8/evidence/reference.md`). Casing-only rows (Transcrust -> transcrust) are labelled name.
  This is script intent, not verified spoken ground truth.
- Definitions follow `tools/d1-context-probe/main.rs`: repair = want name and final name; unwanted edit = want is not name
  and final is name; abstentions counted separately. Rules additionally split settled-and-wrong into wrong-name
  (said name, gold keep or abstain) and wrong-keep (said keep, gold name or abstain). A gold-abstain row the rules
  decide is wrong by construction (overconfident).
- Rules see the whole dictation text of the row (the `two` arm includes the neighbouring sentence, as the models do).

## Method discipline

Rules and word lists were written before the first run and not tuned against gold. I did read the fixtures and
the known failure cases beforehand, so the lists (e.g. `VERB_CAPABLE`, `COUNT_NOUNS`) are not blind;
that is a leak risk to weigh the numbers against. After the first run I made **no revisions**: I found no outright bug.
The wrong decisions below are design consequences, listed so they can be argued about, not patched.
No rule keys on a fixture sentence, id or label. Word lists are in the script, with comments.
No fixture carries timing, so the vocative pause cue only uses punctuation and fired 0 times.

## Results (first and only run)

### Settled / correct / wrong by set and arm
| set | arm | n | settled | correct | wrong-name | wrong-keep | passed |
|---|---|---:|---:|---:|---:|---:|---:|
| household | single | 16 | 12 | 12 | 0 | 0 | 4 |
| household | two | 16 | 16 | 12 | 4 | 0 | 0 |
| household | both | 32 | 28 | 24 | 4 | 0 | 4 |
| synthetic | single | 50 | 17 | 11 | 6 | 0 | 33 |
| synthetic | two | 50 | 23 | 16 | 7 | 0 | 27 |
| synthetic | both | 100 | 40 | 27 | 13 | 0 | 60 |

### Per rule (both sets, both arms)
| rule | fires | correct | wrong-name | wrong-keep |
|---|---:|---:|---:|---:|
| 1 inflection | 0 | 0 | 0 | 0 |
| 2 grammar slots | 34 | 30 | 4 | 0 |
| 3 bare count noun | 12 | 6 | 6 | 0 |
| 4 animacy | 0 | 0 | 0 | 0 |
| 5 fixed phrases | 22 | 15 | 7 | 0 |

### Per rule by set
| set | rule | fires | correct | wrong-name | wrong-keep |
|---|---|---:|---:|---:|---:|
| household | 1 inflection | 0 | 0 | 0 | 0 |
| household | 2 grammar slots | 20 | 20 | 0 | 0 |
| household | 3 bare count noun | 2 | 2 | 0 | 0 |
| household | 4 animacy | 0 | 0 | 0 | 0 |
| household | 5 fixed phrases | 6 | 2 | 4 | 0 |
| synthetic | 1 inflection | 0 | 0 | 0 | 0 |
| synthetic | 2 grammar slots | 14 | 10 | 4 | 0 |
| synthetic | 3 bare count noun | 10 | 4 | 6 | 0 |
| synthetic | 4 animacy | 0 | 0 | 0 | 0 |
| synthetic | 5 fixed phrases | 16 | 13 | 3 | 0 |

### Gold-abstain rows (rules should pass; deciding is an overconfident error)
23 rows; rules decided 4.

### Residue (ids as id/arm)
household (4): parakeet-3/single, parakeet-4/single, phonon-5/single, phonon-6/single
synthetic (60): son-context-name/single, son-context-name/two, son-context-keep/single, son-context-keep/two, mother-context-name/single, mother-context-name/two, mother-context-keep/single, mother-context-keep/two, project-context-name/single, project-context-name/two, project-context-keep/single, project-context-keep/two, habit-context-name/single, habit-context-name/two, habit-context-keep/single, habit-context-keep/two, lightning-context-name/single, lightning-context-name/two, lightning-context-keep/single, lightning-context-keep/two, mater-context-name/single, mater-context-name/two, mater-context-keep/single, mater-context-keep/two, flo-context-name/single, flo-context-keep/single, flo-context-keep/two, son-neighbor-name/single, son-neighbor-name/two, project-neighbor-name/single, project-neighbor-name/two, habit-neighbor-name/single, habit-neighbor-name/two, lightning-neighbor-name/single, lightning-neighbor-name/two, initial-after/single, initial-after/two, quoted-rowen/single, quoted-rowen/two, quoted-flow/single, quoted-flow/two, bluey-bingo-context-name/single, bluey-bingo-context-common/single, bluey-bingo-context-common/two, bluey-bingo-common-1/single, bluey-bingo-common-2/single, bluey-bingo-common-2/two, bluey-chili-context-name/single, bluey-chili-context-common/single, bluey-chili-context-common/two, bluey-chili-common-1/single, bluey-chili-common-2/single, bluey-bluey-explicit-1/single, bluey-bluey-explicit-1/two, bluey-bluey-explicit-2/single, bluey-bluey-explicit-2/two, bluey-bluey-context-name/single, bluey-bluey-context-name/two, bluey-bluey-color-neighbor/single, bluey-bluey-color-neighbor/two

### Wrong decisions (ids, rule, label vs gold)
- parakeet-3/two: rule 5 fixed phrases: said name, gold keep
- parakeet-4/two: rule 5 fixed phrases: said name, gold keep
- phonon-5/two: rule 5 fixed phrases: said name, gold keep
- phonon-6/two: rule 5 fixed phrases: said name, gold keep
- dog-context-name/single: rule 3 bare count noun: said name, gold abstain
- dog-context-keep/single: rule 3 bare count noun: said name, gold abstain
- dog-context-keep/two: rule 3 bare count noun: said name, gold keep
- mater-neighbor-name/single: rule 2 grammar slots: said name, gold keep
- mater-neighbor-name/two: rule 2 grammar slots: said name, gold keep
- initial-before/single: rule 2 grammar slots: said name, gold keep
- initial-before/two: rule 2 grammar slots: said name, gold keep
- bluey-bingo-common-1/two: rule 5 fixed phrases: said name, gold keep
- bluey-chili-common-1/two: rule 5 fixed phrases: said name, gold keep
- bluey-chili-common-2/two: rule 5 fixed phrases: said name, gold keep
- bluey-bandit-context-name/single: rule 3 bare count noun: said name, gold abstain
- bluey-bandit-context-common/single: rule 3 bare count noun: said name, gold abstain
- bluey-bandit-context-common/two: rule 3 bare count noun: said name, gold keep

### Known cases
- parakeet-0/single gold=name -> name (2 grammar slots): named "Gambol": appositive after "dog", which the glossary gives for Gambol
- parakeet-0/two gold=name -> name (2 grammar slots): named "Gambol": appositive after "dog", which the glossary gives for Gambol
- parakeet-1/single gold=keep -> keep (2 grammar slots): kept "gamble": verb slot after "to"
- parakeet-1/two gold=keep -> keep (2 grammar slots): kept "gamble": verb slot after "to"
- phonon-0/single gold=name -> name (2 grammar slots): named "Gambol": appositive after "dog", which the glossary gives for Gambol
- phonon-0/two gold=name -> name (2 grammar slots): named "Gambol": appositive after "dog", which the glossary gives for Gambol
- phonon-1/single gold=keep -> keep (2 grammar slots): kept "gamble": verb slot after "to"
- phonon-1/two gold=keep -> keep (2 grammar slots): kept "gamble": verb slot after "to"
- phonon-2/single gold=name -> name (5 fixed phrases): named "Flo": cars marker phrase "lightning mcqueen" appears in the same text
- phonon-2/two gold=name -> name (5 fixed phrases): named "Flo": cars marker phrase "lightning mcqueen" appears in the same text

### Model comparison (existing saved outputs)
- household, rules alone (pass = no edit): n=32 repairs 12/12, unwanted edits 4/20, abstain-or-pass 4, exact-label 24
- household, D1 alone: n=32 repairs 9/12, unwanted edits 2/20, abstain-or-pass 2, exact-label 27
- household, rules then D1: n=32 repairs 12/12, unwanted edits 4/20, abstain-or-pass 0, exact-label 28
- synthetic, rules alone (pass = no edit): n=100 repairs 17/28, unwanted edits 13/72, abstain-or-pass 60, exact-label 27
- synthetic, D1 alone: n=100 repairs 14/28, unwanted edits 21/72, abstain-or-pass 6, exact-label 44
- synthetic, rules then D1: n=100 repairs 27/28, unwanted edits 28/72, abstain-or-pass 2, exact-label 54
- all, rules alone (pass = no edit): n=132 repairs 29/40, unwanted edits 17/92, abstain-or-pass 64, exact-label 51
- all, D1 alone: n=132 repairs 23/40, unwanted edits 23/92, abstain-or-pass 8, exact-label 71
- all, rules then D1: n=132 repairs 39/40, unwanted edits 32/92, abstain-or-pass 2, exact-label 82
- household, UltraBERT INT4 full-glossary alone: n=32 repairs 8/12, unwanted edits 11/20, abstain-or-pass 0, exact-label 17
- household, rules then UltraBERT INT4: n=32 repairs 12/12, unwanted edits 8/20, abstain-or-pass 0, exact-label 24
- synthetic UltraBERT per-case outputs: not saved in earlier runs; comparison pending.



## Reading the results

- Rules settled 68 of 132 rows (52%), 51 correct (75%), 17 wrong, **all wrong-name, none wrong-keep**.
  Keep decisions (verb slot, article slot) were never wrong; every error is an over-eager name.
- Residue 64 rows (48%): 60 synthetic, 4 household. The residue is mostly gold keep (34), gold abstain (19), gold name (11).
- Rules 1 (inflection) and 4 (animacy) never fired: no fixture has an inflected or animate-subject form.
  They are untested here, not validated.
- Rule 5 (franchise co-occurrence) does the same thing the neighbour sentence does to the models: a Bluey marker in a
  *neighbouring* household sentence names `chili` and `bingo` in "We ate chili and played bingo" (4 wrong-names in the two arm),
  plus 3 synthetic common-neighbour cases. Co-occurrence needs a same-sentence limit or a proximity rule; I did not add one.
- Rule 3 (bare count noun) is wrong on "thinking about gamble" and "asked about bandit": a bare noun after a preposition is
  ungrammatical, yet the gold there is abstain or keep, because the ASR word may be a mis-hearing of either. Grammar says
  "not a common noun here", not "it is the dog".
- Rule 2: "my son O" (initial diagnostic, gold keep) and "Mr Mader" (a different Mr, gold keep) are the two wrong cases.
  The appositive and title cues do not know whether it is the same person.

## Known cases

| case | outcome |
|---|---|
| "to gamble" (parakeet-1, phonon-1, both arms) | **keep, correct**, rule 2: verb slot after "to". D1 got this wrong with neighbour context; UltraBERT INT4 kept it. |
| Flo repair (phonon-2, both arms) | **name, correct**, rule 5: "Lightning McQueen" in the same sentence. Fires without any model. |
| Dog repair, Gamble -> Gambol (parakeet-0, phonon-0, both arms) | **name, correct**, rule 2: appositive after "dog", which the glossary gives for Gambol. |
| "to flow" / "to Flo" weather (parakeet-2, phonon-3, both arms) | keep, correct, verb slot after "to". |
| "a bandit stole" (parakeet-5, phonon-7) | keep, correct, article slot. |
| "Chilean bandit" (phonon-4) | name, correct, rule 3 bare count noun. |
| ordinary chili / bingo, single arm (parakeet-3, -4, phonon-5, -6) | pass: nothing fires; residue. Both are mass nouns with no cue. |
| ordinary chili / bingo, two arm | **wrong-name**, rule 5, via the neighbouring Bluey sentence. |

## Headline

- **Share that genuinely needs a model: 64 / 132 = 48% (household 4/32 = 12%, synthetic 60/100 = 60%).**
  That is an upper bound on "needs a model" only if the 68 settled rows are trusted; they are 75% correct, so the
  effective number is higher. The household set is nearly solved by rules because the household script is natural
  prose with grammar cues; the synthetic set is built from near-bare fragments ("My son drew flow") that are underdetermined
  by design (19 of the residue's gold labels are abstain).
- **Rules-then-D1 versus D1 alone** (existing saved D1 outputs for all 132 rows; D1 gold exact-label match / repairs / unwanted edits):
  - household: rules then D1 28/32 exact, 12/12 repairs, 4/20 unwanted edits; D1 alone 27/32, 9/12, 2/20; rules alone 24/32, 12/12, 4/20.
  - synthetic: rules then D1 54/100, 27/28 repairs, 28/72 unwanted; D1 alone 44/100, 14/28, 21/72.
  - all: rules then D1 82/132 exact, 39/40 repairs, 32/92 unwanted edits; D1 alone 71/132, 23/40, 23/92.
  - So the hybrid wins on repairs and exact-label match, but **loses on unwanted edits** (32 vs 23 of 92): every
    rule error is an extra edit. On the residue alone D1 is right on 31 of 64 (14 of 45 wrong where gold is not abstain);
    on the rows the rules settle D1 is right on 40 of 68, the rules on 51.
  - Caveat: the synthetic rows are correlated (two arms per scenario, contrived labels), and D1's synthetic labels were not fitted here, but the rules were written seeing the fixture designs. Not an accuracy estimate for dictation.
- **Rules-then-UltraBERT INT4** (only the 32 household rows have saved UltraBERT outputs, full glossary): rules then UltraBERT
  24/32 exact, 12/12 repairs, 8/20 unwanted; UltraBERT alone 17/32, 8/12, 11/20. D1 alone and rules-then-D1 are better than both.
  **Synthetic UltraBERT comparison is pending**: no per-case outputs were saved for the 100 synthetic rows.
- Unwanted edits are the cost to watch. A shippable chain needs rule 5 restricted to the same sentence and the rule-3 and
  rule-2 name cues demoted to "pass" unless the glossary description agrees. I did not make those changes.

## Files

- `rules_probe.py`: lists, rules, scoring, report.
- `gold-household.json`: ids and labels for the household rows only, with its derivation noted.
- No contents of `tools/vocab-score/out/` were used or copied.

## v2 variants (2026-11-18, post-hoc, labelled)

Run: `python3 -I variants.py` (full table in `variants-out.md`). It imports v1's rules unchanged
and loops variants × set × arm. "Hybrid" means the rules decide first, and passed cases fall
through to D1's saved choice.

| variant (both arms) | household hybrid exact | household unwanted | synthetic hybrid exact | synthetic repairs | synthetic unwanted |
|---|---:|---:|---:|---:|---:|
| v1 | 28/32 | 4/20 | 54/100 | 27/28 | 28/72 |
| **v2-local** (rule 5 current sentence only) | **32/32** | **0/20** | 54/100 | 25/28 | 26/72 |
| v2-gloss (rule 3 needs an animate glossary description) | 28/32 | 4/20 | 54/100 | 27/28 | 28/72 |
| v2+recency (exact name in an earlier sentence → name) | 30/32 | 2/20 | 52/100 | 25/28 | 28/72 |

- **v2-local is the keeper.** Household rules-then-D1 goes to 32/32 with zero unwanted edits.
  The only changes are in the two-sentence arm: neighbouring-sentence franchise markers stop
  forcing names. The pre-stated gate holds (household single-sentence is unchanged).
  On synthetic, 2 fewer repairs and 2 fewer unwanted edits.
- **v2-gloss changes nothing here:** every rule-3 firing already had an animate description.
- **Recency as a hard rule is rejected:** +5 synthetic wrong-names. The synthetic scenarios use a
  name and then the ordinary word ("Chili … the chili"). At most, keep it as soft evidence for
  the model margin, never as a decision.

## Extra fixtures: inflection, animacy, controls (2026-11-18)

`fixtures-extra.json` has 12 text cases, labelled from linguistics before running. Run: `python3 -I extra.py`.
- **Before the fix: 9/12.** The inflection rule treated "Lightning" as an -ing verb, so it kept
  "Lightning smiled and waved", which is wrong. **Bug fix:** rule 1 now fires only if
  removing the ending leaves a verb (gambled→gamble, flowing→flow), never on lexicalised nouns like
  lightning, morning or king. It also handles -s ("flows").
- **After the fix: 10/12.**
  - Inflection fires correctly on gambled, gambling, flowing and flows.
  - Animacy fires correctly on "Chili laughed", "flow said hello" and "Lightning smiled".
  - The 2 misses are passes, not errors: "played bingo" and "ate chili" are genuinely ambiguous
    without context ("played Bingo" can mean the character), so they go to the model. No new
    wrong-names.
- **Full 132-decision rerun after the fix: unchanged** (v2-local household 32/32, 0 unwanted).
