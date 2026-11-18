# d1 Q8 paired context decisions

Each scenario judges one marked occurrence, with the target sentence alone and then with one neighboring sentence. Expected answers are not model inputs.

Shared glossary SHA-256: d01d31c61937138b25fbb62d9fd66f24744488fe02a5f344420c02d375258ad3

| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |
|---|---|---|---|---:|---|---|
| dog-context-name | single | abstain | keep | 0.007 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-name | two | name | keep | 0.461 | I was thinking about gamble. She is my dog, and I was planning her next walk. | I was thinking about gamble. She is my dog, and I was planning her next walk. |
| dog-context-keep | single | abstain | keep | 0.007 | I was thinking about gamble. | I was thinking about gamble. |
| dog-context-keep | two | keep | keep | 0.008 | I was thinking about gamble. I meant betting money at the casino. | I was thinking about gamble. I meant betting money at the casino. |
| son-context-name | single | abstain | name | 0.567 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-name | two | name | keep | 0.259 | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. | He is my son, and I was reminding him about his lunchbox. I spoke to Rowen. |
| son-context-keep | single | abstain | name | 0.567 | I spoke to Rowen. | I spoke to Rowan. |
| son-context-keep | two | keep | keep | 0.134 | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. |
| mother-context-name | single | abstain | keep | 0.147 | I heard Knell. | I heard Knell. |
| mother-context-name | two | name | name | 0.843 | I heard Knell. She is my son Rowan's mother, and she was calling him home. | I heard Nell. She is my son Rowan's mother, and she was calling him home. |
| mother-context-keep | single | abstain | keep | 0.147 | I heard Knell. | I heard Knell. |
| mother-context-keep | two | keep | keep | 0.022 | I heard Knell. It was the sound of the funeral bell. | I heard Knell. It was the sound of the funeral bell. |
| project-context-name | single | abstain | keep | 0.030 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-name | two | name | name | 0.948 | It is the Linux push-to-talk dictation app I use. I was discussing trans crust. | It is the Linux push-to-talk dictation app I use. I was discussing transcrust. |
| project-context-keep | single | abstain | keep | 0.030 | I was discussing trans crust. | I was discussing trans crust. |
| project-context-keep | two | keep | keep | 0.004 | We were describing how material moves through a crust, not software. I was discussing trans crust. | We were describing how material moves through a crust, not software. I was discussing trans crust. |
| habit-context-name | single | abstain | keep | 0.001 | I updated six. | I updated six. |
| habit-context-name | two | name | name | 0.536 | I updated six. It is the cigarette-counting habit in my tracker. | I updated Cigs. It is the cigarette-counting habit in my tracker. |
| habit-context-keep | single | abstain | keep | 0.001 | I updated six. | I updated six. |
| habit-context-keep | two | keep | keep | 0.345 | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. | I updated six. It was the number of cigarettes I had smoked, and I changed the count to seven. |
| lightning-context-name | single | abstain | keep | 0.045 | My son drew lightning. | My son drew lightning. |
| lightning-context-name | two | name | name | 0.443 | He gave the red race car eyes and the number 95. My son drew lightning. | He gave the red race car eyes and the number 95. My son drew Lightning. |
| lightning-context-keep | single | abstain | keep | 0.045 | My son drew lightning. | My son drew lightning. |
| lightning-context-keep | two | keep | keep | 0.132 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. |
| mater-context-name | single | abstain | keep | 0.209 | My son drew Mader. | My son drew Mader. |
| mater-context-name | two | name | name | 0.449 | My son drew Mader. He added the tow hook and rusty body from Cars. | My son drew Mater. He added the tow hook and rusty body from Cars. |
| mater-context-keep | single | abstain | keep | 0.209 | My son drew Mader. | My son drew Mader. |
| mater-context-keep | two | keep | keep | 0.082 | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. | My son drew Mader. It was a portrait of our neighbor, whose surname is spelled Mader. |
| flo-context-name | single | abstain | name | 0.520 | My son drew flow. | My son drew Flo. |
| flo-context-name | two | name | name | 0.874 | She was serving the other Cars characters at the V8 Cafe. My son drew flow. | She was serving the other Cars characters at the V8 Cafe. My son drew Flo. |
| flo-context-keep | single | abstain | name | 0.520 | My son drew flow. | My son drew Flo. |
| flo-context-keep | two | keep | keep | 0.327 | He drew arrows showing how water moves down a river. My son drew flow. | He drew arrows showing how water moves down a river. My son drew flow. |
| dog-neighbor-name | single | keep | keep | 0.002 | I like to Gamble and play poker with cards. | I like to Gamble and play poker with cards. |
| dog-neighbor-name | two | keep | keep | 0.084 | I like to Gamble and play poker with cards. I love my dog Gambol. | I like to Gamble and play poker with cards. I love my dog Gambol. |
| son-neighbor-name | single | keep | name | 0.792 | My colleague Rowen is presenting the report. | My colleague Rowan is presenting the report. |
| son-neighbor-name | two | keep | name | 0.982 | My son Rowan is at school. My colleague Rowen is presenting the report. | My son Rowan is at school. My colleague Rowan is presenting the report. |
| mother-neighbor-name | single | keep | keep | 0.083 | A Knell is the sound of a bell marking a death. | A Knell is the sound of a bell marking a death. |
| mother-neighbor-name | two | keep | keep | 0.106 | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Knell is the sound of a bell marking a death. Nell is picking up my son. |
| project-neighbor-name | single | keep | abstain | 0.046 | The label uses trans crust to describe movement through a crust. | The label uses trans crust to describe movement through a crust. |
| project-neighbor-name | two | keep | name | 0.598 | I closed the transcrust dictation app. The label uses trans crust to describe movement through a crust. | I closed the transcrust dictation app. The label uses transcrust to describe movement through a crust. |
| habit-neighbor-name | single | keep | name | 0.676 | I smoked six cigarettes today. | I smoked Cigs cigarettes today. |
| habit-neighbor-name | two | keep | name | 0.981 | I smoked six cigarettes today. I track them in my Cigs habit. | I smoked Cigs cigarettes today. I track them in my Cigs habit. |
| lightning-neighbor-name | single | keep | keep | 0.278 | The storm produced bright lightning. | The storm produced bright lightning. |
| lightning-neighbor-name | two | keep | name | 0.925 | My son was playing with his Lightning McQueen toy. The storm produced bright lightning. | My son was playing with his Lightning McQueen toy. The storm produced bright Lightning. |
| mater-neighbor-name | single | keep | name | 0.593 | Our neighbor Mr Mader repaired the fence. | Our neighbor Mr Mater repaired the fence. |
| mater-neighbor-name | two | keep | name | 0.510 | Our neighbor Mr Mader repaired the fence. My son was watching Mater in Cars. | Our neighbor Mr Mater repaired the fence. My son was watching Mater in Cars. |
| flo-neighbor-name | single | keep | name | 0.798 | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| flo-neighbor-name | two | keep | name | 0.919 | My son was talking about Flo from Cars. The river has a strong flow after the rain. | My son was talking about Flo from Cars. The river has a strong Flo after the rain. |
| initial-before | single | keep | keep | 0.173 | My son O needs his lunchbox; use only his initial here. | My son O needs his lunchbox; use only his initial here. |
| initial-before | two | keep | name | 0.650 | His full name is Rowan. My son O needs his lunchbox; use only his initial here. | His full name is Rowan. My son Rowan needs his lunchbox; use only his initial here. |
| initial-after | single | keep | keep | 0.177 | Please refer to my son as O in this message. | Please refer to my son as O in this message. |
| initial-after | two | keep | name | 0.736 | Please refer to my son as O in this message. Rowan is his full name, which I keep private online. | Please refer to my son as Rowan in this message. Rowan is his full name, which I keep private online. |
| quoted-rowen | single | keep | keep | 0.249 | Copy the label exactly as written: Rowen. | Copy the label exactly as written: Rowen. |
| quoted-rowen | two | keep | name | 0.853 | It belongs to my son Rowan. Copy the label exactly as written: Rowen. | It belongs to my son Rowan. Copy the label exactly as written: Rowan. |
| quoted-flow | single | keep | name | 0.690 | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |
| quoted-flow | two | keep | name | 0.877 | Keep the quoted word exactly as written: flow. My son was describing Flo from Cars. | Keep the quoted word exactly as written: Flo. My son was describing Flo from Cars. |
| bluey-bingo-explicit-1 | single | name | abstain | 0.212 | My son wants a toy of bingo, the little sister from Bluey. | My son wants a toy of bingo, the little sister from Bluey. |
| bluey-bingo-explicit-1 | two | name | name | 0.747 | My son wants a toy of bingo, the little sister from Bluey. She is one of his favorite characters. | My son wants a toy of Bingo, the little sister from Bluey. She is one of his favorite characters. |
| bluey-bingo-explicit-2 | single | name | keep | 0.284 | We watched the episode where bin go plays with her older sister Bluey. | We watched the episode where bin go plays with her older sister Bluey. |
| bluey-bingo-explicit-2 | two | name | name | 0.659 | My son loves that character. We watched the episode where bin go plays with her older sister Bluey. | My son loves that character. We watched the episode where Bingo plays with her older sister Bluey. |
| bluey-bingo-context-name | single | abstain | keep | 0.058 | My son asked about bingo. | My son asked about bingo. |
| bluey-bingo-context-name | two | name | abstain | 0.182 | My son asked about bingo. He meant Bluey's four-year-old sister. | My son asked about bingo. He meant Bluey's four-year-old sister. |
| bluey-bingo-context-common | single | abstain | keep | 0.058 | My son asked about bingo. | My son asked about bingo. |
| bluey-bingo-context-common | two | keep | keep | 0.017 | My son asked about bingo. He wanted to know the rules of the number-calling game. | My son asked about bingo. He wanted to know the rules of the number-calling game. |
| bluey-bingo-common-1 | single | keep | keep | 0.035 | Grandma plays bingo at the community center. | Grandma plays bingo at the community center. |
| bluey-bingo-common-1 | two | keep | keep | 0.089 | My son was watching Bingo and Bluey on television. Grandma plays bingo at the community center. | My son was watching Bingo and Bluey on television. Grandma plays bingo at the community center. |
| bluey-bingo-common-2 | single | keep | keep | 0.041 | You win at bingo by marking the called numbers on your card. | You win at bingo by marking the called numbers on your card. |
| bluey-bingo-common-2 | two | keep | keep | 0.010 | You win at bingo by marking the called numbers on your card. We bought a Bingo toy for my son. | You win at bingo by marking the called numbers on your card. We bought a Bingo toy for my son. |
| bluey-chili-explicit-1 | single | name | keep | 0.013 | My son drew chili, Bluey and Bingo's mom. | My son drew chili, Bluey and Bingo's mom. |
| bluey-chili-explicit-1 | two | name | keep | 0.051 | My son drew chili, Bluey and Bingo's mom. She is one of his favorite characters. | My son drew chili, Bluey and Bingo's mom. She is one of his favorite characters. |
| bluey-chili-explicit-2 | single | name | abstain | 0.202 | In Bluey, chilly is the girls' mother. | In Bluey, chilly is the girls' mother. |
| bluey-chili-explicit-2 | two | name | abstain | 0.181 | My son loves that character. In Bluey, chilly is the girls' mother. | My son loves that character. In Bluey, chilly is the girls' mother. |
| bluey-chili-context-name | single | abstain | keep | 0.006 | My son asked about chili. | My son asked about chili. |
| bluey-chili-context-name | two | name | keep | 0.074 | He meant the mom of Bluey and Bingo. My son asked about chili. | He meant the mom of Bluey and Bingo. My son asked about chili. |
| bluey-chili-context-common | single | abstain | keep | 0.006 | My son asked about chili. | My son asked about chili. |
| bluey-chili-context-common | two | keep | keep | 0.030 | He wanted to know whether the dish contains hot peppers. My son asked about chili. | He wanted to know whether the dish contains hot peppers. My son asked about chili. |
| bluey-chili-common-1 | single | keep | keep | 0.002 | I cooked a pot of chili with beans for dinner. | I cooked a pot of chili with beans for dinner. |
| bluey-chili-common-1 | two | keep | name | 0.761 | My son was watching Chili and her daughters in Bluey. I cooked a pot of chili with beans for dinner. | My son was watching Chili and her daughters in Bluey. I cooked a pot of Chili with beans for dinner. |
| bluey-chili-common-2 | single | keep | abstain | 0.011 | It feels chilly outside this morning. | It feels chilly outside this morning. |
| bluey-chili-common-2 | two | keep | keep | 0.000 | It feels chilly outside this morning. My son asked for a toy of Chili from Bluey. | It feels chilly outside this morning. My son asked for a toy of Chili from Bluey. |
| bluey-bandit-explicit-1 | single | name | keep | 0.246 | In Bluey, bandit is the dad of the two girls. | In Bluey, bandit is the dad of the two girls. |
| bluey-bandit-explicit-1 | two | name | name | 0.746 | In Bluey, bandit is the dad of the two girls. He is one of his favorite characters. | In Bluey, Bandit is the dad of the two girls. He is one of his favorite characters. |
| bluey-bandit-explicit-2 | single | name | keep | 0.246 | My son wants a toy of band it, Bluey's father. | My son wants a toy of band it, Bluey's father. |
| bluey-bandit-explicit-2 | two | name | keep | 0.190 | My son loves that character. My son wants a toy of band it, Bluey's father. | My son loves that character. My son wants a toy of band it, Bluey's father. |
| bluey-bandit-context-name | single | abstain | keep | 0.004 | My son asked about bandit. | My son asked about bandit. |
| bluey-bandit-context-name | two | name | keep | 0.060 | My son asked about bandit. He meant the dad of Bluey and Bingo. | My son asked about bandit. He meant the dad of Bluey and Bingo. |
| bluey-bandit-context-common | single | abstain | keep | 0.004 | My son asked about bandit. | My son asked about bandit. |
| bluey-bandit-context-common | two | keep | keep | 0.001 | My son asked about bandit. He meant a robber who steals from travelers. | My son asked about bandit. He meant a robber who steals from travelers. |
| bluey-bandit-common-1 | single | keep | keep | 0.011 | The bandit stole the travelers' money. | The bandit stole the travelers' money. |
| bluey-bandit-common-1 | two | keep | keep | 0.015 | My son was talking about Bandit from Bluey. The bandit stole the travelers' money. | My son was talking about Bandit from Bluey. The bandit stole the travelers' money. |
| bluey-bandit-common-2 | single | keep | keep | 0.019 | The sheriff caught the bandit after the robbery. | The sheriff caught the bandit after the robbery. |
| bluey-bandit-common-2 | two | keep | keep | 0.109 | The sheriff caught the bandit after the robbery. We watched Bandit playing with his daughters in Bluey. | The sheriff caught the bandit after the robbery. We watched Bandit playing with his daughters in Bluey. |
| bluey-bluey-explicit-1 | single | name | name | 0.897 | My son wants a toy of bluey, the blue heeler from the cartoon. | My son wants a toy of Bluey, the blue heeler from the cartoon. |
| bluey-bluey-explicit-1 | two | name | name | 0.906 | My son wants a toy of bluey, the blue heeler from the cartoon. She is his favorite character. | My son wants a toy of Bluey, the blue heeler from the cartoon. She is his favorite character. |
| bluey-bluey-explicit-2 | single | name | name | 0.882 | In the cartoon, blue e is Bingo's older sister. | In the cartoon, Bluey is Bingo's older sister. |
| bluey-bluey-explicit-2 | two | name | name | 0.589 | My son likes her games. In the cartoon, blue e is Bingo's older sister. | My son likes her games. In the cartoon, Bluey is Bingo's older sister. |
| bluey-bluey-context-name | single | abstain | keep | 0.114 | My son asked about blue e. | My son asked about blue e. |
| bluey-bluey-context-name | two | name | name | 0.542 | My son asked about blue e. He meant the six-year-old blue heeler who is Bingo's older sister. | My son asked about Bluey. He meant the six-year-old blue heeler who is Bingo's older sister. |
| bluey-bluey-color-neighbor | single | keep | name | 0.496 | The sky looks blue today. | The sky looks Bluey today. |
| bluey-bluey-color-neighbor | two | keep | keep | 0.205 | My son was watching Bluey with his mom. The sky looks blue today. | My son was watching Bluey with his mom. The sky looks blue today. |
