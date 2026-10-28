# Sentence-and-pair decision probe

Model: local Verdict/OpenJev ModernBERT 151M, fp16 ONNX, CPU. Per-3-choice temperature: 5.0069.

Uses Verdict’s published label/SEP prompt contract. Each call has KEEP, the supplied canonical name, and insufficient evidence. Model probabilities are not validated calibration for this correction domain.

Record-2 rows come from the new **unboosted delivered** Parakeet transcript; the existing phonetic/casing post-pass still ran. Labels use the existing fixture notes, Dayshade observed pairs, and known task names. These selected repair labels are not a complete human reference transcript or full WER. Typed sentence controls are reported separately from recorded speech.

| Source | Confidence floor | Repairs made / wanted | Wrong edits on controls |
|---|---:|---:|---:|
| record-2 | 0.0 | 6/8 | 0/0 |
| record-2 | 0.5 | 5/8 | 0/0 |
| record-2 | 0.7 | 0/8 | 0/0 |
| record-2 | 0.9 | 0/8 | 0/0 |
| typed | 0.0 | 4/4 | 1/7 |
| typed | 0.5 | 4/4 | 1/7 |
| typed | 0.7 | 0/4 | 0/7 |
| typed | 0.9 | 0/4 | 0/7 |

Model load: 1.47s. Median inference: 113ms.

| Case | Heard → candidate | Want | Chose | P(keep) | P(name) | P(abstain) |
|---|---|---|---|---:|---:|---:|
| record-sigs-1 | Sigs → Cigs | name | name | 0.090 | 0.634 | 0.276 |
| record-job-supply-1 | job supply → Jobs apply | name | keep | 0.348 | 0.344 | 0.308 |
| record-terakeet-1 | terrakeet → Terakeet | name | name | 0.188 | 0.447 | 0.365 |
| record-apptrack-1 | At track → Apptrack | name | keep | 0.355 | 0.343 | 0.302 |
| record-orion-1 | orangue → Rowan | name | name | 0.127 | 0.595 | 0.278 |
| record-backroom-1 | backroom vacuum → Back room vacuum | name | name | 0.150 | 0.597 | 0.253 |
| record-dominion-1 | paid dominion → Pay Dominion | name | name | 0.177 | 0.572 | 0.251 |
| record-dominion-2 | paid dominion → Pay Dominion | name | name | 0.160 | 0.613 | 0.226 |
| dog-in-user-sentence | Gamble → Gambol | name | name | 0.252 | 0.546 | 0.202 |
| casino-in-user-sentence | gamble → Gambol | keep | name | 0.255 | 0.534 | 0.210 |
| dog-lowercase | gamble → Gambol | name | name | 0.226 | 0.579 | 0.194 |
| casino-short | gamble → Gambol | keep | keep | 0.546 | 0.164 | 0.290 |
| casino-capitalized | Gamble → Gambol | keep | keep | 0.546 | 0.158 | 0.295 |
| son-in-user-sentence | Rowan → Rowan | name | name | 0.129 | 0.655 | 0.216 |
| six-cards | six → Cigs | keep | keep | 0.574 | 0.094 | 0.333 |
| six-cigarettes | six → Cigs | name | name | 0.297 | 0.507 | 0.196 |
| job-supply-prose | job supply → Jobs apply | keep | keep | 0.470 | 0.234 | 0.297 |
| track-prose | at track → Apptrack | keep | keep | 0.486 | 0.252 | 0.262 |
| dominion-past | paid Dominion → Pay Dominion | keep | keep | 0.469 | 0.197 | 0.333 |

## Contexts

**record-sigs-1** (record-2): by talking about some of the tasks, not just the habits, that I have in my app called Daylight that I use to both track time and keep habit counts. First, the four habits that I keep track of are job ads, which is at 0 out of 16, «Sigs», or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, b

**record-job-supply-1** (record-2): he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, «job supply», terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. At track start. Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA

**record-terakeet-1** (record-2): t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, «terrakeet» 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. At track start. Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 2

**record-apptrack-1** (record-2): tfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. «At track» start. Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. To give an example of a command that I might add, I migh

**record-orion-1** (record-2): nclude Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. To give an example of a command that I might add, I might say paid dominion 360 plus 10. I might also say something like «orangue» laundry complete.

**record-backroom-1** (record-2): , which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, «backroom vacuum», bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. A

**record-dominion-1** (record-2): , trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. At track start. Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, «paid dominion» 360, pay BJWSA 100, Nell 220. To give an example of a command that I might add, I might say paid dominion 360 plus 10. I might also say something like orangue laundry complete.

**record-dominion-2** (record-2): nning 8 30 a.m. At track start. Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. To give an example of a command that I might add, I might say «paid dominion» 360 plus 10. I might also say something like orangue laundry complete.

**dog-in-user-sentence** (typed): I love my dog «Gamble» and my son Rowan. My son Rowan's mother is named Nell. She and I used to go to the casino to gamble and play cards.

**casino-in-user-sentence** (typed): I love my dog Gamble and my son Rowan. My son Rowan's mother is named Nell. She and I used to go to the casino to «gamble» and play cards.

**dog-lowercase** (typed): I love my dog «gamble».

**casino-short** (typed): I like to «gamble» and play poker with cards.

**casino-capitalized** (typed): «Gamble» and play cards at the casino.

**son-in-user-sentence** (typed): I love my dog Gamble and my son Rowan. My son «Rowan»'s mother is named Nell. She and I used to go to the casino to gamble and play cards.

**six-cards** (typed): I dealt «six» cards to each player.

**six-cigarettes** (typed): My cigarette habit is called «six». Add two cigarettes.

**job-supply-prose** (typed): The «job supply» in this town has fallen.

**track-prose** (typed): I am «at track» practice with my son.

**dominion-past** (typed): I «paid Dominion» yesterday and the balance is now zero.
