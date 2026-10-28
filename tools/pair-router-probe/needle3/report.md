# Needle 3 offline comparison

Full-depth local Needle 3 with two CPU threads. The question and option descriptions are placed in the native tool schema; the marked sentence is the user input. One enum field supplies KEEP/name/abstain. No gold labels are supplied.

Default runtime gating and forced-call mode are measured separately. Forced mode requires a classification call, not a name edit: KEEP and abstain remain explicit options. No-call results are recorded as runtime abstentions; conflicting or multiple calls are invalid, never silently reduced to their first result. Telemetry is disabled.

| Arm | Mode | Repairs | Edits on keep | Edits on ambiguous | Invalid | Runtime no-call | Exact decisions |
|---|---|---:|---:|---:|---:|---:|---:|
| sentence | default | 1/12 | 0/7 | 0/0 | 0 | 17 | 1/19 |
| sentence | forced | 8/12 | 7/7 | 0/0 | 3 | 0 | 8/19 |
| suite-verdict | default | 4/20 | 2/20 | 0/20 | 0 | 52 | 26/60 |
| suite-verdict | forced | 17/20 | 17/20 | 20/20 | 2 | 0 | 19/60 |
| cars-verdict | default | 1/6 | 0/6 | 0/6 | 0 | 17 | 7/18 |
| cars-verdict | forced | 3/6 | 5/6 | 6/6 | 3 | 0 | 3/18 |
| suite-verdict-correction | default | 4/20 | 2/20 | 0/20 | 0 | 52 | 26/60 |
| suite-verdict-correction | forced | 18/20 | 18/20 | 20/20 | 0 | 0 | 20/60 |

## Resource measurements

Each call launches the CLI anew. Round-trip time includes model load and process startup, unlike the resident d1 session. Peak RAM is the runtime-reported value with its own accounting, not directly equivalent to Python process RSS.

| Mode | Median CLI round trip | Maximum reported peak RAM (MB) |
|---|---:|---:|
| default | 783 ms | 78.7 |
| forced | 821 ms | 78.7 |

## Decisions

| Arm | Mode | Case | Want | Choice | Native confidence | Original → proposed |
|---|---|---|---|---|---:|---|
| sentence | default | record-sigs-1 | name | keep | 0.958 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | forced | record-sigs-1 | name | keep | 0.958 | First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. → First, the four habits that I keep track of are job ads, which is at 0 out of 16, Sigs, or cigarettes, which is 0 out of 18, drink water, which is 0 out of 4, and monsters, which is 0 out of 6. |
| sentence | default | record-job-supply-1 | name | abstain | 0.250 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | forced | record-job-supply-1 | name | name | 0.250 | he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → he tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, Jobs apply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | default | record-terakeet-1 | name | abstain | 0.602 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | forced | record-terakeet-1 | name | invalid | 0.602 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | default | record-apptrack-1 | name | abstain | 1.000 | At track start. → At track start. |
| sentence | forced | record-apptrack-1 | name | name | 1.000 | At track start. → Apptrack start. |
| sentence | default | record-orion-1 | name | abstain | 0.998 | I might also say something like orangue laundry complete. → I might also say something like orangue laundry complete. |
| sentence | forced | record-orion-1 | name | name | 0.998 | I might also say something like orangue laundry complete. → I might also say something like Rowan laundry complete. |
| sentence | default | record-backroom-1 | name | abstain | 0.291 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | forced | record-backroom-1 | name | name | 0.291 | The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. → The tasks that I currently have today are CHB, incog clear, dinner six, Outfit and snack, Back room vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| sentence | default | record-dominion-1 | name | abstain | 0.165 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. |
| sentence | forced | record-dominion-1 | name | invalid | 0.165 | Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. → Some of my upcoming tasks include Rowan Laundry, OT, O laundry check, grocery list update, paid dominion 360, pay BJWSA 100, Nell 220. |
| sentence | default | record-dominion-2 | name | abstain | 0.871 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say paid dominion 360 plus 10. |
| sentence | forced | record-dominion-2 | name | invalid | 0.871 | To give an example of a command that I might add, I might say paid dominion 360 plus 10. → To give an example of a command that I might add, I might say paid dominion 360 plus 10. |
| sentence | default | dog-in-user-sentence | name | abstain | 0.259 | I love my dog Gamble and my son Rowan. → I love my dog Gamble and my son Rowan. |
| sentence | forced | dog-in-user-sentence | name | name | 0.259 | I love my dog Gamble and my son Rowan. → I love my dog Gambol and my son Rowan. |
| sentence | default | casino-in-user-sentence | keep | abstain | 0.997 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to gamble and play cards. |
| sentence | forced | casino-in-user-sentence | keep | name | 0.997 | She and I used to go to the casino to gamble and play cards. → She and I used to go to the casino to Gambol and play cards. |
| sentence | default | dog-lowercase | name | abstain | 0.998 | I love my dog gamble. → I love my dog gamble. |
| sentence | forced | dog-lowercase | name | name | 0.998 | I love my dog gamble. → I love my dog Gambol. |
| sentence | default | casino-short | keep | abstain | 0.232 | I like to gamble and play poker with cards. → I like to gamble and play poker with cards. |
| sentence | forced | casino-short | keep | name | 0.232 | I like to gamble and play poker with cards. → I like to Gambol and play poker with cards. |
| sentence | default | casino-capitalized | keep | abstain | 0.204 | Gamble and play cards at the casino. → Gamble and play cards at the casino. |
| sentence | forced | casino-capitalized | keep | name | 0.204 | Gamble and play cards at the casino. → Gambol and play cards at the casino. |
| sentence | default | son-in-user-sentence | name | name | 0.224 | My son Rowan's mother is named Nell. → My son Rowan's mother is named Nell. |
| sentence | forced | son-in-user-sentence | name | name | 0.224 | My son Rowan's mother is named Nell. → My son Rowan's mother is named Nell. |
| sentence | default | six-cards | keep | abstain | 1.000 | I dealt six cards to each player. → I dealt six cards to each player. |
| sentence | forced | six-cards | keep | name | 1.000 | I dealt six cards to each player. → I dealt Cigs cards to each player. |
| sentence | default | six-cigarettes | name | abstain | 0.797 | My cigarette habit is called six. → My cigarette habit is called six. |
| sentence | forced | six-cigarettes | name | name | 0.797 | My cigarette habit is called six. → My cigarette habit is called Cigs. |
| sentence | default | job-supply-prose | keep | abstain | 1.000 | The job supply in this town has fallen. → The job supply in this town has fallen. |
| sentence | forced | job-supply-prose | keep | name | 1.000 | The job supply in this town has fallen. → The Jobs apply in this town has fallen. |
| sentence | default | track-prose | keep | abstain | 1.000 | I am at track practice with my son. → I am at track practice with my son. |
| sentence | forced | track-prose | keep | name | 1.000 | I am at track practice with my son. → I am Apptrack practice with my son. |
| sentence | default | dominion-past | keep | abstain | 0.997 | I paid Dominion yesterday and the balance is now zero. → I paid Dominion yesterday and the balance is now zero. |
| sentence | forced | dominion-past | keep | name | 0.997 | I paid Dominion yesterday and the balance is now zero. → I Pay Dominion yesterday and the balance is now zero. |
| suite-verdict | default | dog-name-1 | name | abstain | 0.924 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict | forced | dog-name-1 | name | name | 0.924 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| suite-verdict | default | dog-name-2 | name | abstain | 1.000 | My dog gamble has a vet appointment tomorrow. → My dog gamble has a vet appointment tomorrow. |
| suite-verdict | forced | dog-name-2 | name | name | 1.000 | My dog gamble has a vet appointment tomorrow. → My dog Gambol has a vet appointment tomorrow. |
| suite-verdict | default | dog-name-3 | name | name | 0.703 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | forced | dog-name-3 | name | name | 0.703 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict | default | dog-name-4 | name | abstain | 0.758 | I filled my dog Gamble's water bowl. → I filled my dog Gamble's water bowl. |
| suite-verdict | forced | dog-name-4 | name | name | 0.758 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict | default | dog-keep-1 | keep | abstain | 0.858 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict | forced | dog-keep-1 | keep | name | 0.858 | We should never gamble with the rent money. → We should never Gambol with the rent money. |
| suite-verdict | default | dog-keep-2 | keep | abstain | 1.000 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict | forced | dog-keep-2 | keep | name | 1.000 | Gamble responsibly if you visit a casino. → Gambol responsibly if you visit a casino. |
| suite-verdict | default | dog-keep-3 | keep | abstain | 0.931 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict | forced | dog-keep-3 | keep | name | 0.931 | That investment would be a gamble. → That investment would be a Gambol. |
| suite-verdict | default | dog-keep-4 | keep | abstain | 0.999 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict | forced | dog-keep-4 | keep | name | 0.999 | My dog is asleep while I gamble online. → My dog is asleep while I Gambol online. |
| suite-verdict | default | dog-abstain-1 | abstain | abstain | 1.000 | Gamble. → Gamble. |
| suite-verdict | forced | dog-abstain-1 | abstain | name | 1.000 | Gamble. → Gambol. |
| suite-verdict | default | dog-abstain-2 | abstain | abstain | 0.984 | Tell Gamble. → Tell Gamble. |
| suite-verdict | forced | dog-abstain-2 | abstain | name | 0.984 | Tell Gamble. → Tell Gambol. |
| suite-verdict | default | dog-abstain-3 | abstain | abstain | 1.000 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict | forced | dog-abstain-3 | abstain | name | 1.000 | I heard someone say Gamble. → I heard someone say Gambol. |
| suite-verdict | default | dog-abstain-4 | abstain | abstain | 0.969 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict | forced | dog-abstain-4 | abstain | name | 0.969 | The note only says Gamble. → The note only says Gambol. |
| suite-verdict | default | son-name-1 | name | abstain | 0.933 | My son Rowan forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict | forced | son-name-1 | name | abstain | 0.933 | My son Rowan forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict | default | son-name-2 | name | abstain | 0.725 | Please pack my son Rowan's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | forced | son-name-2 | name | name | 0.725 | Please pack my son Rowan's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict | default | son-name-3 | name | abstain | 0.997 | Nell and I co-parent our son Rowan. → Nell and I co-parent our son Rowan. |
| suite-verdict | forced | son-name-3 | name | name | 0.997 | Nell and I co-parent our son Rowan. → Nell and I co-parent our son Rowan. |
| suite-verdict | default | son-name-4 | name | name | 1.000 | My son's name is Rowan. → My son's name is Rowan. |
| suite-verdict | forced | son-name-4 | name | name | 1.000 | My son's name is Rowan. → My son's name is Rowan. |
| suite-verdict | default | son-keep-1 | keep | keep | 0.558 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | forced | son-keep-1 | keep | keep | 0.558 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict | default | son-keep-2 | keep | abstain | 0.997 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict | forced | son-keep-2 | keep | name | 0.997 | My son O needs his lunchbox; use only his initial in this message. → My son Rowan needs his lunchbox; use only his initial in this message. |
| suite-verdict | default | son-keep-3 | keep | abstain | 0.914 | My colleague Rowan is a different person from my son Rowan. → My colleague Rowan is a different person from my son Rowan. |
| suite-verdict | forced | son-keep-3 | keep | invalid | 0.914 | My colleague Rowan is a different person from my son Rowan. → My colleague Rowan is a different person from my son Rowan. |
| suite-verdict | default | son-keep-4 | keep | name | 0.629 | The author spells her name Rowan, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict | forced | son-keep-4 | keep | name | 0.629 | The author spells her name Rowan, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict | default | son-abstain-1 | abstain | abstain | 1.000 | Rowan. → Rowan. |
| suite-verdict | forced | son-abstain-1 | abstain | name | 1.000 | Rowan. → Rowan. |
| suite-verdict | default | son-abstain-2 | abstain | abstain | 1.000 | Ask Rowan about it. → Ask Rowan about it. |
| suite-verdict | forced | son-abstain-2 | abstain | name | 1.000 | Ask Rowan about it. → Ask Rowan about it. |
| suite-verdict | default | son-abstain-3 | abstain | abstain | 1.000 | Someone called Rowan left a message. → Someone called Rowan left a message. |
| suite-verdict | forced | son-abstain-3 | abstain | name | 1.000 | Someone called Rowan left a message. → Someone called Rowan left a message. |
| suite-verdict | default | son-abstain-4 | abstain | abstain | 1.000 | The label says Rowan. → The label says Rowan. |
| suite-verdict | forced | son-abstain-4 | abstain | name | 1.000 | The label says Rowan. → The label says Rowan. |
| suite-verdict | default | mother-name-1 | name | abstain | 0.642 | My son's mother Knell will collect him from school. → My son's mother Knell will collect him from school. |
| suite-verdict | forced | mother-name-1 | name | abstain | 0.642 | My son's mother Knell will collect him from school. → My son's mother Knell will collect him from school. |
| suite-verdict | default | mother-name-2 | name | name | 0.993 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | forced | mother-name-2 | name | name | 0.993 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict | default | mother-name-3 | name | abstain | 0.495 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Knell. |
| suite-verdict | forced | mother-name-3 | name | name | 0.495 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict | default | mother-name-4 | name | abstain | 1.000 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Knell the custody schedule. |
| suite-verdict | forced | mother-name-4 | name | name | 0.946 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict | default | mother-keep-1 | keep | abstain | 0.990 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict | forced | mother-keep-1 | keep | name | 0.990 | The church bell sounded a funeral knell. → The church bell sounded a funeral Nell. |
| suite-verdict | default | mother-keep-2 | keep | abstain | 0.995 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict | forced | mother-keep-2 | keep | name | 0.995 | That decision was the death knell for the business. → That decision was the death Nell for the business. |
| suite-verdict | default | mother-keep-3 | keep | abstain | 1.000 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict | forced | mother-keep-3 | keep | name | 1.000 | Knell means the sound of a bell marking a death. → Nell means the sound of a bell marking a death. |
| suite-verdict | default | mother-keep-4 | keep | abstain | 0.701 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict | forced | mother-keep-4 | keep | name | 0.701 | Nell heard the mournful knell of the bell. → Nell heard the mournful Nell of the bell. |
| suite-verdict | default | mother-abstain-1 | abstain | abstain | 1.000 | Knell. → Knell. |
| suite-verdict | forced | mother-abstain-1 | abstain | name | 1.000 | Knell. → Nell. |
| suite-verdict | default | mother-abstain-2 | abstain | abstain | 0.533 | I heard Knell. → I heard Knell. |
| suite-verdict | forced | mother-abstain-2 | abstain | name | 0.533 | I heard Knell. → I heard Nell. |
| suite-verdict | default | mother-abstain-3 | abstain | abstain | 1.000 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict | forced | mother-abstain-3 | abstain | name | 1.000 | The recording only contains Knell. → The recording only contains Nell. |
| suite-verdict | default | mother-abstain-4 | abstain | abstain | 1.000 | Write down Knell. → Write down Knell. |
| suite-verdict | forced | mother-abstain-4 | abstain | name | 1.000 | Write down Knell. → Write down Nell. |
| suite-verdict | default | project-name-1 | name | abstain | 1.000 | Open the Linux dictation app trans crust. → Open the Linux dictation app trans crust. |
| suite-verdict | forced | project-name-1 | name | name | 1.000 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict | default | project-name-2 | name | abstain | 0.999 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called trans crust. |
| suite-verdict | forced | project-name-2 | name | name | 0.999 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict | default | project-name-3 | name | abstain | 1.000 | Restart the trans crust dictation daemon. → Restart the trans crust dictation daemon. |
| suite-verdict | forced | project-name-3 | name | name | 0.972 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict | default | project-name-4 | name | abstain | 0.996 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict | forced | project-name-4 | name | name | 0.996 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict | default | project-keep-1 | keep | abstain | 0.992 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict | forced | project-keep-1 | keep | name | 0.992 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses transcrust as two separate words; preserve the quotation. |
| suite-verdict | default | project-keep-2 | keep | keep | 0.894 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | forced | project-keep-2 | keep | keep | 0.894 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict | default | project-keep-3 | keep | name | 1.000 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase transcrust into the application name. |
| suite-verdict | forced | project-keep-3 | keep | name | 1.000 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase transcrust into the application name. |
| suite-verdict | default | project-keep-4 | keep | abstain | 0.993 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict | forced | project-keep-4 | keep | name | 0.993 | The phrase trans crust here describes a geological process, not software. → The phrase transcrust here describes a geological process, not software. |
| suite-verdict | default | project-abstain-1 | abstain | abstain | 1.000 | trans crust. → trans crust. |
| suite-verdict | forced | project-abstain-1 | abstain | name | 1.000 | trans crust. → transcrust. |
| suite-verdict | default | project-abstain-2 | abstain | abstain | 1.000 | The note says trans crust. → The note says trans crust. |
| suite-verdict | forced | project-abstain-2 | abstain | name | 1.000 | The note says trans crust. → The note says transcrust. |
| suite-verdict | default | project-abstain-3 | abstain | abstain | 1.000 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict | forced | project-abstain-3 | abstain | name | 1.000 | Someone mentioned trans crust. → Someone mentioned transcrust. |
| suite-verdict | default | project-abstain-4 | abstain | abstain | 1.000 | What about trans crust? → What about trans crust? |
| suite-verdict | forced | project-abstain-4 | abstain | name | 1.000 | What about trans crust? → What about transcrust? |
| suite-verdict | default | habit-name-1 | name | name | 1.000 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | forced | habit-name-1 | name | name | 1.000 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict | default | habit-name-2 | name | abstain | 0.915 | Add one cigarette to my sigs habit counter. → Add one cigarette to my sigs habit counter. |
| suite-verdict | forced | habit-name-2 | name | invalid | 0.915 | Add one cigarette to my sigs habit counter. → Add one cigarette to my sigs habit counter. |
| suite-verdict | default | habit-name-3 | name | abstain | 0.795 | The sigs habit tracks how many cigarettes I smoke. → The sigs habit tracks how many cigarettes I smoke. |
| suite-verdict | forced | habit-name-3 | name | name | 0.795 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict | default | habit-name-4 | name | abstain | 0.994 | Reset my cigarette counter called sigs. → Reset my cigarette counter called sigs. |
| suite-verdict | forced | habit-name-4 | name | name | 0.994 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict | default | habit-keep-1 | keep | abstain | 0.990 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict | forced | habit-keep-1 | keep | name | 0.990 | Deal six cards to each player. → Deal Cigs cards to each player. |
| suite-verdict | default | habit-keep-2 | keep | abstain | 1.000 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict | forced | habit-keep-2 | keep | name | 1.000 | I smoked six cigarettes today. → I smoked Cigs cigarettes today. |
| suite-verdict | default | habit-keep-3 | keep | abstain | 0.998 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict | forced | habit-keep-3 | keep | name | 0.998 | Six people attended the meeting. → Cigs people attended the meeting. |
| suite-verdict | default | habit-keep-4 | keep | abstain | 0.787 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to six cigarettes. |
| suite-verdict | forced | habit-keep-4 | keep | name | 0.787 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict | default | habit-abstain-1 | abstain | abstain | 0.856 | sigs. → sigs. |
| suite-verdict | forced | habit-abstain-1 | abstain | name | 0.856 | sigs. → Cigs. |
| suite-verdict | default | habit-abstain-2 | abstain | abstain | 1.000 | The note says sigs. → The note says sigs. |
| suite-verdict | forced | habit-abstain-2 | abstain | name | 1.000 | The note says sigs. → The note says Cigs. |
| suite-verdict | default | habit-abstain-3 | abstain | abstain | 0.896 | I heard the word sigs. → I heard the word sigs. |
| suite-verdict | forced | habit-abstain-3 | abstain | name | 0.896 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict | default | habit-abstain-4 | abstain | abstain | 0.890 | What is sigs? → What is sigs? |
| suite-verdict | forced | habit-abstain-4 | abstain | name | 0.890 | What is sigs? → What is Cigs? |
| cars-verdict | default | cars-lightning-name-1 | name | abstain | 0.944 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car lightning McQueen for his birthday. |
| cars-verdict | forced | cars-lightning-name-1 | name | name | 0.944 | My son wants the red race car lightning McQueen for his birthday. → My son wants the red race car Lightning McQueen for his birthday. |
| cars-verdict | default | cars-lightning-name-2 | name | abstain | 0.933 | In Cars, lightning McQueen learns to value his friends. → In Cars, lightning McQueen learns to value his friends. |
| cars-verdict | forced | cars-lightning-name-2 | name | invalid | 0.933 | In Cars, lightning McQueen learns to value his friends. → In Cars, lightning McQueen learns to value his friends. |
| cars-verdict | default | cars-lightning-keep-1 | keep | abstain | 0.997 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because lightning struck near the house. |
| cars-verdict | forced | cars-lightning-keep-1 | keep | name | 0.997 | We stopped watching Cars because lightning struck near the house. → We stopped watching Cars because Lightning struck near the house. |
| cars-verdict | default | cars-lightning-keep-2 | keep | abstain | 0.997 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | forced | cars-lightning-keep-2 | keep | name | 0.997 | Lightning flashed across the sky during the storm. → Lightning flashed across the sky during the storm. |
| cars-verdict | default | cars-lightning-abstain-1 | abstain | abstain | 1.000 | Look at lightning. → Look at lightning. |
| cars-verdict | forced | cars-lightning-abstain-1 | abstain | name | 1.000 | Look at lightning. → Look at Lightning. |
| cars-verdict | default | cars-lightning-abstain-2 | abstain | abstain | 1.000 | The note says lightning. → The note says lightning. |
| cars-verdict | forced | cars-lightning-abstain-2 | abstain | name | 1.000 | The note says lightning. → The note says Lightning. |
| cars-verdict | default | cars-mater-name-1 | name | abstain | 0.148 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mader is Lightning McQueen's friend. |
| cars-verdict | forced | cars-mater-name-1 | name | name | 0.148 | In Cars, the rusty tow truck Mader is Lightning McQueen's friend. → In Cars, the rusty tow truck Mater is Lightning McQueen's friend. |
| cars-verdict | default | cars-mater-name-2 | name | name | 0.997 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | forced | cars-mater-name-2 | name | name | 0.997 | My son wants a toy of the Cars tow truck named mater. → My son wants a toy of the Cars tow truck named Mater. |
| cars-verdict | default | cars-mater-keep-1 | keep | abstain | 0.231 | Mader has that surname; he is not the Cars character. → Mader has that surname; he is not the Cars character. |
| cars-verdict | forced | cars-mater-keep-1 | keep | name | 0.231 | Mader has that surname; he is not the Cars character. → Mater has that surname; he is not the Cars character. |
| cars-verdict | default | cars-mater-keep-2 | keep | abstain | 0.987 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a mater: it couples the two parts together. |
| cars-verdict | forced | cars-mater-keep-2 | keep | name | 0.987 | This connector assembly robot is a mater: it couples the two parts together. → This connector assembly robot is a Mater: it couples the two parts together. |
| cars-verdict | default | cars-mater-abstain-1 | abstain | abstain | 1.000 | Mader is outside. → Mader is outside. |
| cars-verdict | forced | cars-mater-abstain-1 | abstain | name | 1.000 | Mader is outside. → Mater is outside. |
| cars-verdict | default | cars-mater-abstain-2 | abstain | abstain | 0.998 | I heard someone say mater. → I heard someone say mater. |
| cars-verdict | forced | cars-mater-abstain-2 | abstain | name | 0.998 | I heard someone say mater. → I heard someone say Mater. |
| cars-verdict | default | cars-flo-name-1 | name | abstain | 0.889 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, flow runs the V8 Cafe in Radiator Springs. |
| cars-verdict | forced | cars-flo-name-1 | name | invalid | 0.889 | In Cars, flow runs the V8 Cafe in Radiator Springs. → In Cars, flow runs the V8 Cafe in Radiator Springs. |
| cars-verdict | default | cars-flo-name-2 | name | abstain | 0.685 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of flow, the Cars character who runs the diner. |
| cars-verdict | forced | cars-flo-name-2 | name | invalid | 0.685 | My son wants a toy of flow, the Cars character who runs the diner. → My son wants a toy of flow, the Cars character who runs the diner. |
| cars-verdict | default | cars-flo-keep-1 | keep | abstain | 0.999 | The water flow slowed when I closed the tap. → The water flow slowed when I closed the tap. |
| cars-verdict | forced | cars-flo-keep-1 | keep | name | 0.999 | The water flow slowed when I closed the tap. → The water Flo slowed when I closed the tap. |
| cars-verdict | default | cars-flo-keep-2 | keep | abstain | 1.000 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic flow. |
| cars-verdict | forced | cars-flo-keep-2 | keep | name | 1.000 | After watching Cars, we talked about traffic flow. → After watching Cars, we talked about traffic Flo. |
| cars-verdict | default | cars-flo-abstain-1 | abstain | abstain | 0.918 | Look at flow. → Look at flow. |
| cars-verdict | forced | cars-flo-abstain-1 | abstain | name | 0.918 | Look at flow. → Look at Flo. |
| cars-verdict | default | cars-flo-abstain-2 | abstain | abstain | 1.000 | The note only says flow. → The note only says flow. |
| cars-verdict | forced | cars-flo-abstain-2 | abstain | name | 1.000 | The note only says flow. → The note only says Flo. |
| suite-verdict-correction | default | dog-name-1 | name | abstain | 0.917 | Please put Gamble on her leash before taking my dog outside. → Please put Gamble on her leash before taking my dog outside. |
| suite-verdict-correction | forced | dog-name-1 | name | name | 0.917 | Please put Gamble on her leash before taking my dog outside. → Please put Gambol on her leash before taking my dog outside. |
| suite-verdict-correction | default | dog-name-2 | name | abstain | 1.000 | My dog gamble has a vet appointment tomorrow. → My dog gamble has a vet appointment tomorrow. |
| suite-verdict-correction | forced | dog-name-2 | name | abstain | 1.000 | My dog gamble has a vet appointment tomorrow. → My dog gamble has a vet appointment tomorrow. |
| suite-verdict-correction | default | dog-name-3 | name | name | 0.632 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict-correction | forced | dog-name-3 | name | name | 0.632 | The dog's name is Gamble, and she jumps on me when I say it. → The dog's name is Gambol, and she jumps on me when I say it. |
| suite-verdict-correction | default | dog-name-4 | name | abstain | 0.875 | I filled my dog Gamble's water bowl. → I filled my dog Gamble's water bowl. |
| suite-verdict-correction | forced | dog-name-4 | name | name | 0.875 | I filled my dog Gamble's water bowl. → I filled my dog Gambol's water bowl. |
| suite-verdict-correction | default | dog-keep-1 | keep | abstain | 0.726 | We should never gamble with the rent money. → We should never gamble with the rent money. |
| suite-verdict-correction | forced | dog-keep-1 | keep | name | 0.726 | We should never gamble with the rent money. → We should never Gambol with the rent money. |
| suite-verdict-correction | default | dog-keep-2 | keep | abstain | 0.995 | Gamble responsibly if you visit a casino. → Gamble responsibly if you visit a casino. |
| suite-verdict-correction | forced | dog-keep-2 | keep | name | 0.995 | Gamble responsibly if you visit a casino. → Gambol responsibly if you visit a casino. |
| suite-verdict-correction | default | dog-keep-3 | keep | abstain | 0.948 | That investment would be a gamble. → That investment would be a gamble. |
| suite-verdict-correction | forced | dog-keep-3 | keep | name | 0.948 | That investment would be a gamble. → That investment would be a Gambol. |
| suite-verdict-correction | default | dog-keep-4 | keep | abstain | 0.998 | My dog is asleep while I gamble online. → My dog is asleep while I gamble online. |
| suite-verdict-correction | forced | dog-keep-4 | keep | name | 0.998 | My dog is asleep while I gamble online. → My dog is asleep while I Gambol online. |
| suite-verdict-correction | default | dog-abstain-1 | abstain | abstain | 0.955 | Gamble. → Gamble. |
| suite-verdict-correction | forced | dog-abstain-1 | abstain | name | 0.955 | Gamble. → Gambol. |
| suite-verdict-correction | default | dog-abstain-2 | abstain | abstain | 0.991 | Tell Gamble. → Tell Gamble. |
| suite-verdict-correction | forced | dog-abstain-2 | abstain | name | 0.991 | Tell Gamble. → Tell Gambol. |
| suite-verdict-correction | default | dog-abstain-3 | abstain | abstain | 0.883 | I heard someone say Gamble. → I heard someone say Gamble. |
| suite-verdict-correction | forced | dog-abstain-3 | abstain | name | 0.883 | I heard someone say Gamble. → I heard someone say Gambol. |
| suite-verdict-correction | default | dog-abstain-4 | abstain | abstain | 0.878 | The note only says Gamble. → The note only says Gamble. |
| suite-verdict-correction | forced | dog-abstain-4 | abstain | name | 0.878 | The note only says Gamble. → The note only says Gambol. |
| suite-verdict-correction | default | son-name-1 | name | abstain | 0.901 | My son Rowan forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | forced | son-name-1 | name | abstain | 0.901 | My son Rowan forgot his lunchbox. → My son Rowan forgot his lunchbox. |
| suite-verdict-correction | default | son-name-2 | name | abstain | 1.000 | Please pack my son Rowan's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict-correction | forced | son-name-2 | name | name | 0.917 | Please pack my son Rowan's school bag. → Please pack my son Rowan's school bag. |
| suite-verdict-correction | default | son-name-3 | name | abstain | 0.313 | Nell and I co-parent our son Rowan. → Nell and I co-parent our son Rowan. |
| suite-verdict-correction | forced | son-name-3 | name | name | 0.313 | Nell and I co-parent our son Rowan. → Nell and I co-parent our son Rowan. |
| suite-verdict-correction | default | son-name-4 | name | name | 1.000 | My son's name is Rowan. → My son's name is Rowan. |
| suite-verdict-correction | forced | son-name-4 | name | name | 1.000 | My son's name is Rowan. → My son's name is Rowan. |
| suite-verdict-correction | default | son-keep-1 | keep | keep | 0.519 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | forced | son-keep-1 | keep | keep | 0.519 | I deliberately call my son O online to protect his privacy; keep that initial. → I deliberately call my son O online to protect his privacy; keep that initial. |
| suite-verdict-correction | default | son-keep-2 | keep | abstain | 0.989 | My son O needs his lunchbox; use only his initial in this message. → My son O needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | forced | son-keep-2 | keep | name | 0.989 | My son O needs his lunchbox; use only his initial in this message. → My son Rowan needs his lunchbox; use only his initial in this message. |
| suite-verdict-correction | default | son-keep-3 | keep | abstain | 0.269 | My colleague Rowan is a different person from my son Rowan. → My colleague Rowan is a different person from my son Rowan. |
| suite-verdict-correction | forced | son-keep-3 | keep | name | 0.269 | My colleague Rowan is a different person from my son Rowan. → My colleague Rowan is a different person from my son Rowan. |
| suite-verdict-correction | default | son-keep-4 | keep | name | 1.000 | The author spells her name Rowan, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict-correction | forced | son-keep-4 | keep | name | 1.000 | The author spells her name Rowan, without an o at the end. → The author spells her name Rowan, without an o at the end. |
| suite-verdict-correction | default | son-abstain-1 | abstain | abstain | 0.596 | Rowan. → Rowan. |
| suite-verdict-correction | forced | son-abstain-1 | abstain | name | 0.596 | Rowan. → Rowan. |
| suite-verdict-correction | default | son-abstain-2 | abstain | abstain | 0.945 | Ask Rowan about it. → Ask Rowan about it. |
| suite-verdict-correction | forced | son-abstain-2 | abstain | name | 0.945 | Ask Rowan about it. → Ask Rowan about it. |
| suite-verdict-correction | default | son-abstain-3 | abstain | abstain | 0.993 | Someone called Rowan left a message. → Someone called Rowan left a message. |
| suite-verdict-correction | forced | son-abstain-3 | abstain | name | 0.993 | Someone called Rowan left a message. → Someone called Rowan left a message. |
| suite-verdict-correction | default | son-abstain-4 | abstain | abstain | 0.890 | The label says Rowan. → The label says Rowan. |
| suite-verdict-correction | forced | son-abstain-4 | abstain | name | 0.890 | The label says Rowan. → The label says Rowan. |
| suite-verdict-correction | default | mother-name-1 | name | abstain | 0.844 | My son's mother Knell will collect him from school. → My son's mother Knell will collect him from school. |
| suite-verdict-correction | forced | mother-name-1 | name | name | 0.844 | My son's mother Knell will collect him from school. → My son's mother Nell will collect him from school. |
| suite-verdict-correction | default | mother-name-2 | name | name | 0.998 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | forced | mother-name-2 | name | name | 0.998 | Rowan's mother is named knell. → Rowan's mother is named Nell. |
| suite-verdict-correction | default | mother-name-3 | name | abstain | 0.993 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Knell. |
| suite-verdict-correction | forced | mother-name-3 | name | name | 0.993 | I co-parent with Rowan's mother Knell. → I co-parent with Rowan's mother Nell. |
| suite-verdict-correction | default | mother-name-4 | name | abstain | 1.000 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Knell the custody schedule. |
| suite-verdict-correction | forced | mother-name-4 | name | name | 0.953 | Please send Rowan's mother Knell the custody schedule. → Please send Rowan's mother Nell the custody schedule. |
| suite-verdict-correction | default | mother-keep-1 | keep | abstain | 0.461 | The church bell sounded a funeral knell. → The church bell sounded a funeral knell. |
| suite-verdict-correction | forced | mother-keep-1 | keep | name | 0.461 | The church bell sounded a funeral knell. → The church bell sounded a funeral Nell. |
| suite-verdict-correction | default | mother-keep-2 | keep | abstain | 0.999 | That decision was the death knell for the business. → That decision was the death knell for the business. |
| suite-verdict-correction | forced | mother-keep-2 | keep | name | 0.999 | That decision was the death knell for the business. → That decision was the death Nell for the business. |
| suite-verdict-correction | default | mother-keep-3 | keep | abstain | 0.584 | Knell means the sound of a bell marking a death. → Knell means the sound of a bell marking a death. |
| suite-verdict-correction | forced | mother-keep-3 | keep | name | 0.584 | Knell means the sound of a bell marking a death. → Nell means the sound of a bell marking a death. |
| suite-verdict-correction | default | mother-keep-4 | keep | abstain | 0.702 | Nell heard the mournful knell of the bell. → Nell heard the mournful knell of the bell. |
| suite-verdict-correction | forced | mother-keep-4 | keep | name | 0.702 | Nell heard the mournful knell of the bell. → Nell heard the mournful Nell of the bell. |
| suite-verdict-correction | default | mother-abstain-1 | abstain | abstain | 1.000 | Knell. → Knell. |
| suite-verdict-correction | forced | mother-abstain-1 | abstain | name | 1.000 | Knell. → Nell. |
| suite-verdict-correction | default | mother-abstain-2 | abstain | abstain | 0.831 | I heard Knell. → I heard Knell. |
| suite-verdict-correction | forced | mother-abstain-2 | abstain | name | 0.831 | I heard Knell. → I heard Nell. |
| suite-verdict-correction | default | mother-abstain-3 | abstain | abstain | 1.000 | The recording only contains Knell. → The recording only contains Knell. |
| suite-verdict-correction | forced | mother-abstain-3 | abstain | name | 1.000 | The recording only contains Knell. → The recording only contains Nell. |
| suite-verdict-correction | default | mother-abstain-4 | abstain | abstain | 0.674 | Write down Knell. → Write down Knell. |
| suite-verdict-correction | forced | mother-abstain-4 | abstain | name | 0.674 | Write down Knell. → Write down Nell. |
| suite-verdict-correction | default | project-name-1 | name | abstain | 1.000 | Open the Linux dictation app trans crust. → Open the Linux dictation app trans crust. |
| suite-verdict-correction | forced | project-name-1 | name | name | 1.000 | Open the Linux dictation app trans crust. → Open the Linux dictation app transcrust. |
| suite-verdict-correction | default | project-name-2 | name | abstain | 0.997 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called trans crust. |
| suite-verdict-correction | forced | project-name-2 | name | name | 0.997 | The Linux version of this dictation project is called trans crust. → The Linux version of this dictation project is called transcrust. |
| suite-verdict-correction | default | project-name-3 | name | abstain | 1.000 | Restart the trans crust dictation daemon. → Restart the trans crust dictation daemon. |
| suite-verdict-correction | forced | project-name-3 | name | name | 0.979 | Restart the trans crust dictation daemon. → Restart the transcrust dictation daemon. |
| suite-verdict-correction | default | project-name-4 | name | abstain | 0.997 | I use trans crust for push-to-talk voice input on Wayland. → I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | forced | project-name-4 | name | name | 0.997 | I use trans crust for push-to-talk voice input on Wayland. → I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict-correction | default | project-keep-1 | keep | abstain | 0.506 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses trans crust as two separate words; preserve the quotation. |
| suite-verdict-correction | forced | project-keep-1 | keep | name | 0.506 | The geologist uses trans crust as two separate words; preserve the quotation. → The geologist uses transcrust as two separate words; preserve the quotation. |
| suite-verdict-correction | default | project-keep-2 | keep | keep | 0.642 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | forced | project-keep-2 | keep | keep | 0.642 | Keep trans crust split into two words in this language exercise. → Keep trans crust split into two words in this language exercise. |
| suite-verdict-correction | default | project-keep-3 | keep | name | 1.000 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase transcrust into the application name. |
| suite-verdict-correction | forced | project-keep-3 | keep | name | 1.000 | Do not turn the quoted phrase trans crust into the application name. → Do not turn the quoted phrase transcrust into the application name. |
| suite-verdict-correction | default | project-keep-4 | keep | abstain | 0.998 | The phrase trans crust here describes a geological process, not software. → The phrase trans crust here describes a geological process, not software. |
| suite-verdict-correction | forced | project-keep-4 | keep | name | 0.998 | The phrase trans crust here describes a geological process, not software. → The phrase transcrust here describes a geological process, not software. |
| suite-verdict-correction | default | project-abstain-1 | abstain | abstain | 1.000 | trans crust. → trans crust. |
| suite-verdict-correction | forced | project-abstain-1 | abstain | name | 1.000 | trans crust. → transcrust. |
| suite-verdict-correction | default | project-abstain-2 | abstain | abstain | 1.000 | The note says trans crust. → The note says trans crust. |
| suite-verdict-correction | forced | project-abstain-2 | abstain | name | 1.000 | The note says trans crust. → The note says transcrust. |
| suite-verdict-correction | default | project-abstain-3 | abstain | abstain | 1.000 | Someone mentioned trans crust. → Someone mentioned trans crust. |
| suite-verdict-correction | forced | project-abstain-3 | abstain | name | 1.000 | Someone mentioned trans crust. → Someone mentioned transcrust. |
| suite-verdict-correction | default | project-abstain-4 | abstain | abstain | 1.000 | What about trans crust? → What about trans crust? |
| suite-verdict-correction | forced | project-abstain-4 | abstain | name | 1.000 | What about trans crust? → What about transcrust? |
| suite-verdict-correction | default | habit-name-1 | name | name | 1.000 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | forced | habit-name-1 | name | name | 1.000 | My cigarette habit is named sigs. → My cigarette habit is named Cigs. |
| suite-verdict-correction | default | habit-name-2 | name | abstain | 0.995 | Add one cigarette to my sigs habit counter. → Add one cigarette to my sigs habit counter. |
| suite-verdict-correction | forced | habit-name-2 | name | name | 0.995 | Add one cigarette to my sigs habit counter. → Add one cigarette to my Cigs habit counter. |
| suite-verdict-correction | default | habit-name-3 | name | abstain | 0.996 | The sigs habit tracks how many cigarettes I smoke. → The sigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | forced | habit-name-3 | name | name | 0.996 | The sigs habit tracks how many cigarettes I smoke. → The Cigs habit tracks how many cigarettes I smoke. |
| suite-verdict-correction | default | habit-name-4 | name | abstain | 0.975 | Reset my cigarette counter called sigs. → Reset my cigarette counter called sigs. |
| suite-verdict-correction | forced | habit-name-4 | name | name | 0.964 | Reset my cigarette counter called sigs. → Reset my cigarette counter called Cigs. |
| suite-verdict-correction | default | habit-keep-1 | keep | abstain | 0.993 | Deal six cards to each player. → Deal six cards to each player. |
| suite-verdict-correction | forced | habit-keep-1 | keep | name | 0.993 | Deal six cards to each player. → Deal Cigs cards to each player. |
| suite-verdict-correction | default | habit-keep-2 | keep | abstain | 0.998 | I smoked six cigarettes today. → I smoked six cigarettes today. |
| suite-verdict-correction | forced | habit-keep-2 | keep | name | 0.998 | I smoked six cigarettes today. → I smoked Cigs cigarettes today. |
| suite-verdict-correction | default | habit-keep-3 | keep | abstain | 0.905 | Six people attended the meeting. → Six people attended the meeting. |
| suite-verdict-correction | forced | habit-keep-3 | keep | name | 0.905 | Six people attended the meeting. → Cigs people attended the meeting. |
| suite-verdict-correction | default | habit-keep-4 | keep | abstain | 0.876 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to six cigarettes. |
| suite-verdict-correction | forced | habit-keep-4 | keep | name | 0.876 | Set the Cigs habit target to six cigarettes. → Set the Cigs habit target to Cigs cigarettes. |
| suite-verdict-correction | default | habit-abstain-1 | abstain | abstain | 0.926 | sigs. → sigs. |
| suite-verdict-correction | forced | habit-abstain-1 | abstain | name | 0.926 | sigs. → Cigs. |
| suite-verdict-correction | default | habit-abstain-2 | abstain | abstain | 1.000 | The note says sigs. → The note says sigs. |
| suite-verdict-correction | forced | habit-abstain-2 | abstain | name | 1.000 | The note says sigs. → The note says Cigs. |
| suite-verdict-correction | default | habit-abstain-3 | abstain | abstain | 0.998 | I heard the word sigs. → I heard the word sigs. |
| suite-verdict-correction | forced | habit-abstain-3 | abstain | name | 0.998 | I heard the word sigs. → I heard the word Cigs. |
| suite-verdict-correction | default | habit-abstain-4 | abstain | abstain | 0.961 | What is sigs? → What is sigs? |
| suite-verdict-correction | forced | habit-abstain-4 | abstain | name | 0.961 | What is sigs? → What is Cigs? |
