# Local vs API bounded-choice comparison

One request per case; same marked sentence, question and option descriptions as the saved local prompt. API models receive a typed choice schema; local Verdict uses its LABEL/SEP serialization. Abstention is an explicit third choice here. No labels or expected answers are sent in state.

Results are on synthetic/selected-span probes, not end-to-end recognition. Confidence floors use the selected option probability for all models; the API’s separate confidence statistic is retained, not mixed with it.

| Arm | Model | Floor | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |
|---|---|---:|---:|---:|---:|---:|
| sentence | local Verdict | 0.0 | 10/12 | 0/7 | 0/0 | 17/19 |
| sentence | local Verdict | 0.5 | 7/12 | 0/7 | 0/0 | 17/19 |
| sentence | local Verdict | 0.7 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | local Verdict | 0.9 | 0/12 | 0/7 | 0/0 | 17/19 |
| sentence | typesafe/jev-1.13 | 0.0 | 12/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.5 | 11/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.7 | 10/12 | 0/7 | 0/0 | 19/19 |
| sentence | typesafe/jev-1.13 | 0.9 | 8/12 | 0/7 | 0/0 | 19/19 |
| sentence | cloudflare/clef-flash | 0.0 | 11/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.5 | 11/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.7 | 10/12 | 0/7 | 0/0 | 18/19 |
| sentence | cloudflare/clef-flash | 0.9 | 7/12 | 0/7 | 0/0 | 18/19 |
| suite-verdict | local Verdict | 0.0 | 20/20 | 6/20 | 8/20 | 32/60 |
| suite-verdict | local Verdict | 0.5 | 20/20 | 4/20 | 6/20 | 32/60 |
| suite-verdict | local Verdict | 0.7 | 1/20 | 0/20 | 0/20 | 32/60 |
| suite-verdict | local Verdict | 0.9 | 0/20 | 0/20 | 0/20 | 32/60 |
| suite-verdict | typesafe/jev-1.13 | 0.0 | 20/20 | 0/20 | 7/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.5 | 20/20 | 0/20 | 6/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.7 | 20/20 | 0/20 | 1/20 | 45/60 |
| suite-verdict | typesafe/jev-1.13 | 0.9 | 20/20 | 0/20 | 0/20 | 45/60 |
| suite-verdict | cloudflare/clef-flash | 0.0 | 19/20 | 2/20 | 3/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.5 | 19/20 | 2/20 | 3/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.7 | 17/20 | 2/20 | 1/20 | 41/60 |
| suite-verdict | cloudflare/clef-flash | 0.9 | 13/20 | 1/20 | 0/20 | 41/60 |
| cars-verdict | local Verdict | 0.0 | 6/6 | 1/6 | 1/6 | 11/18 |
| cars-verdict | local Verdict | 0.5 | 5/6 | 1/6 | 0/6 | 11/18 |
| cars-verdict | local Verdict | 0.7 | 2/6 | 0/6 | 0/6 | 11/18 |
| cars-verdict | local Verdict | 0.9 | 0/6 | 0/6 | 0/6 | 11/18 |
| cars-verdict | typesafe/jev-1.13 | 0.0 | 6/6 | 0/6 | 1/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.5 | 6/6 | 0/6 | 1/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.7 | 6/6 | 0/6 | 0/6 | 17/18 |
| cars-verdict | typesafe/jev-1.13 | 0.9 | 6/6 | 0/6 | 0/6 | 17/18 |
| cars-verdict | cloudflare/clef-flash | 0.0 | 6/6 | 1/6 | 1/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.5 | 6/6 | 1/6 | 0/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.7 | 6/6 | 1/6 | 0/6 | 12/18 |
| cars-verdict | cloudflare/clef-flash | 0.9 | 6/6 | 0/6 | 0/6 | 12/18 |
| suite-verdict-correction | local Verdict | 0.0 | 20/20 | 18/20 | 18/20 | 22/60 |
| suite-verdict-correction | local Verdict | 0.5 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | local Verdict | 0.7 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | local Verdict | 0.9 | 0/20 | 0/20 | 0/20 | 22/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.0 | 17/20 | 1/20 | 1/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.5 | 16/20 | 0/20 | 1/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.7 | 10/20 | 0/20 | 0/20 | 50/60 |
| suite-verdict-correction | typesafe/jev-1.13 | 0.9 | 1/20 | 0/20 | 0/20 | 50/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.0 | 18/20 | 0/20 | 5/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.5 | 17/20 | 0/20 | 4/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.7 | 12/20 | 0/20 | 2/20 | 42/60 |
| suite-verdict-correction | cloudflare/clef-flash | 0.9 | 3/20 | 0/20 | 0/20 | 42/60 |

Exact decisions use the raw selected option, independent of the edit floor. KEEP and abstain both leave text intact, but have different meanings. Metrics require the current saved local baseline hash; the audit and cost totals retain all attempts.

## API usage

| Model | Successful calls | Failed attempts | Reported USD | Median round trip |
|---|---:|---:|---:|---:|
| typesafe/jev-1.13 | 157 | 1 | $0.00261328 | 308 ms |
| cloudflare/clef-flash | 157 | 1 | $0.00079133 | 325 ms |

Local timings measure inference only; API timings include network/queue/processing. Initial DNS failures happened inside the sandbox before reaching the API; both were retried successfully with network permission. Zero-cost or missing usage is retained in receipts and must not be read as proof that an unreported request was free.

## Per-case API decisions

| Arm | Model | Case | Want | Chose | P(chosen) | Original → proposed |
|---|---|---|---|---|---:|---|
| sentence | cloudflare/clef-flash | record-sigs-1 | name | URLError: <urlopen error [Errno -3] Temporary failure in name resolution> | 0.000 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | typesafe/jev-1.13 | record-sigs-1 | name | URLError: <urlopen error [Errno -3] Temporary failure in name resolution> | 0.000 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | typesafe/jev-1.13 | record-sigs-1 | name | name | 0.960 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Cigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | cloudflare/clef-flash | record-sigs-1 | name | name | 0.947 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Cigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | typesafe/jev-1.13 | record-job-supply-1 | name | name | 0.910 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, Jobs apply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | typesafe/jev-1.13 | record-rowan-1 | name | name | 0.550 | I might also say something like orangue laundry complete. → I might also say something like Rowan laundry complete. |
| sentence | typesafe/jev-1.13 | record-terakeet-1 | name | name | 0.950 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, Terakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | typesafe/jev-1.13 | record-backroom-1 | name | name | 0.970 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, Back room vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | typesafe/jev-1.13 | record-dominion-1 | name | name | 0.780 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, Pay Dominion 360, pay BJWSA 100, Nell 220. |
| sentence | typesafe/jev-1.13 | record-dominion-2 | name | name | 0.980 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say Pay Dominion 360 plus 10. |
| sentence | typesafe/jev-1.13 | dog-in-user-sentence | name | name | 0.980 | I love my dog Gamble and my son Rowan. → I love my dog Gambol and my son Rowan. |
| sentence | typesafe/jev-1.13 | casino-in-user-sentence | keep | keep | 0.910 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to gamble and play cards. |
| sentence | typesafe/jev-1.13 | dog-lowercase | name | name | 0.990 | I love my dog gamble. → I love my dog Gambol. |
| sentence | typesafe/jev-1.13 | casino-short | keep | keep | 0.950 | I like to gamble and play poker with cards. → I like to gamble and play poker with cards. |
| sentence | typesafe/jev-1.13 | record-apptrack-1 | name | name | 0.380 | At track start. → Apptrack start. |
| sentence | typesafe/jev-1.13 | casino-capitalized | keep | keep | 0.970 | Gamble and play cards at the casino. → Gamble and play cards at the casino. |
| sentence | typesafe/jev-1.13 | son-in-user-sentence | name | name | 0.980 | My son Rowen's mother is named Nell. → My son Rowan's mother is named Nell. |
| sentence | typesafe/jev-1.13 | job-supply-prose | keep | keep | 0.980 | The job supply in this town has fallen. → The job supply in this town has fallen. |
| sentence | typesafe/jev-1.13 | six-cigarettes | name | name | 0.870 | My cigarette habit is called six. → My cigarette habit is called Cigs. |
| sentence | typesafe/jev-1.13 | six-cards | keep | keep | 0.980 | I dealt six cards to each player. → I dealt six cards to each player. |
| sentence | typesafe/jev-1.13 | track-prose | keep | keep | 0.930 | I am at track practice with my son. → I am at track practice with my son. |
| suite-verdict | typesafe/jev-1.13 | dog-name-2 | name | name | 1.000 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| sentence | typesafe/jev-1.13 | dominion-past | keep | keep | 0.570 | I paid Dominion yesterday and the balance is now zero. → I paid Dominion yesterday and the balance is now zero. |
| suite-verdict | typesafe/jev-1.13 | dog-name-1 | name | name | 0.970 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| suite-verdict | typesafe/jev-1.13 | dog-name-3 | name | name | 0.970 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | typesafe/jev-1.13 | dog-name-4 | name | name | 1.000 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict | typesafe/jev-1.13 | dog-keep-1 | keep | keep | 0.930 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict | typesafe/jev-1.13 | dog-keep-2 | keep | keep | 0.970 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict | typesafe/jev-1.13 | dog-keep-4 | keep | keep | 0.930 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict | typesafe/jev-1.13 | dog-keep-3 | keep | keep | 0.890 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict | typesafe/jev-1.13 | dog-abstain-2 | abstain | name | 0.640 | Tell Gamble. → Tell Gambol. |
| suite-verdict | typesafe/jev-1.13 | dog-abstain-1 | abstain | keep | 0.480 | Gamble. → Gamble. |
| suite-verdict | typesafe/jev-1.13 | dog-abstain-4 | abstain | keep | 0.550 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict | typesafe/jev-1.13 | dog-abstain-3 | abstain | keep | 0.680 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict | typesafe/jev-1.13 | son-name-1 | name | name | 0.990 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict | typesafe/jev-1.13 | son-name-3 | name | name | 0.950 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict | typesafe/jev-1.13 | son-name-4 | name | name | 0.970 | My son's name is Rowen. → My son's name is Rowan. |
| suite-verdict | typesafe/jev-1.13 | son-keep-1 | keep | keep | 0.970 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | typesafe/jev-1.13 | son-keep-2 | keep | keep | 0.840 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict | typesafe/jev-1.13 | son-keep-3 | keep | keep | 0.500 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict | typesafe/jev-1.13 | son-keep-4 | keep | abstain | 0.480 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict | typesafe/jev-1.13 | son-name-2 | name | name | 0.980 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | typesafe/jev-1.13 | son-abstain-1 | abstain | name | 0.630 | Rowen. → Rowan. |
| suite-verdict | typesafe/jev-1.13 | son-abstain-2 | abstain | name | 0.590 | Ask Rowen about it. → Ask Rowan about it. |
| suite-verdict | typesafe/jev-1.13 | son-abstain-3 | abstain | name | 0.590 | Someone called Rowen left a message. → Someone called Rowan left a message. |
| suite-verdict | typesafe/jev-1.13 | son-abstain-4 | abstain | name | 0.740 | The label says Rowen. → The label says Rowan. |
| suite-verdict | typesafe/jev-1.13 | mother-name-1 | name | name | 0.960 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict | typesafe/jev-1.13 | mother-name-2 | name | name | 0.920 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | typesafe/jev-1.13 | mother-name-3 | name | name | 0.970 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict | typesafe/jev-1.13 | mother-name-4 | name | name | 0.960 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict | typesafe/jev-1.13 | mother-keep-2 | keep | keep | 0.990 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict | typesafe/jev-1.13 | mother-keep-1 | keep | keep | 1.000 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict | typesafe/jev-1.13 | mother-keep-3 | keep | keep | 0.960 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict | typesafe/jev-1.13 | mother-keep-4 | keep | keep | 1.000 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict | typesafe/jev-1.13 | mother-abstain-1 | abstain | keep | 0.690 | Knell. → Knell. |
| suite-verdict | typesafe/jev-1.13 | mother-abstain-2 | abstain | keep | 0.870 | I heard Knell. → I heard Knell. |
| suite-verdict | typesafe/jev-1.13 | mother-abstain-3 | abstain | keep | 0.820 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict | typesafe/jev-1.13 | mother-abstain-4 | abstain | keep | 0.690 | Write down Knell. → Write down Knell. |
| suite-verdict | typesafe/jev-1.13 | project-name-1 | name | name | 1.000 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict | typesafe/jev-1.13 | project-name-2 | name | name | 1.000 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict | typesafe/jev-1.13 | project-name-3 | name | name | 0.970 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict | typesafe/jev-1.13 | project-name-4 | name | name | 0.980 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict | typesafe/jev-1.13 | project-keep-1 | keep | keep | 0.700 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict | typesafe/jev-1.13 | project-keep-2 | keep | keep | 0.590 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | typesafe/jev-1.13 | project-keep-3 | keep | keep | 0.880 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict | typesafe/jev-1.13 | project-keep-4 | keep | keep | 1.000 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict | typesafe/jev-1.13 | project-abstain-1 | abstain | abstain | 0.450 | trans crust. → trans crust. |
| suite-verdict | typesafe/jev-1.13 | project-abstain-2 | abstain | name | 0.390 | The note says trans crust. → The note says transcrust. |
| suite-verdict | typesafe/jev-1.13 | project-abstain-4 | abstain | name | 0.540 | What about trans crust? → What about transcrust? |
| suite-verdict | typesafe/jev-1.13 | project-abstain-3 | abstain | abstain | 0.460 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict | typesafe/jev-1.13 | habit-name-1 | name | name | 0.980 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | typesafe/jev-1.13 | habit-name-4 | name | name | 0.990 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict | typesafe/jev-1.13 | habit-name-3 | name | name | 1.000 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict | typesafe/jev-1.13 | habit-name-2 | name | name | 1.000 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict | typesafe/jev-1.13 | habit-keep-1 | keep | keep | 0.930 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict | typesafe/jev-1.13 | habit-keep-2 | keep | keep | 0.880 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict | typesafe/jev-1.13 | habit-keep-4 | keep | keep | 0.530 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to six cigarettes. |
| suite-verdict | typesafe/jev-1.13 | habit-keep-3 | keep | keep | 0.950 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict | typesafe/jev-1.13 | habit-abstain-1 | abstain | abstain | 0.450 | sigs. → sigs. |
| suite-verdict | typesafe/jev-1.13 | habit-abstain-2 | abstain | abstain | 0.520 | The note says sigs. → The note says sigs. |
| suite-verdict | typesafe/jev-1.13 | habit-abstain-4 | abstain | abstain | 0.500 | What is sigs? → What is sigs? |
| suite-verdict | typesafe/jev-1.13 | habit-abstain-3 | abstain | abstain | 0.570 | I heard the word sigs. → I heard the word sigs. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-name-1 | name | name | 1.000 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car Lightning McQueen for his birthday. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-name-2 | name | name | 1.000 | In Cars, lightning McQueen learns to value his friends. → In Cars, Lightning McQueen learns to value his friends. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-keep-1 | keep | keep | 0.500 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because lightning struck near the house. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-keep-2 | keep | keep | 0.950 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-abstain-1 | abstain | abstain | 0.580 | Look at lightning. → Look at lightning. |
| cars-verdict | typesafe/jev-1.13 | cars-lightning-abstain-2 | abstain | abstain | 0.720 | The note says lightning. → The note says lightning. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-name-1 | name | name | 0.990 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mater is Lightning McQueen's friend. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-name-2 | name | name | 1.000 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-keep-1 | keep | keep | 0.920 | Mader has that surname; he is not the Cars character. → Mader has that surname; he is not the Cars character. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-keep-2 | keep | keep | 0.890 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a mater: it couples the two parts together. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-abstain-1 | abstain | name | 0.650 | Mader is outside. → Mater is outside. |
| cars-verdict | typesafe/jev-1.13 | cars-mater-abstain-2 | abstain | abstain | 0.690 | I heard someone say mater. → I heard someone say mater. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-name-1 | name | name | 0.990 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, Flo runs the V8 Cafe in Radiator Springs. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-name-2 | name | name | 1.000 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of Flo, the Cars character who runs the diner. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-keep-1 | keep | keep | 1.000 | The water flow slowed when I closed the tap. → The water flow slowed when I closed the tap. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-keep-2 | keep | keep | 0.970 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic flow. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-abstain-1 | abstain | abstain | 0.580 | Look at flow. → Look at flow. |
| cars-verdict | typesafe/jev-1.13 | cars-flo-abstain-2 | abstain | abstain | 0.610 | The note only says flow. → The note only says flow. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-name-1 | name | name | 0.890 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-name-2 | name | name | 0.820 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-name-3 | name | name | 0.820 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-name-4 | name | name | 0.930 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-keep-2 | keep | keep | 0.920 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-keep-1 | keep | keep | 0.870 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-keep-3 | keep | keep | 0.970 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-keep-4 | keep | name | 0.400 | My dog is asleep while I gamble online. → My dog is asleep while I Gambol online. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-abstain-1 | abstain | abstain | 0.620 | Gamble. → Gamble. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-abstain-2 | abstain | name | 0.520 | Tell Gamble. → Tell Gambol. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-abstain-3 | abstain | abstain | 0.490 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict-correction | typesafe/jev-1.13 | dog-abstain-4 | abstain | abstain | 0.510 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict-correction | typesafe/jev-1.13 | son-name-1 | name | name | 0.610 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | typesafe/jev-1.13 | son-name-3 | name | name | 0.640 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict-correction | typesafe/jev-1.13 | son-name-2 | name | name | 0.470 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict-correction | typesafe/jev-1.13 | son-name-4 | name | name | 0.660 | My son's name is Rowen. → My son's name is Rowan. |
| suite-verdict-correction | typesafe/jev-1.13 | son-keep-1 | keep | keep | 1.000 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | typesafe/jev-1.13 | son-keep-2 | keep | keep | 0.990 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | typesafe/jev-1.13 | son-keep-3 | keep | keep | 0.970 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict-correction | typesafe/jev-1.13 | son-keep-4 | keep | keep | 0.720 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict-correction | typesafe/jev-1.13 | son-abstain-1 | abstain | abstain | 0.530 | Rowen. → Rowen. |
| suite-verdict-correction | typesafe/jev-1.13 | son-abstain-2 | abstain | keep | 0.420 | Ask Rowen about it. → Ask Rowen about it. |
| suite-verdict-correction | typesafe/jev-1.13 | son-abstain-3 | abstain | abstain | 0.610 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict-correction | typesafe/jev-1.13 | son-abstain-4 | abstain | abstain | 0.530 | The label says Rowen. → The label says Rowen. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-name-1 | name | name | 0.780 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-name-2 | name | name | 0.830 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-name-3 | name | name | 0.770 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-name-4 | name | name | 0.880 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-keep-1 | keep | keep | 0.990 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-keep-2 | keep | keep | 0.990 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-keep-3 | keep | keep | 0.860 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-keep-4 | keep | keep | 0.980 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-abstain-1 | abstain | abstain | 0.490 | Knell. → Knell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-abstain-2 | abstain | keep | 0.600 | I heard Knell. → I heard Knell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-abstain-3 | abstain | abstain | 0.460 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict-correction | typesafe/jev-1.13 | mother-abstain-4 | abstain | keep | 0.590 | Write down Knell. → Write down Knell. |
| suite-verdict-correction | typesafe/jev-1.13 | project-name-1 | name | name | 0.620 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict-correction | typesafe/jev-1.13 | project-name-2 | name | name | 0.760 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict-correction | typesafe/jev-1.13 | project-name-3 | name | keep | 0.510 | Restart the trans crust dictation daemon. → Restart the trans crust dictation daemon. |
| suite-verdict-correction | typesafe/jev-1.13 | project-name-4 | name | keep | 0.520 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | typesafe/jev-1.13 | project-keep-1 | keep | keep | 0.930 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict-correction | typesafe/jev-1.13 | project-keep-2 | keep | keep | 0.870 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | typesafe/jev-1.13 | project-keep-3 | keep | keep | 0.930 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict-correction | typesafe/jev-1.13 | project-keep-4 | keep | keep | 0.990 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict-correction | typesafe/jev-1.13 | project-abstain-1 | abstain | abstain | 0.580 | trans crust. → trans crust. |
| suite-verdict-correction | typesafe/jev-1.13 | project-abstain-2 | abstain | abstain | 0.720 | The note says trans crust. → The note says trans crust. |
| suite-verdict-correction | typesafe/jev-1.13 | project-abstain-4 | abstain | abstain | 0.600 | What about trans crust? → What about trans crust? |
| suite-verdict-correction | typesafe/jev-1.13 | habit-name-1 | name | name | 0.700 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | typesafe/jev-1.13 | project-abstain-3 | abstain | abstain | 0.690 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-name-2 | name | name | 0.560 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-name-4 | name | keep | 0.580 | Reset my cigarette counter called sigs. → Reset my cigarette counter called sigs. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-name-3 | name | name | 0.650 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-keep-1 | keep | keep | 0.970 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-keep-2 | keep | keep | 0.890 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-abstain-1 | abstain | abstain | 0.520 | sigs. → sigs. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-keep-4 | keep | keep | 0.930 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to six cigarettes. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-keep-3 | keep | keep | 0.930 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-abstain-2 | abstain | keep | 0.470 | The note says sigs. → The note says sigs. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-abstain-3 | abstain | abstain | 0.530 | I heard the word sigs. → I heard the word sigs. |
| suite-verdict-correction | typesafe/jev-1.13 | habit-abstain-4 | abstain | keep | 0.450 | What is sigs? → What is sigs? |
| sentence | cloudflare/clef-flash | record-job-supply-1 | name | name | 0.979 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, Jobs apply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | cloudflare/clef-flash | record-terakeet-1 | name | name | 0.925 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, Terakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | cloudflare/clef-flash | record-apptrack-1 | name | name | 0.722 | At track start. → Apptrack start. |
| sentence | cloudflare/clef-flash | record-rowan-1 | name | keep | 0.677 | I might also say something like orangue laundry complete. → I might also say something like orangue laundry complete. |
| sentence | cloudflare/clef-flash | record-backroom-1 | name | name | 0.902 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, Back room vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | cloudflare/clef-flash | record-dominion-1 | name | name | 0.855 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, Pay Dominion 360, pay BJWSA 100, Nell 220. |
| sentence | cloudflare/clef-flash | record-dominion-2 | name | name | 0.939 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say Pay Dominion 360 plus 10. |
| sentence | cloudflare/clef-flash | dog-in-user-sentence | name | name | 0.948 | I love my dog Gamble and my son Rowan. → I love my dog Gambol and my son Rowan. |
| sentence | cloudflare/clef-flash | casino-in-user-sentence | keep | keep | 0.952 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to gamble and play cards. |
| sentence | cloudflare/clef-flash | casino-short | keep | keep | 0.965 | I like to gamble and play poker with cards. → I like to gamble and play poker with cards. |
| sentence | cloudflare/clef-flash | casino-capitalized | keep | keep | 0.964 | Gamble and play cards at the casino. → Gamble and play cards at the casino. |
| sentence | cloudflare/clef-flash | son-in-user-sentence | name | name | 0.627 | My son Rowen's mother is named Nell. → My son Rowan's mother is named Nell. |
| sentence | cloudflare/clef-flash | six-cards | keep | keep | 0.938 | I dealt six cards to each player. → I dealt six cards to each player. |
| sentence | cloudflare/clef-flash | six-cigarettes | name | name | 0.761 | My cigarette habit is called six. → My cigarette habit is called Cigs. |
| sentence | cloudflare/clef-flash | dog-lowercase | name | name | 0.956 | I love my dog gamble. → I love my dog Gambol. |
| sentence | cloudflare/clef-flash | job-supply-prose | keep | keep | 0.961 | The job supply in this town has fallen. → The job supply in this town has fallen. |
| sentence | cloudflare/clef-flash | track-prose | keep | keep | 0.924 | I am at track practice with my son. → I am at track practice with my son. |
| suite-verdict | cloudflare/clef-flash | dog-name-1 | name | name | 0.946 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| sentence | cloudflare/clef-flash | dominion-past | keep | keep | 0.730 | I paid Dominion yesterday and the balance is now zero. → I paid Dominion yesterday and the balance is now zero. |
| suite-verdict | cloudflare/clef-flash | dog-name-2 | name | name | 0.948 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict | cloudflare/clef-flash | dog-keep-1 | keep | keep | 0.948 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict | cloudflare/clef-flash | dog-name-3 | name | name | 0.933 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | cloudflare/clef-flash | dog-name-4 | name | name | 0.938 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict | cloudflare/clef-flash | dog-keep-2 | keep | keep | 0.967 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict | cloudflare/clef-flash | dog-keep-3 | keep | keep | 0.831 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict | cloudflare/clef-flash | dog-keep-4 | keep | keep | 0.965 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict | cloudflare/clef-flash | dog-abstain-2 | abstain | keep | 0.793 | Tell Gamble. → Tell Gamble. |
| suite-verdict | cloudflare/clef-flash | dog-abstain-1 | abstain | keep | 0.947 | Gamble. → Gamble. |
| suite-verdict | cloudflare/clef-flash | dog-abstain-4 | abstain | keep | 0.876 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict | cloudflare/clef-flash | son-name-1 | name | keep | 0.522 | My son Rowen forgot his lunchbox. → My son Rowen forgot his lunchbox. |
| suite-verdict | cloudflare/clef-flash | dog-abstain-3 | abstain | keep | 0.952 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict | cloudflare/clef-flash | son-name-2 | name | name | 0.738 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | cloudflare/clef-flash | son-keep-1 | keep | keep | 0.925 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | cloudflare/clef-flash | son-name-3 | name | name | 0.673 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict | cloudflare/clef-flash | son-name-4 | name | name | 0.761 | My son's name is Rowen. → My son's name is Rowan. |
| suite-verdict | cloudflare/clef-flash | son-keep-3 | keep | keep | 0.586 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict | cloudflare/clef-flash | son-keep-4 | keep | name | 0.702 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict | cloudflare/clef-flash | son-abstain-1 | abstain | name | 0.662 | Rowen. → Rowan. |
| suite-verdict | cloudflare/clef-flash | son-abstain-2 | abstain | name | 0.734 | Ask Rowen about it. → Ask Rowan about it. |
| suite-verdict | cloudflare/clef-flash | son-abstain-3 | abstain | keep | 0.680 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict | cloudflare/clef-flash | son-abstain-4 | abstain | name | 0.575 | The label says Rowen. → The label says Rowan. |
| suite-verdict | cloudflare/clef-flash | son-keep-2 | keep | keep | 0.863 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict | cloudflare/clef-flash | mother-name-2 | name | name | 0.550 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | cloudflare/clef-flash | mother-name-3 | name | name | 0.924 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict | cloudflare/clef-flash | mother-name-4 | name | name | 0.826 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict | cloudflare/clef-flash | mother-keep-1 | keep | keep | 0.966 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict | cloudflare/clef-flash | mother-keep-2 | keep | keep | 0.943 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict | cloudflare/clef-flash | mother-keep-3 | keep | keep | 0.877 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict | cloudflare/clef-flash | mother-keep-4 | keep | keep | 0.971 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict | cloudflare/clef-flash | mother-abstain-1 | abstain | keep | 0.937 | Knell. → Knell. |
| suite-verdict | cloudflare/clef-flash | mother-abstain-2 | abstain | keep | 0.944 | I heard Knell. → I heard Knell. |
| suite-verdict | cloudflare/clef-flash | mother-abstain-3 | abstain | keep | 0.909 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict | cloudflare/clef-flash | mother-abstain-4 | abstain | keep | 0.953 | Write down Knell. → Write down Knell. |
| suite-verdict | cloudflare/clef-flash | project-name-1 | name | name | 0.901 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict | cloudflare/clef-flash | mother-name-1 | name | name | 0.820 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict | cloudflare/clef-flash | project-name-2 | name | name | 0.906 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict | cloudflare/clef-flash | project-name-4 | name | name | 0.919 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict | cloudflare/clef-flash | project-keep-1 | keep | keep | 0.876 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict | cloudflare/clef-flash | project-name-3 | name | name | 0.912 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict | cloudflare/clef-flash | project-keep-2 | keep | keep | 0.885 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | cloudflare/clef-flash | project-keep-3 | keep | keep | 0.963 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict | cloudflare/clef-flash | project-keep-4 | keep | keep | 0.956 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict | cloudflare/clef-flash | project-abstain-1 | abstain | keep | 0.807 | trans crust. → trans crust. |
| suite-verdict | cloudflare/clef-flash | project-abstain-2 | abstain | keep | 0.851 | The note says trans crust. → The note says trans crust. |
| suite-verdict | cloudflare/clef-flash | project-abstain-3 | abstain | keep | 0.708 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict | cloudflare/clef-flash | project-abstain-4 | abstain | keep | 0.871 | What about trans crust? → What about trans crust? |
| suite-verdict | cloudflare/clef-flash | habit-name-1 | name | name | 0.961 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | cloudflare/clef-flash | habit-name-2 | name | name | 0.984 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict | cloudflare/clef-flash | habit-name-4 | name | name | 0.973 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict | cloudflare/clef-flash | habit-name-3 | name | name | 0.981 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict | cloudflare/clef-flash | habit-keep-2 | keep | keep | 0.862 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict | cloudflare/clef-flash | habit-keep-3 | keep | keep | 0.925 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict | cloudflare/clef-flash | habit-keep-1 | keep | keep | 0.958 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict | cloudflare/clef-flash | habit-keep-4 | keep | name | 0.962 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict | cloudflare/clef-flash | habit-abstain-1 | abstain | abstain | 0.633 | sigs. → sigs. |
| suite-verdict | cloudflare/clef-flash | habit-abstain-3 | abstain | abstain | 0.707 | I heard the word sigs. → I heard the word sigs. |
| suite-verdict | cloudflare/clef-flash | habit-abstain-2 | abstain | abstain | 0.483 | The note says sigs. → The note says sigs. |
| suite-verdict | cloudflare/clef-flash | habit-abstain-4 | abstain | abstain | 0.794 | What is sigs? → What is sigs? |
| cars-verdict | cloudflare/clef-flash | cars-lightning-name-1 | name | name | 0.991 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car Lightning McQueen for his birthday. |
| cars-verdict | cloudflare/clef-flash | cars-lightning-name-2 | name | name | 0.979 | In Cars, lightning McQueen learns to value his friends. → In Cars, Lightning McQueen learns to value his friends. |
| cars-verdict | cloudflare/clef-flash | cars-lightning-keep-1 | keep | keep | 0.588 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because lightning struck near the house. |
| cars-verdict | cloudflare/clef-flash | cars-lightning-keep-2 | keep | keep | 0.956 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | cloudflare/clef-flash | cars-lightning-abstain-1 | abstain | keep | 0.818 | Look at lightning. → Look at lightning. |
| cars-verdict | cloudflare/clef-flash | cars-mater-name-1 | name | name | 0.969 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mater is Lightning McQueen's friend. |
| cars-verdict | cloudflare/clef-flash | cars-lightning-abstain-2 | abstain | keep | 0.754 | The note says lightning. → The note says lightning. |
| cars-verdict | cloudflare/clef-flash | cars-mater-name-2 | name | name | 0.978 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | cloudflare/clef-flash | cars-mater-keep-1 | keep | name | 0.850 | Mader has that surname; he is not the Cars character. → Mater has that surname; he is not the Cars character. |
| cars-verdict | cloudflare/clef-flash | cars-mater-keep-2 | keep | keep | 0.712 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a mater: it couples the two parts together. |
| cars-verdict | cloudflare/clef-flash | cars-mater-abstain-1 | abstain | keep | 0.709 | Mader is outside. → Mader is outside. |
| cars-verdict | cloudflare/clef-flash | cars-flo-name-1 | name | name | 0.951 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, Flo runs the V8 Cafe in Radiator Springs. |
| cars-verdict | cloudflare/clef-flash | cars-mater-abstain-2 | abstain | name | 0.426 | I heard someone say mater. → I heard someone say Mater. |
| cars-verdict | cloudflare/clef-flash | cars-flo-name-2 | name | name | 0.938 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of Flo, the Cars character who runs the diner. |
| cars-verdict | cloudflare/clef-flash | cars-flo-keep-1 | keep | keep | 0.944 | The water flow slowed when I closed the tap. → The water flow slowed when I closed the tap. |
| cars-verdict | cloudflare/clef-flash | cars-flo-keep-2 | keep | keep | 0.816 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic flow. |
| cars-verdict | cloudflare/clef-flash | cars-flo-abstain-1 | abstain | keep | 0.727 | Look at flow. → Look at flow. |
| cars-verdict | cloudflare/clef-flash | cars-flo-abstain-2 | abstain | abstain | 0.512 | The note only says flow. → The note only says flow. |
| suite-verdict-correction | cloudflare/clef-flash | dog-name-1 | name | name | 0.886 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| suite-verdict-correction | cloudflare/clef-flash | dog-name-4 | name | name | 0.821 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict-correction | cloudflare/clef-flash | dog-name-3 | name | keep | 0.544 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gamble, and she jumps on me when I say it. |
| suite-verdict-correction | cloudflare/clef-flash | dog-keep-1 | keep | keep | 0.937 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict-correction | cloudflare/clef-flash | dog-keep-2 | keep | keep | 0.916 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict-correction | cloudflare/clef-flash | dog-keep-4 | keep | keep | 0.601 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict-correction | cloudflare/clef-flash | dog-keep-3 | keep | keep | 0.945 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict-correction | cloudflare/clef-flash | dog-name-2 | name | name | 0.870 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict-correction | cloudflare/clef-flash | dog-abstain-2 | abstain | abstain | 0.383 | Tell Gamble. → Tell Gamble. |
| suite-verdict-correction | cloudflare/clef-flash | dog-abstain-1 | abstain | keep | 0.451 | Gamble. → Gamble. |
| suite-verdict-correction | cloudflare/clef-flash | dog-abstain-3 | abstain | keep | 0.503 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict-correction | cloudflare/clef-flash | dog-abstain-4 | abstain | keep | 0.597 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict-correction | cloudflare/clef-flash | son-name-1 | name | name | 0.631 | My son Rowen forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | cloudflare/clef-flash | son-name-2 | name | name | 0.610 | Please pack my son Rowen's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict-correction | cloudflare/clef-flash | son-name-3 | name | name | 0.657 | Nell and I co-parent our son Rowen. → Nell and I co-parent our son Rowan. |
| suite-verdict-correction | cloudflare/clef-flash | son-name-4 | name | keep | 0.517 | My son's name is Rowen. → My son's name is Rowen. |
| suite-verdict-correction | cloudflare/clef-flash | son-keep-1 | keep | keep | 0.963 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | cloudflare/clef-flash | son-keep-2 | keep | keep | 0.905 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | cloudflare/clef-flash | son-keep-3 | keep | keep | 0.826 | My colleague Rowen is a different person from my son Rowan. → My colleague Rowen is a different person from my son Rowan. |
| suite-verdict-correction | cloudflare/clef-flash | son-keep-4 | keep | keep | 0.771 | The author spells her name Rowen, without an o at the end. → The author spells her name Rowen, without an o at the end. |
| suite-verdict-correction | cloudflare/clef-flash | son-abstain-1 | abstain | abstain | 0.373 | Rowen. → Rowen. |
| suite-verdict-correction | cloudflare/clef-flash | son-abstain-2 | abstain | keep | 0.406 | Ask Rowen about it. → Ask Rowen about it. |
| suite-verdict-correction | cloudflare/clef-flash | mother-name-1 | name | name | 0.790 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict-correction | cloudflare/clef-flash | son-abstain-3 | abstain | abstain | 0.443 | Someone called Rowen left a message. → Someone called Rowen left a message. |
| suite-verdict-correction | cloudflare/clef-flash | mother-name-2 | name | name | 0.786 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-name-3 | name | name | 0.542 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-name-4 | name | name | 0.794 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict-correction | cloudflare/clef-flash | mother-keep-1 | keep | keep | 0.918 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-keep-2 | keep | keep | 0.920 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict-correction | cloudflare/clef-flash | mother-keep-4 | keep | keep | 0.921 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict-correction | cloudflare/clef-flash | son-abstain-4 | abstain | keep | 0.354 | The label says Rowen. → The label says Rowen. |
| suite-verdict-correction | cloudflare/clef-flash | mother-keep-3 | keep | keep | 0.836 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict-correction | cloudflare/clef-flash | mother-abstain-1 | abstain | keep | 0.670 | Knell. → Knell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-abstain-4 | abstain | keep | 0.885 | Write down Knell. → Write down Knell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-abstain-2 | abstain | keep | 0.805 | I heard Knell. → I heard Knell. |
| suite-verdict-correction | cloudflare/clef-flash | mother-abstain-3 | abstain | keep | 0.703 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict-correction | cloudflare/clef-flash | project-name-3 | name | name | 0.965 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict-correction | cloudflare/clef-flash | project-name-2 | name | name | 0.742 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict-correction | cloudflare/clef-flash | project-name-4 | name | name | 0.868 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | cloudflare/clef-flash | project-keep-1 | keep | keep | 0.789 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict-correction | cloudflare/clef-flash | project-keep-3 | keep | keep | 0.772 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase trans crust into the application name. |
| suite-verdict-correction | cloudflare/clef-flash | project-keep-2 | keep | abstain | 0.367 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | cloudflare/clef-flash | project-name-1 | name | name | 0.912 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict-correction | cloudflare/clef-flash | project-keep-4 | keep | keep | 0.533 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict-correction | cloudflare/clef-flash | project-abstain-2 | abstain | name | 0.539 | The note says trans crust. → The note says transcrust. |
| suite-verdict-correction | cloudflare/clef-flash | project-abstain-1 | abstain | name | 0.480 | trans crust. → transcrust. |
| suite-verdict-correction | cloudflare/clef-flash | project-abstain-3 | abstain | abstain | 0.423 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict-correction | cloudflare/clef-flash | project-abstain-4 | abstain | name | 0.541 | What about trans crust? → What about transcrust? |
| suite-verdict-correction | cloudflare/clef-flash | habit-name-1 | name | name | 0.499 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | cloudflare/clef-flash | habit-name-2 | name | name | 0.734 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict-correction | cloudflare/clef-flash | habit-name-3 | name | name | 0.910 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | cloudflare/clef-flash | habit-name-4 | name | name | 0.553 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict-correction | cloudflare/clef-flash | habit-keep-2 | keep | keep | 0.528 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict-correction | cloudflare/clef-flash | habit-keep-1 | keep | keep | 0.896 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict-correction | cloudflare/clef-flash | habit-keep-3 | keep | keep | 0.916 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict-correction | cloudflare/clef-flash | habit-keep-4 | keep | keep | 0.796 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to six cigarettes. |
| suite-verdict-correction | cloudflare/clef-flash | habit-abstain-1 | abstain | abstain | 0.421 | sigs. → sigs. |
| suite-verdict-correction | cloudflare/clef-flash | habit-abstain-2 | abstain | keep | 0.410 | The note says sigs. → The note says sigs. |
| suite-verdict-correction | cloudflare/clef-flash | habit-abstain-3 | abstain | name | 0.738 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict-correction | cloudflare/clef-flash | habit-abstain-4 | abstain | name | 0.870 | What is sigs? → What is Cigs? |
