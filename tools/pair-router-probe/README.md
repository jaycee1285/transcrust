# Sentence-and-pair probe — 2026-10-07

> Historical experiment notes. “No live changes” below describes the individual
> probe at its measurement date. The app now has native Rust batched `--dec`;
> its current contract is `../../design-dec.md`, and work/acceptance is tracked
> only in `../../TASKBOARD.md`. These Python scripts remain offline research
> tools, not the application runtime.

An offline first shot at the bounded-choice correction seam John proposed.
Uses the existing local Verdict/OpenJev ModernBERT 151M fp16 ONNX model at
`~/repos/dz/jev-options/rlcd-151m`. No model download or production integration.

The family application's decision stage must run offline alongside local NVIDIA
ASR. Needle is the deployment competitor for d1; API results are explicitly
authorized quality baselines only, never a planned runtime or fallback.

The source is `~/syncthing/Record-2.wav`: 143 seconds of John's voice. The new
unboosted delivered Parakeet transcript is `record-2-unboosted.md`; it still
includes the legacy phonetic/casing post-pass. `provenance.json` records the
WAV hash, transcript hash and generation command. Eight selected repair spans
use the existing Record-2 fixture notes, observed Dayshade pairs and task names.
Eleven separate typed cases include John's dog/son/casino example and negative
controls. Their family-name descriptions are supplied facts, not inferred
personal knowledge. They do not measure audio recognition.

## Reproduce

From this repository:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/run.py --sentence-only --output-dir tools/pair-router-probe/sentence
```

Omit `--sentence-only` to repeat the passage/context-window arm; use another
`--output-dir` to retain previous measurements. `--model-dir` accepts a different
local model bundle with the same ONNX/tokenizer/calibrator contract. The optional
Nix shell supplies Python, ONNX Runtime, tokenizers and NumPy; these are not added
to the app runtime.

The runner copies Verdict's *prompt contract* (label markers and SEP, natural
language descriptions with “It is …”, and the native insufficient-evidence
label). It loads temperature 5.0069 for three choices from the installed
calibrator. There is no token generation and no silent context truncation.
`results.json` retains every input, raw logit, probability, chosen option and
inference time. Context scope is the only changed variable between the two arms.
The same interpretation labels are supplied on positive and negative examples.

## Results

| Scope | Recording repairs ranked correctly | Typed repairs ranked correctly | Wrong edits on 7 typed controls | Median inference |
|---|---:|---:|---:|---:|
| Passage/window | 6/8 | 4/4 | 1 | 113 ms |
| Marked sentence | 6/8 | 4/4 | 0 | 85 ms |

The passage arm changes the casino verb to Gambol when the earlier dog sentence
is in its input, despite marking the casino occurrence. Sentence scope instead
chooses Gambol for the dog, keeps gamble at the casino, and chooses Rowan for
Rowen. This is evidence for scoping the classifier to the affected sentence,
not proof of a model's ability to attend to arbitrary marked spans in long text.

Both arms miss `job supply → Jobs apply` and `At track → Apptrack`. Those title
repairs need more than the current semantic question/labels supply. At a 0.5
confidence floor, the sentence arm makes 4/8 recording repairs and 3/4 typed
repairs, still with no wrong edits on the seven controls. At 0.7 it makes none.
Do not choose a production floor from this tiny probe: the shipped calibrator
was fitted on other decision domains, not spelling corrections.

`sentence/report.md` is the detailed result. `sentence/record-2-proposed.md` is
an inspection copy with the four recording edits qualifying at 0.5, generated
from the saved run. It is not ground truth and never overwrites the source.
`report.md` and `results.json` retain the first, whole-passage arm.

This is selected-span scoring with supplied candidates, not end-to-end WER.
Candidate discovery, automatic semantic descriptions, confidence calibration,
broader false-positive measurement and Apple platform parity remain untested.
No live dictation behavior was changed by this experiment. Decoder weighting
failed its preceding human smoke and is not established as an improvement.

## Expanded behavioral suite

See [SUITE.md](SUITE.md) for 60 separate synthetic repair, preservation and
ambiguity cases, including privacy initials. The broader suite exposes false
edits and unreliable abstention despite ranking all positive repairs correctly.
It retains both the referent and explicit correction prompt measurements.

## API baselines

[OpenRouter comparison](openrouter-baseline/SUMMARY.md) records 314 successful
Jev 1.13 / Clef Flash requests across all saved local arms. Jev is strongest on
preservation and Cars in this run; ambiguity remains a distinct failure mode.
Requests, raw responses, probabilities, usage and provider IDs are retained.
`openrouter.py --report-only` rebuilds the report without authentication or API calls.

## Offline candidates

`d1.py` runs text-only d1-omni Q8 or Q4 bundles with the inspected upstream prompt
formatter and shipped calibration, on the same saved inputs. `download-d1.py`
fetches a revision-pinned text bundle into `/tmp/transcrust-d1-omni-600M-q8`;
it downloads no media encoder. `needle3.py` uses the existing full-depth local
Needle 3 bundle and tests both native gating and forced classification, with
telemetry disabled and input-overflow rejection enabled.

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/offline-report.py
```

This reads saved results only. The [offline comparison](offline-comparison.md)
is the deployment comparison; the API report remains a quality reference.

The [Q4 versus Q8 report](d1-q4/SUMMARY.md) compares identical native inputs and
calibration at matching pinned upstream revisions. Q4 needs only its text graph
and sidecar; the downloader reuses verified shared tokenizer/prompt files:

```sh
python3 tools/pair-router-probe/download-d1.py --quantization q4 --reference-dir /tmp/transcrust-d1-omni-600M-q8
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4
```

Inspect the saved paired comparison without loading models:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/quantization-report.py
```

## Neighboring-sentence context

[Two-sentence experiment](TWO-SENTENCE.md): 28 paired scenarios (56 decisions
per local model), testing helpful context, opposing interpretations, nearby-name
contamination, privacy initials and exact spelling. The question/labels remain
those of the referent probe, isolating the effect of adding a neighboring
sentence. [Saved Q4/Q8 comparison](two-sentence-report.md).

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/context_suite.py
```

## Five-entry shared glossary

[Glossary ablation](glossary-report.md) tests the user's exact five term/description
entries from [glossary.md](glossary.md), added as a shared table before the marked
dictation. Candidate-specific option descriptions and all gold labels stay fixed.
Each quantization runs all 56 inputs; the original no-glossary results are retained.
Nell, Cigs and Mater remain unlisted controls. No API calls or live changes.

Re-run Q4:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4 --cases tools/pair-router-probe/two-sentence.json --glossary tools/pair-router-probe/glossary.md
```

Read/regenerate the saved comparison without loading models:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/glossary-report.py
```

## OpenJev with the shared glossary

[OpenJev/d1 comparison](openjev-glossary-report.md) runs the same 56 context inputs
with and without the five-entry glossary through the existing 151M fp16 export.
It preserves OpenJev's native LABEL/SEP contract and three-choice calibration.
The comparison separates initial/quoted-spelling stress cases from the primary
repair/control table and records process memory, token counts and inference time.

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/openjev-context.py
```

Add `--report-only` to read saved results without loading the model.

## Bluey glossary expansion

[Bluey Q8 comparison](bluey-report.md) appends the four supplied character
entries to the original five-entry glossary, retaining the user's spelling
Chili. [bluey.json](bluey.json) defines 22 new paired scenarios; the combined
[fixture](bluey-expanded-suite.json) also retains the 28 earlier scenarios.
Each glossary arm runs 100 local decisions. Gold labels and questions are
identical between arms. Candidate descriptions already identify the characters
in both arms; this measures adding a shared glossary, not first exposure to the
character facts. No API calls, audio replay, tuning or live changes.

The three common/proper splits are bingo/Bingo, chili/Chili and bandit/Bandit.
The Bluey cases add character references and a blue/Bluey color near-match.
Some name cases require capitalization; synthetic bin go/chilly/band it/blue e
cases probe spelling repair. These are not recorded NVIDIA transcription errors.
Names occur both before and after marked targets. Earlier initial and quoted
spelling cases remain diagnostic controls, outside the primary regression table.

Inspect the saved comparison:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/bluey-report.py
```

Re-run expanded-glossary Q8:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q8 --cases tools/pair-router-probe/bluey-expanded-suite.json --glossary tools/pair-router-probe/glossary-bluey.md --output-dir tools/pair-router-probe/bluey-q8-expanded-glossary
```

The original-glossary control uses `--glossary tools/pair-router-probe/glossary.md`
and `--output-dir tools/pair-router-probe/bluey-q8-original-glossary`.

## Name scrub (2026-11-18)

Name scrub (2026-11-18): the household child's name and its misheard variants were replaced (Rowan / Rowen) in saved inputs, outputs and receipts after the runs. Probabilities were computed on the original text; reruns on the scrubbed files may differ slightly.
