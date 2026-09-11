# Long-form dictation — four routes, one picker

A test plan, not a decision. The decision it is *meant* to enable is whether
transcrust ends up as two binaries' worth of behaviour behind one flag —
`transcrust` and `transcrust --long` — with Granite demoted to research. That is
the likely destination, not the premise. Nothing here commits to it.

Everything in this document is markdown. No code changes ship with it.

---

## 0. Why Nemotron is on the table at all

`Parakeet-v3.md` §8 says streaming is irrelevant because push-to-talk does not
need text appearing while you talk, and that is correct — for push-to-talk.
Toggled paragraph dictation is a different problem, and the sentence stops being
true the moment the hold becomes a toggle. **Nemotron Speech Streaming EN 0.6B**
is a cache-aware FastConformer encoder with an RNN-T decoder that processes
audio in strictly non-overlapping chunks of 80/160/560/1120 ms, reusing cached
encoder state rather than recomputing an overlapping window, so the encoder runs
*while you are still speaking* and the wall clock after you stop is roughly one
chunk plus a tail decode — constant, not proportional to how long you talked.
Against a measured Parakeet direct-driver RTF of 0.11–0.13 that is ~0.3 s
against ~36 s on a five-minute capture, and unlike the Granite crossover the
margin never plateaus, because this is a latency win (when the compute happens)
rather than a throughput win (how much compute there is) — Nemotron's total
compute is very likely *higher*, since it has no duration head and a 1025-token
vocabulary against Parakeet's 8193. It ships native punctuation and
capitalisation, which is the larger prize: the only reason `Profile::Long`
exists is that Granite emits bare lowercase, so an engine that emits finished
prose deletes the repair pass rather than improving it. It costs English-only
(fine for this user), the NVIDIA Open Model License instead of CC BY 4.0, about
8% relative WER against Parakeet on the OpenASR average, and a 5.68-second
attention memory (`sliding_window: 71` × 80 ms) where Parakeet has full
attention across the whole utterance — that last one is the trade this plan
exists to measure, because no public benchmark scores a five-minute continuous
context and the corpus can.

**Sources.** Chunk sizes, `sliding_window`, vocab and licence from the model
card and `config.json` **[card]/[config]**. Parakeet RTF 0.11–0.13 from
`TASKBOARD.md` B.1/B.3 **[measured]**. Granite RTF 0.062 derived from D.4's
36:35-in-2:17 **[measured]**.

**One stale number this corrects.** D.4 reads *"a five-minute capture is 63 s of
Parakeet against ~43 s."* That 63 s is the `parakeet-rs` crate at RTF 0.21.
B.3 made the direct driver the live path at 0.11–0.13, so it is **~36 s**, and
Granite's real margin at five minutes is ~2×, not the 3.6× the crossover note
implies.

---

## 1. A naming collision to fix before it spreads

Two different things are called "long" and this plan makes the overlap worse:

| name | what it is | where |
|---|---|---|
| `--long` | a **run mode**: capture starts and stops on `transcrust --toggle` (SIGUSR1) instead of on a key hold | `main.rs`, `RunMode::long` |
| `Profile::Long` | a **text profile**: `restore_contractions` over the engine's raw output | `mode.rs` |

They are orthogonal. `Granite — Long` is a profile; `transcrust --long` is a
capture mode. Route 4 below is `--long` capture with `Profile::Raw`, which reads
as a contradiction and is not one.

**Recommendation, not in this PR:** rename `Profile::Long` → `Profile::Repair`.
`mode.rs`'s own doc comment already describes it as "long-form repair for
engines that emit bare, system-shaped text" — the repair is the noun. If
`transcrust --long` becomes the shipped second binary-shaped flag, the collision
becomes user-facing.

---

## 2. The four required routes

All four must be reachable and measurable before anything is chosen. Three
already exist; the work is route 4 and the picker.

| # | route | capture | profile | engine | status |
|---|---|---|---|---|---|
| 1 | **Parakeet, hold** | key hold | `Raw` | Parakeet TDT v3 | **ships today** — the default, the control |
| 2 | **Parakeet, `--long`** | toggle | `Raw` | Parakeet TDT v3 | **works today**, undocumented as a route |
| 3 | **Granite, `--long`** | toggle | `Long` | Granite TurboCTC | **works today** (`Granite — Long` mode) |
| 4 | **Nemotron, `--long`** | toggle | `Raw` | *not present* | **to build** |

### Route 1 — Parakeet, hold (standard)
The reference point. Nothing changes. It is in the matrix because every latency
and WER number below is meaningless without it, and because if routes 2–4 all
lose, this is what ships unchanged.

### Route 2 — Parakeet, `--long` (comparative)
Already reachable: run the daemon with `--long`, leave Parakeet selected. This
is the honest control for route 4, and the one most likely to be skipped because
it feels like a null result. It is not. It isolates **capture mode** from
**engine**: if toggled Parakeet is already tolerable at paragraph length, the
Nemotron case collapses to "0.3 s instead of ~1–36 s," which is a comfort
argument and should be argued as one rather than as a necessity.

It also establishes the accuracy ceiling for the other two, because it is the
only route with full attention across the entire capture.

### Route 3 — Granite, `--long`
Already reachable as the `Granite — Long` mode. Carries the repair pass, and
therefore carries its known wall: `Profile::Long` gets 13.71% → 10.11% WER on
contractions and then stops dead at possessives (`today is sponsor`), because
`today is` → `today's` is not safely reversible. D.4 proposes s1-mini for the
residue. **This route is in the matrix to be beaten**, and the likely outcome —
stated in the brief, so it is not a surprise later — is that Granite becomes
`--wav` research only.

### Route 4 — Nemotron, `--long`
Does not exist. Needs a `ModelKind`, an engine, a discovery keyword, and an ONNX
export that survives §3. The profile is `Raw`: if it needs a repair pass, the
route has failed, because not needing one is the reason it is here.

---

## 3. Testing whatever ONNX you find — acceptance gates

There are several Nemotron exports in circulation and they will not agree. Do
not adopt one because it loads. Run the gates in order; the first failure stops
the route rather than starting a workaround.

### Gate 0 — the corpus must exist first

**`observe.corpus` is `false` and the corpus is 0 WAVs.** Every accuracy gate
below (G3, G5, G6) reads from it. Nothing in this plan can be answered until it
is on and has banked real dictation, ideally including several paragraph-length
captures, since paragraph-length is the thing being tested.

This is not a prerequisite that can be worked around with bench clips.
`tools/bench-clips/` is 48 kHz interview audio and the device is 44100 Hz 2ch;
its own README says treat it as timing fixtures only. The corpus banks mono f32
**at the device rate, pre-resample**, through the live path — which is the only
audio that exercises `audio.rs`'s polyphase decimator the way a real dictation
does.

Fill `reference` only on entries that came out wrong. A failure-weighted corpus
is the only kind that gets finished (C.2).

### Gate 1 — probe the graph before running it

`--probe-onnx` prints graph inputs and outputs. Every export-naming difference
in the Moonshine work was found with it rather than guessed; use it first.

What you are looking for decides which route the export can serve:

| graph shape | what it is | serves |
|---|---|---|
| encoder in → encodings out, no cache tensors | an **offline/batch** export | route 4's *output shape* and *WER* only — no latency benefit |
| encoder + ~72 cache inputs and ~72 cache outputs | a **cache-aware streaming** export | the full route 4 |

Both are worth testing and they answer different questions. A batch export is
still useful: it settles the punctuation and WER questions cheaply, against the
existing batch path, with no streaming loop written. **Do that first if both are
available** — it is most of the answer for a fraction of the work.

Also record from the probe: whether decoder and joint are fused
(`decoder_joint-model.onnx`, Parakeet's layout) or split
(`decoder` + `joiner`, the sherpa-onnx layout). Split means a three-session step
loop rather than two.

### Gate 2 — declared dynamic is not honoured dynamic

This repo has been bitten twice. Granite's axes are declared dynamic and only
multiples of 512 execute; VibeVoice's encoder returned the traced frame count
for any input. **Verify shapes by running the graph, not by reading its
metadata.**

For a streaming export, run every chunk size the card claims — lookahead
`{0, 1, 6, 13}`, i.e. 1/2/7/14 frames of 80 ms — and confirm each executes and
produces the frame count it should. An export that only runs at its traced
lookahead is a batch export wearing a streaming graph, and fails this gate.

### Gate 3 — the mel is shared, but prove it

Parakeet TDT v3 and Nemotron declare byte-identical feature extractors: 128 mel,
hop 160, n_fft 512, win 400, preemphasis 0.97, 16 kHz **[config, both
`processor_config.json`]**. So `nemo128.onnx` — the graph whose use made the
direct driver 35–43% faster — should feed Nemotron unchanged.

Should. Verify numerically against the export's own expected input before
building on it. And note that streaming needs the mel computed **incrementally**
with frame alignment carried across chunk boundaries
(`start_idx = mel_frame_idx * hop_length - n_fft // 2`), not one shot over the
whole buffer — the graph is reusable, the calling code is not.

### Gate 4 — vocabulary, blank, and discovery

- Vocab is **1025** tokens against Parakeet's 8193, and `blank_token_id` is
  **1024** **[config]**. Read both from the export's own config; do not carry
  Parakeet's constants across.
- The tokenizer file may arrive as `vocab.txt` (Parakeet layout), `tokens.txt`
  (sherpa layout) or `tokenizer.json`. `model.rs`'s discovery is
  family-keyed and greedy — the directory name nominates via
  `FAMILY_KEYWORDS`, `model_kind` confirms by contents. Adding Nemotron is a
  third keyword, a third `ModelKind`, and a `pick_onnx` arm; this is exactly the
  extension point the family-keyed design was built for.
- `max_symbols_per_step` is 10 on both.

### Gate 5 — output shape, which is the whole point

Run the corpus through it and read the raw text, before any profile.

Pass: contractions, casing, sentence punctuation, spelled-out numerals — the
shape §4 of `Parakeet-v3.md` measured on Parakeet. Fail: anything that would
need `Profile::Long`. **If this gate needs a repair pass, route 4 is dead**,
because deleting the repair pass is its main argument and the latency is the
secondary one.

### Gate 6 — WER against the corpus, paired

Replay corpus WAVs through each route and score against filled `reference`
fields. C.4 (teach `--bench` to read the corpus) is the wiring this needs and is
already on the board.

Score **paired on identical audio**, not as two independent aggregates. From the
error-budget arithmetic: at a realistic 2–3% dictation WER you must speak
300–500 words to save one word of correction, so an unpaired comparison on a
small corpus will not resolve anything. Paired, roughly **2,000 hand-corrected
words (~15 minutes of dictation)** is enough to see a ~10% relative difference.
That is the corpus target — and it is why gate 0 is not optional.

Expect Nemotron to lose here. The OpenASR average is 6.93% at its slowest chunk
against Parakeet's 6.34%. The question is not whether it loses but whether it
loses more on *your* five-minute paragraphs than on segmented benchmarks, which
is the 5.68-second-memory hypothesis and the one thing the corpus can answer
that no leaderboard can.

### Gate 7 — timing, and the failure mode batch does not have

`--bench` gives batch wall-clock. For a streaming route that is the wrong
measurement: what matters is whether it **holds cadence over a sustained
capture**. Run a ten-minute stream with a compile and a browser running. Batch
degrades gracefully — you just wait longer. Streaming degrades by falling behind,
growing a backlog, and then making you wait anyway, having also spent the RAM.

Load is not a differentiator and should not be measured as one: same encoder,
same parameter count, ~1% smaller weights from the smaller vocab, so ~1.4 s at
int4 against Parakeet's measured 1.43 s. **Load tracks bytes, so the
quantisation the export lands on matters far more than the model** — int8 or
q8_0 roughly doubles it. Note also that in a toggled streaming route the load is
hidden behind the first ~2 s of speech rather than felt after it, which inverts
the tap-to-talk finding that cold load is the metric. For this mode, steady-state
RTF is the metric.

### Gate 8 — licence

Per export, not per model. The base is the NVIDIA Open Model License; a
community quantisation may add its own terms. CC BY 4.0 this is not.

### Kill criteria for route 4

Any one of these ends it, and the route is recorded as closed rather than
carried:

1. No available export honours its declared chunk sizes (gate 2).
2. Raw output needs a repair pass (gate 5).
3. Paired corpus WER is worse than Parakeet by more than ~15% relative (gate 6).
4. Post-toggle wait is not under ~1 s on a five-minute capture, or cadence is
   not held over ten minutes (gate 7).

If route 4 dies, route 2 is the fallback and the conclusion is that toggled
Parakeet was the answer all along — which is a result, cheaply obtained, and the
reason route 2 is required rather than optional.

---

## 4. Wiring the four routes into the UI with fuzzel

The tray already owns mode activation and the main loop already owns the swap.
The picker is a second front end onto that same contract, not a new mechanism.

### Why fuzzel

`fuzzel --dmenu` reads newline-separated items on stdin, shows a Wayland-native
layer-shell fuzzy picker, and writes the chosen line to stdout with a non-zero
exit on cancel. No toolkit, no window in the compositor's normal stack, no GUI
dependency added to a keyboard-driven desktop. It is bindable from labwc
`rc.xml` exactly like the existing chords, which matches the convention
`config.example.toml` already documents.

### The contract it must reuse, unchanged

`traverse/runtime-stack.md` is the authority here and none of it is negotiable:

- Feed it `mode::discover_modes()` labels. They are already unique strings and
  already correct (`Granite Speech 5 470m TurboCTC — Long`).
- Map the selection back to an index and send on the **existing**
  `mpsc::UnboundedSender<usize>` that `tray.rs` uses. The picker never builds a
  model, same as the tray.
- The switch is still **refused unless Idle**, with the "Busy" notification.
- Every path that declines a switch still sends on the `watch` channel, or the
  tray's radio will claim an engine that never loaded.

Prefer `fuzzel --dmenu --index`, which returns the 0-based index directly and
removes string matching from the path entirely — **verify it exists in the
installed version** (`fuzzel --help`) before depending on it; the fallback is an
exact-string match against the label list, which must then not be prettified.

### How it gets triggered

SIGUSR1 is taken by `--toggle`. Use **SIGUSR2** for `--pick`, mirroring the
existing pattern exactly — including its hard-won lesson, recorded at
`main.rs:550`: the default disposition for both signals is *terminate*, so a
daemon that does not handle SIGUSR2 **dies silently** the first time someone runs
`transcrust --pick`. Register it unconditionally, the way SIGUSR1 already is.

The daemon spawns fuzzel, not the CLI: the daemon is already inside the user's
Wayland session, and it is the only process that knows the current mode list.
Spawn it as a tokio child so the `select!` loop is never blocked on a picker the
user walked away from.

### Edge cases worth writing down now

- **fuzzel absent.** Log, notify, continue. Never crash a dictation daemon over a
  missing picker. `--doctor` should report its presence alongside the model list.
- **Cancelled.** Non-zero exit is a no-op, not an error state.
- **One mode installed.** The tray hides its submenu below two entries; the
  picker should do the same rather than showing a one-item list.
- **Marking the active mode.** dmenu has no "current" concept. With `--index`,
  prefix the active row with a marker and strip nothing, since the index is what
  comes back. Without it, do not prettify labels at all.

### Kill criterion

If the picker cannot be built against the existing `mpsc` + `watch` contract
without restructuring how modes are activated, stop and re-scope. The value here
is a second front end onto a working mechanism; a picker that forces the
mechanism to change has stopped being worth ~30 lines.

---

## 5. The measurement matrix

One table, filled once gate 0 has volume. This is the artefact the whole plan
exists to produce.

| route | post-toggle wait, 5 min | paired WER vs corpus | raw output needs repair? | confidence signal |
|---|---|---|---|---|
| 1 — Parakeet, hold | n/a (hold) | | no | yes (0.75 gate, live) |
| 2 — Parakeet, `--long` | ~36 s | | no | yes |
| 3 — Granite, `--long` | ~19 s | | **yes** | no |
| 4 — Nemotron, `--long` | ~0.3 s (claim) | | ? | ? |

The latency column is projected from measured RTFs; everything else is empty on
purpose.

---

## 6. What this plan is not

- **Not a commitment to drop Granite.** It stays permanently for `--wav` — 36:35
  in 2:17 against Parakeet's 7:37 — and D.4 is unaffected. The likely outcome is
  that it stops being a *dictation* route, which is a narrower claim.
- **Not a restructure of `TASKBOARD.md`.** The board stays the authority on
  ordering. This document is the route map it can point at.
- **Not an ONNX export project.** It assumes exports are found, and tells you how
  to reject the bad ones.
- **Not E.1.** The boost tree still owns the proper-noun errors, which are the
  errors that actually cost a re-read (`CentOS 0.42`, `CentaWes. 0.27`,
  `RHEL 0.64`). Nothing in this plan touches them, and no engine swap will.
