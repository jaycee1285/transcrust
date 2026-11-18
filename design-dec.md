# Live contextual vocabulary dogfood — `--dec`

Status updated 2026-11-01: at John's request, **UltraBERT INT4 is now the primary
`--dec` backend** for desktop dogfood. D1 Q8 remains selectable with
`--dec-backend d1`; its 2026-10-29 acceptance is retained as baseline evidence.
UltraBERT's live human smoke is pending. Mobile latency/RAM must be measured
separately. See `Smoke-Human-transcrust.md` for the ledger
and `TASKBOARD.md` for the next steps; research history is `decision-experiments.md`.
The Linux experiment leaves the Apple review-before-paste contract unchanged.

Stop the existing daemon through its tray Exit, then run:

```sh
nix develop -c cargo run --release -- --dec --smoke
```

Use the normal push-to-talk binding and an ordinary editor. The same daemon
continues to serve `--correct`, its configured correction chord, and the tray.
`--smoke` echoes phase/timing/choice logs to the terminal; it still types output.
Without `--dec`, the previous decoder weighting and phonetic path remain active.

## Runtime

`crates/decision/src/lib.rs` owns candidate generation, two-sentence context,
resident model lifetime, backend selection and replacement. Desktop
`src/decision.rs` retains audit logging. Shared `ultra.rs` and
`decision_tensor.rs` implement the native backend promoted from the research
harness; shared `d1.rs` retains the original ORT backend. No interpreter,
subprocess or JSON pipe is involved. No model is loaded in the correction UI.

UltraBERT uses the tested Q4_0 encoder/embeddings with FP32 norms/head. Matrices
stay packed, one operator uses FP32 scratch, and only requested embedding rows
are decoded. The tokenizer is pinned; packed source identity, size and checksum
are verified at load. Native CPU work uses a dedicated pool of up to four
threads. Model load and a warmup occur at daemon startup, before dictation.
No network request or automatic download exists in the runtime.

Resolution: `TRANSCRUST_DEC_MODEL`, then the selected backend's local-share
directory. UltraBERT uses `~/.local/share/transcrust/models/ultrabert-380m-curation-q4`,
with the current checkout's `target/schema-probe/ultra-q4` as development fallback.
D1 uses `~/.local/share/transcrust/models/d1-omni-600M-q8`, with its old `/tmp`
cache as fallback. The environment override applies to whichever backend was
selected; it does not auto-detect or silently switch backends. Only INT4 bundles
are accepted by the live Ultra adapter. The original D1 graph/config/weights
verification remains intact. Model licences remain with their respective bundles.

`--dec-backend ultra|d1` also enables `--dec`. A missing/unknown backend value
is rejected, and the selector is removed from `--fix` input text. For D1:

```sh
nix develop -c cargo run --release -- --dec-backend d1 --smoke
```

`--dec` sets decoder boosting to zero after config loading, regardless of its
config/environment setting, and bypasses the entire legacy vocabulary pass.
Punctuation, repetition/filler cleanup and deterministic sentence capitalization
remain. In long mode the existing prose polisher precedes the decision backend. It is the final
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
UltraBERT receives only the marked context and the candidate's three answer
descriptions, including its canonical spelling and meaning. **The global glossary
table is omitted**, matching the tested stripped formulation. D1 keeps its
original global glossary. The referent question and option meanings are shared:
keep/name/insufficient evidence. All candidates use the immutable original cleaned text; replacements
are applied afterward in reverse UTF-8 byte-offset order.

Native option order is explicitly keep/name/abstain even when Rust serializes
JSON object keys alphabetically. Any state/option truncation is rejected, not silently
accepted. No unvalidated probability floor was selected. A name choice applies
that candidate; KEEP/abstention preserve it. All eligible occurrences become
independent sequences in one batch. UltraBERT packs them on one token axis with
separate attention ranges and reset positions; it rejects sequences over 4,096
tokens or a total over 8,192. D1 uses right-padded rows, per_option=64 and its
shipped three-choice calibration, following LiquidAI's
[`probabilities_batch` implementation](https://huggingface.co/LiquidAI/d1-omni-600M/blob/02b55d7076f15129e59ab3f94783f32c4b088674/modeling_d1.py),
with independent question rows. Native ORT uses
up to four CPU threads (bounded by available parallelism). The warmed model
remains loaded between utterances. A D1 batch exceeding upstream's 65,536 padded
token budget is rejected with text preserved.

Startup has a 30s budget. All decisions for one utterance share a 15s budget.
Missing model, invalid results and timeout preserve the
cleaned transcript without falling back to forced vocabulary substitutions.
A failed model is disabled until daemon restart. ORT runs on a blocking Rust task;
a timeout sets its RunOptions termination flag. UltraBERT runs on the same
blocking-task boundary and checks a cancellation flag before work, between
encoder layers and before readout. Results from an expired task
cannot reach injection. A request-level failure applies no partial edits.

## Observation and acceptance

`logs/latest.log` records raw, cleaned and final text samples, per-candidate
choices/probabilities/shared batch time, ASR time and total wait to injection.
`logs/decisions.jsonl` appends full raw/input/final text, marked candidate contexts,
backend identity, meanings, choices/probabilities, decision time, ASR time, total wait, mode and
delivery status. `--correct` stores the raw and actual delivered strings as before;
its wait display now includes processing through the decision stage.

An optional no-microphone/no-injection path uses the same code:

```sh
nix develop -c cargo run --release -- --fix 'I love my dog Gamble. I like to gamble and play poker with cards. My son Rowen likes Flo from Cars.' --dec
```

Its phase log is `logs/dec-probe.log`, preserving the daemon's latest log.
It appends an audit with source `--fix`, but does not change last-dictation state.
The previous Python adapter's compact-prompt check took 2.61s for three candidates; it repaired the dog's
name, overcorrected the casino verb and missed Rowan. This is a working dogfood
path, not a new claim that the model solves those meanings. John evaluates actual
ASR output, correction quality and tolerable delay in the live smoke.

Machine verification:

- UltraBERT promotion (2026-11-01): release suite **151 passed, three installed-model
  tests ignored**. The new installed-model test was then run explicitly: all
  32 frozen stripped-input distributions match the research results within
  1e-5, and both pre-start and in-flight layer-boundary cancellation pass.
  CLI tests verify primary selection, explicit D1 selection and selector removal
  from `--fix` text. Candidate tests verify Ultra receives stripped context
  with unchanged option descriptions, while D1 retains the global table.
- Real release binary `--fix --dec` loaded UltraBERT INT4, then repaired
  “My dog Gamble is an Australian shepherd.” in .47s decision time (2.22s
  startup/warmup). Explicit `--dec-backend d1` kept ordinary gamble in .82s.
  These are single text probes through the shared vocabulary, not microphone
  latency measurements. A missing Ultra bundle preserved input, and an invalid
  selector exited with status 2. No injection or running-daemon restart occurred.
- The debug suite passed 150 functional tests but missed the unchanged
  resampler timing assertion (199.8 ms). The optimized suite passes that timing
  test. No resampler or timing-threshold changes belong to this slice.

Historical D1 verification:

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
