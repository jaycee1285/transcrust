# d1 Q4 versus Q8 — text-only, 2026-10-08

Q4 retains comparable quality on this small suite, with mixed individual changes. It gains two main-suite repairs and avoids one ambiguous edit, while adding an ambiguous flow → Flo edit in Cars and losing one original-probe repair. Both privacy-initial controls remain unchanged in both prompt arms. The smaller footprint makes Q4 worth carrying forward as the local candidate; these untuned results do not establish safe automatic correction.

157 paired local decisions. Pinned source revisions, tokenizer, native prompt, temperature, complete input IDs and saved baseline hashes match. Only the quantized graph/weights differ. No API requests or media encoders.

| Arm | Quantization | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |
|---|---|---:|---:|---:|---:|
| sentence | Q8 | 5/12 | 0/7 | 0/0 | 12/19 |
| sentence | Q4 | 4/12 | 0/7 | 0/0 | 11/19 |
| suite-verdict | Q8 | 16/20 | 1/20 | 7/20 | 37/60 |
| suite-verdict | Q4 | 18/20 | 1/20 | 6/20 | 38/60 |
| cars-verdict | Q8 | 5/6 | 0/6 | 0/6 | 12/18 |
| cars-verdict | Q4 | 5/6 | 0/6 | 1/6 | 12/18 |
| suite-verdict-correction | Q8 | 12/20 | 10/20 | 16/20 | 22/60 |
| suite-verdict-correction | Q4 | 13/20 | 9/20 | 16/20 | 24/60 |

15/157 choices changed: 7 became exact, 5 lost exactness, 3 changed between incorrect choices. Mean absolute option probability difference 0.0673; maximum 0.4359.

KEEP and abstain both leave the sentence unchanged. Exact decisions distinguish them. Counts above apply no confidence floor. The editing-prompt arm repeats the main 60 sentences with a different question; these are not 157 independent utterances.

| Measurement | Q8 | Q4 |
|---|---:|---:|
| Text graph + weights (decimal MB) | 406.33 | 244.66 |
| Process RSS at end (MiB) | 522.34 | 332.47 |
| Process peak RSS (MiB) | 521.16 | 332.69 |
| Median inference (ms) | 1064.61 | 330.05 |
| Load (s) | 0.93 | 0.40 |

RAM includes Python, tokenizer and ONNX Runtime. Timings are separate laptop runs under varying load, not an isolated speed benchmark or a mobile measurement.

## Every changed choice

| Arm / case | Expected | Q8 → Q4 | P(name), Q8 → Q4 | Original | Q8 output | Q4 output |
|---|---|---|---:|---|---|---|
| sentence / record-terakeet-1 | name | name → keep | 0.514 → 0.286 | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, Terakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. | t I currently have today are CHB, incog clear, dinner six, Outfit and snack, backroom vacuum, bath and candies 8, BR reminder, GSC check, echo check in, transcrust work, trash and recycling out and to curb, homework, job supply, terrakeet 1P phone 30M, DMV 28 Munch Road beginning 8 30 a.m. |
| suite-verdict / son-name-1 | name | keep → name | 0.462 → 0.887 | My son Rowan forgot his lunchbox. | My son Rowan forgot his lunchbox. | My son Rowan forgot his lunchbox. |
| suite-verdict / son-abstain-4 | abstain | name → keep | 0.469 → 0.342 | The label says Rowan. | The label says Rowan. | The label says Rowan. |
| suite-verdict / project-name-4 | name | abstain → name | 0.146 → 0.531 | I use trans crust for push-to-talk voice input on Wayland. | I use trans crust for push-to-talk voice input on Wayland. | I use transcrust for push-to-talk voice input on Wayland. |
| suite-verdict / project-abstain-2 | abstain | abstain → keep | 0.003 → 0.023 | The note says trans crust. | The note says trans crust. | The note says trans crust. |
| cars-verdict / cars-flo-abstain-2 | abstain | keep → name | 0.239 → 0.445 | The note only says flow. | The note only says flow. | The note only says Flo. |
| suite-verdict-correction / dog-name-4 | name | keep → name | 0.384 → 0.458 | I filled my dog Gamble's water bowl. | I filled my dog Gamble's water bowl. | I filled my dog Gambol's water bowl. |
| suite-verdict-correction / dog-keep-1 | keep | keep → name | 0.250 → 0.439 | We should never gamble with the rent money. | We should never gamble with the rent money. | We should never Gambol with the rent money. |
| suite-verdict-correction / dog-keep-4 | keep | keep → name | 0.399 → 0.409 | My dog is asleep while I gamble online. | My dog is asleep while I gamble online. | My dog is asleep while I Gambol online. |
| suite-verdict-correction / son-keep-4 | keep | name → keep | 0.471 → 0.290 | The author spells her name Rowan, without an o at the end. | The author spells her name Rowan, without an o at the end. | The author spells her name Rowan, without an o at the end. |
| suite-verdict-correction / mother-name-1 | name | keep → name | 0.359 → 0.392 | My son's mother Knell will collect him from school. | My son's mother Knell will collect him from school. | My son's mother Nell will collect him from school. |
| suite-verdict-correction / mother-keep-3 | keep | name → keep | 0.518 → 0.285 | Knell means the sound of a bell marking a death. | Nell means the sound of a bell marking a death. | Knell means the sound of a bell marking a death. |
| suite-verdict-correction / project-name-1 | name | name → keep | 0.492 → 0.333 | Open the Linux dictation app trans crust. | Open the Linux dictation app transcrust. | Open the Linux dictation app trans crust. |
| suite-verdict-correction / project-name-4 | name | abstain → keep | 0.127 → 0.054 | I use trans crust for push-to-talk voice input on Wayland. | I use trans crust for push-to-talk voice input on Wayland. | I use trans crust for push-to-talk voice input on Wayland. |
| suite-verdict-correction / habit-keep-2 | keep | name → keep | 0.668 → 0.423 | I smoked six cigarettes today. | I smoked Cigs cigarettes today. | I smoked six cigarettes today. |

Re-run Q4 locally:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4
```

Regenerate this paired report from saved results:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/quantization-report.py
```
