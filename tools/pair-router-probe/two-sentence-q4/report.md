# d1 Q4 paired context decisions

Each scenario judges one marked occurrence, with the target sentence alone and then with one neighboring sentence. Expected answers are not model inputs.

| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |
|---|---|---|---|---:|---|---|
| dog-context-name | single | abstain | keep | 0.000 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-name | two | name | keep | 0.208 | I was thinking about gamble. She is my dog, and I was planning her next walk. | I was thinking about gamble. She is my dog, and I was planning her next walk. |
| dog-context-keep | single | abstain | keep | 0.000 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-keep | two | keep | keep | 0.000 | I was thinking about gamble. I meant betting money at the casino. | I was thinking about gamble. I meant betting money at the casino. |
| son-context-name | single | abstain | keep | 0.341 | I spoke to Rowen. | I spoke to Rowen. |
| son-context-name | two | name | keep | 0.392 | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. |
| son-context-keep | single | abstain | keep | 0.341 | I spoke to Rowen. | I spoke to Rowen. |
| son-context-keep | two | keep | keep | 0.024 | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. |
| mother-context-name | single | abstain | keep | 0.024 | I heard Knell. | I heard Knell. |
| mother-context-name | two | name | name | 0.734 | I heard Knell. She is my son Rowan's mother, and she was calling him home. | I heard Nell. She is my son Rowan's mother, and she was calling him home. |
| mother-context-keep | single | abstain | keep | 0.024 | I heard Knell. | I heard Knell. |
| mother-context-keep | two | keep | keep | 0.004 | I heard Knell. It was the sound of the funeral bell. | I heard Knell. It was the sound of the funeral bell. |
| project-context-name | single | abstain | keep | 0.001 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-name | two | name | name | 0.964 | It is the Linux push-to-talk dictation app I use. I was discussing trans crust. | It is the Linux push-to-talk dictation app I use. I was discussing transcrust. |
| project-context-keep | single | abstain | keep | 0.001 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-keep | two | keep | keep | 0.000 | We were describing how material moves through a crust, not software. I was discussing trans crust. | We were describing how material moves through a crust, not software. I was discussing trans crust. |
| habit-context-name | single | abstain | keep | 0.000 | I updated six. | I updated six. |
| habit-context-name | two | name | keep | 0.384 | I updated six. It is the cigarette-counting habit in my tracker. | I updated six. It is the cigarette-counting habit in my tracker. |
| habit-context-keep | single | abstain | keep | 0.000 | I updated six. | I updated six. |
| habit-context-keep | two | keep | keep | 0.129 | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. |
| lightning-context-name | single | abstain | keep | 0.008 | My son drew lightning. | My son drew lightning. |
| lightning-context-name | two | name | abstain | 0.318 | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew lightning. |
| lightning-context-keep | single | abstain | keep | 0.008 | My son drew lightning. | My son drew lightning. |
| lightning-context-keep | two | keep | abstain | 0.022 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| mater-context-name | single | abstain | abstain | 0.125 | My son drew Mader. | My son drew Mader. |
| mater-context-name | two | name | name | 0.380 | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. |
| mater-context-keep | single | abstain | abstain | 0.125 | My son drew Mader. | My son drew Mader. |
| mater-context-keep | two | keep | keep | 0.022 | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. |
| flo-context-name | single | abstain | keep | 0.078 | My son drew flow. | My son drew flow. |
| flo-context-name | two | name | name | 0.914 | She was serving the other Cars characters at the V8 Cafe. My son drew flow. | She was serving the other Cars characters at the V8 Cafe. My son drew Flo. |
| flo-context-keep | single | abstain | keep | 0.078 | My son drew flow. | My son drew flow. |
| flo-context-keep | two | keep | keep | 0.247 | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew flow. |
| dog-neighbor-name | single | keep | keep | 0.000 | I like to Gamble and play poker with cards. | I like to Gamble and play poker with cards. |
| dog-neighbor-name | two | keep | keep | 0.031 | I like to Gamble and play poker with cards. I love my dog Gambol. | I like to Gamble and play poker with cards. I love my dog Gambol. |
| son-neighbor-name | single | keep | keep | 0.203 | My colleague Rowen is presenting the report. | My colleague Rowen is presenting the report. |
| son-neighbor-name | two | keep | name | 0.733 | My son Rowan is at school. My colleague Rowen is presenting the report. | My son Rowan is at school. My colleague Rowan is presenting the report. |
| mother-neighbor-name | single | keep | keep | 0.006 | A Knell is the sound of a bell marking a death. | A Knell is the sound of a bell marking a death. |
| mother-neighbor-name | two | keep | keep | 0.013 | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Knell is the sound of a bell marking a death. Nell is picking up my son. |
| project-neighbor-name | single | keep | keep | 0.040 | The label uses trans crust to describe movement through a crust. | The label uses trans crust to describe movement through a crust. |
| project-neighbor-name | two | keep | name | 0.811 | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. |
| habit-neighbor-name | single | keep | keep | 0.440 | I smoked six cigarettes today. | I smoked six cigarettes today. |
| habit-neighbor-name | two | keep | name | 0.923 | I smoked six cigarettes today. I track them in my Cigs habit. | I smoked Cigs cigarettes today. I track them in my Cigs habit. |
| lightning-neighbor-name | single | keep | keep | 0.004 | The storm produced bright lightning. | The storm produced bright lightning. |
| lightning-neighbor-name | two | keep | name | 0.953 | My son was playing with his Lightning McQueen toy. The storm produced bright lightning. | My son was playing with his Lightning McQueen toy. The storm produced bright Lightning. |
| mater-neighbor-name | single | keep | abstain | 0.015 | Our neighbor Mr Mader repaired the fence. | Our neighbor Mr Mader repaired the fence. |
| mater-neighbor-name | two | keep | name | 0.890 | Our neighbor Mr Mader repaired the fence. My son was watching Mater in Cars. | Our neighbor Mr Mater repaired the fence. My son was watching Mater in Cars. |
| flo-neighbor-name | single | keep | keep | 0.237 | The river has a strong flow after the rain. | The river has a strong flow after the rain. |
| flo-neighbor-name | two | keep | name | 0.874 | My son was talking about Flo from Cars. The river has a strong flow after the rain. | My son was talking about Flo from Cars. The river has a strong Flo after the rain. |
| initial-before | single | keep | keep | 0.173 | My son O needs his lunchbox; use only his initial here. | My son O needs his lunchbox; use only his initial here. |
| initial-before | two | keep | name | 0.525 | His full name is Rowan. My son O needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. |
| initial-after | single | keep | keep | 0.093 | Please refer to my son as O in this message. | Please refer to my son as O in this message. |
| initial-after | two | keep | keep | 0.491 | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. |
| quoted-rowen | single | keep | keep | 0.272 | Copy the label exactly as written: Rowen. | Copy the label exactly as written: Rowen. |
| quoted-rowen | two | keep | name | 0.873 | It belongs to my son Rowan. Copy the label exactly as written: Rowen. | It belongs to my son Rowan. Copy the label exactly as written: Rowan. |
| quoted-flow | single | keep | keep | 0.322 | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: flow. |
| quoted-flow | two | keep | name | 0.816 | Keep the quoted word exactly as written: flow. My son was describing Flo from Cars. | Keep the quoted word exactly as written: Flo. My son was describing Flo from Cars. |
