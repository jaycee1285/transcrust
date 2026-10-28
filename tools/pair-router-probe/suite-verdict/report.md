# Sentence-and-pair decision probe

Model: local Verdict/OpenJev ModernBERT 151M, fp16 ONNX, CPU. Per-3-choice temperature: 5.0069.

Context scope: marked sentence.

Uses Verdict’s published label/SEP prompt contract. Each call has KEEP, the supplied canonical name, and insufficient evidence. Model probabilities are not validated calibration for this correction domain.

Cases: `suite.json`. Fixture SHA-256: `cbe6682ee9036dddcad327892101a8f9365eb93a5b5dc2c9d80f0a93e0c9956b`. Synthetic sentences measure supplied-candidate decisions, not audio recognition or end-to-end WER. Record-2 rows, when included, are selected repairs from the unboosted delivered transcript, including its phonetic/casing post-pass.

| Source | Confidence floor | Repairs made / wanted | Wrong edits on keep cases | Edits on ambiguous cases |
|---|---:|---:|---:|---:|
| synthetic | 0.0 | 20/20 | 6/20 | 8/20 |
| synthetic | 0.5 | 20/20 | 4/20 | 6/20 |
| synthetic | 0.7 | 1/20 | 0/20 | 0/20 |
| synthetic | 0.9 | 0/20 | 0/20 | 0/20 |

## Decision accuracy by category

| Category | Correct choice | Name edits on non-repair cases |
|---|---:|---:|
| dog/abstain | 0/4 | 0/4 |
| dog/keep | 3/4 | 1/4 |
| dog/name | 4/4 | 0/0 |
| habit/abstain | 0/4 | 4/4 |
| habit/keep | 2/4 | 2/4 |
| habit/name | 4/4 | 0/0 |
| mother/abstain | 0/4 | 0/4 |
| mother/keep | 4/4 | 0/4 |
| mother/name | 4/4 | 0/0 |
| project/abstain | 0/4 | 0/4 |
| project/keep | 3/4 | 0/4 |
| project/name | 4/4 | 0/0 |
| son/abstain | 0/4 | 4/4 |
| son/keep | 0/4 | 3/4 |
| son/name | 4/4 | 0/0 |

Model load: 1.67s. Median inference: 83ms.

| Case | Heard → candidate | Want | Chose | P(keep) | P(name) | P(abstain) |
|---|---|---|---|---:|---:|---:|
| dog-name-1 | Gamble → Gambol | name | name | 0.253 | 0.522 | 0.225 |
| dog-name-2 | gamble → Gambol | name | name | 0.127 | 0.612 | 0.260 |
| dog-name-3 | Gamble → Gambol | name | name | 0.180 | 0.595 | 0.225 |
| dog-name-4 | Gamble → Gambol | name | name | 0.186 | 0.566 | 0.248 |
| dog-keep-1 | gamble → Gambol | keep | keep | 0.493 | 0.153 | 0.355 |
| dog-keep-2 | Gamble → Gambol | keep | keep | 0.569 | 0.129 | 0.302 |
| dog-keep-3 | gamble → Gambol | keep | keep | 0.593 | 0.142 | 0.266 |
| dog-keep-4 | gamble → Gambol | keep | name | 0.283 | 0.478 | 0.238 |
| dog-abstain-1 | Gamble → Gambol | abstain | keep | 0.510 | 0.194 | 0.296 |
| dog-abstain-2 | Gamble → Gambol | abstain | keep | 0.468 | 0.227 | 0.305 |
| dog-abstain-3 | Gamble → Gambol | abstain | keep | 0.518 | 0.180 | 0.301 |
| dog-abstain-4 | Gamble → Gambol | abstain | keep | 0.523 | 0.187 | 0.290 |
| son-name-1 | Rowan → Rowan | name | name | 0.143 | 0.615 | 0.242 |
| son-name-2 | Rowan → Rowan | name | name | 0.145 | 0.606 | 0.249 |
| son-name-3 | Rowan → Rowan | name | name | 0.148 | 0.640 | 0.212 |
| son-name-4 | Rowan → Rowan | name | name | 0.223 | 0.561 | 0.216 |
| son-keep-1 | O → Rowan | keep | name | 0.251 | 0.502 | 0.247 |
| son-keep-2 | O → Rowan | keep | name | 0.198 | 0.554 | 0.248 |
| son-keep-3 | Rowan → Rowan | keep | name | 0.286 | 0.508 | 0.206 |
| son-keep-4 | Rowan → Rowan | keep | abstain | 0.338 | 0.309 | 0.353 |
| son-abstain-1 | Rowan → Rowan | abstain | name | 0.164 | 0.566 | 0.270 |
| son-abstain-2 | Rowan → Rowan | abstain | name | 0.153 | 0.566 | 0.281 |
| son-abstain-3 | Rowan → Rowan | abstain | name | 0.274 | 0.437 | 0.289 |
| son-abstain-4 | Rowan → Rowan | abstain | name | 0.213 | 0.469 | 0.318 |
| mother-name-1 | Knell → Nell | name | name | 0.200 | 0.545 | 0.255 |
| mother-name-2 | knell → Nell | name | name | 0.132 | 0.666 | 0.202 |
| mother-name-3 | Knell → Nell | name | name | 0.129 | 0.672 | 0.199 |
| mother-name-4 | Knell → Nell | name | name | 0.148 | 0.596 | 0.257 |
| mother-keep-1 | knell → Nell | keep | keep | 0.600 | 0.134 | 0.266 |
| mother-keep-2 | knell → Nell | keep | keep | 0.488 | 0.179 | 0.333 |
| mother-keep-3 | Knell → Nell | keep | keep | 0.607 | 0.124 | 0.269 |
| mother-keep-4 | knell → Nell | keep | keep | 0.475 | 0.237 | 0.288 |
| mother-abstain-1 | Knell → Nell | abstain | keep | 0.458 | 0.205 | 0.337 |
| mother-abstain-2 | Knell → Nell | abstain | keep | 0.452 | 0.210 | 0.338 |
| mother-abstain-3 | Knell → Nell | abstain | keep | 0.374 | 0.260 | 0.366 |
| mother-abstain-4 | Knell → Nell | abstain | keep | 0.396 | 0.255 | 0.349 |
| project-name-1 | trans crust → transcrust | name | name | 0.142 | 0.687 | 0.171 |
| project-name-2 | trans crust → transcrust | name | name | 0.128 | 0.713 | 0.159 |
| project-name-3 | trans crust → transcrust | name | name | 0.205 | 0.561 | 0.233 |
| project-name-4 | trans crust → transcrust | name | name | 0.169 | 0.612 | 0.219 |
| project-keep-1 | trans crust → transcrust | keep | abstain | 0.307 | 0.345 | 0.348 |
| project-keep-2 | trans crust → transcrust | keep | keep | 0.375 | 0.315 | 0.310 |
| project-keep-3 | trans crust → transcrust | keep | keep | 0.418 | 0.327 | 0.255 |
| project-keep-4 | trans crust → transcrust | keep | keep | 0.514 | 0.227 | 0.259 |
| project-abstain-1 | trans crust → transcrust | abstain | keep | 0.426 | 0.276 | 0.298 |
| project-abstain-2 | trans crust → transcrust | abstain | keep | 0.387 | 0.336 | 0.277 |
| project-abstain-3 | trans crust → transcrust | abstain | keep | 0.374 | 0.299 | 0.327 |
| project-abstain-4 | trans crust → transcrust | abstain | keep | 0.409 | 0.299 | 0.292 |
| habit-name-1 | sigs → Cigs | name | name | 0.096 | 0.691 | 0.214 |
| habit-name-2 | sigs → Cigs | name | name | 0.105 | 0.676 | 0.219 |
| habit-name-3 | sigs → Cigs | name | name | 0.106 | 0.678 | 0.216 |
| habit-name-4 | sigs → Cigs | name | name | 0.122 | 0.629 | 0.249 |
| habit-keep-1 | six → Cigs | keep | keep | 0.515 | 0.108 | 0.377 |
| habit-keep-2 | six → Cigs | keep | name | 0.345 | 0.437 | 0.218 |
| habit-keep-3 | Six → Cigs | keep | keep | 0.585 | 0.084 | 0.331 |
| habit-keep-4 | six → Cigs | keep | name | 0.271 | 0.547 | 0.182 |
| habit-abstain-1 | sigs → Cigs | abstain | name | 0.118 | 0.616 | 0.266 |
| habit-abstain-2 | sigs → Cigs | abstain | name | 0.118 | 0.600 | 0.283 |
| habit-abstain-3 | sigs → Cigs | abstain | name | 0.120 | 0.571 | 0.309 |
| habit-abstain-4 | sigs → Cigs | abstain | name | 0.116 | 0.620 | 0.263 |

## Contexts

**dog-name-1** (synthetic): Please put «Gamble» on her leash before taking my dog outside.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**dog-name-2** (synthetic): My dog «gamble» has a vet appointment tomorrow.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**dog-name-3** (synthetic): The dog's name is «Gamble», and she jumps on me when I say it.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**dog-name-4** (synthetic): I filled my dog «Gamble»'s water bowl.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**dog-keep-1** (synthetic): We should never «gamble» with the rent money.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**dog-keep-2** (synthetic): «Gamble» responsibly if you visit a casino.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**dog-keep-3** (synthetic): That investment would be a «gamble».

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**dog-keep-4** (synthetic): My dog is asleep while I «gamble» online.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**dog-abstain-1** (synthetic): «Gamble».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**dog-abstain-2** (synthetic): Tell «Gamble».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**dog-abstain-3** (synthetic): I heard someone say «Gamble».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**dog-abstain-4** (synthetic): The note only says «Gamble».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**son-name-1** (synthetic): My son «Rowan» forgot his lunchbox.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**son-name-2** (synthetic): Please pack my son «Rowan»'s school bag.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**son-name-3** (synthetic): Nell and I co-parent our son «Rowan».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**son-name-4** (synthetic): My son's name is «Rowan».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**son-keep-1** (synthetic): I deliberately call my son «O» online to protect his privacy; keep that initial.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**son-keep-2** (synthetic): My son «O» needs his lunchbox; use only his initial in this message.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**son-keep-3** (synthetic): My colleague «Rowan» is a different person from my son Rowan.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**son-keep-4** (synthetic): The author spells her name «Rowan», without an o at the end.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**son-abstain-1** (synthetic): «Rowan».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**son-abstain-2** (synthetic): Ask «Rowan» about it.

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**son-abstain-3** (synthetic): Someone called «Rowan» left a message.

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**son-abstain-4** (synthetic): The label says «Rowan».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**mother-name-1** (synthetic): My son's mother «Knell» will collect him from school.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**mother-name-2** (synthetic): Rowan's mother is named «knell».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**mother-name-3** (synthetic): I co-parent with Rowan's mother «Knell».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**mother-name-4** (synthetic): Please send Rowan's mother «Knell» the custody schedule.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**mother-keep-1** (synthetic): The church bell sounded a funeral «knell».

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**mother-keep-2** (synthetic): That decision was the death «knell» for the business.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**mother-keep-3** (synthetic): «Knell» means the sound of a bell marking a death.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**mother-keep-4** (synthetic): Nell heard the mournful «knell» of the bell.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**mother-abstain-1** (synthetic): «Knell».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**mother-abstain-2** (synthetic): I heard «Knell».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**mother-abstain-3** (synthetic): The recording only contains «Knell».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**mother-abstain-4** (synthetic): Write down «Knell».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**project-name-1** (synthetic): Open the Linux dictation app «trans crust».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**project-name-2** (synthetic): The Linux version of this dictation project is called «trans crust».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**project-name-3** (synthetic): Restart the «trans crust» dictation daemon.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**project-name-4** (synthetic): I use «trans crust» for push-to-talk voice input on Wayland.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**project-keep-1** (synthetic): The geologist uses «trans crust» as two separate words; preserve the quotation.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**project-keep-2** (synthetic): Keep «trans crust» split into two words in this language exercise.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**project-keep-3** (synthetic): Do not turn the quoted phrase «trans crust» into the application name.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**project-keep-4** (synthetic): The phrase «trans crust» here describes a geological process, not software.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**project-abstain-1** (synthetic): «trans crust».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**project-abstain-2** (synthetic): The note says «trans crust».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**project-abstain-3** (synthetic): Someone mentioned «trans crust».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**project-abstain-4** (synthetic): What about «trans crust»?

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**habit-name-1** (synthetic): My cigarette habit is named «sigs».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**habit-name-2** (synthetic): Add one cigarette to my «sigs» habit counter.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**habit-name-3** (synthetic): The «sigs» habit tracks how many cigarettes I smoke.

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**habit-name-4** (synthetic): Reset my cigarette counter called «sigs».

Expected **name**: The sentence explicitly identifies the supplied glossary referent.

**habit-keep-1** (synthetic): Deal «six» cards to each player.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**habit-keep-2** (synthetic): I smoked «six» cigarettes today.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**habit-keep-3** (synthetic): «Six» people attended the meeting.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**habit-keep-4** (synthetic): Set the Cigs habit target to «six» cigarettes.

Expected **keep**: The sentence specifies an ordinary meaning, another referent, or spelling that must be preserved.

**habit-abstain-1** (synthetic): «sigs».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**habit-abstain-2** (synthetic): The note says «sigs».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**habit-abstain-3** (synthetic): I heard the word «sigs».

Expected **abstain**: The sentence does not establish whether this is the glossary referent.

**habit-abstain-4** (synthetic): What is «sigs»?

Expected **abstain**: The sentence does not establish whether this is the glossary referent.
