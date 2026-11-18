# Native d3-lite Q8 — 2026-10-31

**The native implementation passed its upstream agreement gate, but this
prompt/model combination failed ordinary-word preservation on the household
test. Keep the accepted D1 setup.** The broader 100-row synthetic correction
surface was not run after this real-input preservation failure; the runner
supports it for a future justified experiment.

```sh
nix develop -c bash tools/d3-probe/run.sh
```

[Implementation and exact methodology](../../d3-probe/README.md). Optional
Rust/Candle CPU research binary, using the pinned decision.js text-only packed
Q8 bundle. No interpreter, browser, API inference, live backend switch, shared
glossary changes, audio replay or mobile integration.

## Reference validation

- First native formatter: **147 tokens, exact match** to the public fixture.
- **56/56 supported public top answers match** the shipped Q8 reference outputs.
- Largest absolute probability drift: **0.00114151** (0.114 percentage points).
  The predeclared gate required all 56 tops and drift ≤ 0.02; this is close
  numerical agreement, not bit-exact equality.
- Those public cases exercise choice, yes/no and ordinal score readouts. Seven
  unsupported request combinations were skipped exactly as in the reference.
- The completed weights' SHA-256 matches the published LFS object; tokenizer
  and manifest are pinned too. Source signatures bind correction runs to the
  code that passed the agreement gate.
- The first dog occurrence has identical probabilities standalone and in its
  packed batch. The later ordinary-water-flow occurrence was also isolated in
  four checks (both engines × both scopes): **zero probability drift**, and
  the same wrong name choice in every case. This checks the surprising result
  rather than attributing it to untested ragged-batch behavior.

Unit tests cover offset-u8 → signed Q8 packing/scales and recurrent state
update/decay. Initial checksum/packing failures were fixed before private text
evaluation; their details are retained in the implementation notes. No poor
output from those failed builds is counted as model quality evidence.

## Actual phone corrections

32 decisions: seven Parakeet and nine Phonon occurrences, each evaluated with
the target sentence alone and the original neighboring sentence. Same frozen
glossary, candidates, question and keep/name/abstain meanings as D1. Each
utterance/scope is one packed native forward call; no approximate prefix sharing.

| Model | Parakeet word errors: sentence / context | Phonon word errors: sentence / context |
|---|---:|---:|
| Raw phone text | 3 / 3 | 5 / 5 |
| Native D1 Q8, same-day ablation | **2 / 3** | **4 / 4** |
| Native d3-lite Q8 reference | **3 / 4** | **4 / 5** |

Expected reference: 79-word read-aloud script, not independently verified spoken
ground truth. Scores ignore punctuation/case. This hides some semantic mistakes:
ordinary chili, bingo and bandit being capitalized do not increase WER.

| Meaning / occurrence | d3 sentence only | d3 with context |
|---|---|---|
| Dog Gamble | Gambol, correct | Gambol, correct |
| “I like to gamble and play poker…” | Kept, correct | Gambol, wrong |
| Phonon character flow | Flo, correct | Flo, correct |
| Ordinary water flow | Flo, wrong | Flo, wrong |
| Food chili / game bingo / robber bandit | Chili / Bingo / Bandit, wrong | Same wrong edits |
| Phonon character bandit | Bandit, casing repair | Same |
| Transcrust project | transcrust, casing repair | Same |
| Intentional O | Unchanged, no candidate | Same |

For each phone transcript, d3 changes **4/5 ordinary-word controls** in the
sentence-only arm and **5/5** with context. D1 changes 0/5 and 1/5 respectively.
These controls are ordinary gamble, water flow, food chili, game bingo and robber
bandit; they are actual phone outputs, not explicit-spelling stress cases.

The wrong ordinary-word decisions can be confident: on Parakeet's two-sentence
input P(name) was .938 for the gambling verb, .892 for water flow, .982 for food
chili, .825 for bingo and .951 for a robber bandit. No confidence floor or prompt
tuning was introduced after seeing this. `meter` and `Chilean` remain unoffered
aliases, so neither model is given a chance to repair them.

This establishes a failure of the **current glossary/context/question setup
with this model**, not a claim that d3-lite cannot be adapted or fine-tuned for
the task. Upstream decision benchmark scores do not establish this capability.

## Resource observations

Native CPU reference computation, four Rayon threads. Matrices remain packed
Q8 in persistent storage, with only one operator temporarily expanded into
F16-rounded F32 compute scratch. Embedding rows are read on demand from the
packed file. This is not an optimized packed GPU/integer-kernel benchmark.

| Phase | Wall time | Peak process RSS |
|---|---:|---:|
| All 56 public agreement rows | 298.78 s | 836,232 KiB ≈ 816.6 MiB |
| All 32 household decisions | 165.19 s | 1,119,844 KiB ≈ **1.07 GiB** |

Model load after verification was .473s / .543s for those phases; verification,
warmup and output serialization are included in wall time. The first private
standalone warmup took 6.819s. Persistent packed matrices occupy 530,595,840
bytes, while the complete bundle is 802,817,536 bytes. File-cache memory outside
process RSS is not included, so these figures are not total device RAM use.

| Phone batch | Occurrences / packed tokens | Native CPU seconds |
|---|---:|---:|
| Parakeet sentence | 7 / 2,488 | 35.487 |
| Parakeet context | 7 / 2,557 | 33.711 |
| Phonon sentence | 9 / 3,209 | 43.172 |
| Phonon context | 9 / 3,297 | 44.056 |

Those delays do not meet the intended correction latency. They measure this
correctness-first CPU implementation, not the model's potential with the
reference's optimized GPU kernels. Machine load was uncontrolled. No phone
latency/RAM or Apple build/parity claim is made.

## Evidence / disposition

`parity.json` retains every supported reference/native distribution and the
debug snapshots. `corrections.json` retains full frozen states, native token IDs,
option meanings, probabilities, choices and delivered text. `isolation*.json`
checks the water-flow error separately. Resource logs, scoring TSV and artifact/
source hashes are adjacent. Model and audio files stay outside Git; no private
text went to a service or became a public-CI fixture.

Do not promote this backend or replace D1's live two-sentence policy. The
[D1 ablation](../d1-native-context/README.md) is mixed: sentence-only fixes the
infinitive mistake, but useful neighbor-dependent repairs are lost. Both
findings are now reproducible in native Rust for future controlled experiments.
