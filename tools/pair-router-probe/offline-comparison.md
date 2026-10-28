# Offline correction candidates

Deployment comparison: local d1 Q8 versus local Needle 3 alongside NVIDIA ASR. Previously authorized API calls are quality references only, not a runtime option or fallback for the family application. This report makes no network requests.

| Surface | Candidate | Repairs | Edits on keep | Edits on ambiguous | Invalid |
|---|---|---:|---:|---:|---:|
| Original 19 | d1 Q8 | 5/12 | 0/7 | 0/0 | 0 |
| Original 19 | Needle 3 default | 1/12 | 0/7 | 0/0 | 0 |
| Original 19 | Needle 3 forced | 8/12 | 7/7 | 0/0 | 3 |
| Main 60 | d1 Q8 | 16/20 | 1/20 | 7/20 | 0 |
| Main 60 | Needle 3 default | 4/20 | 2/20 | 0/20 | 0 |
| Main 60 | Needle 3 forced | 17/20 | 17/20 | 20/20 | 2 |
| Cars 18 | d1 Q8 | 5/6 | 0/6 | 0/6 | 0 |
| Cars 18 | Needle 3 default | 1/6 | 0/6 | 0/6 | 0 |
| Cars 18 | Needle 3 forced | 3/6 | 5/6 | 6/6 | 3 |
| Correction question 60 | d1 Q8 | 12/20 | 10/20 | 16/20 | 0 |
| Correction question 60 | Needle 3 default | 4/20 | 2/20 | 0/20 | 0 |
| Correction question 60 | Needle 3 forced | 18/20 | 18/20 | 20/20 | 0 |

No confidence floor is applied. Invalid or multiple tool calls are counted separately and never interpreted as permission to edit. Native no-call results leave text intact and are recorded as runtime abstention in Needle receipts.

## Interpretation

These are untuned, supplied-candidate adapters. d1 uses its upstream masked-option formatter and shipped choice temperature; Needle uses one classification tool with a three-value enum. Default Needle gating and forced-call mode are shown separately. Forced-call mode still offers KEEP and abstain; it is not a forced edit. This is not a verdict on tuned Needle or on its best possible task representation.

Sentence scope, question and option meanings match the saved probe inputs. The original sentence splitter’s abbreviation behavior is inherited by both. d1 uses per_option=64 with verified zero truncation to retain all descriptions. Needle refuses context overflow; complete raw tool calls, suppressed calls and confidence scores are retained. The candidates’ confidence statistics are not interchangeable, so this report does not apply a shared numerical floor.

## Memory and time

d1 text bundle: 411.1 MB, including tokenizer and references. Only the text graph and its weight sidecar were downloaded. No vision/audio encoder is loaded.

d1 process peak RSS: 521 MiB; end RSS: 522 MiB. Load: 0.93s. Median inference: 1065ms. These include the Python/ORT process; there were competing Rust builds. They do not establish isolated speed or mobile memory/latency.

Needle model file: 35.3 MB. Maximum runtime-reported peak RAM: 78.7 MB. Median CLI round trip: 803ms. Needle CLI timing includes loading the model for every case. Its runtime RAM counter has different accounting from process RSS, so those figures are not equivalent memory measurements. Needle telemetry was explicitly disabled.

Detailed reports: [d1](d1-q8/report.md), [Needle 3](needle3/report.md). d1’s stock billing/technical control questions both selected the expected answer above 0.99; see `d1-q8/contract-smoke.json`.

Rerun this saved-result comparison:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/offline-report.py
```

No live dictation, vocabulary, or Apple review-before-paste behavior was changed.
