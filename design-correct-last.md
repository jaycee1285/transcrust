# Correct last dictation

Status reconciled 2026-10-28: Linux GUI and GTK palette accepted by John on
2026-10-06. Vocabulary weighting failed its subsequent quality gate. Native
opt-in D1 adjudication is implemented separately in `design-dec.md`; its
post-port live acceptance remains open. The shared crate/FFI extraction and
Apple shells are not built. Current order and gates: `TASKBOARD.md`.

Linux's keyboard-first dogfooding route for Swiftype's shared correction engine.
Apple's review-before-paste contract remains unchanged. iOS supplies its own
review surface over the shared speech/correction data, not Linux automation.

The daemon retains raw engine output, the delivered transcript and subsequent
corrected text separately. `transcrust --correct`, bound by the compositor to
Ctrl–Alt–C, and the tray's “Correct last dictation…” open the same compact egui
editor. Enter makes a newline; Ctrl+Enter submits; Escape cancels.

Linux replacement assumes the original editor regains focus when the review
window closes and the caret is still immediately after the delivered text.
Select backward by grapheme count, then replace. Copy is an explicit alternative.
No automatic primary-selection capture: it can contain an old selection.
Do not claim Wayland can recover an editable text range from clipboard contents.

Diff delivered → corrected text into short replacement spans. Keep/Drop each
proposal; pure insertions, deletions, punctuation edits and long rewrites do not
become vocabulary. Save kept `heard:` learning records in the configured Markdown file;
reload before the next dictation. Preserve habit/task exclusions.

### Learning semantics — corrected by human smoke (2026-10-06)

The interface and GTK palette passed John's smoke. Learning is a vocabulary
preference, not an unconditional substitution. The first `heard:` application
did the latter and failed this semantic acceptance gate.
For example, `heard: gamble` under Gambol rewrote the ordinary verb in a sentence
about poker. Kept pairs should supply candidates for bounded decode-time
weighting (E.1), leaving acoustic/model evidence able to select the original.
Do not substitute a hand-written dog/poker context rule for that capability.

Acceptance utterance: “I love my dog Gambol. I like to gamble and play poker
with cards.” Both spellings must survive in the same dictation. Retain the
successful Rowan, transcrust and Nell cases as regression smoke material.
The hard alias pass is now removed. Entries with kept pairs also bypass fuzzy
post-correction; their exact spelling still receives canonical casing. Original
vocabulary entries without learned pairs retain their existing phonetic behavior.

`boost.rs` provides the same prefix/failure/backoff scorer to the direct Parakeet
TDT driver and Nemotron's streaming RNN-T driver. Canonical terms (including
case variants) are segmented exactly into each export's vocabulary. The scorer
adds a finite preference before argmax, gated by raw top-5 rank (top-20 after
three engaged prefix tokens) and a raw-logit-gap ceiling. Confidence uses the
unmodified logits. Blank emissions preserve prefix state; abandoning a prefix
refunds its pending score; completed phrases keep their credit. Nemotron carries
the scorer across chunks and resets it at the start of each dictation.

The alpha/depth calibration comes from `murmure-reference.md`; the implementation
is written fresh from the published
[NeMo boosting graph design](https://github.com/NVIDIA-NeMo/Speech/blob/main/nemo/collections/asr/parts/context_biasing/boosting_graph_batched.py),
not copied from the AGPL app. `[vocabulary].boost` scales the preference from
0 to 1 (default 1); `TRANSCRUST_VOCAB_BOOST` overrides it for comparison runs.
Read vocabulary at each new dictation so kept terms apply without restarting.

Granite, Moonshine and the `parakeet-rs` fallback do not yet have decoder
weighting. Their learned pairs likewise no longer force substitutions. No
Apple crate split/parity claim follows from this Linux implementation.
The E.2 double-decode divergence guard remains a separate, unimplemented item.

Validation: scorer/learning tests plus actual Parakeet and Nemotron inference on
the five-second interview fixture. Nemotron's weighted and unweighted delivered
text matched on that fixture; this is a regression spot-check, not an accuracy
study. The later mixed Gambol/gamble human smoke failed; see the taskboard.

Human acceptance: dictate into an editor; open correction; edit one name;
replace; keep its proposal; dictate again and see it applied. Also cancel a
review, reopen after Copy, and exercise multiline text. This is a Linux smoke,
not proof of macOS focus restoration, permissions, paste or iOS integration.
