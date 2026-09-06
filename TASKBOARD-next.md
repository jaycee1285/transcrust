# TASKBOARD — what's next for transcrust

> Revised 2026-09-06, replacing `TASKBOARD-command-channel.md`. Three things
> changed my mind about my own ordering from earlier the same night:
>
> 1. The audio path has an unfixed defect that **confounds every measurement
>    below it**, so it has to go first — not because it is the most valuable
>    but because everything measured on top of it is polluted.
> 2. Phase 1 got *built* and left *unwired*. That is the worst state a change
>    can be in, and it needs resolving before anything is stacked on it.
> 3. The board was organised around the command channel, which is the least
>    validated thing on it. John's measured bottleneck is his review pass, and
>    only one phase attacks that.
>
> The command channel is still here. It is no longer the organising principle.

> Several items here trace back to mechanisms the pre-neural dictation systems
> solved and the industry dropped. `Dragon-Mechanisms.md` is the inventory and
> the reasoning; this board is the work.

## The bet

Everything in `murmure` that transcrust never ported — confidence gating,
bigram repair, phrase boosting — is scaffolding that reconstructs output shape
Parakeet already emits natively. That is why it was never ported for dictation,
and that reasoning stands.

It is worth pointing at **commands**, where output shape is irrelevant and the
vocabulary is closed. And it runs on **Parakeet**, not Granite: the joint's
vocab logits were always in `decoder_joint-model.onnx`; `parakeet-rs` runs the
decode internally and returns a `String`. `design-dictation-as-control.md`
recorded that as a model property. It is a crate boundary. See `Parakeet-v3.md` §5.

---

## Phase A — Fix the audio path (do this first)

`audio.rs::resample` is linear interpolation with **no anti-aliasing filter**,
and this machine captures at 44100 Hz. Measured response, and where each band
folds to when decimated to 16 kHz:

| input | attenuation | aliases onto |
|---:|---:|---:|
| 10 kHz | −1.5 dB | 6 kHz |
| 12 kHz | −2.2 dB | 4 kHz |
| 14 kHz | −3.0 dB | 2 kHz |

Content at 12 kHz arrives barely 2 dB down and lands on 4 kHz — the core speech
band, and precisely where sibilant and plosive-burst cues live. That is a
credible mechanism for the proper-noun errors that actually hurt
(`Met Gala` → `Meg Calendar`, `recognition` → `dilation`).

- [ ] **A.1 — Decimate properly.** Windowed-sinc or a polyphase FIR lowpass at
      0.45 × target rate before downsampling. `rubato` is not in the lock; a
      hand-rolled decimator is ~50 lines and avoids a dependency, but either is
      fine. Applies to `resample_to_16k` and any future rate.
- [ ] **A.2 — Prove it on real audio.** Replay corpus WAVs (stored
      *pre-resample*, exactly for this) through old and new resamplers into the
      same engine and diff the transcripts. This is the one experiment on the
      board that needs no ground truth to be informative — a changed word is a
      changed word.

**Why first:** it is not the highest-value item in isolation. It is the item
that makes every later measurement mean something. Improve the dictionary on
top of a broken audio path and you cannot tell which change helped.

**Kill criterion:** if A.2 changes no transcripts across the corpus, the
resampler is not your problem. Say so in `traverse/` and stop — that is a real
result and it retires a suspicion I have raised three times.

---

## Phase B — Resolve the direct driver

`src/parakeet_ort.rs` exists, works, and has 10 passing tests. **Nothing calls
it** except `--parakeet-direct`; the live path still goes through
`parakeet-rs`. A parallel implementation that nothing uses rots silently.

- [ ] **B.1 — Measure it.** The kill criterion I wrote was never evaluated.
      Add it to `--bench` as a fourth row and compare against the crate.
- [ ] **B.2 — Reconcile the one-token divergence.** Verification demanded
      byte-identical output. It is identical on `chat-10s` and differs by a
      single comma on `chat-20s` (`album cover, by the way`). Cause is known and
      benign — the crate computes mel in Rust, the direct driver uses NeMo's own
      `nemo128.onnx` — but "known and benign" needs to be *shown*, not asserted.
      Marginal punctuation tokens score 0.3–0.7, so this is the expected place
      to drift.
- [ ] **B.3 — Then choose, and actually do it.** Either route `parakeet.rs`
      through the direct driver (unlocking Phase D on the live path) **or**
      delete `parakeet_ort.rs`. Do not leave it where it is.

**Kill criterion:** >15% slower than the crate → profile before proceeding.

---

## Phase C — Ground truth, earned rather than sat down for

- [x] **C.1 — Corpus capture.** Built. `observe.corpus = true` banks each
      dictation as WAV + JSON under `~/.local/share/transcrust/corpus/`, through
      the live path, pre-resample, at the device's real 44100 Hz. `--record`
      banks audio alone.
- [ ] **C.2 — Fill `reference` only where it came out wrong.** A
      failure-weighted corpus is worth more per minute of attention than a
      balanced one, and it is the only kind that gets finished.
- [ ] **C.3 — A misroute is the free signal.** Dictating real project notes
      makes distinctive project and library names do three jobs at once: hard
      ASR vocabulary, phonetic-dictionary entries, routing keys. A note landing
      in `unrouted/` means the ASR mangled a name — a continuous, unlabelled
      error signal that tells C.2 which entries deserve attention.
- [ ] **C.4 — Teach `--bench` to read the corpus** and report WER per mode once
      references exist.

### Routing, in support of C.3

- [ ] **C.5 — Project registry.** One list feeding dictionary, boost tree and
      router. Build it from what actually gets dictated, not from a guess.
- [ ] **C.6 — Deterministic router first.** Keyword match with an `unrouted/`
      fallback. A subagent per note is the wrong tool — each spawn starts cold
      to do a two-sentence classification.
- [ ] **C.7 — needle2 for the ambiguous remainder.** `Literal`-constrained
      project set, calibrated confidence head, route above threshold. Same
      component as E.3, second use.

---

## Phase D — The correction legs

**This is the phase that attacks the actual bottleneck.** John reviews every
sentence, which is why he never perceives the engine speed differences at all.
Reducing corrections is worth more than reducing latency. Depends on B.3.

- [ ] **D.1 — Confidence-gate the phonetic dictionary.** `murmure`'s
      `POSTCORR_CONF_THRESHOLD = 0.45` (all constants in
      `murmure-reference.md`): words the model was sure of are never
      fuzzy-corrected. This is the direct answer to the Harper problem — Harper
      split `tori` into `tor i` because it had no idea the model was confident.
      Also `POSTCORR_MAX_DICT_WORDS = 100` and length-scaled `max_distance_for`.
      Confidence is already recovered and measured discriminative: on the bench
      clip, correct words scored 0.99–1.00 and the two wrong ones (`Meg`,
      `Calendar.`) scored 0.393 and 0.570.
- [ ] **D.2 — Bigram segmentation repair.** `bigram_match` joins split
      dictionary words (`app image` → `AppImage`). Exact joins unconditional,
      fuzzy joins need both fragments below the gate. A repair, not a
      re-ranker — structurally cannot have Harper's failure mode.
- [ ] **D.0 — Point at the words to check (do this first in D).** The
      pre-neural systems underlined low-confidence words rather than silently
      correcting them; Dragon's insight was that the human is going to proofread
      anyway, so the cheap win is telling them *where*. That is this repo's
      actual bottleneck: John reviews every sentence blind. Confidence is
      already recovered and sharply discriminative — on the bench clip, correct
      words scored 0.99–1.00 while `Meg` and `Calendar.` scored 0.393 and 0.570.
      Surface the sub-threshold words in the desktop notification and the smoke
      log (**not** in the injected text — that goes into a real buffer). Turns a
      whole-sentence proofread into a two-word glance, and needs no model change
      at all.
- [ ] **D.3 — Widen the seam.** `TranscriptionService::transcribe` returns
      `Result<String, String>`; carrying confidence means a struct. Degrades
      cleanly: engines that cannot supply it return `None` and consumers fall
      back to ungated behaviour.

---

## Phase E — The command channel

Demoted deliberately. It is the most-designed and least-validated thing here —
John encountered the idea this week. Building six subtasks ahead of evidence is
the VibeVoice mistake in a different costume: 705 lines written before the
protocol was checked.

Do not start before C.2 has enough entries to evaluate the kill criterion.

- [ ] **E.1 — Port `boost_tree.rs`.** 295 lines, weighted Aho-Corasick, written
      for *greedy* decode — it adds trie scores to non-blank logits before
      argmax, which is exactly where B.3 puts you. Take the calibrated
      constants: top-K 5 at phrase start relaxing to 20 at depth 3 (*"in greedy
      decoding only near-misses are recoverable"*), alpha 3.5 → 1.0 with
      vocabulary size, backoff reimbursing abandoned partials.
      **Implement from the design, not from murmure's source:** murmure is
      AGPL-3.0-or-later and transcrust carries no licence, so a verbatim lift
      would pull the whole app under AGPL. The algorithm is NeMo's published
      GPU-PB technique and the constants are recorded in
      `murmure-reference.md`; write it fresh against those.
- [ ] **E.2 — Divergence guard, kept gated.** `murmure` runs both decodes and
      falls back above 0.35 divergence over ≥24 tokens. On TDT that doubles the
      joint's ONNX calls, so keep the length gate. (An earlier version of this
      board said the guard was free; that is true of CTC, not of a transducer.)
- [ ] **E.3 — The diamond.** Reuse the top-K gate as classifier: a command is a
      boosted decode that matches the grammar **and** whose tokens were already
      in the raw top-K. Acoustic decision, not a text heuristic. needle2 (C.7)
      is the text-level complement — the two answer different questions.
- [ ] **E.4 — Grammar + intent store.** ~300 lines, no new dependencies. Not
      before E.3 has a measured false-positive rate.

**Kill criterion:** false positives on ordinary prose above ~2% means the
channel eats your dictation. Ship push-to-talk-with-a-modifier instead.

---

## Settled

- [x] **Granite pad-to-512 is a hard graph constraint.** Axes declared dynamic,
      only multiples of 512 execute; 256/128/64/100 fail on a frozen reshape in
      `/model/encoder/layers.N/self_attn/Reshape_4`.
- [x] **The NVIDIA repos do not change the plan.** Both `parakeet-tdt-0.6b-v3`
      and `nemotron-3.5-asr-streaming-0.6b` are now `transformers`-native
      (≥ 5.13; 5.16.1 on PyPI) and ship q8_0 GGUF; neither ships ONNX. Nemotron
      is plain RNN-T (`"durations": []`) — no duration head, so likely *slower*
      per utterance than v3 despite the same 0.6 B. Its only real edge is
      streaming.
- [x] **Parakeet/Granite crossover is ~3.0 s.** Below it Parakeet is 1.5–2.9×
      faster; above it Granite pulls to 3.6×. Commands live below the crossover,
      which is a second independent reason the command channel belongs on
      Parakeet.
- [x] `design-dictation-as-control.md` annotated with the crate-boundary
      correction.
- [x] `traverse/runtime-stack.md` corrected: missing Granite export repo,
      measured pad-to-512, VibeVoice removal.

## Deferred, with reasons

| item | why |
|---|---|
| Granite as command engine | crate-boundary finding removed its only advantage; its 10.24 s quantum was already a latency problem below 3 s |
| Granite export toolchain rebuild | `~/repos/granite-speech-5.0-470m-turboctc` is **gone**; no longer on the critical path |
| Nemotron 3.5 adoption | RNN-T without a duration head; streaming is its only edge and PTT does not need it |
| Rolling our own Parakeet ONNX export | community int4 has been fine for six months; `tools/vibevoice-export/` is the template if it ever proves lossy |
| LM adaptation from John's own prose | the biggest unexploited win here and the one Dragon did best: bias decoding toward the writer's actual vocabulary and phrasing. Needs E.1's boost tree as the mechanism, plus a corpus of his writing. Real project, not a task |
| Enrollment / speaker adaptation | dropped industry-wide because large models generalise — but generalising is what you need for *many* speakers, and this is a single-speaker app with corpus capture now running. Revisit once C.2 has volume |
| s1-mini behind `Profile::Long` | real candidate, but the ordering against the phonetic dictionary is unresolved — it is a more aggressive general-English re-ranker than Harper was |
| Dictation-shape fixes for Granite | only needed if Granite becomes the *content* engine. It should not |
| VibeVoice | **removed 2026-09-06.** Lost on size, latency and quality; postmortem at `~/syncthing/vibevoice-asr-15/` |
