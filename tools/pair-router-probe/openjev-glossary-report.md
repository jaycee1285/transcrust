# OpenJev versus d1 — shared glossary context

112 new local OpenJev/Verdict 151M fp16 inferences: 56 without glossary and 56 with the exact five-entry glossary. Same states, marked occurrences, questions, option meanings and pre-existing gold labels as d1. OpenJev uses its native LABEL/SEP format and shipped three-choice temperature 5.0069; d1 uses its own native mask format/calibration. No API calls, audio replay, training or live changes.

Results count raw name selections, mapped to proposed replacements by the harness. They do not demonstrate an edit policy. Initial and quoted-spelling stress cases are shown separately; their expected edit behavior is not the same as referent identification. These are small, correlated diagnostic cases.

## Two-sentence comparison excluding initial and quoted-spelling stress cases

| Model | Glossary | Repairs | Name selections on ordinary/other-person controls |
|---|---|---:|---:|
| OpenJev 151M fp16 | no | 8/8 | 8/16 |
| OpenJev 151M fp16 | yes | 8/8 | 12/16 |
| d1 Q4 | no | 4/8 | 6/16 |
| d1 Q4 | yes | 6/8 | 7/16 |
| d1 Q8 | no | 4/8 | 6/16 |
| d1 Q8 | yes | 7/8 | 6/16 |

The remaining controls include deliberately difficult ambiguity and different-person cases; their error count is not an everyday dictation error rate.

## Full diagnostic counts

| Model | Scope | Glossary | Repairs | Name selections on non-repairs | Exact choices |
|---|---|---|---:|---:|---:|
| OpenJev 151M fp16 | single | no | 0/0 | 11/28 | 5/28 |
| OpenJev 151M fp16 | single | yes | 0/0 | 22/28 | 2/28 |
| OpenJev 151M fp16 | two | no | 8/8 | 12/20 | 16/28 |
| OpenJev 151M fp16 | two | yes | 8/8 | 16/20 | 12/28 |
| d1 Q4 | single | no | 0/0 | 0/28 | 13/28 |
| d1 Q4 | single | yes | 0/0 | 6/28 | 9/28 |
| d1 Q4 | two | no | 4/8 | 9/20 | 14/28 |
| d1 Q4 | two | yes | 6/8 | 10/20 | 16/28 |
| d1 Q8 | single | no | 0/0 | 0/28 | 13/28 |
| d1 Q8 | single | yes | 0/0 | 8/28 | 7/28 |
| d1 Q8 | two | no | 4/8 | 9/20 | 14/28 |
| d1 Q8 | two | yes | 7/8 | 9/20 | 17/28 |

Single-sentence cases require no repairs: ambiguous targets have abstention gold. KEEP and abstention both preserve text. No confidence floor.

## Two-sentence results by category

| Model | Category | Without glossary: names selected | With glossary: names selected |
|---|---|---:|---:|
| OpenJev 151M fp16 | disambiguate-name | 8/8 | 8/8 |
| OpenJev 151M fp16 | disambiguate-keep | 1/8 | 4/8 |
| OpenJev 151M fp16 | name-distractor | 7/8 | 8/8 |
| OpenJev 151M fp16 | privacy-initial | 2/2 | 2/2 |
| OpenJev 151M fp16 | intentional-spelling | 2/2 | 2/2 |
| d1 Q4 | disambiguate-name | 4/8 | 6/8 |
| d1 Q4 | disambiguate-keep | 0/8 | 1/8 |
| d1 Q4 | name-distractor | 6/8 | 6/8 |
| d1 Q4 | privacy-initial | 1/2 | 1/2 |
| d1 Q4 | intentional-spelling | 2/2 | 2/2 |
| d1 Q8 | disambiguate-name | 4/8 | 7/8 |
| d1 Q8 | disambiguate-keep | 0/8 | 1/8 |
| d1 Q8 | name-distractor | 6/8 | 5/8 |
| d1 Q8 | privacy-initial | 1/2 | 1/2 |
| d1 Q8 | intentional-spelling | 2/2 | 2/2 |

## Resources

| Model | Glossary | Two-sentence median inference (ms) | End process RSS (MiB) | Input tokens |
|---|---|---:|---:|---|
| OpenJev 151M fp16 | no | 94 | 649 | 50–93 |
| OpenJev 151M fp16 | yes | 169 | 653 | 114–157 |
| d1 Q4 | no | 235 | 318 | 56–97 |
| d1 Q4 | yes | 442 | 327 | 111–152 |
| d1 Q8 | no | 572 | 516 | 56–97 |
| d1 Q8 | yes | 834 | 517 | 111–152 |

OpenJev graph: 303.8 MB; load 1.37s; process peak RSS 849 MiB. Both OpenJev arms share one loaded session, run without glossary first, with one warm-up per arm. d1 timings are from earlier runs under different laptop load. These are not isolated speed comparisons or mobile measurements. RAM includes Python/ORT and allocator caches. Inputs are rejected above the existing 512-token export contract, with zero truncation.

## Every OpenJev choice changed by the glossary

| Scope / scenario | Expected | Without → with | P(name) without → with | Dictation | Output without | Output with |
|---|---|---|---:|---|---|---|
| single / dog-context-name | abstain | keep → name | 0.171 → 0.542 | I was thinking about «gamble». | I was thinking about gamble. | I was thinking about Gambol. |
| single / dog-context-keep | abstain | keep → name | 0.171 → 0.542 | I was thinking about «gamble». | I was thinking about gamble. | I was thinking about Gambol. |
| two / dog-context-keep | keep | keep → name | 0.123 → 0.494 | I was thinking about «gamble». I meant betting money at the casino. | I was thinking about gamble. I meant betting money at the casino. | I was thinking about Gambol. I meant betting money at the casino. |
| two / son-context-keep | keep | keep → name | 0.312 → 0.571 | She is my colleague, and her name really is spelled Rowen. I spoke to «Rowen». | She is my colleague, and her name really is spelled Rowen. I spoke to Rowen. | She is my colleague, and her name really is spelled Rowen. I spoke to Rowan. |
| single / mother-context-name | abstain | keep → name | 0.210 → 0.450 | I heard «Knell». | I heard Knell. | I heard Nell. |
| single / mother-context-keep | abstain | keep → name | 0.210 → 0.450 | I heard «Knell». | I heard Knell. | I heard Nell. |
| two / lightning-context-keep | keep | keep → name | 0.231 → 0.613 | He drew a jagged electrical bolt coming out of a storm cloud. My son drew «lightning». | He drew a jagged electrical bolt coming out of a storm cloud. My son drew lightning. | He drew a jagged electrical bolt coming out of a storm cloud. My son drew Lightning. |
| single / flo-context-name | abstain | keep → name | 0.259 → 0.406 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| single / flo-context-keep | abstain | keep → name | 0.259 → 0.406 | My son drew «flow». | My son drew flow. | My son drew Flo. |
| single / dog-neighbor-name | keep | keep → name | 0.179 → 0.573 | I like to «Gamble» and play poker with cards. | I like to Gamble and play poker with cards. | I like to Gambol and play poker with cards. |
| two / mother-neighbor-name | keep | keep → name | 0.383 → 0.469 | A «Knell» is the sound of a bell marking a death. Nell is picking up my son. | A Knell is the sound of a bell marking a death. Nell is picking up my son. | A Nell is the sound of a bell marking a death. Nell is picking up my son. |
| single / project-neighbor-name | keep | keep → name | 0.296 → 0.441 | The label uses «trans crust» to describe movement through a crust. | The label uses trans crust to describe movement through a crust. | The label uses transcrust to describe movement through a crust. |
| single / lightning-neighbor-name | keep | abstain → name | 0.261 → 0.680 | The storm produced bright «lightning». | The storm produced bright lightning. | The storm produced bright Lightning. |
| single / flo-neighbor-name | keep | abstain → name | 0.159 → 0.357 | The river has a strong «flow» after the rain. | The river has a strong flow after the rain. | The river has a strong Flo after the rain. |
| single / quoted-flow | keep | keep → name | 0.200 → 0.431 | Keep the quoted word exactly as written: «flow». | Keep the quoted word exactly as written: flow. | Keep the quoted word exactly as written: Flo. |

Full OpenJev inputs and outputs: [without glossary](two-sentence-openjev/report.md), [with glossary](two-sentence-glossary-openjev/report.md).

Re-run both OpenJev arms:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/openjev-context.py
```

Read/regenerate this saved report without model loading:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/openjev-context.py --report-only
```
