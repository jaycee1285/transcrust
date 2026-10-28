# Two-sentence context experiment

Question: does one neighboring sentence help the local decision model choose
what a marked word means, without letting a nearby name override ordinary
language or intentional privacy initials?

`two-sentence.json` fixes 28 scenarios before inference. Each contains the
exact target sentence, the two-sentence version, independent expected choices
for each scope, and the existing vocabulary descriptions. No expected choice
or scenario category enters the model input. The question, option order,
formatter and calibration match the prior d1 referent probe.

- Eight ambiguous targets receive a neighbor establishing the vocabulary name.
- The same eight targets receive opposing context establishing an ordinary word
  or another person. Both single-sentence copies have an abstention gold label.
- Eight ordinary-word/other-person controls receive a neighboring mention of
  the competing vocabulary name.
- Two initial controls and two explicit spelling controls receive the full
  competing name nearby. Their marked text must remain unchanged.

Neighbors occur before and after targets. Each input has exactly one marked
occurrence. For example, the gambling occurrence is judged independently of
the nearby dog's name; the experiment never replaces every matching word.
Full snippets and proposed outputs are retained in each model's report.

Q4 and Q8 use matching pinned upstream revisions and identical token sequences.
Each runs 56 local inferences, with zero formatter truncation. No API requests,
ASR replay, model tuning or changes to live dictation are part of this test.
KEEP and abstain both preserve text, while exact-choice scoring distinguishes
them. Scope-specific gold labels make an ambiguous single sentence different
from a two-sentence input that supplies evidence. The eight repeated targets
and paired scopes mean these are correlated diagnostics, not 56 independent
utterances or evidence of production calibration.

Run Q4:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4 --cases tools/pair-router-probe/two-sentence.json
```

Run Q8:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q8 --cases tools/pair-router-probe/two-sentence.json
```

Read/regenerate the saved [comparison](two-sentence-report.md):

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/context_suite.py
```
