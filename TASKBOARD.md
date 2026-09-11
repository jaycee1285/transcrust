# TASKBOARD

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

> **One board.** This file is everything live: the phases, what is settled, what
> was deferred and why, the operational backlog, and the standing risks. What has
> already shipped is `DONE.md`.
>
> Merged 2026-09-10 from the old `TASKBOARD.md` + `TASKBOARD-next.md`. The old split was
> not history-versus-live — both files held open items, which is how two updates
> in one day each landed on only one of them and a third landed on neither.

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
      Card: <https://huggingface.co/Cactus-Compute/needle2>.

---

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

      The original reasoning, which still stands:

      The
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
- [ ] **D.4 — s1-mini as the readability pass for Granite's `--wav` output.**
      **Scoped to batch, not to dictation.** Granite is in the tree permanently
      for `--wav`: 36:35 of YouTube in 2:17 against Parakeet's 7:37. It is not a
      candidate that has to earn its keep, so the marginal cost here is s1-mini
      alone.

      The problem it solves is readability, not accuracy. Granite emits no
      punctuation and no casing, so a long transcript arrives as one run-on per
      window:

      > *whether you are using expensive best in class stuff like fable 5 or
      > surprisingly cheap and effective stuff like deep seek v 4 flash it is
      > hard to go wrong but what if you are would not it be nice to know*

      `Profile::Long` gets 13.71% → **10.11%** WER on contractions alone, for no
      measurable time, and then stops at the possessive: `today is sponsor`,
      `when is the last time`. `today is` → `today's` is not safely reversible
      (`today is Tuesday`), the same wall `mode.rs` documents for `I have`.
      s1-mini's `Styling` axis *is* that decision, and it resolves false starts
      no rule can reach.

      **Batch removes the risk that made this hard.** The unmeasured number was
      s1-mini's token rate, and interactively it swings the answer from "never
      wins" to "wins after 26 s". Nobody waits on a `--wav` run, so it does not
      matter here. Latency stops being a variable and the question reduces to
      one thing: does the output read better?

      **Test with what already exists.** `~/repos/transcrust` holds Granite
      `--wav` transcripts of two long talks, and `tools/wer/` compares them.
      Run s1-mini over a Granite transcript, diff against the Parakeet one, and
      read both. No wiring required to answer it.

      **Do not extend this to dictation without a separate argument.** Live,
      Granite costs roughly one extra wrong word per 22-second sentence and
      forfeits the confidence signal that reached the live path on 2026-09-10 —
      CTC posteriors could supply it, but nobody has written that. Reducing
      corrections beats reducing latency here, and Granite trades the wrong way.
      `--long` is the one dictation case worth revisiting, because a five-minute
      capture is 63 s of Parakeet against ~43 s, and that gap is felt.

      **Ordering is still the open question.** s1-mini is a more aggressive
      general-English re-ranker than Harper, and Harper was removed for starving
      the phonetic dictionary. Before the dictionary it repeats that; after it,
      it may undo the dictionary's corrections. Decide by measurement.
- [ ] **D.3 — Widen the seam.** `TranscriptionService::transcribe` returns
      `Result<String, String>`; carrying confidence means a struct. Degrades
      cleanly: engines that cannot supply it return `None` and consumers fall
      back to ungated behaviour.

---

---

## Phase E — The command channel

> **Largely superseded 2026-09-10 by `~/repos/dayshade/spec.md`.** The command
> channel was designed as a decode-time capability inside transcrust on the
> desktop. The use case that actually wanted it — logging time and habits into
> DayLight — turned out to live on the phone, behind a Quick Settings tile, with
> a different engine and an explicit grammar. That removes E.3 and E.4 from this
> repo: intent classification is unnecessary when the utterance is
> `<name> <verb> <detail>` and the name resolves against ~35 candidates.
>
> **E.1 survives and is still the largest item on this board**, because it is not
> really about commands — it is decode-time vocabulary biasing, which is Dragon
> mechanism 2 and the direct answer to the proper-noun errors measured on
> 2026-09-10: `Orang`/Orion, `terrake`/Terakeet, `at track`/Apptrack,
> `sigs`/Cigs. E.2 only matters if E.1 lands.

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

---

## Settled

- [x] **Moonshine is the command engine; Parakeet stays the dictation engine.**
      Measured 2026-09-10 on a 5 s command from real phone audio: Moonshine at
      0.57 s cold load plus 0.17 s inference (RTF 0.033), against Parakeet at
      ~0.9 s plus ~1.0 s. It emits digits where Parakeet emits number words, and
      it transcribed `Orion Laundry` correctly where Parakeet produced `Orang`
      and flagged it at 0.46. Two limits keep it out of dictation: it truncated
      partway through a 60 s clip, and it repeated a phrase on a 5 s one — the
      autoregressive hallucination a transducer structurally cannot produce.
      `src/moonshine.rs` and `--moonshine <dir> <wav>`. The **split-decoder**
      export is required: the merged one fuses across its own `optimum::if`
      under ORT and dies on the first token.
- [x] **Cold load, not RTF, is the metric for a tap-to-talk surface.** A tile is
      cold every time, so load dominates the wall clock on a 1-5 s utterance.
      `--bench` reports both columns; read the load one.
- [x] **`--probe-onnx` prints graph inputs and outputs.** Every export naming
      difference in the Moonshine work was found with it rather than guessed.

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

---

## Deferred, with reasons

| item | why |
|---|---|
| Granite as command engine | crate-boundary finding removed its only advantage; its 10.24 s quantum was already a latency problem below 3 s |
| Granite export toolchain rebuild | `~/repos/granite-speech-5.0-470m-turboctc` is **gone**; no longer on the critical path |
| Nemotron 3.5 adoption | RNN-T without a duration head; streaming is its only edge and PTT does not need it. **Reopened 2026-09-11 for the toggled long-form case, where streaming is the point — see `design-long-form-routes.md`.** Blocked on C.2: every accuracy gate there reads the corpus |
| Rolling our own Parakeet ONNX export | community int4 has been fine for six months; `tools/vibevoice-export/` is the template if it ever proves lossy |
| LM adaptation from John's own prose | the biggest unexploited win here and the one Dragon did best: bias decoding toward the writer's actual vocabulary and phrasing. Needs E.1's boost tree as the mechanism, plus a corpus of his writing. Real project, not a task |
| Enrollment / speaker adaptation | dropped industry-wide because large models generalise — but generalising is what you need for *many* speakers, and this is a single-speaker app with corpus capture now running. Revisit once C.2 has volume |
| s1-mini behind `Profile::Long` | **promoted to D.4 on 2026-09-10**, now that the deterministic baseline it has to beat is measured (10.11% WER) and the residual is characterised |
| Dictation-shape fixes for Granite | only needed if Granite becomes the *content* engine. It should not |
| VibeVoice | **removed 2026-09-06.** Lost on size, latency and quality; postmortem at `~/syncthing/vibevoice-asr-15/` |

---

## Operations — pending, and needing John
- **The command channel moved.** It is now `~/repos/dayshade/spec.md` — a Quick
  Settings tile on Android that writes DayLight Markdown directly. What stays in
  this repo is `TASKBOARD.md` **E.1**, the decode-time boost tree, which was
  never really about commands: it is vocabulary biasing, and it is the answer to
  the proper-noun errors this repo actually has.
- **Ship what is built.** Five commits are local-only and the published release
  asset is now seven behind: `./release.sh`, then
  `git push -u origin engines-and-corpus`, then check the `tauri.nix` entry in
  the config repo against the new asset.
- **Turn on `observe.corpus`.** It is `false`, so nothing is banked and the
  corpus is empty. C.2, C.4, the A.2 re-run against real microphone audio, and
  Dragon mechanism 1 all wait behind one boolean.
- **Bump yt-dlp.** The system binary is `2025.12.08` against nixpkgs
  `2026.08.19`, and the stale one 403s on every download — so anything shelling
  out to it is broken, not just the new skills.
- If a grammar/punctuation pass is wanted back, build a small purpose-built deterministic one (or the future small-LM toggle) rather than re-adding Harper.
- Add explicit first-load tray/icon feedback so users can see model warmup instead of only paying hidden latency on first transcription.
- Tighten startup/log ergonomics so steady-state smoke logs stay high-signal.
- **Log path is cwd-relative.** `observe.rs` builds `./logs/latest.log` from
  `current_dir()`, so a tray- or systemd-launched instance writes somewhere
  arbitrary and `--doctor` does not say where. `~/.local/state/transcrust/` is
  the XDG-correct home; two lines plus a `--doctor` line.

---

## Risks
- Pure Nix builds still need scrutiny because `ort` binary provisioning is touchy across environments.
- Tray/status behavior depends on SNI/DBus availability in the running desktop session.
- Feature brittleness remains around cross-session tray rendering and first-use model warmup UX because the repo does not control the status-notifier host theme path.
