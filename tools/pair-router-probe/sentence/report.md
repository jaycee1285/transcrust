# Sentence-and-pair decision probe

Model: local Verdict/OpenJev ModernBERT 151M, fp16 ONNX, CPU. Per-3-choice temperature: 5.0069.

Context scope: marked sentence.

Uses Verdict’s published label/SEP prompt contract. Each call has KEEP, the supplied canonical name, and insufficient evidence. Model probabilities are not validated calibration for this correction domain.

Record-2 rows come from the new **unboosted delivered** Parakeet transcript; the existing phonetic/casing post-pass still ran. Labels use the existing fixture notes, Dayshade observed pairs, and known task names. These selected repair labels are not a complete human reference transcript or full WER. Typed sentence controls are reported separately from recorded speech.

| Source | Confidence floor | Repairs made / wanted | Wrong edits on controls |
|---|---:|---:|---:|
| record-2 | 0.0 | 6/8 | 0/0 |
| record-2 | 0.5 | 4/8 | 0/0 |
| record-2 | 0.7 | 0/8 | 0/0 |
| record-2 | 0.9 | 0/8 | 0/0 |
| typed | 0.0 | 4/4 | 0/7 |
| typed | 0.5 | 3/4 | 0/7 |
| typed | 0.7 | 0/4 | 0/7 |
| typed | 0.9 | 0/4 | 0/7 |

Model load: 1.20s. Median inference: 83ms.

| Case | Heard → candidate | Want | Chose | P(keep) | P(name) | P(abstain) |
|---|---|---|---|---:|---:|---:|
| record-sigs-1 | Sigs → Cigs | name | name | 0.119 | 0.618 | 0.263 |
| record-job-supply-1 | job supply → Jobs apply | name | keep | 0.377 | 0.315 | 0.308 |
| record-terakeet-1 | terrakeet → Terakeet | name | name | 0.179 | 0.471 | 0.350 |
| record-apptrack-1 | At track → Apptrack | name | keep | 0.480 | 0.255 | 0.265 |
| record-rowan-1 | orangue → Rowan | name | name | 0.166 | 0.553 | 0.281 |
| record-backroom-1 | backroom vacuum → Back room vacuum | name | name | 0.119 | 0.659 | 0.223 |
| record-dominion-1 | paid dominion → Pay Dominion | name | name | 0.166 | 0.538 | 0.296 |
| record-dominion-2 | paid dominion → Pay Dominion | name | name | 0.337 | 0.376 | 0.287 |
| dog-in-user-sentence | Gamble → Gambol | name | name | 0.124 | 0.661 | 0.215 |
| casino-in-user-sentence | gamble → Gambol | keep | keep | 0.498 | 0.174 | 0.329 |
| dog-lowercase | gamble → Gambol | name | name | 0.226 | 0.579 | 0.194 |
| casino-short | gamble → Gambol | keep | keep | 0.546 | 0.164 | 0.290 |
| casino-capitalized | Gamble → Gambol | keep | keep | 0.546 | 0.158 | 0.295 |
| son-in-user-sentence | Rowen → Rowan | name | name | 0.166 | 0.615 | 0.219 |
| six-cards | six → Cigs | keep | keep | 0.574 | 0.094 | 0.333 |
| six-cigarettes | six → Cigs | name | name | 0.335 | 0.479 | 0.186 |
| job-supply-prose | job supply → Jobs apply | keep | keep | 0.470 | 0.234 | 0.297 |
| track-prose | at track → Apptrack | keep | keep | 0.486 | 0.252 | 0.262 |
| dominion-past | paid Dominion → Pay Dominion | keep | keep | 0.469 | 0.197 | 0.333 |

## Contexts

**record-sigs-1** (record-2): First, the four habits that I keep track of are job ads, which is at 0 out of 16, «Sigs», or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6.

**record-job-supply-1** (record-2): he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, «job supply», terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m.

**record-terakeet-1** (record-2): t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, «terrakeet» 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m.

**record-apptrack-1** (record-2): «At track» start.

**record-rowan-1** (record-2): I might also say something like «orangue» laundry complete.

**record-backroom-1** (record-2): The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, «backroom vacuum», bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m.

**record-dominion-1** (record-2): Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, «paid dominion» 360, pay BJWSA 100, Nell 220.

**record-dominion-2** (record-2): To give an example of a command that I might add, I might say «paid dominion» 360 plus 10.

**dog-in-user-sentence** (typed): I love my dog «Gamble» and my son Rowan.

**casino-in-user-sentence** (typed): She and I used to go to the casino to «gamble» and play cards.

**dog-lowercase** (typed): I love my dog «gamble».

**casino-short** (typed): I like to «gamble» and play poker with cards.

**casino-capitalized** (typed): «Gamble» and play cards at the casino.

**son-in-user-sentence** (typed): My son «Rowen»'s mother is named Nell.

**six-cards** (typed): I dealt «six» cards to each player.

**six-cigarettes** (typed): My cigarette habit is called «six».

**job-supply-prose** (typed): The «job supply» in this town has fallen.

**track-prose** (typed): I am «at track» practice with my son.

**dominion-past** (typed): I «paid Dominion» yesterday and the balance is now zero.
