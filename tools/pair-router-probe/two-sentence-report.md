# One versus two sentences — d1 Q8 / Q4

Extra context helps some repairs but also causes nearby-name contamination under the existing referent prompt. Both quantizations repair 4/8 newly disambiguated names, but adding a neighboring name introduces edits in 6/8 ordinary/other-person controls, 1/2 initial controls and both quoted spelling controls. These preservation failures were absent with the target sentence alone. The inputs are only 56–97 tokens: capacity is ample; respecting the marked occurrence and preservation contract is the issue.

28 synthetic paired scenarios, 56 decisions per model. Eight names: Gambol, Rowan, Nell, transcrust, Cigs, Lightning, Mater and Flo. Both preceding and following context are included. Each inference judges exactly one marked occurrence; ordinary words and nearby names are never edited together.

The eight ambiguous target sentences each appear with two opposing neighbors: one establishes the vocabulary referent and one establishes an ordinary/other referent. Eight further controls put vocabulary names near explicit ordinary uses; four preserve initials or explicitly quoted spellings. Gold labels were written before inference. No API calls, learned substitutions or live changes.

| Category | Model | Scope | Exact decisions | Repairs | Unwanted edits |
|---|---|---|---:|---:|---:|
| disambiguate-name | Q8 | single | 1/8 | 0/0 | 0/8 |
| disambiguate-name | Q8 | two | 4/8 | 4/8 | 0/0 |
| disambiguate-name | Q4 | single | 1/8 | 0/0 | 0/8 |
| disambiguate-name | Q4 | two | 4/8 | 4/8 | 0/0 |
| disambiguate-keep | Q8 | single | 1/8 | 0/0 | 0/8 |
| disambiguate-keep | Q8 | two | 7/8 | 0/0 | 0/8 |
| disambiguate-keep | Q4 | single | 1/8 | 0/0 | 0/8 |
| disambiguate-keep | Q4 | two | 7/8 | 0/0 | 0/8 |
| name-distractor | Q8 | single | 7/8 | 0/0 | 0/8 |
| name-distractor | Q8 | two | 2/8 | 0/0 | 6/8 |
| name-distractor | Q4 | single | 7/8 | 0/0 | 0/8 |
| name-distractor | Q4 | two | 2/8 | 0/0 | 6/8 |
| privacy-initial | Q8 | single | 2/2 | 0/0 | 0/2 |
| privacy-initial | Q8 | two | 1/2 | 0/0 | 1/2 |
| privacy-initial | Q4 | single | 2/2 | 0/0 | 0/2 |
| privacy-initial | Q4 | two | 1/2 | 0/0 | 1/2 |
| intentional-spelling | Q8 | single | 2/2 | 0/0 | 0/2 |
| intentional-spelling | Q8 | two | 0/2 | 0/0 | 2/2 |
| intentional-spelling | Q4 | single | 2/2 | 0/0 | 0/2 |
| intentional-spelling | Q4 | two | 0/2 | 0/0 | 2/2 |

Exact decisions distinguish KEEP from abstention; both preserve the text. An ambiguous single sentence and its disambiguated two-sentence version can have different expected answers. No confidence threshold is applied.

Q8: two-sentence exact 14/28; 10 scenarios change from incorrect to correct relative to their scope-specific gold, 9 from correct to incorrect. Complete native inputs use 56–97 tokens.
Q4: two-sentence exact 14/28; 9 scenarios change from incorrect to correct relative to their scope-specific gold, 8 from correct to incorrect. Complete native inputs use 56–97 tokens.

This is a small diagnostic, not a trained or calibrated production benchmark. Counterfactual pairs deliberately reuse target sentences, so decisions are correlated. No ASR audio is involved. This preserves the existing referent question rather than evaluating a newly tuned correction prompt; it is not a claim that every possible two-sentence interface fails. Review the individual sentences and gold interpretations.

## All paired decisions

| Scenario | Single sentence / expected | Added context / expected | Q8 single → two | Q4 single → two |
|---|---|---|---|---|
| dog-context-name | I was thinking about «gamble». / abstain | She is my dog, and I was planning her next walk. / name | keep (0.000) → name (0.642) | keep (0.000) → keep (0.208) |
| dog-context-keep | I was thinking about «gamble». / abstain | I meant betting money at the casino. / keep | keep (0.000) → keep (0.000) | keep (0.000) → keep (0.000) |
| son-context-name | I spoke to «Rowen». / abstain | He is my son, and I was reminding him about his lunchbox. / name | keep (0.193) → keep (0.105) | keep (0.341) → keep (0.392) |
| son-context-keep | I spoke to «Rowen». / abstain | She is my colleague, and her name really is spelled Rowen. / keep | keep (0.193) → keep (0.083) | keep (0.341) → keep (0.024) |
| mother-context-name | I heard «Knell». / abstain | She is my son Rowan's mother, and she was calling him home. / name | keep (0.008) → name (0.819) | keep (0.024) → name (0.734) |
| mother-context-keep | I heard «Knell». / abstain | It was the sound of the funeral bell. / keep | keep (0.008) → keep (0.002) | keep (0.024) → keep (0.004) |
| project-context-name | I was discussing «trans crust». / abstain | It is the Linux push-to-talk dictation app I use. / name | keep (0.001) → name (0.974) | keep (0.001) → name (0.964) |
| project-context-keep | I was discussing «trans crust». / abstain | We were describing how material moves through a crust, not software. / keep | keep (0.001) → keep (0.000) | keep (0.001) → keep (0.000) |
| habit-context-name | I updated «six». / abstain | It is the cigarette-counting habit in my tracker. / name | keep (0.001) → keep (0.284) | keep (0.000) → keep (0.384) |
| habit-context-keep | I updated «six». / abstain | It was the number of cigarettes I had smoked, and I changed the count to seven. / keep | keep (0.001) → keep (0.044) | keep (0.000) → keep (0.129) |
| lightning-context-name | My son drew «lightning». / abstain | He gave the red race car eyes and the number 95. / name | keep (0.006) → abstain (0.310) | keep (0.008) → abstain (0.318) |
| lightning-context-keep | My son drew «lightning». / abstain | He drew a jagged electrical bolt coming out of a storm cloud. / keep | keep (0.006) → abstain (0.016) | keep (0.008) → abstain (0.022) |
| mater-context-name | My son drew «Mader». / abstain | He added the tow hook and rusty body from Cars. / name | abstain (0.081) → abstain (0.288) | abstain (0.125) → name (0.380) |
| mater-context-keep | My son drew «Mader». / abstain | It was a portrait of our neighbor, whose surname is spelled Mader. / keep | abstain (0.081) → keep (0.036) | abstain (0.125) → keep (0.022) |
| flo-context-name | My son drew «flow». / abstain | She was serving the other Cars characters at the V8 Cafe. / name | keep (0.027) → name (0.964) | keep (0.078) → name (0.914) |
| flo-context-keep | My son drew «flow». / abstain | He drew arrows showing how water moves down a river. / keep | keep (0.027) → keep (0.058) | keep (0.078) → keep (0.247) |
| dog-neighbor-name | I like to «Gamble» and play poker with cards. / keep | I love my dog Gambol. / keep | keep (0.000) → keep (0.039) | keep (0.000) → keep (0.031) |
| son-neighbor-name | My colleague «Rowen» is presenting the report. / keep | My son Rowan is at school. / keep | keep (0.077) → name (0.739) | keep (0.203) → name (0.733) |
| mother-neighbor-name | A «Knell» is the sound of a bell marking a death. / keep | Nell is picking up my son. / keep | keep (0.001) → keep (0.027) | keep (0.006) → keep (0.013) |
| project-neighbor-name | The label uses «trans crust» to describe movement through a crust. / keep | I closed the transcrust dictation app. / keep | keep (0.030) → name (0.813) | keep (0.040) → name (0.811) |
| habit-neighbor-name | I smoked «six» cigarettes today. / keep | I track them in my Cigs habit. / keep | keep (0.386) → name (0.964) | keep (0.440) → name (0.923) |
| lightning-neighbor-name | The storm produced bright «lightning». / keep | My son was playing with his Lightning McQueen toy. / keep | keep (0.005) → name (0.931) | keep (0.004) → name (0.953) |
| mater-neighbor-name | Our neighbor Mr «Mader» repaired the fence. / keep | My son was watching Mater in Cars. / keep | abstain (0.017) → name (0.832) | abstain (0.015) → name (0.890) |
| flo-neighbor-name | The river has a strong «flow» after the rain. / keep | My son was talking about Flo from Cars. / keep | keep (0.047) → name (0.819) | keep (0.237) → name (0.874) |
| initial-before | My son «O» needs his lunchbox; use only his initial here. / keep | His full name is Rowan. / keep | keep (0.060) → name (0.684) | keep (0.173) → name (0.525) |
| initial-after | Please refer to my son as «O» in this message. / keep | Rowan is his full name, which I keep private online. / keep | keep (0.012) → keep (0.192) | keep (0.093) → keep (0.491) |
| quoted-rowen | Copy the label exactly as written: «Rowen». / keep | It belongs to my son Rowan. / keep | keep (0.153) → name (0.760) | keep (0.272) → name (0.873) |
| quoted-flow | Keep the quoted word exactly as written: «flow». / keep | My son was describing Flo from Cars. / keep | keep (0.281) → name (0.917) | keep (0.322) → name (0.816) |

Parenthesized probabilities are P(name), not P(chosen). [Q4 full inputs/outputs](two-sentence-q4/report.md) · [Q8 full inputs/outputs](two-sentence-q8/report.md)

Regenerate the saved comparison:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/context_suite.py
```
