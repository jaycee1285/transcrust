# Sentence-and-pair decision probe

Model: local Verdict/OpenJev ModernBERT 151M, fp16 ONNX, CPU. Per-3-choice temperature: 5.0069.

Context scope: marked sentence.

Decision task: **referent**. Referent identifies meaning; correction asks whether to replace text, preserving intentional spellings.

Uses Verdict’s published label/SEP prompt contract. Each call has KEEP, the supplied canonical name, and insufficient evidence. Model probabilities are not validated calibration for this correction domain.

Cases: `cars.json`. Fixture SHA-256: `288a5a5445cc6d8642dee8b5411b46f159bbf7497214355903b455f33912bfde`. Synthetic sentences measure supplied-candidate decisions, not audio recognition or end-to-end WER. Record-2 rows, when included, are selected repairs from the unboosted delivered transcript, including its phonetic/casing post-pass.

| Source | Confidence floor | Repairs made / wanted | Wrong edits on keep cases | Edits on ambiguous cases |
|---|---:|---:|---:|---:|
| synthetic-cars | 0.0 | 6/6 | 1/6 | 1/6 |
| synthetic-cars | 0.5 | 5/6 | 1/6 | 0/6 |
| synthetic-cars | 0.7 | 2/6 | 0/6 | 0/6 |
| synthetic-cars | 0.9 | 0/6 | 0/6 | 0/6 |

## Decision accuracy by category

| Category | Correct choice | Name edits on non-repair cases |
|---|---:|---:|
| flo/abstain | 0/2 | 0/2 |
| flo/keep | 1/2 | 0/2 |
| flo/name | 2/2 | 0/0 |
| lightning/abstain | 1/2 | 1/2 |
| lightning/keep | 1/2 | 1/2 |
| lightning/name | 2/2 | 0/0 |
| mater/abstain | 0/2 | 0/2 |
| mater/keep | 2/2 | 0/2 |
| mater/name | 2/2 | 0/0 |

Model load: 1.27s. Median inference: 104ms.

| Case | Heard → candidate | Want | Chose | P(keep) | P(name) | P(abstain) |
|---|---|---|---|---:|---:|---:|
| cars-lightning-name-1 | lightning → Lightning | name | name | 0.055 | 0.749 | 0.196 |
| cars-lightning-name-2 | lightning → Lightning | name | name | 0.065 | 0.733 | 0.202 |
| cars-lightning-keep-1 | lightning → Lightning | keep | name | 0.147 | 0.529 | 0.323 |
| cars-lightning-keep-2 | Lightning → Lightning | keep | keep | 0.405 | 0.233 | 0.362 |
| cars-lightning-abstain-1 | lightning → Lightning | abstain | name | 0.219 | 0.404 | 0.377 |
| cars-lightning-abstain-2 | lightning → Lightning | abstain | abstain | 0.213 | 0.384 | 0.403 |
| cars-mater-name-1 | Mader → Mater | name | name | 0.145 | 0.675 | 0.180 |
| cars-mater-name-2 | mater → Mater | name | name | 0.247 | 0.520 | 0.234 |
| cars-mater-keep-1 | Mader → Mater | keep | keep | 0.602 | 0.190 | 0.209 |
| cars-mater-keep-2 | mater → Mater | keep | keep | 0.392 | 0.262 | 0.346 |
| cars-mater-abstain-1 | Mader → Mater | abstain | keep | 0.534 | 0.191 | 0.275 |
| cars-mater-abstain-2 | mater → Mater | abstain | keep | 0.490 | 0.227 | 0.282 |
| cars-flo-name-1 | flow → Flo | name | name | 0.133 | 0.682 | 0.185 |
| cars-flo-name-2 | flow → Flo | name | name | 0.207 | 0.485 | 0.308 |
| cars-flo-keep-1 | flow → Flo | keep | abstain | 0.367 | 0.157 | 0.476 |
| cars-flo-keep-2 | flow → Flo | keep | keep | 0.419 | 0.171 | 0.409 |
| cars-flo-abstain-1 | flow → Flo | abstain | keep | 0.410 | 0.243 | 0.347 |
| cars-flo-abstain-2 | flow → Flo | abstain | keep | 0.412 | 0.213 | 0.375 |

## Contexts

**cars-lightning-name-1** (synthetic-cars): My son wants the red race car «lightning» McQueen for his birthday.

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-lightning-name-2** (synthetic-cars): In Cars, «lightning» McQueen learns to value his friends.

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-lightning-keep-1** (synthetic-cars): We stopped watching Cars because «lightning» struck near the house.

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-lightning-keep-2** (synthetic-cars): «Lightning» flashed across the sky during the storm.

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-lightning-abstain-1** (synthetic-cars): Look at «lightning».

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.

**cars-lightning-abstain-2** (synthetic-cars): The note says «lightning».

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.

**cars-mater-name-1** (synthetic-cars): In Cars, the rusty tow truck «Mader» is Lightning McQueen's friend.

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-mater-name-2** (synthetic-cars): My son wants a toy of the Cars tow truck named «mater».

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-mater-keep-1** (synthetic-cars): «Mader» has that surname; he is not the Cars character.

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-mater-keep-2** (synthetic-cars): This connector assembly robot is a «mater»: it couples the two parts together.

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-mater-abstain-1** (synthetic-cars): «Mader» is outside.

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.

**cars-mater-abstain-2** (synthetic-cars): I heard someone say «mater».

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.

**cars-flo-name-1** (synthetic-cars): In Cars, «flow» runs the V8 Cafe in Radiator Springs.

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-flo-name-2** (synthetic-cars): My son wants a toy of «flow», the Cars character who runs the diner.

Expected **name**: The sentence identifies the Cars character; restore the canonical spelling or capitalization.

**cars-flo-keep-1** (synthetic-cars): The water «flow» slowed when I closed the tap.

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-flo-keep-2** (synthetic-cars): After watching Cars, we talked about traffic «flow».

Expected **keep**: The sentence establishes a common-word meaning or a different person.

**cars-flo-abstain-1** (synthetic-cars): Look at «flow».

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.

**cars-flo-abstain-2** (synthetic-cars): The note only says «flow».

Expected **abstain**: The sentence does not identify a character or establish a common-word meaning.
