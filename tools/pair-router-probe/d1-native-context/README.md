# Native D1 context ablation — 2026-10-31

Same frozen inputs and glossary as the paired-encoder experiment, using the
**unchanged production `src/d1.rs`** encoder, Q8 bundle, calibration and argmax.
132 decisions: both sentence/context arms for 16 phone occurrences plus the
existing 100 synthetic rows. No threshold fitting, prompts, aliases, app flag
or live context policy changed. All candidates in a phone utterance/arm are
submitted in one batch; synthetic batches contain up to eight independent rows.

```sh
nix develop -c bash tools/d1-context-probe/run.sh
```

## Actual phone text

| Input | Sentence alone | With neighboring sentence |
|---|---:|---:|
| Parakeet, 79-word script | **2 errors** | 3 errors |
| Phonon, same recording | **4 errors** | 4 errors |

Both sentence-only outputs fix dog Gamble→Gambol and **preserve “I like to
gamble and play poker with cards.”** Both context outputs fix the dog but
wrongly change that verb to Gambol. However, Phonon's `raced past meter and flow`
gets Flo only with context. Sentence-only leaves that repair undone. Other
ordinary controls and O survive; `meter` and `Chilean` remain unoffered aliases.
Both arms fix project casing. Initial native two-sentence results are reproduced.

The reference is the expected read-aloud script, not independently checked
spoken ground truth. WER ignores case/punctuation. Full raw candidate states,
choices and probabilities are retained; case-only edits need separate inspection.

## Broader fixtures

On the same 33 synthetic evaluation rows per scope held out from the *other
experiment's* threshold fitting (D1 does not fit anything):

| Context | Repairs | Unwanted edits |
|---|---:|---:|
| Sentence alone | 2 / 6 | 8 / 27 |
| Two sentences | 10 / 16 | 3 / 17 |

**These denominators differ deliberately.** An ambiguous sentence may have an
abstain label alone but a name label after its neighbor establishes meaning.
These totals are not evidence that sentence-only loses eight comparable repairs.
The cases are correlated development diagnostics, with some contrived labels;
they are not representative dictation accuracy estimates.

A more direct paired subset consists of the 23 non-initial/non-quotation
scenarios whose desired decision stays the same in both scopes:

| Context | Repairs / 8 | Unwanted edits / 15 |
|---|---:|---:|
| Sentence alone | 2 | 5 |
| Two sentences | 5 | 7 |

Neighbors add three Bluey character repairs while also adding net two unwanted
edits. Thus **do not switch the live app globally to sentence-only based on the
one gambling success**. Target focus is worth further testing, but avoiding
neighbor contamination must be balanced against useful disambiguation. No
special “to gamble” rule or confidence threshold was added.

## Verification / resources

Model files are manifest-verified by the production loader. Identical first
batch repeat had zero probability drift. No input truncation or inference
failure occurred. The first standalone row warms the model before measurement.
The existing frozen context fixture and `src/d1.rs` hashes are recorded.

Standalone full-run peak RSS: **729,420 KiB ≈ 712.3 MiB**, without ASR.
Process wall: 179.21s, including load, warmup, repeat and all 132 decisions.
Phone batch graph times: Parakeet 11.960 / 13.068s (single/two), Phonon
16.769 / 16.092s. This run was substantially slower than earlier observations;
machine load was uncontrolled. These are single desktop observations, not
mobile measurements or a controlled performance comparison. This research
driver calls the model directly and does not apply the daemon's 15-second
utterance timeout; a live timeout would preserve the cleaned text.

Models/audio remain outside Git; no injection, last-dictation mutation or
external inference occurred. The accepted live two-sentence path remains as-is.
