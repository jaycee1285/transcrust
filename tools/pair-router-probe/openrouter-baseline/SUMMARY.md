# API comparison — 2026-10-08

Jev 1.13 is the strongest baseline in this run. There were 314 successful paid
requests: 157 per model, across the original 19-case probe, the 60-case suite,
18 Cars cases, and the 60-case explicit correction-question arm. Each request
reuses the saved local sentence, question and three option descriptions, with
the model's native typed-choice serialization. Expected labels are never sent.

## Original referent question, no confidence floor

| Surface | Model | Repairs selected | Edits on preservation cases | Edits on ambiguous cases |
|---|---|---:|---:|---:|
| Main 60 | Verdict 151M local | 20/20 | 6/20 | 8/20 |
| Main 60 | Jev 1.13 | 20/20 | 0/20 | 7/20 |
| Main 60 | Clef Flash | 19/20 | 2/20 | 3/20 |
| Cars 18 | Verdict 151M local | 6/6 | 1/6 | 1/6 |
| Cars 18 | Jev 1.13 | 6/6 | 0/6 | 1/6 |
| Cars 18 | Clef Flash | 6/6 | 1/6 | 1/6 |

Both API models preserve both privacy initials in both prompt arms. Jev also
preserves the storm lightning sentence, ordinary gamble, numbers, and the
different person named Rowen. Clef's preservation errors are the explicitly
different spelling Rowen (P(name) 0.7021) and "Set the Cigs habit target to six
cigarettes" (0.9619). In Cars it changes "Mader has that surname; he is not
the Cars character" to Mater (0.85). This is the saved local sentence scope:
the existing sentence extractor splits after the abbreviation "Mr." and drops
the preceding "Our neighbor Mr." from every model's input in that case.

Jev still guesses on underspecified sentences: seven ambiguous main-suite edits
and "Mader is outside" -> "Mater is outside" (0.65). Native abstention selects
the expected label on 6/20 main ambiguous cases and 5/6 Cars ambiguous cases.
KEEP and abstention both preserve text but express different knowledge.

At an illustrative P(selected) floor of 0.9, Jev retains all 26 main/Cars repairs
and makes zero unwanted edits across those 78 cases. This is an observed result,
not a validated deployment threshold: the same floor retains only 8/12 repairs
from the original 19-case probe. No floor was fitted and no calibration claim
is established by these small, related fixtures.

## Explicit correction-question arm

On the same 60 cases, Jev selects 17/20 repairs, makes one preservation edit and
one ambiguous edit; exact KEEP/name/abstain accuracy is 50/60. Clef selects 18/20
repairs, makes zero preservation edits and five ambiguous edits; exact accuracy
is 42/60. The local model's corresponding counts were 20/20, 18 and 18, with
22/60 exact accuracy. The question changes the repair/preservation trade-off;
it does not automatically solve abstention.

On the original eight selected Record-2 repairs, Jev selects 8/8, Clef 7/8 and
Verdict 6/8. All preserve the seven original prose controls. This uses supplied
candidate spans in an existing transcript, not new audio inference or full WER.

## Cost and provenance

- Jev: $0.002613282, median client round trip 308 ms, TypeSafe provider on all
  157 calls, resolved snapshot `typesafe/jev-1.13-20260917`.
- Clef: $0.000791334, median client round trip 325 ms; OpenRouter routed 152
  calls to PrimeIntellect and five to Cloudflare. This is a routed baseline,
  not a provider-pinned latency comparison.
- Total reported cost: **$0.003404616**, approximately **0.34 US cents**.
- Two sandbox DNS failures occurred before network access was allowed. Every
  intended case/model combination subsequently completed successfully.

`receipts.jsonl` retains requests (without authentication headers), raw API
responses, model/provider IDs, usage, option probabilities, the separate API
confidence statistic, elapsed time and source baseline hashes. `report.md`
contains every decision and the 0/0.5/0.7/0.9 floor tables. Local latency measures
inference only; API latency includes network and queueing.

Read saved results without loading a key or making paid requests:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/openrouter.py --report-only
```

Omit `--report-only` to resume missing API results using the same key file as
`~/.pi/agent/extensions/pi-opentest.ts`. Successful receipts are reused when
the baseline hash matches; failed attempts are retried. Changing saved local
results creates a new baseline hash and therefore new paid calls. No live
dictation or vocabulary behavior was changed.

API references:
[Decisions endpoint](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request),
[Jev tutorial](https://openrouter.ai/docs/guides/community/jev-tutorial),
[Jev model](https://openrouter.ai/typesafe/jev-1.13),
[Clef Flash model](https://openrouter.ai/cloudflare/clef-flash).
