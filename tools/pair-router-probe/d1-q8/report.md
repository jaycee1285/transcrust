# d1-omni text Q8 comparison

One CPU session, two inference threads, one warm-up call. Text graph only; no media encoders. Same saved questions, descriptions and sentence scopes as prior probes, using d1’s native formatter and choice calibration.

Load 0.93s; median inference 1065ms; RSS after load 112 MiB; RSS at end 522 MiB; process peak RSS 521 MiB.

RSS includes Python, tokenizer, ONNX Runtime and CPU allocator caches. These are laptop measurements, not mobile RAM/latency measurements. API timings include network overhead; local timings measure inference.

| Arm | Model | Floor | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |
|---|---|---:|---:|---:|---:|---:|
| sentence | Verdict | 0.0 | 10/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.5 | 7/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.7 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | Verdict | 0.9 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | d1 Q8 | 0.0 | 5/12 | 0/7 | 0/0 | 12/19 |
| sentence | d1 Q8 | 0.5 | 4/12 | 0/7 | 0/0 | 12/19 |
| sentence | d1 Q8 | 0.7 | 2/12 | 0/7 | 0/0 | 12/19 |
| sentence | d1 Q8 | 0.9 | 0/12 | 0/7 | 0/0 | 12/19 |
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
| suite-verdict | d1 Q8 | 0.0 | 16/20 | 1/20 | 7/20 | 37/60 |
| suite-verdict | d1 Q8 | 0.5 | 16/20 | 1/20 | 5/20 | 37/60 |
| suite-verdict | d1 Q8 | 0.7 | 12/20 | 1/20 | 4/20 | 37/60 |
| suite-verdict | d1 Q8 | 0.9 | 10/20 | 1/20 | 4/20 | 37/60 |
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
| cars-verdict | d1 Q8 | 0.0 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | d1 Q8 | 0.5 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | d1 Q8 | 0.7 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | d1 Q8 | 0.9 | 5/6 | 0/6 | 0/6 | 12/18 |
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
| suite-verdict-correction | d1 Q8 | 0.0 | 12/20 | 10/20 | 16/20 | 22/60 |
| suite-verdict-correction | d1 Q8 | 0.5 | 9/20 | 8/20 | 13/20 | 22/60 |
| suite-verdict-correction | d1 Q8 | 0.7 | 4/20 | 2/20 | 5/20 | 22/60 |
| suite-verdict-correction | d1 Q8 | 0.9 | 2/20 | 1/20 | 0/20 | 22/60 |
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
| sentence | record-sigs-1 | name | name | 0.016 | 0.893 | 0.091 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Cigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | record-job-supply-1 | name | keep | 0.415 | 0.314 | 0.271 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-terakeet-1 | name | name | 0.312 | 0.514 | 0.173 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, Terakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-apptrack-1 | name | keep | 0.447 | 0.133 | 0.420 | At track start. → At track start. |
| sentence | record-rowan-1 | name | name | 0.197 | 0.429 | 0.374 | I might also say something like orangue laundry complete. → I might also say something like Rowan laundry complete. |
| sentence | record-backroom-1 | name | name | 0.362 | 0.532 | 0.106 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, Back room vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | record-dominion-1 | name | keep | 0.826 | 0.066 | 0.108 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. |
| sentence | record-dominion-2 | name | keep | 0.659 | 0.048 | 0.293 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say paid dominion 360 plus 10. |
| sentence | dog-in-user-sentence | name | name | 0.093 | 0.854 | 0.053 | I love my dog Gamble and my son Rowan. → I love my dog Gambol and my son Rowan. |
| sentence | casino-in-user-sentence | keep | keep | 0.998 | 0.000 | 0.002 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to gamble and play cards. |
| sentence | dog-lowercase | name | keep | 0.625 | 0.365 | 0.010 | I love my dog gamble. → I love my dog gamble. |
| sentence | casino-short | keep | keep | 0.997 | 0.000 | 0.003 | I like to gamble and play poker with cards. → I like to gamble and play poker with cards. |
| sentence | casino-capitalized | keep | keep | 0.986 | 0.000 | 0.013 | Gamble and play cards at the casino. → Gamble and play cards at the casino. |
| sentence | son-in-user-sentence | name | keep | 0.944 | 0.007 | 0.049 | My son Rowen's mother is named Nell. → My son Rowen's mother is named Nell. |
| sentence | six-cards | keep | keep | 0.993 | 0.000 | 0.007 | I dealt six cards to each player. → I dealt six cards to each player. |
| sentence | six-cigarettes | name | keep | 0.881 | 0.112 | 0.007 | My cigarette habit is called six. → My cigarette habit is called six. |
| sentence | job-supply-prose | keep | keep | 0.759 | 0.064 | 0.177 | The job supply in this town has fallen. → The job supply in this town has fallen. |
| sentence | track-prose | keep | keep | 0.680 | 0.040 | 0.280 | I am at track practice with my son. → I am at track practice with my son. |
| sentence | dominion-past | keep | keep | 0.981 | 0.002 | 0.017 | I paid Dominion yesterday and the balance is now zero. → I paid Dominion yesterday and the balance is now zero. |
| suite-verdict | dog-name-1 | name | keep | 0.837 | 0.132 | 0.031 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict | dog-name-2 | name | name | 0.307 | 0.634 | 0.060 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict | dog-name-3 | name | name | 0.241 | 0.662 | 0.097 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | dog-name-4 | name | name | 0.060 | 0.835 | 0.105 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict | dog-keep-1 | keep | keep | 0.963 | 0.000 | 0.037 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict | dog-keep-2 | keep | keep | 0.997 | 0.000 | 0.003 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict | dog-keep-3 | keep | keep | 0.981 | 0.000 | 0.019 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict | dog-keep-4 | keep | keep | 0.975 | 0.010 | 0.015 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict | dog-abstain-1 | abstain | keep | 0.934 | 0.000 | 0.066 | Gamble. → Gamble. |
| suite-verdict | dog-abstain-2 | abstain | keep | 0.943 | 0.000 | 0.057 | Tell Gamble. → Tell Gamble. |
| suite-verdict | dog-abstain-3 | abstain | keep | 0.903 | 0.000 | 0.097 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict | dog-abstain-4 | abstain | keep | 0.692 | 0.001 | 0.307 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict | son-name-1 | name | keep | 0.527 | 0.462 | 0.011 | My son Rowen forgot his lunchbox. → My son Rowen forgot his lunchbox. |
| suite-verdict | son-name-2 | name | name | 0.412 | 0.577 | 0.011 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | son-name-3 | name | name | 0.043 | 0.938 | 0.020 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict | son-name-4 | name | name | 0.444 | 0.521 | 0.034 | My son's name is Rowen. → My son's name is Rowan. |
| suite-verdict | son-keep-1 | keep | keep | 0.982 | 0.015 | 0.003 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | son-keep-2 | keep | keep | 0.965 | 0.032 | 0.003 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict | son-keep-3 | keep | keep | 0.838 | 0.110 | 0.052 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict | son-keep-4 | keep | abstain | 0.268 | 0.146 | 0.585 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict | son-abstain-1 | abstain | name | 0.409 | 0.427 | 0.164 | Rowen. → Rowan. |
| suite-verdict | son-abstain-2 | abstain | name | 0.286 | 0.664 | 0.050 | Ask Rowen about it. → Ask Rowan about it. |
| suite-verdict | son-abstain-3 | abstain | keep | 0.889 | 0.106 | 0.004 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict | son-abstain-4 | abstain | name | 0.422 | 0.469 | 0.109 | The label says Rowen. → The label says Rowan. |
| suite-verdict | mother-name-1 | name | name | 0.018 | 0.956 | 0.025 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict | mother-name-2 | name | name | 0.078 | 0.890 | 0.031 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | mother-name-3 | name | name | 0.034 | 0.944 | 0.022 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict | mother-name-4 | name | name | 0.018 | 0.959 | 0.023 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict | mother-keep-1 | keep | keep | 0.990 | 0.001 | 0.009 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict | mother-keep-2 | keep | keep | 0.726 | 0.011 | 0.263 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict | mother-keep-3 | keep | keep | 0.990 | 0.002 | 0.009 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict | mother-keep-4 | keep | keep | 0.947 | 0.032 | 0.021 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict | mother-abstain-1 | abstain | keep | 0.847 | 0.038 | 0.114 | Knell. → Knell. |
| suite-verdict | mother-abstain-2 | abstain | keep | 0.958 | 0.008 | 0.034 | I heard Knell. → I heard Knell. |
| suite-verdict | mother-abstain-3 | abstain | keep | 0.513 | 0.191 | 0.296 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict | mother-abstain-4 | abstain | keep | 0.789 | 0.033 | 0.178 | Write down Knell. → Write down Knell. |
| suite-verdict | project-name-1 | name | name | 0.002 | 0.985 | 0.013 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict | project-name-2 | name | abstain | 0.133 | 0.085 | 0.782 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called trans crust. |
| suite-verdict | project-name-3 | name | name | 0.001 | 0.995 | 0.004 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict | project-name-4 | name | abstain | 0.094 | 0.146 | 0.761 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict | project-keep-1 | keep | keep | 0.593 | 0.001 | 0.406 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict | project-keep-2 | keep | keep | 0.920 | 0.000 | 0.080 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | project-keep-3 | keep | abstain | 0.480 | 0.028 | 0.491 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict | project-keep-4 | keep | keep | 0.970 | 0.000 | 0.030 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict | project-abstain-1 | abstain | abstain | 0.266 | 0.001 | 0.733 | trans crust. → trans crust. |
| suite-verdict | project-abstain-2 | abstain | abstain | 0.132 | 0.003 | 0.864 | The note says trans crust. → The note says trans crust. |
| suite-verdict | project-abstain-3 | abstain | abstain | 0.172 | 0.002 | 0.826 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict | project-abstain-4 | abstain | abstain | 0.326 | 0.001 | 0.673 | What about trans crust? → What about trans crust? |
| suite-verdict | habit-name-1 | name | name | 0.002 | 0.909 | 0.089 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | habit-name-2 | name | name | 0.001 | 0.999 | 0.001 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict | habit-name-3 | name | name | 0.001 | 0.997 | 0.002 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict | habit-name-4 | name | name | 0.001 | 0.997 | 0.002 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict | habit-keep-1 | keep | keep | 0.990 | 0.000 | 0.010 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict | habit-keep-2 | keep | keep | 0.610 | 0.386 | 0.004 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict | habit-keep-3 | keep | keep | 0.934 | 0.000 | 0.066 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict | habit-keep-4 | keep | name | 0.079 | 0.919 | 0.002 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict | habit-abstain-1 | abstain | name | 0.008 | 0.979 | 0.013 | sigs. → Cigs. |
| suite-verdict | habit-abstain-2 | abstain | name | 0.032 | 0.931 | 0.036 | The note says sigs. → The note says Cigs. |
| suite-verdict | habit-abstain-3 | abstain | name | 0.020 | 0.957 | 0.023 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict | habit-abstain-4 | abstain | name | 0.002 | 0.995 | 0.003 | What is sigs? → What is Cigs? |
| cars-verdict | cars-lightning-name-1 | name | name | 0.001 | 0.995 | 0.004 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car Lightning McQueen for his birthday. |
| cars-verdict | cars-lightning-name-2 | name | name | 0.010 | 0.984 | 0.006 | In Cars, lightning McQueen learns to value his friends. → In Cars, Lightning McQueen learns to value his friends. |
| cars-verdict | cars-lightning-keep-1 | keep | keep | 0.618 | 0.134 | 0.248 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because lightning struck near the house. |
| cars-verdict | cars-lightning-keep-2 | keep | keep | 0.983 | 0.004 | 0.013 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | cars-lightning-abstain-1 | abstain | keep | 0.971 | 0.006 | 0.023 | Look at lightning. → Look at lightning. |
| cars-verdict | cars-lightning-abstain-2 | abstain | keep | 0.938 | 0.020 | 0.042 | The note says lightning. → The note says lightning. |
| cars-verdict | cars-mater-name-1 | name | name | 0.019 | 0.955 | 0.025 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mater is Lightning McQueen's friend. |
| cars-verdict | cars-mater-name-2 | name | name | 0.005 | 0.982 | 0.013 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | cars-mater-keep-1 | keep | keep | 0.688 | 0.052 | 0.260 | Mader has that surname; he is not the Cars character. → Mader has that surname; he is not the Cars character. |
| cars-verdict | cars-mater-keep-2 | keep | keep | 0.578 | 0.060 | 0.362 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a mater: it couples the two parts together. |
| cars-verdict | cars-mater-abstain-1 | abstain | keep | 0.524 | 0.076 | 0.400 | Mader is outside. → Mader is outside. |
| cars-verdict | cars-mater-abstain-2 | abstain | abstain | 0.089 | 0.088 | 0.823 | I heard someone say mater. → I heard someone say mater. |
| cars-verdict | cars-flo-name-1 | name | keep | 0.583 | 0.271 | 0.147 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, flow runs the V8 Cafe in Radiator Springs. |
| cars-verdict | cars-flo-name-2 | name | name | 0.010 | 0.970 | 0.019 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of Flo, the Cars character who runs the diner. |
| cars-verdict | cars-flo-keep-1 | keep | keep | 0.930 | 0.010 | 0.060 | The water flow slowed when I closed the tap. → The water flow slowed when I closed the tap. |
| cars-verdict | cars-flo-keep-2 | keep | keep | 0.806 | 0.081 | 0.113 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic flow. |
| cars-verdict | cars-flo-abstain-1 | abstain | keep | 0.717 | 0.105 | 0.178 | Look at flow. → Look at flow. |
| cars-verdict | cars-flo-abstain-2 | abstain | keep | 0.425 | 0.239 | 0.335 | The note only says flow. → The note only says flow. |
| suite-verdict-correction | dog-name-1 | name | keep | 0.407 | 0.383 | 0.209 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict-correction | dog-name-2 | name | keep | 0.555 | 0.300 | 0.145 | My dog gamble has a vet appointment tomorrow. → My dog gamble has a vet appointment tomorrow. |
| suite-verdict-correction | dog-name-3 | name | name | 0.377 | 0.502 | 0.121 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict-correction | dog-name-4 | name | keep | 0.435 | 0.384 | 0.182 | I filled my dog Gamble's water bowl. → I filled my dog Gamble's water bowl. |
| suite-verdict-correction | dog-keep-1 | keep | keep | 0.469 | 0.250 | 0.281 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict-correction | dog-keep-2 | keep | name | 0.117 | 0.865 | 0.018 | Gamble responsibly if you visit a casino. → Gambol responsibly if you visit a casino. |
| suite-verdict-correction | dog-keep-3 | keep | name | 0.209 | 0.647 | 0.144 | That investment would be a gamble. → That investment would be a Gambol. |
| suite-verdict-correction | dog-keep-4 | keep | keep | 0.425 | 0.399 | 0.177 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict-correction | dog-abstain-1 | abstain | name | 0.088 | 0.843 | 0.069 | Gamble. → Gambol. |
| suite-verdict-correction | dog-abstain-2 | abstain | name | 0.229 | 0.554 | 0.217 | Tell Gamble. → Tell Gambol. |
| suite-verdict-correction | dog-abstain-3 | abstain | name | 0.205 | 0.665 | 0.131 | I heard someone say Gamble. → I heard someone say Gambol. |
| suite-verdict-correction | dog-abstain-4 | abstain | name | 0.143 | 0.737 | 0.120 | The note only says Gamble. → The note only says Gambol. |
| suite-verdict-correction | son-name-1 | name | name | 0.352 | 0.492 | 0.157 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | son-name-2 | name | keep | 0.591 | 0.242 | 0.166 | Please pack my son Rowen's school bag. → Please pack my son Rowen's school bag. |
| suite-verdict-correction | son-name-3 | name | keep | 0.634 | 0.156 | 0.209 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowen. |
| suite-verdict-correction | son-name-4 | name | keep | 0.567 | 0.249 | 0.184 | My son's name is Rowen. → My son's name is Rowen. |
| suite-verdict-correction | son-keep-1 | keep | keep | 0.971 | 0.008 | 0.021 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | son-keep-2 | keep | keep | 0.974 | 0.008 | 0.018 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | son-keep-3 | keep | keep | 0.713 | 0.046 | 0.242 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict-correction | son-keep-4 | keep | name | 0.414 | 0.471 | 0.115 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict-correction | son-abstain-1 | abstain | keep | 0.470 | 0.382 | 0.148 | Rowen. → Rowen. |
| suite-verdict-correction | son-abstain-2 | abstain | keep | 0.504 | 0.265 | 0.231 | Ask Rowen about it. → Ask Rowen about it. |
| suite-verdict-correction | son-abstain-3 | abstain | keep | 0.807 | 0.139 | 0.054 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict-correction | son-abstain-4 | abstain | name | 0.366 | 0.459 | 0.175 | The label says Rowen. → The label says Rowan. |
| suite-verdict-correction | mother-name-1 | name | keep | 0.415 | 0.359 | 0.226 | My son's mother Knell will collect him from school. → My son's mother Knell will collect him from school. |
| suite-verdict-correction | mother-name-2 | name | name | 0.196 | 0.691 | 0.114 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | mother-name-3 | name | name | 0.322 | 0.475 | 0.203 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict-correction | mother-name-4 | name | name | 0.332 | 0.565 | 0.104 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict-correction | mother-keep-1 | keep | name | 0.152 | 0.678 | 0.170 | The church bell sounded a funeral knell. → The church bell sounded a funeral Nell. |
| suite-verdict-correction | mother-keep-2 | keep | name | 0.301 | 0.494 | 0.205 | That decision was the death knell for the business. → That decision was the death Nell for the business. |
| suite-verdict-correction | mother-keep-3 | keep | name | 0.304 | 0.518 | 0.178 | Knell means the sound of a bell marking a death. → Nell means the sound of a bell marking a death. |
| suite-verdict-correction | mother-keep-4 | keep | name | 0.175 | 0.686 | 0.140 | Nell heard the mournful knell of the bell. → Nell heard the mournful Nell of the bell. |
| suite-verdict-correction | mother-abstain-1 | abstain | name | 0.244 | 0.602 | 0.153 | Knell. → Nell. |
| suite-verdict-correction | mother-abstain-2 | abstain | name | 0.198 | 0.577 | 0.225 | I heard Knell. → I heard Nell. |
| suite-verdict-correction | mother-abstain-3 | abstain | name | 0.183 | 0.672 | 0.145 | The recording only contains Knell. → The recording only contains Nell. |
| suite-verdict-correction | mother-abstain-4 | abstain | name | 0.254 | 0.588 | 0.158 | Write down Knell. → Write down Nell. |
| suite-verdict-correction | project-name-1 | name | name | 0.338 | 0.492 | 0.170 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict-correction | project-name-2 | name | name | 0.318 | 0.541 | 0.141 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict-correction | project-name-3 | name | name | 0.013 | 0.853 | 0.134 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict-correction | project-name-4 | name | abstain | 0.336 | 0.127 | 0.537 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | project-keep-1 | keep | keep | 0.509 | 0.340 | 0.151 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict-correction | project-keep-2 | keep | keep | 0.714 | 0.074 | 0.212 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | project-keep-3 | keep | keep | 0.462 | 0.201 | 0.337 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict-correction | project-keep-4 | keep | name | 0.259 | 0.528 | 0.214 | The phrase trans crust here describes a geological process, not software. → The phrase transcrust here describes a geological process, not software. |
| suite-verdict-correction | project-abstain-1 | abstain | name | 0.349 | 0.382 | 0.269 | trans crust. → transcrust. |
| suite-verdict-correction | project-abstain-2 | abstain | name | 0.280 | 0.555 | 0.166 | The note says trans crust. → The note says transcrust. |
| suite-verdict-correction | project-abstain-3 | abstain | keep | 0.409 | 0.376 | 0.215 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict-correction | project-abstain-4 | abstain | name | 0.286 | 0.366 | 0.349 | What about trans crust? → What about transcrust? |
| suite-verdict-correction | habit-name-1 | name | name | 0.274 | 0.643 | 0.083 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | habit-name-2 | name | name | 0.074 | 0.911 | 0.015 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict-correction | habit-name-3 | name | name | 0.086 | 0.910 | 0.004 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | habit-name-4 | name | name | 0.127 | 0.845 | 0.028 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict-correction | habit-keep-1 | keep | keep | 0.493 | 0.147 | 0.359 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict-correction | habit-keep-2 | keep | name | 0.300 | 0.668 | 0.032 | I smoked six cigarettes today. → I smoked Cigs cigarettes today. |
| suite-verdict-correction | habit-keep-3 | keep | keep | 0.671 | 0.076 | 0.253 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict-correction | habit-keep-4 | keep | name | 0.035 | 0.963 | 0.003 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict-correction | habit-abstain-1 | abstain | name | 0.130 | 0.796 | 0.074 | sigs. → Cigs. |
| suite-verdict-correction | habit-abstain-2 | abstain | name | 0.144 | 0.744 | 0.112 | The note says sigs. → The note says Cigs. |
| suite-verdict-correction | habit-abstain-3 | abstain | name | 0.224 | 0.642 | 0.134 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict-correction | habit-abstain-4 | abstain | name | 0.136 | 0.843 | 0.022 | What is sigs? → What is Cigs? |
