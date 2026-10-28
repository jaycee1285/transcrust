# Bluey glossary expansion — d1 Q8

200 local inferences: 100 with the original five-entry glossary and 100 with four Bluey entries appended. Same pinned text-only graph, tokenizer, question, option descriptions, gold labels and native choice calibration. The 22 new scenarios each have a single- and two-sentence version; the 28 original paired scenarios are retained as regression controls.

## Added glossary

```text
| Bluey | 6-year-old main character, blue heeler |
| Bingo | Bluey's 4 year old sister |
| Chili | Bluey and Bingo's mom |
| Bandit | Bluey and Bingo's dad |
```

Names and descriptions follow the supplied glossary, including Chili. The three direct common/proper splits are bingo/Bingo, chili/Chili and bandit/Bandit. Bluey adds character positives and a blue/Bluey near-match color control. Some positives require only capitalization; others use synthetic split/homophone forms bin go, chilly, band it and blue e. These are supplied-candidate semantic probes, not observed NVIDIA errors or end-to-end transcription accuracy.

The original baseline still supplies candidate-specific descriptions in the choices; the change is the additional shared glossary. No gold answers enter model state. No API calls, audio replay, training or live changes.

## New Bluey cases

| Scope | Glossary | Character repairs | Name selections on common/ambiguous controls | Exact choices |
|---|---|---:|---:|---:|
| single | original | 2/8 | 0/14 | 10/22 |
| single | expanded | 2/8 | 1/14 | 7/22 |
| two | original | 7/12 | 1/10 | 16/22 |
| two | expanded | 6/12 | 1/10 | 15/22 |

## Two-sentence results per character

| Character | Glossary | Repairs | Name selections on common-word controls |
|---|---|---:|---:|
| Bluey | original | 2/3 | 0/1 |
| Bluey | expanded | 3/3 | 0/1 |
| Bingo | original | 2/3 | 0/3 |
| Bingo | expanded | 2/3 | 0/3 |
| Chili | original | 1/3 | 1/3 |
| Chili | expanded | 0/3 | 1/3 |
| Bandit | original | 2/3 | 0/3 |
| Bandit | expanded | 1/3 | 0/3 |

## Existing cases with expanded glossary

Excludes deliberate initials and quoted-spelling stress controls from this primary table. Those remain in the full receipts. Referent identification and edit permission are distinct; name selections below are proposed replacements by the harness, not an implemented edit policy.

| Scope | Glossary | Repairs | Name selections on non-repairs | Exact choices |
|---|---|---:|---:|---:|
| single | original | 0/0 | 7/24 | 4/24 |
| single | expanded | 0/0 | 8/24 | 3/24 |
| two | original | 7/8 | 6/16 | 16/24 |
| two | expanded | 6/8 | 6/16 | 16/24 |

KEEP and abstention both preserve text; exact choices distinguish them. No confidence floor. Cases are small and deliberately include ambiguity and nearby-name distractions; their error fraction is not everyday dictation WER.

## Resources

| Glossary | Input tokens | Median two-sentence inference (ms) | End RSS (MiB) |
|---|---|---:|---:|
| original | 111–153 | 901 | 519 |
| expanded | 163–205 | 1211 | 518 |

The expansion adds 52 actual input tokens. Zero truncation. CPU, two inference threads, one warm-up. Timings are separate laptop runs under varying load, not an isolated benchmark or mobile measurement.

## Every Bluey decision and proposed output

| Scope / case | Expected | Original → expanded choice | P(name) original → expanded | Dictation | Original-glossary output | Expanded-glossary output |
|---|---|---|---:|---|---|---|
| single / bluey-bingo-explicit-1 | name | name → abstain | 0.903 → 0.212 | My son wants a toy of «bingo», the little sister from Bluey. | My son wants a toy of Bingo, the little sister from Bluey. | My son wants a toy of bingo, the little sister from Bluey. |
| two / bluey-bingo-explicit-1 | name | name → name | 0.984 → 0.747 | My son wants a toy of «bingo», the little sister from Bluey. She is one of his favorite characters. | My son wants a toy of Bingo, the little sister from Bluey. She is one of his favorite characters. | My son wants a toy of Bingo, the little sister from Bluey. She is one of his favorite characters. |
| single / bluey-bingo-explicit-2 | name | keep → keep | 0.262 → 0.284 | We watched the episode where «bin go» plays with her older sister Bluey. | We watched the episode where bin go plays with her older sister Bluey. | We watched the episode where bin go plays with her older sister Bluey. |
| two / bluey-bingo-explicit-2 | name | name → name | 0.704 → 0.659 | My son loves that character. We watched the episode where «bin go» plays with her older sister Bluey. | My son loves that character. We watched the episode where Bingo plays with her older sister Bluey. | My son loves that character. We watched the episode where Bingo plays with her older sister Bluey. |
| single / bluey-bingo-context-name | abstain | keep → keep | 0.011 → 0.058 | My son asked about «bingo». | My son asked about bingo. | My son asked about bingo. |
| two / bluey-bingo-context-name | name | keep → abstain | 0.229 → 0.182 | My son asked about «bingo». He meant Bluey's four-year-old sister. | My son asked about bingo. He meant Bluey's four-year-old sister. | My son asked about bingo. He meant Bluey's four-year-old sister. |
| single / bluey-bingo-context-common | abstain | keep → keep | 0.011 → 0.058 | My son asked about «bingo». | My son asked about bingo. | My son asked about bingo. |
| two / bluey-bingo-context-common | keep | keep → keep | 0.002 → 0.017 | My son asked about «bingo». He wanted to know the rules of the number-calling game. | My son asked about bingo. He wanted to know the rules of the number-calling game. | My son asked about bingo. He wanted to know the rules of the number-calling game. |
| single / bluey-bingo-common-1 | keep | keep → keep | 0.008 → 0.035 | Grandma plays «bingo» at the community center. | Grandma plays bingo at the community center. | Grandma plays bingo at the community center. |
| two / bluey-bingo-common-1 | keep | keep → keep | 0.053 → 0.089 | My son was watching Bingo and Bluey on television. Grandma plays «bingo» at the community center. | My son was watching Bingo and Bluey on television. Grandma plays bingo at the community center. | My son was watching Bingo and Bluey on television. Grandma plays bingo at the community center. |
| single / bluey-bingo-common-2 | keep | keep → keep | 0.008 → 0.041 | You win at «bingo» by marking the called numbers on your card. | You win at bingo by marking the called numbers on your card. | You win at bingo by marking the called numbers on your card. |
| two / bluey-bingo-common-2 | keep | keep → keep | 0.005 → 0.010 | You win at «bingo» by marking the called numbers on your card. We bought a Bingo toy for my son. | You win at bingo by marking the called numbers on your card. We bought a Bingo toy for my son. | You win at bingo by marking the called numbers on your card. We bought a Bingo toy for my son. |
| single / bluey-chili-explicit-1 | name | keep → keep | 0.019 → 0.013 | My son drew «chili», Bluey and Bingo's mom. | My son drew chili, Bluey and Bingo's mom. | My son drew chili, Bluey and Bingo's mom. |
| two / bluey-chili-explicit-1 | name | keep → keep | 0.123 → 0.051 | My son drew «chili», Bluey and Bingo's mom. She is one of his favorite characters. | My son drew chili, Bluey and Bingo's mom. She is one of his favorite characters. | My son drew chili, Bluey and Bingo's mom. She is one of his favorite characters. |
| single / bluey-chili-explicit-2 | name | abstain → abstain | 0.373 → 0.202 | In Bluey, «chilly» is the girls' mother. | In Bluey, chilly is the girls' mother. | In Bluey, chilly is the girls' mother. |
| two / bluey-chili-explicit-2 | name | name → abstain | 0.540 → 0.181 | My son loves that character. In Bluey, «chilly» is the girls' mother. | My son loves that character. In Bluey, Chili is the girls' mother. | My son loves that character. In Bluey, chilly is the girls' mother. |
| single / bluey-chili-context-name | abstain | keep → keep | 0.001 → 0.006 | My son asked about «chili». | My son asked about chili. | My son asked about chili. |
| two / bluey-chili-context-name | name | keep → keep | 0.263 → 0.074 | He meant the mom of Bluey and Bingo. My son asked about «chili». | He meant the mom of Bluey and Bingo. My son asked about chili. | He meant the mom of Bluey and Bingo. My son asked about chili. |
| single / bluey-chili-context-common | abstain | keep → keep | 0.001 → 0.006 | My son asked about «chili». | My son asked about chili. | My son asked about chili. |
| two / bluey-chili-context-common | keep | keep → keep | 0.004 → 0.030 | He wanted to know whether the dish contains hot peppers. My son asked about «chili». | He wanted to know whether the dish contains hot peppers. My son asked about chili. | He wanted to know whether the dish contains hot peppers. My son asked about chili. |
| single / bluey-chili-common-1 | keep | keep → keep | 0.001 → 0.002 | I cooked a pot of «chili» with beans for dinner. | I cooked a pot of chili with beans for dinner. | I cooked a pot of chili with beans for dinner. |
| two / bluey-chili-common-1 | keep | name → name | 0.970 → 0.761 | My son was watching Chili and her daughters in Bluey. I cooked a pot of «chili» with beans for dinner. | My son was watching Chili and her daughters in Bluey. I cooked a pot of Chili with beans for dinner. | My son was watching Chili and her daughters in Bluey. I cooked a pot of Chili with beans for dinner. |
| single / bluey-chili-common-2 | keep | keep → abstain | 0.003 → 0.011 | It feels «chilly» outside this morning. | It feels chilly outside this morning. | It feels chilly outside this morning. |
| two / bluey-chili-common-2 | keep | keep → keep | 0.002 → 0.000 | It feels «chilly» outside this morning. My son asked for a toy of Chili from Bluey. | It feels chilly outside this morning. My son asked for a toy of Chili from Bluey. | It feels chilly outside this morning. My son asked for a toy of Chili from Bluey. |
| single / bluey-bandit-explicit-1 | name | keep → keep | 0.288 → 0.246 | In Bluey, «bandit» is the dad of the two girls. | In Bluey, bandit is the dad of the two girls. | In Bluey, bandit is the dad of the two girls. |
| two / bluey-bandit-explicit-1 | name | name → name | 0.863 → 0.746 | In Bluey, «bandit» is the dad of the two girls. He is one of his favorite characters. | In Bluey, Bandit is the dad of the two girls. He is one of his favorite characters. | In Bluey, Bandit is the dad of the two girls. He is one of his favorite characters. |
| single / bluey-bandit-explicit-2 | name | abstain → keep | 0.317 → 0.246 | My son wants a toy of «band it», Bluey's father. | My son wants a toy of band it, Bluey's father. | My son wants a toy of band it, Bluey's father. |
| two / bluey-bandit-explicit-2 | name | name → keep | 0.446 → 0.190 | My son loves that character. My son wants a toy of «band it», Bluey's father. | My son loves that character. My son wants a toy of Bandit, Bluey's father. | My son loves that character. My son wants a toy of band it, Bluey's father. |
| single / bluey-bandit-context-name | abstain | keep → keep | 0.003 → 0.004 | My son asked about «bandit». | My son asked about bandit. | My son asked about bandit. |
| two / bluey-bandit-context-name | name | keep → keep | 0.069 → 0.060 | My son asked about «bandit». He meant the dad of Bluey and Bingo. | My son asked about bandit. He meant the dad of Bluey and Bingo. | My son asked about bandit. He meant the dad of Bluey and Bingo. |
| single / bluey-bandit-context-common | abstain | keep → keep | 0.003 → 0.004 | My son asked about «bandit». | My son asked about bandit. | My son asked about bandit. |
| two / bluey-bandit-context-common | keep | keep → keep | 0.000 → 0.001 | My son asked about «bandit». He meant a robber who steals from travelers. | My son asked about bandit. He meant a robber who steals from travelers. | My son asked about bandit. He meant a robber who steals from travelers. |
| single / bluey-bandit-common-1 | keep | keep → keep | 0.001 → 0.011 | The «bandit» stole the travelers' money. | The bandit stole the travelers' money. | The bandit stole the travelers' money. |
| two / bluey-bandit-common-1 | keep | keep → keep | 0.005 → 0.015 | My son was talking about Bandit from Bluey. The «bandit» stole the travelers' money. | My son was talking about Bandit from Bluey. The bandit stole the travelers' money. | My son was talking about Bandit from Bluey. The bandit stole the travelers' money. |
| single / bluey-bandit-common-2 | keep | keep → keep | 0.001 → 0.019 | The sheriff caught the «bandit» after the robbery. | The sheriff caught the bandit after the robbery. | The sheriff caught the bandit after the robbery. |
| two / bluey-bandit-common-2 | keep | keep → keep | 0.012 → 0.109 | The sheriff caught the «bandit» after the robbery. We watched Bandit playing with his daughters in Bluey. | The sheriff caught the bandit after the robbery. We watched Bandit playing with his daughters in Bluey. | The sheriff caught the bandit after the robbery. We watched Bandit playing with his daughters in Bluey. |
| single / bluey-bluey-explicit-1 | name | name → name | 0.842 → 0.897 | My son wants a toy of «bluey», the blue heeler from the cartoon. | My son wants a toy of Bluey, the blue heeler from the cartoon. | My son wants a toy of Bluey, the blue heeler from the cartoon. |
| two / bluey-bluey-explicit-1 | name | name → name | 0.889 → 0.906 | My son wants a toy of «bluey», the blue heeler from the cartoon. She is his favorite character. | My son wants a toy of Bluey, the blue heeler from the cartoon. She is his favorite character. | My son wants a toy of Bluey, the blue heeler from the cartoon. She is his favorite character. |
| single / bluey-bluey-explicit-2 | name | abstain → name | 0.470 → 0.882 | In the cartoon, «blue e» is Bingo's older sister. | In the cartoon, blue e is Bingo's older sister. | In the cartoon, Bluey is Bingo's older sister. |
| two / bluey-bluey-explicit-2 | name | abstain → name | 0.301 → 0.589 | My son likes her games. In the cartoon, «blue e» is Bingo's older sister. | My son likes her games. In the cartoon, blue e is Bingo's older sister. | My son likes her games. In the cartoon, Bluey is Bingo's older sister. |
| single / bluey-bluey-context-name | abstain | abstain → keep | 0.165 → 0.114 | My son asked about «blue e». | My son asked about blue e. | My son asked about blue e. |
| two / bluey-bluey-context-name | name | name → name | 0.414 → 0.542 | My son asked about «blue e». He meant the six-year-old blue heeler who is Bingo's older sister. | My son asked about Bluey. He meant the six-year-old blue heeler who is Bingo's older sister. | My son asked about Bluey. He meant the six-year-old blue heeler who is Bingo's older sister. |
| single / bluey-bluey-color-neighbor | keep | keep → name | 0.127 → 0.496 | The sky looks «blue» today. | The sky looks blue today. | The sky looks Bluey today. |
| two / bluey-bluey-color-neighbor | keep | keep → keep | 0.174 → 0.205 | My son was watching Bluey with his mom. The sky looks «blue» today. | My son was watching Bluey with his mom. The sky looks blue today. | My son was watching Bluey with his mom. The sky looks blue today. |

## Changes to existing cases

| Scope / case | Expected | Original → expanded | P(name) original → expanded | Dictation |
|---|---|---|---:|---|
| two / dog-context-name | name | name → keep | 0.632 → 0.461 | I was thinking about «gamble». She is my dog, and I was planning her next walk. |
| two / flo-context-keep | keep | name → keep | 0.720 → 0.327 | He drew arrows showing how water moves down a river. My son drew «flow». |
| single / project-neighbor-name | keep | keep → abstain | 0.026 → 0.046 | The label uses «trans crust» to describe movement through a crust. |
| two / project-neighbor-name | keep | abstain → name | 0.294 → 0.598 | I closed the transcrust dictation app. The label uses «trans crust» to describe movement through a crust. |
| single / mater-neighbor-name | keep | abstain → name | 0.258 → 0.593 | Our neighbor Mr «Mader» repaired the fence. |
| two / initial-before | keep | keep → name | 0.294 → 0.650 | His full name is Rowan. My son «O» needs his lunchbox; use only his initial here. |

Regenerate the saved comparison:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/bluey-report.py
```

Re-run expanded-glossary Q8:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q8 --cases tools/pair-router-probe/bluey-expanded-suite.json --glossary tools/pair-router-probe/glossary-bluey.md --output-dir tools/pair-router-probe/bluey-q8-expanded-glossary
```
