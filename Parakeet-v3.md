# Why Parakeet TDT 0.6B v3 wins on-device

A reference for anyone building single-user, on-device speech control who is
about to reach for Whisper because everyone else did.

This is not a benchmark writeup. It is the set of *architectural* reasons this
model fits the job, most of which are invisible on a WER leaderboard, all of
which were either measured on one 12-core desktop CPU running `transcrust`, or
read out of the checkpoint's own config. Sources are marked: **[measured]**,
**[config]**, **[card]**.

---

## 0. The one-paragraph version

Push-to-talk device control is a **short-utterance, low-latency, no-network,
no-supervision** problem. Almost every open ASR model is built for a different
problem — long-form batch transcription with a human proofreader downstream.
Parakeet TDT v3 is one of the few that is shaped for the first problem, and the
reasons are structural rather than a matter of accuracy. If you take one thing:
**the binding constraints here are utterance latency, silence behaviour, and
output shape. None of the three is WER.**

---

## 1. No fixed window — cost tracks what you actually said

Whisper pads every input to a **30-second** window. A 2-second command costs 30
seconds of encoder work. That is not a tuning issue, it is the architecture:
the positional embeddings and the encoder are built for 1500 frames.

This is not an abstract concern. Measured on the same machine **[measured]**:

| engine | cost model | 2 s command costs |
|---|---|---|
| Parakeet TDT v3 | linear in duration | ~0.4 s |
| Granite TurboCTC | `ceil(dur / 10.24 s) × 0.53 s` | 0.50 s (a full 10.24 s block) |
| Whisper (any size) | fixed 30 s window | 30 s of encoder work |

Granite's quantum comes from a hard 512-frame constraint baked into the graph —
its axes are *declared* dynamic but only multiples of 512 execute; 256, 128 and
64 all fail on a frozen reshape in self-attention. Parakeet has no such
quantum. Measured RTF stayed flat at **0.17–0.20 across 5, 10, 20, 40 and 60
second clips** **[measured]** — cost is proportional to audio, with no floor and
no step function.

For a device that wakes on a two-word command, this is the single most important
property in the document.

## 2. TDT: the decode is sublinear in tokens

`"durations": [0, 1, 2, 3, 4]` **[config]**. Token-and-Duration Transducer.
Alongside each token the model predicts how many encoder frames to *skip*, so
the decoder does not step once per frame — it jumps.

Why this matters more than it sounds: a transducer or an encoder-decoder LLM
pays one decoder invocation per emitted token, and on CPU each of those is a
separate kernel launch with fixed overhead. Frame-skipping collapses the number
of invocations. It is the difference between a decode loop that is a rounding
error and one that dominates.

Contrast, measured on the same box **[measured]**:

- **VibeVoice ASR 1.5B** (autoregressive Qwen2 decoder): RTF **1.1–1.6**, and
  crucially the cost scaled with *how much was said*, not clip length — dense
  English cost 2–3× what sparse speech did.
- **Nemotron 3.5 ASR streaming 0.6B**: `"durations": []` **[config]** — plain
  RNN-T, no duration head, no frame skip. Same 0.6 B parameter count, but it
  emits per frame. Same size is not the same speed.

If you are comparing two 0.6 B models, **check for a duration head before you
check the WER table.**

## 3. A transducer cannot hallucinate a paragraph into silence

Whisper's silence hallucination is well documented and has caused real trouble
in clinical transcription — it is an encoder-decoder language model, and a
language model handed nothing will happily generate a fluent something.

A transducer emits per frame, conditioned on the acoustics of that frame. Given
silence it emits blank. There is no generative prior to run away with.

For push-to-talk this is a **correctness** property, not a quality one. You
*will* release the key early. You *will* trigger on a cough. The failure mode
you want is "nothing," not "Thank you for watching."

## 4. It speaks grown-up English natively — no reconstruction pipeline

`Automatic punctuation and capitalization` **[card]**, and in practice also
contractions and spelled-out numerals. Measured side by side on the same
20 seconds of audio **[measured]**:

| engine | output |
|---|---|
| Parakeet | `It's awesome. I can't wait... I've heard three songs` |
| Granite | `it is awesome I can not wait ... I have heard 3 songs` |

Granite is not wrong. It is built to feed a system, not a human — IBM's cut
assumes a downstream normalisation pipeline exists. If you pick it for
dictation you inherit the job of writing that pipeline: contraction
restoration, digit spelling, casing, sentence segmentation. All mechanical, all
deterministic, all code you maintain forever.

**Parakeet's constraint is the feature.** It was trained on properly-spoken
input and it rewards speaking properly. Talk like a grown-up, get grown-up
results. That is why a transcrust-shaped app needs *zero* post-processing on
Parakeet beyond the user's own vocabulary dictionary — and why the whole
`murmure` correction stack (confidence gating, bigram repair, phrase boosting)
was never ported: it is scaffolding to rebuild output shape Parakeet emits for
free.

## 5. The logits are already there — most wrappers just hide them

**This is the finding that costs people the most time, so it goes in the
reference.**

The `parakeet-rs` crate (and most wrappers) run the decode internally and hand
back a `String`. That makes it *look* like Parakeet exposes no confidence
signal, which in turn makes people reach for a CTC model when they want
confidence-gated correction or command classification.

It is not true. The ONNX export is two graphs:

```
encoder-model.onnx        mel [1, T, 128]        -> encodings [1, T/8, 1024]
decoder_joint-model.onnx  encoding + pred state  -> vocab logits + duration logits
```

The joint network's vocab logits are right there. Driving the two sessions
directly with `ort` gets you per-token probability, word-level confidence and
timestamps out of **the same files already on disk** — no re-export, no
different model. `murmure`'s `src-tauri/src/engine/engine.rs` is a working Rust
reference that does exactly this and never depends on `parakeet-rs` at all.

If a design doc in your repo says "Parakeet exposes no logits," it is describing
a crate boundary, not the model. Ours did. It was wrong, and it sent an entire
plan toward the wrong engine.

## 6. Everything else that matters for the device case

| property | value | source |
|---|---|---|
| parameters | 600 M | [card] |
| int4 ONNX on disk | **391 MB** | [measured] |
| cold load | **1.43 s** | [measured] |
| licence | **CC BY 4.0** — commercial use OK | [card] |
| languages | **25 European**, auto-detected, no prompt needed | [card] |
| timestamps | word-level and segment-level | [card] |
| tokenizer | one SentencePiece vocab, 8192 tokens, shared across all 25 languages | [card] |
| encoder | 24 layers, hidden 1024, subsampling ×8, 128 mel | [config] |
| decoder | 2 layers, hidden 640 — *tiny* | [config] |
| max audio | 24 min full attention, 3 h local attention | [card] |
| frontend | 128 mel, hop 160, n_fft 512, win 400, preemphasis 0.97, 16 kHz | [config] |

Three of those deserve a sentence:

- **Big encoder, 2-layer decoder.** The expensive half runs once over the
  audio; the cheap half runs per token. That ratio is exactly backwards in an
  encoder-decoder LLM, and it is why this model is fast on a CPU that has no
  business running ASR.
- **One tokenizer for 25 languages, auto-detected.** No language-selection UI,
  no per-language model download, no prompt id. For a shipped device that is a
  whole category of configuration that never has to exist.
- **CC BY 4.0.** Not a research licence. You can ship it.

## 7. The frontend is shared with the rest of the family

Parakeet TDT v3 and Nemotron 3.5 ASR declare **identical** feature extractors —
128 mel, hop 160, n_fft 512, win length 400, preemphasis 0.97, 16 kHz
**[config]**. One mel pipeline feeds both encoders.

Compare what the alternatives cost you: Granite needs a bespoke 80-mel + delta +
stack-2 + pad-to-512 frontend (546 lines in our tree, with a `librosa`-matching
test suite to prove it), and VibeVoice needs 24 kHz and its own conv tokenizers.
Staying inside one vendor's family is not brand loyalty, it is one frontend
instead of three.

## 8. What it does *not* give you

Being honest so nobody discovers these the hard way:

- **No speaker diarization.** Not weak — absent. NVIDIA ships diarization as a
  separate NeMo module by design. If you need who-said-what (medical, legal,
  meetings), this is the wrong family and VibeVoice-class models are the right
  one.
- **No native decode-time context biasing in the released export.** NVIDIA has
  the technology (NeMo GPU-PB, `context_graph_universal.py`), but the hooks are
  not in the ONNX. You can add it yourself over the joint logits — that is what
  `murmure`'s `boost_tree.rs` does — but it is your code.
- **No streaming in v3.** Full attention, no cache-aware encoder. For
  push-to-talk that is irrelevant. If you need text appearing *while* the user
  talks, Nemotron 3.5 ASR streaming (`sliding_window: 57`, chunks of
  80/160/320/560/1120 ms, `set_num_lookahead_tokens({0,3,6,13})` at runtime) is
  the sibling built for it — at the cost of the duration head.
- **No official quantized ONNX.** The int4/int8 builds in circulation are
  community work. NVIDIA ships `.nemo`, safetensors, and a q8_0 GGUF. Since
  transformers ≥ 5.13 loads it natively as `ParakeetForTDT`, rolling your own
  export is tractable, but it is a project.

## 9. Model families have a cut, and it predicts fit

The fastest triage heuristic in this whole document. Vendors have a house
shape, and it tells you the answer before you read the benchmarks:

- **NVIDIA / NeMo** — builds for Riva and for throughput. Transducers and CTC,
  frame-synchronous, no fixed window, big encoder / tiny decoder. *Fits
  on-device control.*
- **IBM / Granite** — builds to feed a system. Small, efficient, permissive,
  bare output, assumes your normalisation pipeline exists. *Fits pipelines, not
  humans.*
- **OpenAI / Whisper** — builds "works on anything." One big encoder-decoder,
  30 s window, fluent hallucination. *Fits batch transcription with a
  proofreader.*
- **Microsoft Research / VibeVoice** — builds capability demos. Over-parameterised,
  brilliant at what it demos, no deployment story. *Fits when you need the
  capability and can pay for a server.*

### A 60-second triage for a new release

1. Architecture line — transducer / TDT / CTC? Keep reading. Encoder-decoder?
   Find the window before anything else.
2. `durations` non-empty in the config? Frame skipping, real speed advantage.
3. Sample output on the card — contractions and punctuation, or bare lowercase?
4. Parameters against the leaderboard RTFx column. The **ratio** is the signal;
   neither number alone is.
5. ONNX in the repo, or only `.nemo`/GGUF? The latter is a conversion project.
6. Licence. CC BY 4.0 and Apache/MIT ship; research licences do not.

---

## 10. Ranked, for the device case

If you are building single-user on-device speech control, this is the order the
properties actually matter in:

1. **No fixed window** (§1) — decides whether short commands are viable at all
2. **Silence behaviour** (§3) — decides whether it is safe unsupervised
3. **Native output shape** (§4) — decides how much code you own forever
4. **Duration head** (§2) — decides the latency ceiling
5. **Licence and footprint** (§6) — decides whether you can ship it
6. **WER** — decides almost nothing at this point; the top models are within
   noise of each other for this use, and the errors that hurt are proper nouns,
   which are a *biasing* problem, not an accuracy one

Ninety percent of open-source dictation tooling runs Whisper. On items 1, 2 and
3 — the three that decide the device case — Whisper is the worst common choice
available, and the reason is architectural, not a matter of the team's skill.

The instinct to check the architecture before the benchmark is the whole
lesson. Everything above follows from `config.json`.
