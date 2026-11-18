# d1 Q4 text comparison

One CPU session, two inference threads, one warm-up call. Text graph only; no media encoders. Same saved questions, descriptions and sentence scopes as prior probes, using d1’s native formatter and choice calibration.

Load 0.40s; median inference 330ms; RSS after load 246 MiB; RSS at end 332 MiB; process peak RSS 333 MiB.

RSS includes Python, tokenizer, ONNX Runtime and CPU allocator caches. These are laptop measurements, not mobile RAM/latency measurements. API timings include network overhead; local timings measure inference.

| Arm | Model | Floor | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |
|---|---|---:|---:|---:|---:|---:|
| sentence | Verdict | 0.0 | 10/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.5 | 7/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.7 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.9 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | d1 Q4 | 0.0 | 4/12 | 0/7 | 0/0 | 11/19 |
| sentence | d1 Q4 | 0.5 | 2/12 | 0/7 | 0/0 | 11/19 |
| sentence | d1 Q4 | 0.7 | 2/12 | 0/7 | 0/0 | 11/19 |
| sentence | d1 Q4 | 0.9 | 0/12 | 0/7 | 0/0 | 11/19 |
| sentence | typesafe/jev-1.13 | 0.0 | 12/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.5 | 11/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.7 | 10/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.9 | 8/12 | 0/7 | 0/0 | 19/19 |
| sentence | cloudflare/clef-flash | 0.0 | 11/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.5 | 11/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.7 | 10/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.9 | 7/12 | 0/7 | 0/0 | 18/19 |
| suite-verdict | Verdict | 0.0 | 20/20 | 6/20 | 8/20 | 32/60 |
| suite-verdict | Verdict | 0.5 | 20/20 | 4/20 | 6/20 | 32/60 |
| suite-verdict | Verdict | 0.7 | 1/20 | 0/20 | 0/20 | 32/60 |
| suite-verdict | Verdict | 0.9 | 0/20 | 0/20 | 0/20 | 32/60 |
| suite-verdict | d1 Q4 | 0.0 | 18/20 | 1/20 | 6/20 | 38/60 |
| suite-verdict | d1 Q4 | 0.5 | 18/20 | 1/20 | 6/20 | 38/60 |
| suite-verdict | d1 Q4 | 0.7 | 14/20 | 1/20 | 5/20 | 38/60 |
| suite-verdict | d1 Q4 | 0.9 | 7/20 | 0/20 | 3/20 | 38/60 |
| suite-verdict | typesafe/jev-1.13 | 0.0 | 20/20 | 0/20 | 7/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.5 | 20/20 | 0/20 | 6/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.7 | 20/20 | 0/20 | 1/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.9 | 20/20 | 0/20 | 0/20 | 45/60 |
| suite-verdict | cloudflare/clef-flash | 0.0 | 19/20 | 2/20 | 3/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.5 | 19/20 | 2/20 | 3/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.7 | 17/20 | 2/20 | 1/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.9 | 13/20 | 1/20 | 0/20 | 41/60 |
| cars-verdict | Verdict | 0.0 | 6/6 | 1/6 | 1/6 | 11/18 |
| cars-verdict | Verdict | 0.5 | 5/6 | 1/6 | 0/6 | 11/18 |
| cars-verdict | Verdict | 0.7 | 2/6 | 0/6 | 0/6 | 11/18 |
| cars-verdict | Verdict | 0.9 | 0/6 | 0/6 | 0/6 | 11/18 |
| cars-verdict | d1 Q4 | 0.0 | 5/6 | 0/6 | 1/6 | 12/18 |
| cars-verdict | d1 Q4 | 0.5 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | d1 Q4 | 0.7 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | d1 Q4 | 0.9 | 4/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | typesafe/jev-1.13 | 0.0 | 6/6 | 0/6 | 1/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.5 | 6/6 | 0/6 | 1/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.7 | 6/6 | 0/6 | 0/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.9 | 6/6 | 0/6 | 0/6 | 17/18 |
| cars-verdict | cloudflare/clef-flash | 0.0 | 6/6 | 1/6 | 1/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.5 | 6/6 | 1/6 | 0/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.7 | 6/6 | 1/6 | 0/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.9 | 6/6 | 0/6 | 0/6 | 12/18 |
| suite-verdict-correction | Verdict | 0.0 | 20/20 | 18/20 | 18/20 | 22/60 |
| suite-verdict-correction | Verdict | 0.5 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | Verdict | 0.7 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | Verdict | 0.9 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | d1 Q4 | 0.0 | 13/20 | 9/20 | 16/20 | 24/60 |
| suite-verdict-correction | d1 Q4 | 0.5 | 9/20 | 6/20 | 14/20 | 24/60 |
| suite-verdict-correction | d1 Q4 | 0.7 | 3/20 | 0/20 | 8/20 | 24/60 |
| suite-verdict-correction | d1 Q4 | 0.9 | 0/20 | 0/20 | 0/20 | 24/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.0 | 17/20 | 1/20 | 1/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.5 | 16/20 | 0/20 | 1/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.7 | 10/20 | 0/20 | 0/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.9 | 1/20 | 0/20 | 0/20 | 50/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.0 | 18/20 | 0/20 | 5/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.5 | 17/20 | 0/20 | 4/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.7 | 12/20 | 0/20 | 2/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.9 | 3/20 | 0/20 | 0/20 | 42/60 |

Exact decisions are raw choices, independent of the edit floor. KEEP and abstain both preserve the text. Floors are illustrative, not fitted correction-domain calibration.

## d1 decisions

| Arm | Case | Want | Choice | P(keep) | P(name) | P(abstain) | Original → proposed |
|---|---|---|---|---:|---:|---:|---|
| sentence | record-sigs-1 | name | name | 0.036 | 0.763 | 0.201 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Cigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | record-job-supply-1 | name | keep | 0.527 | 0.343 | 0.130 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-terakeet-1 | name | keep | 0.440 | 0.286 | 0.274 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-apptrack-1 | name | keep | 0.466 | 0.224 | 0.310 | At track start. → At track start. |
| sentence | record-rowan-1 | name | name | 0.179 | 0.464 | 0.356 | I might also say something like orangue laundry complete. → I might also say something like Rowan laundry complete. |
| sentence | record-backroom-1 | name | name | 0.415 | 0.495 | 0.090 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, Back room vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-dominion-1 | name | keep | 0.882 | 0.007 | 0.110 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. |
| sentence | record-dominion-2 | name | keep | 0.507 | 0.034 | 0.459 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say paid dominion 360 plus 10. |
| sentence | dog-in-user-sentence | name | name | 0.139 | 0.742 | 0.119 | I love my dog Gamble and my son Rowan. → I love my dog Gambol and my son Rowan. |
| sentence | casino-in-user-sentence | keep | keep | 0.997 | 0.000 | 0.003 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to gamble and play cards. |
| sentence | dog-lowercase | name | keep | 0.617 | 0.354 | 0.029 | I love my dog gamble. → I love my dog gamble. |
| sentence | casino-short | keep | keep | 0.996 | 0.000 | 0.004 | I like to gamble and play poker with cards. → I like to gamble and play poker with cards. |
| sentence | casino-capitalized | keep | keep | 0.976 | 0.000 | 0.024 | Gamble and play cards at the casino. → Gamble and play cards at the casino. |
| sentence | son-in-user-sentence | name | keep | 0.885 | 0.011 | 0.104 | My son Rowen's mother is named Nell. → My son Rowen's mother is named Nell. |
| sentence | six-cards | keep | keep | 0.997 | 0.000 | 0.003 | I dealt six cards to each player. → I dealt six cards to each player. |
| sentence | six-cigarettes | name | keep | 0.846 | 0.148 | 0.007 | My cigarette habit is called six. → My cigarette habit is called six. |
| sentence | job-supply-prose | keep | keep | 0.467 | 0.210 | 0.323 | The job supply in this town has fallen. → The job supply in this town has fallen. |
| sentence | track-prose | keep | keep | 0.723 | 0.045 | 0.232 | I am at track practice with my son. → I am at track practice with my son. |
| sentence | dominion-past | keep | keep | 0.967 | 0.007 | 0.026 | I paid Dominion yesterday and the balance is now zero. → I paid Dominion yesterday and the balance is now zero. |
| suite-verdict | dog-name-1 | name | keep | 0.888 | 0.042 | 0.070 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict | dog-name-2 | name | name | 0.389 | 0.541 | 0.070 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict | dog-name-3 | name | name | 0.374 | 0.532 | 0.094 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | dog-name-4 | name | name | 0.089 | 0.772 | 0.139 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict | dog-keep-1 | keep | keep | 0.875 | 0.001 | 0.124 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict | dog-keep-2 | keep | keep | 0.994 | 0.000 | 0.005 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict | dog-keep-3 | keep | keep | 0.968 | 0.000 | 0.032 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict | dog-keep-4 | keep | keep | 0.893 | 0.029 | 0.078 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict | dog-abstain-1 | abstain | keep | 0.924 | 0.001 | 0.075 | Gamble. → Gamble. |
| suite-verdict | dog-abstain-2 | abstain | keep | 0.950 | 0.000 | 0.049 | Tell Gamble. → Tell Gamble. |
| suite-verdict | dog-abstain-3 | abstain | keep | 0.958 | 0.000 | 0.042 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict | dog-abstain-4 | abstain | keep | 0.872 | 0.003 | 0.125 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict | son-name-1 | name | name | 0.091 | 0.887 | 0.022 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict | son-name-2 | name | name | 0.295 | 0.643 | 0.061 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | son-name-3 | name | name | 0.078 | 0.885 | 0.037 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict | son-name-4 | name | name | 0.082 | 0.892 | 0.026 | My son's name is Rowen. → My son's name is Rowan. |
| suite-verdict | son-keep-1 | keep | keep | 0.896 | 0.089 | 0.015 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | son-keep-2 | keep | keep | 0.673 | 0.312 | 0.015 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict | son-keep-3 | keep | keep | 0.769 | 0.127 | 0.104 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict | son-keep-4 | keep | abstain | 0.375 | 0.210 | 0.415 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict | son-abstain-1 | abstain | name | 0.284 | 0.511 | 0.205 | Rowen. → Rowan. |
| suite-verdict | son-abstain-2 | abstain | name | 0.245 | 0.706 | 0.049 | Ask Rowen about it. → Ask Rowan about it. |
| suite-verdict | son-abstain-3 | abstain | keep | 0.908 | 0.082 | 0.010 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict | son-abstain-4 | abstain | keep | 0.538 | 0.342 | 0.120 | The label says Rowen. → The label says Rowen. |
| suite-verdict | mother-name-1 | name | name | 0.068 | 0.804 | 0.128 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict | mother-name-2 | name | name | 0.080 | 0.892 | 0.029 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | mother-name-3 | name | name | 0.102 | 0.843 | 0.054 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict | mother-name-4 | name | name | 0.051 | 0.905 | 0.044 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict | mother-keep-1 | keep | keep | 0.989 | 0.003 | 0.008 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict | mother-keep-2 | keep | keep | 0.565 | 0.116 | 0.318 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict | mother-keep-3 | keep | keep | 0.977 | 0.007 | 0.016 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict | mother-keep-4 | keep | keep | 0.956 | 0.020 | 0.024 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict | mother-abstain-1 | abstain | keep | 0.686 | 0.106 | 0.208 | Knell. → Knell. |
| suite-verdict | mother-abstain-2 | abstain | keep | 0.933 | 0.024 | 0.043 | I heard Knell. → I heard Knell. |
| suite-verdict | mother-abstain-3 | abstain | keep | 0.454 | 0.253 | 0.293 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict | mother-abstain-4 | abstain | keep | 0.706 | 0.073 | 0.221 | Write down Knell. → Write down Knell. |
| suite-verdict | project-name-1 | name | name | 0.006 | 0.978 | 0.016 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict | project-name-2 | name | abstain | 0.196 | 0.181 | 0.622 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called trans crust. |
| suite-verdict | project-name-3 | name | name | 0.006 | 0.972 | 0.022 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict | project-name-4 | name | name | 0.105 | 0.531 | 0.364 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict | project-keep-1 | keep | keep | 0.551 | 0.004 | 0.445 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict | project-keep-2 | keep | keep | 0.890 | 0.002 | 0.108 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | project-keep-3 | keep | abstain | 0.427 | 0.087 | 0.487 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict | project-keep-4 | keep | keep | 0.682 | 0.001 | 0.316 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict | project-abstain-1 | abstain | abstain | 0.348 | 0.003 | 0.649 | trans crust. → trans crust. |
| suite-verdict | project-abstain-2 | abstain | keep | 0.543 | 0.023 | 0.435 | The note says trans crust. → The note says trans crust. |
| suite-verdict | project-abstain-3 | abstain | abstain | 0.320 | 0.007 | 0.673 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict | project-abstain-4 | abstain | abstain | 0.283 | 0.011 | 0.706 | What about trans crust? → What about trans crust? |
| suite-verdict | habit-name-1 | name | name | 0.010 | 0.914 | 0.076 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | habit-name-2 | name | name | 0.010 | 0.987 | 0.003 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict | habit-name-3 | name | name | 0.008 | 0.989 | 0.002 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict | habit-name-4 | name | name | 0.036 | 0.957 | 0.006 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict | habit-keep-1 | keep | keep | 0.988 | 0.000 | 0.012 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict | habit-keep-2 | keep | keep | 0.554 | 0.440 | 0.006 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict | habit-keep-3 | keep | keep | 0.992 | 0.000 | 0.008 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict | habit-keep-4 | keep | name | 0.274 | 0.720 | 0.005 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict | habit-abstain-1 | abstain | name | 0.027 | 0.955 | 0.017 | sigs. → Cigs. |
| suite-verdict | habit-abstain-2 | abstain | name | 0.203 | 0.754 | 0.043 | The note says sigs. → The note says Cigs. |
| suite-verdict | habit-abstain-3 | abstain | name | 0.059 | 0.928 | 0.013 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict | habit-abstain-4 | abstain | name | 0.003 | 0.993 | 0.003 | What is sigs? → What is Cigs? |
| cars-verdict | cars-lightning-name-1 | name | name | 0.002 | 0.981 | 0.018 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car Lightning McQueen for his birthday. |
| cars-verdict | cars-lightning-name-2 | name | name | 0.007 | 0.986 | 0.007 | In Cars, lightning McQueen learns to value his friends. → In Cars, Lightning McQueen learns to value his friends. |
| cars-verdict | cars-lightning-keep-1 | keep | keep | 0.542 | 0.227 | 0.232 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because lightning struck near the house. |
| cars-verdict | cars-lightning-keep-2 | keep | keep | 0.958 | 0.005 | 0.038 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | cars-lightning-abstain-1 | abstain | keep | 0.968 | 0.005 | 0.027 | Look at lightning. → Look at lightning. |
| cars-verdict | cars-lightning-abstain-2 | abstain | keep | 0.893 | 0.033 | 0.075 | The note says lightning. → The note says lightning. |
| cars-verdict | cars-mater-name-1 | name | name | 0.041 | 0.921 | 0.038 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mater is Lightning McQueen's friend. |
| cars-verdict | cars-mater-name-2 | name | name | 0.010 | 0.966 | 0.024 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | cars-mater-keep-1 | keep | keep | 0.837 | 0.037 | 0.126 | Mader has that surname; he is not the Cars character. → Mader has that surname; he is not the Cars character. |
| cars-verdict | cars-mater-keep-2 | keep | keep | 0.806 | 0.029 | 0.166 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a mater: it couples the two parts together. |
| cars-verdict | cars-mater-abstain-1 | abstain | keep | 0.587 | 0.076 | 0.338 | Mader is outside. → Mader is outside. |
| cars-verdict | cars-mater-abstain-2 | abstain | abstain | 0.198 | 0.128 | 0.675 | I heard someone say mater. → I heard someone say mater. |
| cars-verdict | cars-flo-name-1 | name | keep | 0.460 | 0.364 | 0.177 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, flow runs the V8 Cafe in Radiator Springs. |
| cars-verdict | cars-flo-name-2 | name | name | 0.059 | 0.852 | 0.089 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of Flo, the Cars character who runs the diner. |
| cars-verdict | cars-flo-keep-1 | keep | keep | 0.911 | 0.047 | 0.042 | The water flow slowed when I closed the tap. → The water flow slowed when I closed the tap. |
| cars-verdict | cars-flo-keep-2 | keep | keep | 0.678 | 0.211 | 0.111 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic flow. |
| cars-verdict | cars-flo-abstain-1 | abstain | keep | 0.678 | 0.178 | 0.144 | Look at flow. → Look at flow. |
| cars-verdict | cars-flo-abstain-2 | abstain | name | 0.322 | 0.445 | 0.234 | The note only says flow. → The note only says Flo. |
| suite-verdict-correction | dog-name-1 | name | keep | 0.389 | 0.372 | 0.239 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict-correction | dog-name-2 | name | keep | 0.464 | 0.366 | 0.170 | My dog gamble has a vet appointment tomorrow. → My dog gamble has a vet appointment tomorrow. |
| suite-verdict-correction | dog-name-3 | name | name | 0.287 | 0.615 | 0.098 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict-correction | dog-name-4 | name | name | 0.343 | 0.458 | 0.199 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict-correction | dog-keep-1 | keep | name | 0.423 | 0.439 | 0.138 | We should never gamble with the rent money. → We should never Gambol with the rent money. |
| suite-verdict-correction | dog-keep-2 | keep | name | 0.421 | 0.522 | 0.057 | Gamble responsibly if you visit a casino. → Gambol responsibly if you visit a casino. |
| suite-verdict-correction | dog-keep-3 | keep | name | 0.257 | 0.604 | 0.139 | That investment would be a gamble. → That investment would be a Gambol. |
| suite-verdict-correction | dog-keep-4 | keep | name | 0.372 | 0.409 | 0.219 | My dog is asleep while I gamble online. → My dog is asleep while I Gambol online. |
| suite-verdict-correction | dog-abstain-1 | abstain | name | 0.098 | 0.843 | 0.059 | Gamble. → Gambol. |
| suite-verdict-correction | dog-abstain-2 | abstain | name | 0.157 | 0.660 | 0.183 | Tell Gamble. → Tell Gambol. |
| suite-verdict-correction | dog-abstain-3 | abstain | name | 0.255 | 0.584 | 0.161 | I heard someone say Gamble. → I heard someone say Gambol. |
| suite-verdict-correction | dog-abstain-4 | abstain | name | 0.144 | 0.754 | 0.101 | The note only says Gamble. → The note only says Gambol. |
| suite-verdict-correction | son-name-1 | name | name | 0.358 | 0.536 | 0.107 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | son-name-2 | name | keep | 0.497 | 0.276 | 0.226 | Please pack my son Rowen's school bag. → Please pack my son Rowen's school bag. |
| suite-verdict-correction | son-name-3 | name | keep | 0.619 | 0.199 | 0.182 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowen. |
| suite-verdict-correction | son-name-4 | name | keep | 0.705 | 0.158 | 0.137 | My son's name is Rowen. → My son's name is Rowen. |
| suite-verdict-correction | son-keep-1 | keep | keep | 0.943 | 0.021 | 0.037 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | son-keep-2 | keep | keep | 0.949 | 0.014 | 0.037 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | son-keep-3 | keep | keep | 0.707 | 0.078 | 0.215 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict-correction | son-keep-4 | keep | keep | 0.607 | 0.290 | 0.103 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict-correction | son-abstain-1 | abstain | keep | 0.534 | 0.289 | 0.177 | Rowen. → Rowen. |
| suite-verdict-correction | son-abstain-2 | abstain | keep | 0.602 | 0.139 | 0.259 | Ask Rowen about it. → Ask Rowen about it. |
| suite-verdict-correction | son-abstain-3 | abstain | keep | 0.680 | 0.223 | 0.097 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict-correction | son-abstain-4 | abstain | name | 0.322 | 0.513 | 0.165 | The label says Rowen. → The label says Rowan. |
| suite-verdict-correction | mother-name-1 | name | name | 0.352 | 0.392 | 0.256 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict-correction | mother-name-2 | name | name | 0.209 | 0.646 | 0.146 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | mother-name-3 | name | name | 0.325 | 0.380 | 0.295 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict-correction | mother-name-4 | name | name | 0.294 | 0.491 | 0.216 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict-correction | mother-keep-1 | keep | name | 0.145 | 0.674 | 0.181 | The church bell sounded a funeral knell. → The church bell sounded a funeral Nell. |
| suite-verdict-correction | mother-keep-2 | keep | name | 0.237 | 0.546 | 0.216 | That decision was the death knell for the business. → That decision was the death Nell for the business. |
| suite-verdict-correction | mother-keep-3 | keep | keep | 0.397 | 0.285 | 0.317 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict-correction | mother-keep-4 | keep | name | 0.237 | 0.522 | 0.241 | Nell heard the mournful knell of the bell. → Nell heard the mournful Nell of the bell. |
| suite-verdict-correction | mother-abstain-1 | abstain | name | 0.250 | 0.599 | 0.151 | Knell. → Nell. |
| suite-verdict-correction | mother-abstain-2 | abstain | name | 0.121 | 0.760 | 0.120 | I heard Knell. → I heard Nell. |
| suite-verdict-correction | mother-abstain-3 | abstain | name | 0.108 | 0.794 | 0.098 | The recording only contains Knell. → The recording only contains Nell. |
| suite-verdict-correction | mother-abstain-4 | abstain | name | 0.088 | 0.832 | 0.080 | Write down Knell. → Write down Nell. |
| suite-verdict-correction | project-name-1 | name | keep | 0.354 | 0.333 | 0.312 | Open the Linux dictation app trans crust. → Open the Linux dictation app trans crust. |
| suite-verdict-correction | project-name-2 | name | name | 0.322 | 0.566 | 0.112 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict-correction | project-name-3 | name | name | 0.066 | 0.543 | 0.391 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict-correction | project-name-4 | name | keep | 0.685 | 0.054 | 0.261 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | project-keep-1 | keep | keep | 0.606 | 0.160 | 0.233 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict-correction | project-keep-2 | keep | keep | 0.564 | 0.050 | 0.386 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | project-keep-3 | keep | keep | 0.635 | 0.319 | 0.047 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict-correction | project-keep-4 | keep | name | 0.384 | 0.406 | 0.210 | The phrase trans crust here describes a geological process, not software. → The phrase transcrust here describes a geological process, not software. |
| suite-verdict-correction | project-abstain-1 | abstain | name | 0.379 | 0.430 | 0.191 | trans crust. → transcrust. |
| suite-verdict-correction | project-abstain-2 | abstain | name | 0.220 | 0.604 | 0.175 | The note says trans crust. → The note says transcrust. |
| suite-verdict-correction | project-abstain-3 | abstain | keep | 0.451 | 0.348 | 0.201 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict-correction | project-abstain-4 | abstain | name | 0.293 | 0.477 | 0.230 | What about trans crust? → What about transcrust? |
| suite-verdict-correction | habit-name-1 | name | name | 0.329 | 0.639 | 0.032 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | habit-name-2 | name | name | 0.200 | 0.786 | 0.014 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict-correction | habit-name-3 | name | name | 0.270 | 0.726 | 0.004 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | habit-name-4 | name | name | 0.240 | 0.722 | 0.038 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict-correction | habit-keep-1 | keep | keep | 0.433 | 0.218 | 0.349 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict-correction | habit-keep-2 | keep | keep | 0.547 | 0.423 | 0.030 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict-correction | habit-keep-3 | keep | keep | 0.803 | 0.068 | 0.129 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict-correction | habit-keep-4 | keep | name | 0.286 | 0.682 | 0.032 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict-correction | habit-abstain-1 | abstain | name | 0.150 | 0.809 | 0.041 | sigs. → Cigs. |
| suite-verdict-correction | habit-abstain-2 | abstain | name | 0.096 | 0.841 | 0.062 | The note says sigs. → The note says Cigs. |
| suite-verdict-correction | habit-abstain-3 | abstain | name | 0.177 | 0.663 | 0.161 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict-correction | habit-abstain-4 | abstain | name | 0.207 | 0.750 | 0.043 | What is sigs? → What is Cigs? |
