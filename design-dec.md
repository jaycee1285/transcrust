# Live contextual vocabulary dogfood — `--dec`

Status updated 2026-10-29: native implementation and machine checks exist;
John reports really good live corrections and tolerable but slower-than-desired
latency. Preserve Q8 quality; explicit preservation/reload checks and latency
improvement remain open. See `Smoke-Human-transcrust.md` for the ledger
and `TASKBOARD.md` for the next steps; research history is `decision-experiments.md`.
The Linux experiment leaves the Apple review-before-paste contract unchanged.

Stop the existing daemon through its tray Exit, then run:

```sh
nix develop -c cargo run -- --dec --smoke
```

Use the normal push-to-talk binding and an ordinary editor. The same daemon
continues to serve `--correct`, its configured correction chord, and the tray.
`--smoke` echoes phase/timing/choice logs to the terminal; it still types output.
Without `--dec`, the previous decoder weighting and phonetic path remain active.

## Runtime

`src/decision.rs` owns candidate generation, two-sentence context, resident model
lifetime, replacement and audit logging. `src/d1.rs` drives the Q8 graph directly
through the existing Rust ORT/tokenizers stack. No interpreter, subprocess or
JSON pipe is involved. No model is loaded in the correction UI.

The model uses only d1's Q8 text graph and weight sidecar. Tokenizer, graph,
weights and config are verified against the local download manifest before load.
The pinned upstream formatter is transliterated in Rust. No network request or automatic
download exists in the runtime. The bundle's Liquid licence stays with the model.

Resolution: `TRANSCRUST_DEC_MODEL`, then
`~/.local/share/transcrust/models/d1-omni-600M-q8`, then the previous probe's
`/tmp/transcrust-d1-omni-600M-q8` cache. The persistent model is installed on
John's machine. Python is absent from the app's Nix shell/runtime dependencies;
the separate optional research shell retains historical model-evaluation tools.

`--dec` sets decoder boosting to zero after config loading, regardless of its
config/environment setting, and bypasses the entire legacy vocabulary pass.
Punctuation, repetition/filler cleanup and deterministic sentence capitalization
remain. In long mode the existing prose polisher precedes d1. Q8 is the final
vocabulary stage, immediately before injection. Switching ASR modes keeps the
same decision model resident.

## Vocabulary

`~/syncthing/john-vocab.md` is the sole vocabulary source, shared with mopvox
and dayshade. The existing `[vocabulary].path` setting overrides it for all
transcrust vocabulary consumers. It reloads each utterance; no restart is needed
for aliases or descriptions. There is no local sidecar or bundled fallback.
Missing/unreadable files preserve text and report the error.

```text
## Gambol
kind: person
description: pet dog
ordinary: ordinary English about gambling or betting money
heard: gamble
```

`description:` and `ordinary:` are optional metadata alongside existing `kind:`
and `heard:` fields. New kept correction pairs are candidates on the next
dictation. Removing an alias or entry removes it from the decision surface.
Entries without descriptions receive a generic term description; unknown
misspellings are never guessed into a phonetic neighborhood. Habit/task entries
stay excluded. One-character aliases are ignored. `tools/decision-vocab.md` is
an example template only; older local decision-vocab files are no longer read.
Other devices can share the metadata; their existing parsers keep their existing
behavior until a contextual decision adapter is wired there.

Only whole-word/phrase aliases and differently cased canonical spellings become
candidates. Exact canonical spellings are already correct and incur no inference.
Unknown misspellings require adding/keeping an alias. Overlapping alternatives
are left unchanged rather than selecting one by file order. Descriptions and
aliases identify potential replacements, never unconditional substitutions.

## Decisions and latency

Each candidate sees its containing sentence and the preceding sentence, or the
following sentence for the first sentence. There is no cross-dictation history.
The shared glossary is supplied with one marked occurrence. The choice question
is the existing referent question: KEEP/name/insufficient evidence. Short glossary
descriptions also form the candidate's name option with its canonical spelling;
the live starter prompt is not byte-identical to the earlier long-description
benchmarks. All candidates use the immutable original cleaned text; replacements
are applied afterward in reverse UTF-8 byte-offset order.

Native option order is explicitly keep/name/abstain even when Rust serializes
JSON object keys alphabetically. The formatter uses per_option=64 and shipped
three-choice calibration. Any state/option truncation is rejected, not silently
accepted. No unvalidated probability floor was selected. A name choice applies
that candidate; KEEP/abstention preserve it. All eligible occurrences become
independent right-padded rows in one graph call. Each row keeps its original
two-sentence context and three options. This follows LiquidAI's
[`probabilities_batch` implementation](https://huggingface.co/LiquidAI/d1-omni-600M/blob/02b55d7076f15129e59ab3f94783f32c4b088674/modeling_d1.py),
rather than concatenating questions into an untrained sequence. Native ORT uses
up to four CPU threads (bounded by available parallelism). The warmed model
remains loaded between utterances. A batch exceeding upstream's 65,536 padded
token budget is rejected with text preserved.

Startup has a 30s budget. All decisions for one utterance share a 15s budget.
Missing model, invalid results and timeout preserve the
cleaned transcript without falling back to forced vocabulary substitutions.
A failed model is disabled until daemon restart. ORT runs on a blocking Rust task;
a timeout sets its RunOptions termination flag. Results from an expired task
cannot reach injection. A request-level failure applies no partial edits.

## Observation and acceptance

`logs/latest.log` records raw, cleaned and final text samples, per-candidate
choices/probabilities/shared batch time, ASR time and total wait to injection.
`logs/decisions.jsonl` appends full raw/input/final text, marked candidate contexts,
meanings, choices/probabilities, decision time, ASR time, total wait, mode and
delivery status. `--correct` stores the raw and actual delivered strings as before;
its wait display now includes processing through the decision stage.

An optional no-microphone/no-injection path uses the same code:

```sh
nix develop -c cargo run -- --fix 'I love my dog Gamble. I like to gamble and play poker with cards. My son Rowan likes Flo from Cars.' --dec
```

Its phase log is `logs/dec-probe.log`, preserving the daemon's latest log.
It appends an audit with source `--fix`, but does not change last-dictation state.
The previous Python adapter's compact-prompt check took 2.61s for three candidates; it repaired the dog's
name, overcorrected the casino verb and missed Rowan. This is a working dogfood
path, not a new claim that the model solves those meanings. John evaluates actual
ASR output, correction quality and tolerable delay in the live smoke.

Machine verification:

- Native Rust port (2026-10-10): exact token IDs and marker positions match the
  saved Q8 formatter; padded batch probabilities match saved and single-row
  probabilities within 1e-4. Two-question timing: 1.88s batch / 2.07s sequential
  in the first two-thread run; a later thread comparison measured 2.59s with one
  thread, 1.37s with two, and 1.15s with four. These are laptop observations,
  not an acceptance claim or constant latency guarantee.
- Eight candidate/parser/edit tests: UTF-8, occurrence isolation, independent edits,
  overlaps, initials, abbreviations/decimals, phrase line breaks, task exclusion and invalid replies.
- Native live transcript probe: two Gambol repairs in one batch, 1.52s total
  decision-stage time (1.46s graph inference). This uses the actual shared file.
- Functional suite: 148 pass, two installed-model tests ignored; the new native
  model test was run separately and passed. The pre-existing
  debug resampler timing assertion failed (185ms); excluded from the functional
  rerun. No resampler change belongs to this slice.
- `nix develop -c cargo test d1::tests::native_batch_parity_timing_and_recovery -- --ignored --nocapture`:
  real native batch probabilities, token IDs and marker positions compared with
  saved Q8 and single-row runs; reports batch versus sequential timing, rejects
  overflow, and checks native ORT termination and subsequent fresh-run recovery.
- Native `--fix --dec` with a missing model preserves cleaned text and logs the
  startup/request error. No external runtime executable is needed.

Human smoke: dictate names plus their ordinary counterparts, check the terminal
for candidates/choices, and use Correct last dictation to fix mistakes. Keep a
new pair and verify that the next occurrence is considered without restarting.
An utterance with no eligible words should have zero decision inferences. Report
quality and delay; live typing and replacement acceptance remain human gates.
