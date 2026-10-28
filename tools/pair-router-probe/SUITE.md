# Bounded correction suite — 2026-10-07

`suite.json` contains 60 new, synthetic sentences, separate from the initial
19-case Record-2/typed probe. Five supplied pairs each have four repair cases,
four preservation cases and four ambiguous cases. The glossary covers Gambol,
Rowan, Nell, transcrust and Cigs; these are supplied local facts, not inferred
personal information. No audio or additional personal writing is required.

## Contract to review

- **name:** the sentence identifies the glossary referent and the marked spelling
  should be replaced with the supplied canonical spelling.
- **keep:** preserve the marked text, including ordinary words, numbers, other
  people, deliberate quotations and privacy initials.
- **abstain:** the sentence does not establish the intended referent; preserve
  the text and make no automatic replacement.

Every case includes its expected decision and a rationale. Markers select one
occurrence; candidates are supplied, not discovered. Initials deliberately test
what happens if a candidate generator presents an inappropriate expansion.
The runner rejects duplicate IDs, mismatched spans and context overflow.

The intended acceptance gates are zero changes to preservation or ambiguous
cases, especially zero expansion of privacy initials. Repair coverage is
reported separately: declining every edit satisfies preservation but supplies
no useful repairs. Strict KEEP-versus-abstain accuracy is also reported, because
the two leave the same text but communicate different knowledge.

These fixtures need John's review before treating the expectations as accepted.
They include explicit spelling instructions and deliberately contrived quotations;
they are a behavioral stress suite, not a representative usage distribution.
After using these cases for prompt development, use fresh cases for final evaluation.

## Run

From the transcrust repository, using the existing local Verdict model:

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/run.py --cases tools/pair-router-probe/suite.json --sentence-only --output-dir tools/pair-router-probe/suite-verdict
```

This repeats the original **referent** prompt. Add `--decision-task correction`
and a separate `--output-dir` to reproduce the explicit editing-question arm.
The JSON fixture can be reused with other models, but this runner implements
Verdict's ONNX/tokenizer/calibrator contract; Needle and Laya need their own adapters.

## First measurements

| Prompt task | Repairs ranked correctly | Wrong edits on 20 preservation cases | Edits on 20 ambiguous cases | Native abstention on ambiguous cases |
|---|---:|---:|---:|---:|
| Referent | 20/20 | 6 | 8 | 0/20 |
| Correction | 20/20 | 18 | 18 | 2/20 |

Referent chooses KEEP on 12/20 preservation cases and abstains on two others.
Both intentional initial cases are expanded. At an illustrative 0.5 floor it
retains all 20 repairs, but still makes four preservation errors and six edits
on ambiguous cases. At 0.7 it makes just one repair and no unwanted edits.

The explicit correction prompt performs worse; at 0.5 it makes no edits at all.
Changing the question and descriptions is insufficient to establish the desired
editing behavior. This diagnoses the current model/prompt combination, not all
bounded decision models. The installed temperature is calibrated on a different
domain; these floors are measurement points, not a production policy.

`suite-verdict/report.md` and `suite-verdict-correction/report.md` retain every
sentence and decision. Each `results.json` records fixture/model/tokenizer/
calibrator hashes, prompts, logits, probabilities and CPU inference times.
Live dictation and vocabulary files are untouched.

## Cars surface — 2026-10-08

`cars.json` adds 18 separate cases: Lightning, Mater (including Mader), and Flo
(including flow), each with two repairs, two preservation cases, and two
ambiguous cases. Lightning's distinction is capitalization in these text cases;
this does not establish whether ASR recognizes the spoken names. The mater
coupling example is a deliberate lexical stress case, not a natural-use sample.

```sh
nix develop .#pair-probe -c python3 tools/pair-router-probe/run.py --cases tools/pair-router-probe/cars.json --sentence-only --output-dir tools/pair-router-probe/cars-verdict
```

The unchanged referent prompt ranks 6/6 repairs correctly, edits 1/6 preservation
cases and 1/6 ambiguous cases. Four preservation cases choose KEEP, one abstains;
one ambiguous case chooses abstention, four choose KEEP. The false edits are
both lightning -> Lightning: the storm sentence mentioning Cars and the
underspecified sentence "Look at lightning." At a 0.5 floor, 5/6 repairs remain,
along with the storm error; the ambiguous edit falls below the floor. These
are model decisions on synthetic text with supplied candidates, not audio tests.
Details are in `cars-verdict/report.md`.

`unwanted-edits.md` lists all 14 proposed unwanted replacements from the original
60-case referent run, as original and proposed full sentences. Both privacy
initials were selected for expansion to Rowan; no live text was changed.
