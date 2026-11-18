# Decision model experiments — recorded evidence

For the full model inventory and cross-model results through 2026-11-01, see
[decision-model-results.md](decision-model-results.md). This file retains the
earlier experiment chronology.

These are completed measurements from 2026-10-07 through 2026-10-08, retained
from the taskboard without changing their numbers. Each “no live changes” claim
describes that experiment at that time. Native opt-in integration was added
afterward; current status and remaining work belong only in [TASKBOARD.md](TASKBOARD.md).

Some adversarial expectations were challenged by John: quoted instructions are
not an editing-policy test for a referent classifier, and initial handling must
be reported separately from ordinary-word preservation. Do not treat this suite
as everyday WER or use every historical mismatch as an automatic quality failure.

**Sentence-and-pair adjudication (2026-10-07):** offline first shot in
      `tools/pair-router-probe/`, using the locally available Verdict/OpenJev
      151M fp16 encoder, the unboosted Record-2 transcript, and KEEP/name/abstain
      choices. Sentence scope ranks 6/8 selected recording repairs correctly,
      4/4 typed repair examples, and preserves 7/7 typed prose controls. At a
      0.5 confidence floor only 4/8 recording repairs qualify. Whole-passage
      scope repeats the casino/Gambol error; sentence scope avoids it.
      Median sentence inference ~85 ms on this laptop. Not a full WER study
      or calibrated correction confidence. Live opt-in `--dec` is now wired;
      human dogfood acceptance is pending (see `design-dec.md`).
      Inspect the report/proposed transcript before expanding the corpus.
      Expanded 60-case synthetic suite: `tools/pair-router-probe/SUITE.md`.
      Referent prompt ranks 20/20 repairs, but edits 6/20 preservation cases
      (including both privacy initials) and 8/20 ambiguous cases. Explicit
      correction prompt performs worse. Expectations await John's review;
      neither arm establishes safe automatic replacement or abstention.
      API baseline (2026-10-08): `tools/pair-router-probe/openrouter-baseline/SUMMARY.md`.
      Jev 1.13 repairs 20/20 main positives with 0/20 preservation edits;
      Cars 6/6 repairs and 0/6 preservation edits. Ambiguous edits remain
      7/20 main and 1/6 Cars before a floor. Both API models preserve initials.
      Clef Flash repairs 19/20 main positives with 2/20 preservation edits.
      314 successful calls cost $0.003404616; no live API path is wired.
      **Deployment constraint:** the family app's decision stage stays offline
      alongside local NVIDIA ASR. Needle is d1's deployment competitor; API
      measurements are quality references only, with no API fallback planned.
      Offline comparison: `tools/pair-router-probe/d1-q8/SUMMARY.md`.
      Text-only d1 Q8 selects 16/20 main repairs, with 1/20 preservation edits
      and 7/20 ambiguous edits; Cars 5/6 repairs and no unwanted edits.
      Existing full-depth Needle 3 selects 4/20 main repairs with default
      gating. Forced classification raises repairs to 17/20 but also edits
      17/20 preservation cases and all 20 ambiguous cases. These are untuned
      adapters; raw multi-call results remain invalid, not guessed edits.
      d1 process footprint ~522 MiB, measured with no media encoders loaded.
      Q4 paired follow-up: `tools/pair-router-probe/d1-q4/SUMMARY.md`.
      Identical native inputs/revisions: 18/20 main repairs, 1/20 preservation
      edits, 6/20 ambiguous edits; Cars 5/6 repairs, one ambiguous flow -> Flo
      edit. Original probe drops from 5/12 to 4/12 repairs. Q4 process RSS
      ~332 MiB; graph+weights 244.7 MB versus Q8 406.3 MB.
      Two-sentence diagnostic: `tools/pair-router-probe/two-sentence-report.md`.
      28 paired scenarios per quantization, identical inputs/calibration. Both
      Q4/Q8 repair 4/8 context-disambiguated names, but nearby names induce
      6/8 ordinary-control edits, 1/2 initial edits and 2/2 quoted-spelling edits
      under the existing referent prompt. Capacity is ample (56-97 tokens);
      target isolation/preservation remains unresolved. No live changes.
      Five-entry glossary ablation: `tools/pair-router-probe/glossary-report.md`.
      56 added local calls per model. Two-sentence repairs Q4 4/8 -> 6/8,
      Q8 4/8 -> 7/8; unwanted edits Q4 9/20 -> 10/20, Q8 stays 9/20.
      Single-sentence unsupported edits rise from 0/28 to Q4 6/28, Q8 8/28.
      Both still fail one initial and both quoted-spelling controls. Glossary
      plus headings adds 55 tokens; no truncation or live changes.
      OpenJev glossary follow-up: `tools/pair-router-probe/openjev-glossary-report.md`.
      112 new local inferences, native LABEL/SEP + temperature 5.0069.
      Both arms repair 8/8 contextual names; excluding initials/quoted spellings,
      ordinary/other-person name selections rise 8/16 -> 12/16 with glossary.
      Two-sentence median 94 -> 169 ms; fp16 process RSS ~649-653 MiB,
      startup/process peak ~849 MiB. No truncation, API calls or live changes.
      Bluey expansion: `tools/pair-router-probe/bluey-report.md`. Four supplied
      entries appended, 22 new paired scenarios plus 28 earlier pairs; 200 Q8
      calls comparing original vs expanded glossary. New two-sentence repairs
      7/12 -> 6/12, common-word name selections 1/10 -> 1/10. Expanded:
      Bluey 3/3, Bingo 2/3, Chili 0/3, Bandit 1/3; one food-chili error.
      Earlier repairs 7/8 -> 6/8; ordinary/other-person controls stay 6/16
      (excludes initials/quoted spellings). Expansion adds 52 tokens; RSS
      ~518 MiB, median two-sentence inference ~1.21 s under varying load.
      Synthetic homophones/splits are not recorded NVIDIA errors. No live changes.
      Human review of the fixtures/results remains open; live behavior unchanged.
