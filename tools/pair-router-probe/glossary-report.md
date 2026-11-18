# Five-entry glossary ablation — d1 Q8 / Q4

56 local decisions per model with the glossary, compared with the 56 saved decisions without it. Same 28 paired scenarios, marked occurrences, questions, option descriptions, gold labels, model files and calibration. The only input change is the following shared glossary and section headings in state:

```text
Vocabulary glossary:
| Term | Description |
|---|---|
| Gambol | pet dog |
| Rowan | son |
| Transcrust | software project |
| Flo | Cars movie character |
| Lightning McQueen | Cars movie character |

Dictation:
<original one- or two-sentence marked text>
```

No instructions to favor names were added. The previous options already describe the candidate name; this tests an additional shared glossary, not first exposure to those relationships. Nell, Cigs and Mater are absent from the supplied glossary and remain in the suite as unlisted controls. Gold labels and scenario categories are never supplied as model inputs.

| Model | Scope | Glossary | Name repairs | Edits on preserve/ambiguous | Exact choices |
|---|---|---|---:|---:|---:|
| Q8 | single | no | 0/0 | 0/28 | 13/28 |
| Q8 | single | yes | 0/0 | 8/28 | 7/28 |
| Q8 | two | no | 4/8 | 9/20 | 14/28 |
| Q8 | two | yes | 7/8 | 9/20 | 17/28 |
| Q4 | single | no | 0/0 | 0/28 | 13/28 |
| Q4 | single | yes | 0/0 | 6/28 | 9/28 |
| Q4 | two | no | 4/8 | 9/20 | 14/28 |
| Q4 | two | yes | 6/8 | 10/20 | 16/28 |

Single-sentence ambiguous cases have abstention gold labels: no single-sentence case requires a name repair. KEEP and abstain both preserve text, while exact-choice scoring distinguishes them. No confidence floor.

## Two-sentence preservation detail

| Model | Category | Without glossary | With glossary |
|---|---|---:|---:|
| Q8 | disambiguate-keep | 0/8 | 1/8 |
| Q8 | name-distractor | 6/8 | 5/8 |
| Q8 | privacy-initial | 1/2 | 1/2 |
| Q8 | intentional-spelling | 2/2 | 2/2 |
| Q4 | disambiguate-keep | 0/8 | 1/8 |
| Q4 | name-distractor | 6/8 | 6/8 |
| Q4 | privacy-initial | 1/2 | 1/2 |
| Q4 | intentional-spelling | 2/2 | 2/2 |

Cells count unwanted edits, so smaller is better.

Q8: 18/56 choices change; 5 become exact and 8 lose exactness. Complete native inputs with glossary: 111–152 tokens.
Q4: 16/56 choices change; 5 become exact and 7 lose exactness. Complete native inputs with glossary: 111–152 tokens.

The five-entry glossary plus headings adds 55 tokens to the actual native inputs. Zero truncation in both runs. No API calls, ASR replay, model tuning or live dictation changes. Synthetic paired cases are correlated and are not production calibration. The glossary improves two-sentence repair recall, but the current referent prompt still fails the preservation contract; single-sentence unsupported edits increase. This does not establish a safe automatic-correction policy.

## Every changed choice

| Model | Scope / scenario | Expected | Choice without → with | P(name) without → with | Dictation | Output without | Output with |
|---|---|---|---|---:|---|---|---|
| Q8 | single / son-context-name | abstain | keep → name | 0.193 → 0.728 | I spoke to «Rowen». | I spoke to Rowen. | I spoke to Rowan. |
| Q8 | single / son-context-keep | abstain | keep → name | 0.193 → 0.728 | I spoke to «Rowen». | I spoke to Rowen. | I spoke to Rowan. |
| Q8 | two / habit-context-name | name | keep → name | 0.284 → 0.719 | I updated «six». It is the cigarette-counting habit in my tracker. | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| Q8 | two / lightning-context-name | name | abstain → name | 0.310 → 0.774 | He gave the red race car eyes and the number 95. My son drew «lightning». | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| Q8 | two / lightning-context-keep | keep | abstain → keep | 0.016 → 0.310 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew «lightning». | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| Q8 | single / mater-context-name | abstain | abstain → keep | 0.081 → 0.164 | My son drew «Mader». | My son drew Mader. | My son drew Mader. |
| Q8 | two / mater-context-name | name | abstain → name | 0.288 → 0.412 | My son drew «Mader». He added the tow hook and rusty body from Cars. | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. |
| Q8 | single / mater-context-keep | abstain | abstain → keep | 0.081 → 0.164 | My son drew «Mader». | My son drew Mader. | My son drew Mader. |
| Q8 | single / flo-context-name | abstain | keep → name | 0.027 → 0.670 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| Q8 | single / flo-context-keep | abstain | keep → name | 0.027 → 0.670 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| Q8 | two / flo-context-keep | keep | keep → name | 0.058 → 0.720 | He drew arrows showing how water moves down a river. My son drew «flow». | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew Flo. |
| Q8 | single / son-neighbor-name | keep | keep → name | 0.077 → 0.493 | My colleague «Rowen» is presenting the report. | My colleague Rowen is presenting the report. | My colleague Rowan is presenting the report. |
| Q8 | two / project-neighbor-name | keep | name → abstain | 0.813 → 0.294 | I closed the transcrust dictation app. The label uses «trans crust» to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. |
| Q8 | single / habit-neighbor-name | keep | keep → name | 0.386 → 0.723 | I smoked «six» cigarettes today. | I smoked six cigarettes today. | I smoked Cigs cigarettes today. |
| Q8 | single / flo-neighbor-name | keep | keep → name | 0.047 → 0.824 | The river has a strong «flow» after the rain. | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| Q8 | two / initial-before | keep | name → keep | 0.684 → 0.294 | His full name is Rowan. My son «O» needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. | His full name is Rowan. My son O needs his lunchbox; use only his initial here. |
| Q8 | two / initial-after | keep | keep → name | 0.192 → 0.640 | Please refer to my son as «O» in this message. Rowan is his full name, which I keep private online. | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| Q8 | single / quoted-flow | keep | keep → name | 0.281 → 0.855 | Keep the quoted word exactly as written: «flow». | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |
| Q4 | single / son-context-name | abstain | keep → name | 0.341 → 0.712 | I spoke to «Rowen». | I spoke to Rowen. | I spoke to Rowan. |
| Q4 | two / son-context-name | name | keep → name | 0.392 → 0.555 | He is my son, and I was reminding him about his lunchbox. I spoke to «Rowen». | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowan. |
| Q4 | single / son-context-keep | abstain | keep → name | 0.341 → 0.712 | I spoke to «Rowen». | I spoke to Rowen. | I spoke to Rowan. |
| Q4 | two / habit-context-name | name | keep → name | 0.384 → 0.770 | I updated «six». It is the cigarette-counting habit in my tracker. | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| Q4 | two / lightning-context-name | name | abstain → name | 0.318 → 0.681 | He gave the red race car eyes and the number 95. My son drew «lightning». | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| Q4 | two / lightning-context-keep | keep | abstain → keep | 0.022 → 0.366 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew «lightning». | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| Q4 | single / mater-context-name | abstain | abstain → keep | 0.125 → 0.199 | My son drew «Mader». | My son drew Mader. | My son drew Mader. |
| Q4 | two / mater-context-name | name | name → keep | 0.380 → 0.278 | My son drew «Mader». He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. | My son drew Mader. He added the tow hook and rusty body from Cars. |
| Q4 | single / mater-context-keep | abstain | abstain → keep | 0.125 → 0.199 | My son drew «Mader». | My son drew Mader. | My son drew Mader. |
| Q4 | single / flo-context-name | abstain | keep → name | 0.078 → 0.487 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| Q4 | single / flo-context-keep | abstain | keep → name | 0.078 → 0.487 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| Q4 | two / flo-context-keep | keep | keep → name | 0.247 → 0.779 | He drew arrows showing how water moves down a river. My son drew «flow». | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew Flo. |
| Q4 | single / flo-neighbor-name | keep | keep → name | 0.237 → 0.829 | The river has a strong «flow» after the rain. | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| Q4 | two / initial-before | keep | name → keep | 0.525 → 0.180 | His full name is Rowan. My son «O» needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. | His full name is Rowan. My son O needs his lunchbox; use only his initial here. |
| Q4 | two / initial-after | keep | keep → name | 0.491 → 0.563 | Please refer to my son as «O» in this message. Rowan is his full name, which I keep private online. | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| Q4 | single / quoted-flow | keep | keep → name | 0.322 → 0.874 | Keep the quoted word exactly as written: «flow». | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |

Full glossary-run decisions: [Q4](two-sentence-glossary-q4/report.md), [Q8](two-sentence-glossary-q8/report.md). [Original paired-context report](two-sentence-report.md).

Regenerate this saved comparison:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/glossary-report.py
```

Re-run Q4 locally:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4 --cases tools/pair-router-probe/two-sentence.json --glossary tools/pair-router-probe/glossary.md
```
