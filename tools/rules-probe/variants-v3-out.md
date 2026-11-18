# variants v3 (public: ids and terms only)

v3 = v2 + a fixed vocative cue, in `variants_v3.py` (v1 and v2 untouched; rule 2 is wrapped, not edited).

## Rule (stated before running)
Rule 2's vocative cue (clause-initial span followed by a pause mark) now also requires that the span is not
list-signalled within its sentence: L1 `, and|or` follows the span; L2 `, <1-3 word item>(, <1-3 word item>)*, and|or` follows.
A preceding comma-separated item is not a signal. Known cost: a true vocative followed by a short clause and ", and" (L2 shape) passes instead of naming.

## Gate (stated before running)
v3 loses no correct v2 decision on any existing set and adds zero wrong-names. Result: PASS (0 lost, 0 added, 0 label changes on the
existing sets, old and real glossary header). On the new `fixtures-vocative.json` v3 intentionally loses one v2 "correct" (term Flo, id `list-middle-name`),
which v2 reached via the vocative cue for the wrong reason, and removes two wrong-names (`list-middle-keep`, `list-initial-keep`, term Flo).

## Aggregate (both arms)
| set | header | variant | n | settled | correct | wrong-name | wrong-keep | passed |
|---|---|---|--:|--:|--:|--:|--:|--:|
| read-aloud | old | v2 | 32 | 20 | 20 | 0 | 0 | 12 |
| read-aloud | old | v3 | 32 | 20 | 20 | 0 | 0 | 12 |
| read-aloud | real | v2 | 32 | 22 | 22 | 0 | 0 | 10 |
| read-aloud | real | v3 | 32 | 22 | 22 | 0 | 0 | 10 |
| phonon-trial | old | v2 | 90 | 58 | 58 | 0 | 0 | 32 |
| phonon-trial | old | v3 | 90 | 58 | 58 | 0 | 0 | 32 |
| phonon-trial | real | v2 | 90 | 64 | 64 | 0 | 0 | 26 |
| phonon-trial | real | v3 | 90 | 64 | 64 | 0 | 0 | 26 |
| orig132-household | old | v2 | 32 | 24 | 24 | 0 | 0 | 8 |
| orig132-household | old | v3 | 32 | 24 | 24 | 0 | 0 | 8 |
| orig132-household | real | v2 | 32 | 24 | 24 | 0 | 0 | 8 |
| orig132-household | real | v3 | 32 | 24 | 24 | 0 | 0 | 8 |
| orig132-synthetic | old | v2 | 100 | 34 | 24 | 10 | 0 | 66 |
| orig132-synthetic | old | v3 | 100 | 34 | 24 | 10 | 0 | 66 |
| orig132-synthetic | real | v2 | 100 | 34 | 24 | 10 | 0 | 66 |
| orig132-synthetic | real | v3 | 100 | 34 | 24 | 10 | 0 | 66 |
| fixtures-extra | old | v2 | 12 | 10 | 10 | 0 | 0 | 2 |
| fixtures-extra | old | v3 | 12 | 10 | 10 | 0 | 0 | 2 |
| fixtures-extra | real | v2 | 12 | 10 | 10 | 0 | 0 | 2 |
| fixtures-extra | real | v3 | 12 | 10 | 10 | 0 | 0 | 2 |
| fixtures-vocative | old | v2 | 6 | 5 | 3 | 2 | 0 | 1 |
| fixtures-vocative | old | v3 | 6 | 2 | 2 | 0 | 0 | 4 |
| fixtures-vocative | real | v2 | 6 | 5 | 3 | 2 | 0 | 1 |
| fixtures-vocative | real | v3 | 6 | 2 | 2 | 0 | 0 | 4 |

## Per-case differences v2 -> v3 (fixtures-vocative.json only; no other set changed)
| id | term | v2 | v3 | gold |
|---|---|---|---|---|
| list-middle-keep | Flo | name | pass | keep |
| list-initial-keep | Flo | name | pass | keep |
| list-middle-name | Flo | name | pass | name |

New-set labels: voc-sentence-start and voc-after-interjection named by v2 and v3 (gold name); list-end-keep pass in both (gold keep).
Caveat: no existing fixture contains a list-shaped vocative, so v3 is verified only on the 6 synthetic cases; real-audio list data (the smoke take) was not rerun.
