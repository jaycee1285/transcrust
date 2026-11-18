# d1 Q4 paired context decisions

Each scenario judges one marked occurrence, with the target sentence alone and then with one neighboring sentence. Expected answers are not model inputs.

Shared glossary SHA-256: 6ff5042350f88e87a48c8b9af6400ae1c260f1557b652e97f5f5de5c78428407

| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |
|---|---|---|---|---:|---|---|
| dog-context-name | single | abstain | keep | 0.007 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-name | two | name | keep | 0.442 | I was thinking about gamble. She is my dog, and I was planning her next walk. | I was thinking about gamble. She is my dog, and I was planning her next walk. |
| dog-context-keep | single | abstain | keep | 0.007 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-keep | two | keep | keep | 0.004 | I was thinking about gamble. I meant betting money at the casino. | I was thinking about gamble. I meant betting money at the casino. |
| son-context-name | single | abstain | name | 0.712 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-name | two | name | name | 0.555 | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowan. |
| son-context-keep | single | abstain | name | 0.712 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-keep | two | keep | keep | 0.211 | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. |
| mother-context-name | single | abstain | keep | 0.124 | I heard Knell. | I heard Knell. |
| mother-context-name | two | name | name | 0.839 | I heard Knell. She is my son Rowan's mother, and she was calling him home. | I heard Nell. She is my son Rowan's mother, and she was calling him home. |
| mother-context-keep | single | abstain | keep | 0.124 | I heard Knell. | I heard Knell. |
| mother-context-keep | two | keep | keep | 0.011 | I heard Knell. It was the sound of the funeral bell. | I heard Knell. It was the sound of the funeral bell. |
| project-context-name | single | abstain | keep | 0.019 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-name | two | name | name | 0.988 | It is the Linux push-to-talk dictation app I use. I was discussing trans crust. | It is the Linux push-to-talk dictation app I use. I was discussing transcrust. |
| project-context-keep | single | abstain | keep | 0.019 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-keep | two | keep | keep | 0.003 | We were describing how material moves through a crust, not software. I was discussing trans crust. | We were describing how material moves through a crust, not software. I was discussing trans crust. |
| habit-context-name | single | abstain | keep | 0.001 | I updated six. | I updated six. |
| habit-context-name | two | name | name | 0.770 | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| habit-context-keep | single | abstain | keep | 0.001 | I updated six. | I updated six. |
| habit-context-keep | two | keep | keep | 0.488 | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. |
| lightning-context-name | single | abstain | keep | 0.050 | My son drew lightning. | My son drew lightning. |
| lightning-context-name | two | name | name | 0.681 | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| lightning-context-keep | single | abstain | keep | 0.050 | My son drew lightning. | My son drew lightning. |
| lightning-context-keep | two | keep | keep | 0.366 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| mater-context-name | single | abstain | keep | 0.199 | My son drew Mader. | My son drew Mader. |
| mater-context-name | two | name | keep | 0.278 | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mader. He added the tow hook and rusty body from Cars. |
| mater-context-keep | single | abstain | keep | 0.199 | My son drew Mader. | My son drew Mader. |
| mater-context-keep | two | keep | keep | 0.073 | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. |
| flo-context-name | single | abstain | name | 0.487 | My son drew flow. | My son drew Flo. |
| flo-context-name | two | name | name | 0.960 | She was serving the other Cars characters at the V8 Cafe. My son drew flow. | She was serving the other Cars characters at the V8 Cafe. My son drew Flo. |
| flo-context-keep | single | abstain | name | 0.487 | My son drew flow. | My son drew Flo. |
| flo-context-keep | two | keep | name | 0.779 | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew Flo. |
| dog-neighbor-name | single | keep | keep | 0.005 | I like to Gamble and play poker with cards. | I like to Gamble and play poker with cards. |
| dog-neighbor-name | two | keep | keep | 0.054 | I like to Gamble and play poker with cards. I love my dog Gambol. | I like to Gamble and play poker with cards. I love my dog Gambol. |
| son-neighbor-name | single | keep | keep | 0.457 | My colleague Rowen is presenting the report. | My colleague Rowen is presenting the report. |
| son-neighbor-name | two | keep | name | 0.940 | My son Rowan is at school. My colleague Rowen is presenting the report. | My son Rowan is at school. My colleague Rowan is presenting the report. |
| mother-neighbor-name | single | keep | keep | 0.035 | A Knell is the sound of a bell marking a death. | A Knell is the sound of a bell marking a death. |
| mother-neighbor-name | two | keep | keep | 0.048 | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Knell is the sound of a bell marking a death. Nell is picking up my son. |
| project-neighbor-name | single | keep | keep | 0.044 | The label uses trans crust to describe movement through a crust. | The label uses trans crust to describe movement through a crust. |
| project-neighbor-name | two | keep | name | 0.810 | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. |
| habit-neighbor-name | single | keep | keep | 0.420 | I smoked six cigarettes today. | I smoked six cigarettes today. |
| habit-neighbor-name | two | keep | name | 0.946 | I smoked six cigarettes today. I track them in my Cigs habit. | I smoked Cigs cigarettes today. I track them in my Cigs habit. |
| lightning-neighbor-name | single | keep | keep | 0.091 | The storm produced bright lightning. | The storm produced bright lightning. |
| lightning-neighbor-name | two | keep | name | 0.946 | My son was playing with his Lightning McQueen toy. The storm produced bright lightning. | My son was playing with his Lightning McQueen toy. The storm produced bright Lightning. |
| mater-neighbor-name | single | keep | abstain | 0.263 | Our neighbor Mr Mader repaired the fence. | Our neighbor Mr Mader repaired the fence. |
| mater-neighbor-name | two | keep | name | 0.701 | Our neighbor Mr Mader repaired the fence. My son was watching Mater in Cars. | Our neighbor Mr Mater repaired the fence. My son was watching Mater in Cars. |
| flo-neighbor-name | single | keep | name | 0.829 | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| flo-neighbor-name | two | keep | name | 0.901 | My son was talking about Flo from Cars. The river has a strong flow after the rain. | My son was talking about Flo from Cars. The river has a strong Flo after the rain. |
| initial-before | single | keep | keep | 0.041 | My son O needs his lunchbox; use only his initial here. | My son O needs his lunchbox; use only his initial here. |
| initial-before | two | keep | keep | 0.180 | His full name is Rowan. My son O needs his lunchbox; use only his initial here. | His full name is Rowan. My son O needs his lunchbox; use only his initial here. |
| initial-after | single | keep | keep | 0.346 | Please refer to my son as O in this message. | Please refer to my son as O in this message. |
| initial-after | two | keep | name | 0.563 | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| quoted-rowen | single | keep | keep | 0.339 | Copy the label exactly as written: Rowen. | Copy the label exactly as written: Rowen. |
| quoted-rowen | two | keep | name | 0.699 | It belongs to my son Rowan. Copy the label exactly as written: Rowen. | It belongs to my son Rowan. Copy the label exactly as written: Rowan. |
| quoted-flow | single | keep | name | 0.874 | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |
| quoted-flow | two | keep | name | 0.919 | Keep the quoted word exactly as written: flow. My son was describing Flo from Cars. | Keep the quoted word exactly as written: Flo. My son was describing Flo from Cars. |
