# OpenJev/Verdict 151M fp16 paired context decisions

Each scenario judges one marked occurrence, with the target sentence alone and then with one neighboring sentence. Expected answers are not model inputs.

No shared glossary.

| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |
|---|---|---|---|---:|---|---|
| dog-context-name | single | abstain | keep | 0.171 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-name | two | name | name | 0.514 | I was thinking about gamble. She is my dog, and I was planning her next walk. | I was thinking about Gambol. She is my dog, and I was planning her next walk. |
| dog-context-keep | single | abstain | keep | 0.171 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-keep | two | keep | keep | 0.123 | I was thinking about gamble. I meant betting money at the casino. | I was thinking about gamble. I meant betting money at the casino. |
| son-context-name | single | abstain | name | 0.510 | I spoke to Rowan. | I spoke to Rowan. |
| son-context-name | two | name | name | 0.597 | He is my son, and I was reminding him about his lunchbox. I spoke to Rowan. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowan. |
| son-context-keep | single | abstain | name | 0.510 | I spoke to Rowan. | I spoke to Rowan. |
| son-context-keep | two | keep | keep | 0.312 | She is my colleague, and her name really is spelled Rowan. I spoke to Rowan. | She is my colleague, and her name really is spelled Rowan. I spoke to Rowan. |
| mother-context-name | single | abstain | keep | 0.210 | I heard Knell. | I heard Knell. |
| mother-context-name | two | name | name | 0.682 | I heard Knell. She is my son Rowan's mother, and she was calling him home. | I heard Nell. She is my son Rowan's mother, and she was calling him home. |
| mother-context-keep | single | abstain | keep | 0.210 | I heard Knell. | I heard Knell. |
| mother-context-keep | two | keep | keep | 0.124 | I heard Knell. It was the sound of the funeral bell. | I heard Knell. It was the sound of the funeral bell. |
| project-context-name | single | abstain | name | 0.350 | I was discussing trans crust. | I was discussing transcrust. |
| project-context-name | two | name | name | 0.698 | It is the Linux push-to-talk dictation app I use. I was discussing trans crust. | It is the Linux push-to-talk dictation app I use. I was discussing transcrust. |
| project-context-keep | single | abstain | name | 0.350 | I was discussing trans crust. | I was discussing transcrust. |
| project-context-keep | two | keep | keep | 0.265 | We were describing how material moves through a crust, not software. I was discussing trans crust. | We were describing how material moves through a crust, not software. I was discussing trans crust. |
| habit-context-name | single | abstain | keep | 0.113 | I updated six. | I updated six. |
| habit-context-name | two | name | name | 0.598 | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| habit-context-keep | single | abstain | keep | 0.113 | I updated six. | I updated six. |
| habit-context-keep | two | keep | name | 0.499 | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. | I updated Cigs. It was the number of cigarettes I had smoked, and I changed the count to seven. |
| lightning-context-name | single | abstain | name | 0.410 | My son drew lightning. | My son drew Lightning. |
| lightning-context-name | two | name | name | 0.594 | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| lightning-context-keep | single | abstain | name | 0.410 | My son drew lightning. | My son drew Lightning. |
| lightning-context-keep | two | keep | keep | 0.231 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| mater-context-name | single | abstain | keep | 0.196 | My son drew Mader. | My son drew Mader. |
| mater-context-name | two | name | name | 0.369 | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. |
| mater-context-keep | single | abstain | keep | 0.196 | My son drew Mader. | My son drew Mader. |
| mater-context-keep | two | keep | keep | 0.163 | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. |
| flo-context-name | single | abstain | keep | 0.259 | My son drew flow. | My son drew flow. |
| flo-context-name | two | name | name | 0.479 | She was serving the other Cars characters at the V8 Cafe. My son drew flow. | She was serving the other Cars characters at the V8 Cafe. My son drew Flo. |
| flo-context-keep | single | abstain | keep | 0.259 | My son drew flow. | My son drew flow. |
| flo-context-keep | two | keep | keep | 0.187 | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew flow. |
| dog-neighbor-name | single | keep | keep | 0.179 | I like to Gamble and play poker with cards. | I like to Gamble and play poker with cards. |
| dog-neighbor-name | two | keep | name | 0.628 | I like to Gamble and play poker with cards. I love my dog Gambol. | I like to Gambol and play poker with cards. I love my dog Gambol. |
| son-neighbor-name | single | keep | name | 0.477 | My colleague Rowan is presenting the report. | My colleague Rowan is presenting the report. |
| son-neighbor-name | two | keep | name | 0.702 | My son Rowan is at school. My colleague Rowan is presenting the report. | My son Rowan is at school. My colleague Rowan is presenting the report. |
| mother-neighbor-name | single | keep | keep | 0.112 | A Knell is the sound of a bell marking a death. | A Knell is the sound of a bell marking a death. |
| mother-neighbor-name | two | keep | keep | 0.383 | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Knell is the sound of a bell marking a death. Nell is picking up my son. |
| project-neighbor-name | single | keep | keep | 0.296 | The label uses trans crust to describe movement through a crust. | The label uses trans crust to describe movement through a crust. |
| project-neighbor-name | two | keep | name | 0.605 | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. |
| habit-neighbor-name | single | keep | name | 0.437 | I smoked six cigarettes today. | I smoked Cigs cigarettes today. |
| habit-neighbor-name | two | keep | name | 0.576 | I smoked six cigarettes today. I track them in my Cigs habit. | I smoked Cigs cigarettes today. I track them in my Cigs habit. |
| lightning-neighbor-name | single | keep | abstain | 0.261 | The storm produced bright lightning. | The storm produced bright lightning. |
| lightning-neighbor-name | two | keep | name | 0.558 | My son was playing with his Lightning McQueen toy. The storm produced bright lightning. | My son was playing with his Lightning McQueen toy. The storm produced bright Lightning. |
| mater-neighbor-name | single | keep | keep | 0.176 | Our neighbor Mr Mader repaired the fence. | Our neighbor Mr Mader repaired the fence. |
| mater-neighbor-name | two | keep | name | 0.425 | Our neighbor Mr Mader repaired the fence. My son was watching Mater in Cars. | Our neighbor Mr Mater repaired the fence. My son was watching Mater in Cars. |
| flo-neighbor-name | single | keep | abstain | 0.159 | The river has a strong flow after the rain. | The river has a strong flow after the rain. |
| flo-neighbor-name | two | keep | name | 0.449 | My son was talking about Flo from Cars. The river has a strong flow after the rain. | My son was talking about Flo from Cars. The river has a strong Flo after the rain. |
| initial-before | single | keep | name | 0.534 | My son O needs his lunchbox; use only his initial here. | My son Rowan needs his lunchbox; use only his initial here. |
| initial-before | two | keep | name | 0.664 | His full name is Rowan. My son O needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. |
| initial-after | single | keep | name | 0.598 | Please refer to my son as O in this message. | Please refer to my son as Rowan in this message. |
| initial-after | two | keep | name | 0.671 | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| quoted-orian | single | keep | name | 0.436 | Copy the label exactly as written: Rowan. | Copy the label exactly as written: Rowan. |
| quoted-orian | two | keep | name | 0.755 | It belongs to my son Rowan. Copy the label exactly as written: Rowan. | It belongs to my son Rowan. Copy the label exactly as written: Rowan. |
| quoted-flow | single | keep | keep | 0.200 | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: flow. |
| quoted-flow | two | keep | name | 0.528 | Keep the quoted word exactly as written: flow. My son was describing Flo from Cars. | Keep the quoted word exactly as written: Flo. My son was describing Flo from Cars. |
