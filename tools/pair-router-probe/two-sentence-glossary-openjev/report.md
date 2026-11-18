# OpenJev/Verdict 151M fp16 paired context decisions

Each scenario judges one marked occurrence, with the target sentence alone and then with one neighboring sentence. Expected answers are not model inputs.

Shared glossary SHA-256: 6ff5042350f88e87a48c8b9af6400ae1c260f1557b652e97f5f5de5c78428407

| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |
|---|---|---|---|---:|---|---|
| dog-context-name | single | abstain | name | 0.542 | I was thinking about gamble. | I was thinking about Gambol. |
| dog-context-name | two | name | name | 0.659 | I was thinking about gamble. She is my dog, and I was planning her next walk. | I was thinking about Gambol. She is my dog, and I was planning her next walk. |
| dog-context-keep | single | abstain | name | 0.542 | I was thinking about gamble. | I was thinking about Gambol. |
| dog-context-keep | two | keep | name | 0.494 | I was thinking about gamble. I meant betting money at the casino. | I was thinking about Gambol. I meant betting money at the casino. |
| son-context-name | single | abstain | name | 0.659 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-name | two | name | name | 0.698 | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowan. |
| son-context-keep | single | abstain | name | 0.659 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-keep | two | keep | name | 0.571 | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. | She is my colleague, and her name really is spelled Rowen. I spoke to Rowan. |
| mother-context-name | single | abstain | name | 0.450 | I heard Knell. | I heard Nell. |
| mother-context-name | two | name | name | 0.671 | I heard Knell. She is my son Rowan's mother, and she was calling him home. | I heard Nell. She is my son Rowan's mother, and she was calling him home. |
| mother-context-keep | single | abstain | name | 0.450 | I heard Knell. | I heard Nell. |
| mother-context-keep | two | keep | keep | 0.318 | I heard Knell. It was the sound of the funeral bell. | I heard Knell. It was the sound of the funeral bell. |
| project-context-name | single | abstain | name | 0.501 | I was discussing trans crust. | I was discussing transcrust. |
| project-context-name | two | name | name | 0.687 | It is the Linux push-to-talk dictation app I use. I was discussing trans crust. | It is the Linux push-to-talk dictation app I use. I was discussing transcrust. |
| project-context-keep | single | abstain | name | 0.501 | I was discussing trans crust. | I was discussing transcrust. |
| project-context-keep | two | keep | keep | 0.379 | We were describing how material moves through a crust, not software. I was discussing trans crust. | We were describing how material moves through a crust, not software. I was discussing trans crust. |
| habit-context-name | single | abstain | keep | 0.090 | I updated six. | I updated six. |
| habit-context-name | two | name | name | 0.538 | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| habit-context-keep | single | abstain | keep | 0.090 | I updated six. | I updated six. |
| habit-context-keep | two | keep | name | 0.497 | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. | I updated Cigs. It was the number of cigarettes I had smoked, and I changed the count to seven. |
| lightning-context-name | single | abstain | name | 0.717 | My son drew lightning. | My son drew Lightning. |
| lightning-context-name | two | name | name | 0.746 | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| lightning-context-keep | single | abstain | name | 0.717 | My son drew lightning. | My son drew Lightning. |
| lightning-context-keep | two | keep | name | 0.613 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew Lightning. |
| mater-context-name | single | abstain | keep | 0.294 | My son drew Mader. | My son drew Mader. |
| mater-context-name | two | name | name | 0.480 | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. |
| mater-context-keep | single | abstain | keep | 0.294 | My son drew Mader. | My son drew Mader. |
| mater-context-keep | two | keep | keep | 0.262 | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. |
| flo-context-name | single | abstain | name | 0.406 | My son drew flow. | My son drew Flo. |
| flo-context-name | two | name | name | 0.625 | She was serving the other Cars characters at the V8 Cafe. My son drew flow. | She was serving the other Cars characters at the V8 Cafe. My son drew Flo. |
| flo-context-keep | single | abstain | name | 0.406 | My son drew flow. | My son drew Flo. |
| flo-context-keep | two | keep | keep | 0.339 | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew flow. |
| dog-neighbor-name | single | keep | name | 0.573 | I like to Gamble and play poker with cards. | I like to Gambol and play poker with cards. |
| dog-neighbor-name | two | keep | name | 0.678 | I like to Gamble and play poker with cards. I love my dog Gambol. | I like to Gambol and play poker with cards. I love my dog Gambol. |
| son-neighbor-name | single | keep | name | 0.638 | My colleague Rowen is presenting the report. | My colleague Rowan is presenting the report. |
| son-neighbor-name | two | keep | name | 0.725 | My son Rowan is at school. My colleague Rowen is presenting the report. | My son Rowan is at school. My colleague Rowan is presenting the report. |
| mother-neighbor-name | single | keep | keep | 0.298 | A Knell is the sound of a bell marking a death. | A Knell is the sound of a bell marking a death. |
| mother-neighbor-name | two | keep | name | 0.469 | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Nell is the sound of a bell marking a death. Nell is picking up my son. |
| project-neighbor-name | single | keep | name | 0.441 | The label uses trans crust to describe movement through a crust. | The label uses transcrust to describe movement through a crust. |
| project-neighbor-name | two | keep | name | 0.578 | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. |
| habit-neighbor-name | single | keep | name | 0.459 | I smoked six cigarettes today. | I smoked Cigs cigarettes today. |
| habit-neighbor-name | two | keep | name | 0.568 | I smoked six cigarettes today. I track them in my Cigs habit. | I smoked Cigs cigarettes today. I track them in my Cigs habit. |
| lightning-neighbor-name | single | keep | name | 0.680 | The storm produced bright lightning. | The storm produced bright Lightning. |
| lightning-neighbor-name | two | keep | name | 0.695 | My son was playing with his Lightning McQueen toy. The storm produced bright lightning. | My son was playing with his Lightning McQueen toy. The storm produced bright Lightning. |
| mater-neighbor-name | single | keep | keep | 0.301 | Our neighbor Mr Mader repaired the fence. | Our neighbor Mr Mader repaired the fence. |
| mater-neighbor-name | two | keep | name | 0.430 | Our neighbor Mr Mader repaired the fence. My son was watching Mater in Cars. | Our neighbor Mr Mater repaired the fence. My son was watching Mater in Cars. |
| flo-neighbor-name | single | keep | name | 0.357 | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| flo-neighbor-name | two | keep | name | 0.477 | My son was talking about Flo from Cars. The river has a strong flow after the rain. | My son was talking about Flo from Cars. The river has a strong Flo after the rain. |
| initial-before | single | keep | name | 0.630 | My son O needs his lunchbox; use only his initial here. | My son Rowan needs his lunchbox; use only his initial here. |
| initial-before | two | keep | name | 0.655 | His full name is Rowan. My son O needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. |
| initial-after | single | keep | name | 0.689 | Please refer to my son as O in this message. | Please refer to my son as Rowan in this message. |
| initial-after | two | keep | name | 0.672 | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| quoted-rowen | single | keep | name | 0.671 | Copy the label exactly as written: Rowen. | Copy the label exactly as written: Rowan. |
| quoted-rowen | two | keep | name | 0.751 | It belongs to my son Rowan. Copy the label exactly as written: Rowen. | It belongs to my son Rowan. Copy the label exactly as written: Rowan. |
| quoted-flow | single | keep | name | 0.431 | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |
| quoted-flow | two | keep | name | 0.550 | Keep the quoted word exactly as written: flow. My son was describing Flo from Cars. | Keep the quoted word exactly as written: Flo. My son was describing Flo from Cars. |
