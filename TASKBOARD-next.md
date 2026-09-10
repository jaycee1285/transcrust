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

- [x] **A.1 — Decimate properly.** Done 2026-09-09. Windowed-sinc via a
      polyphase bank, no new dependency. Cutoff 0.45 × the lower Nyquist, 16
      zero crossings, Blackman window. The bank is built once per rate pair —
      44100→16000 reduces to 160 phases — so the inner loop is a dot product
      with no transcendentals. **Cost: 18 ms for a 10-second clip**, 1.8 ms per
      second of audio, which is nothing on either the dictation or `--wav` path.
      - Tests pin the defect rather than the implementation:
        `alias_band_is_rejected` requires ≥40 dB at 10/12/14 kHz. The replaced
        code measures −1.5/−2.1/−2.8 dB there, reproducing the table above from
        an independent implementation, so the bar discriminates by a factor of 79.
- [x] **A.2 — Prove it on real audio.** Done 2026-09-09, and the answer is
      **the resampler is not the proper-noun problem.**
      - Material: an 18-minute technical talk, native 48 kHz, resampled to
        44.1 kHz with soxr, then fed through both resamplers into Parakeet. The
        corpus is still empty (0 WAVs), so this is *not* the replay the item
        asked for — see the caveat below.
      - **The change is real in the transcript**: 8 differing spans, 0.87% of
        words. So the kill criterion as written does not fire.
      - **The direction is not measurable.** Against yt-dlp's auto-captions:
        old 7.43%, new 7.39% WER — a one-word difference across 2,673 words.
        Against a soxr-16 kHz control the old resampler was *closer* (1.64% vs
        1.90%). Inspecting the spans is a wash: `rhe`→`rel` is better,
        `whilst there's`→`while still` is worse.
      - **Keep the change anyway.** Its stated purpose was removing a confound
        from every later measurement, and it does that. It is simply not itself
        an improvement, and nothing downstream should be justified by it.

**Why first:** it is not the highest-value item in isolation. It is the item
that makes every later measurement mean something. Improve the dictionary on
top of a broken audio path and you cannot tell which change helped.

**The caveat that keeps A.2 half-open.** The audio above is YouTube-sourced,
band-limited by a lossy codec long before it reached 44.1 kHz, so it may simply
not carry the 10–14 kHz energy the aliasing argument depends on. A real
microphone in a real room does. Re-run this against actual `--record` captures
once C.2 has volume; until then, read the result as *"not demonstrated on
codec-limited speech"* rather than *"the resampler never mattered."*

---

## Phase B — Resolve the direct driver

`src/parakeet_ort.rs` exists, works, and has 10 passing tests. **Nothing calls
it** except `--parakeet-direct`; the live path still goes through
`parakeet-rs`. A parallel implementation that nothing uses rots silently.

- [x] **B.1 — Measure it.** Done 2026-09-09. `--bench` now emits a fourth row
      that bypasses `TranscriptionService` deliberately — the question is what
      the crate boundary costs, so measuring through the seam the crate sits
      behind would measure nothing. On 45 s and 90 s of the same talk:

      | clip | crate | direct | |
      |---|---:|---:|---|
      | 45 s | 7.59 s (RTF 0.17) | **4.34 s (RTF 0.10)** | 43% faster |
      | 90 s | 17.94 s (RTF 0.20) | **11.58 s (RTF 0.13)** | 35% faster |

      **The kill criterion fires in the opposite direction.** It guarded against
      >15% *slower*; the direct driver is 35-43% faster. Likely the mel: the
      crate computes it in Rust, the driver runs NeMo's own `nemo128.onnx`.
- [x] **B.2 — Reconcile the one-token divergence.** Done 2026-09-09, and the
      answer is better than "benign". Byte-identical on the 45 s clip. The 90 s
      clip diverges once, and **the direct driver is the correct one**: the
      crate writes `Centaurus Stream 10` where the driver writes
      `CentOS Stream 10`. Confidence flags the same span — `CentOS` scores 0.42
      and a neighbouring mangle `CentaWes.` scores 0.27, against >0.75 for
      ordinary words.
- [x] **B.3 — Then choose, and actually do it.** Done 2026-09-09. `parakeet.rs`
      now loads an `Engine` that prefers the direct driver and falls back to the
      crate, so there is one live path rather than two implementations and a
      flag. Both arms verified through `--wav`, which drives the same worker:
      - **Direct**, when `nemo128.onnx` is present: RTF **0.11** on a 45 s clip
        against the crate's 0.17, and `transcription.confidence` in the log
        (`EUL (0.52)`).
      - **Crate**, with the graph absent (tested via a symlink farm, no
        `nemo128.onnx`): RTF 0.19, logs *"no nemo128.onnx; using parakeet-rs (no
        confidence signal)"*, and produces a **byte-identical transcript**.
      - A present-but-broken graph set reports the error and falls through
        rather than stranding the user with no dictation.
      - `CONFIDENCE_GATE = 0.75` and the log line are D.0's substrate. The
        notification half is still D.0's own work.

**Kill criterion:** >15% slower than the crate → profile before proceeding.
**Result: cleared.** 35-43% faster, so nothing to profile.

**What B.1 incidentally proved.** The confidence column is Dragon mechanism 5
running live: on 90 s of speech every word above 0.75 was right, and the ones
below were `CentOS 0.42`, `CentaWes. 0.27`, `RHEL 0.64`, `Alma 0.69`,
`Hadron 0.68` — the proper nouns, which is exactly the class of error that
actually costs John a re-read. **D.0 is now a display problem, not a research
problem.**

---

## Phase C — Ground truth, earned rather than sat down for

- [x] **C.1 — Corpus capture.** Built. `observe.corpus = true` banks each
      dictation as WAV + JSON under `~/.local/share/transcrust/corpus/`, through
      the live path, pre-resample, at the device's real 44100 Hz. `--record`
      banks audio alone.
- [ ] **C.2 — Fill `reference` only where it came out wrong.**
      **Blocked on a config flag, not on effort**: `observe.corpus` is `false` in
      John’s config, so nothing is being banked and the corpus is empty (0 WAVs
      as of 2026-09-10). Turn it on and it accumulates passively from ordinary
      dictation. Several things wait on this: C.4, the A.2 re-run against real
      microphone audio rather than codec-limited YouTube, and Dragon mechanism 1.
      A
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

**Unblocked 2026-09-09 by B.3.** Confidence now reaches the live path.

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
- [ ] **D.0 — Point at the words to check (do this first in D).**
      **Unblocked and now nearly free.** B.3 put confidence on the live path and
      `parakeet.rs` already logs `transcription.confidence` with the
      sub-threshold words; `CONFIDENCE_GATE` is 0.75. What remains is a surface.
      - **The plan's assumed surface does not work for John**: his config sets
        `observe.desktop_notifications = false`, and `Observer::notify` returns
        early on that flag, so notification-based surfacing is a no-op for the
        one user. Pick a surface he actually sees — the tray tooltip, a
        state, or an opt-in that turns notifications on for this alone.
      The original reasoning follows.
- [ ] **D.0 (original note).** The
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
