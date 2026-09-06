# The Dragon inventory

Mechanisms the pre-neural dictation systems built, what they were for, and
which ones are still load-bearing.

## Why this exists

Dragon and its contemporaries shipped usable dictation on acoustic models that
could barely hear. Everything they built was compensation for that — and
compensation for a deaf model is, almost by definition, **orthogonal to model
quality**. It composes with a good model rather than being obsoleted by one.
That is why an inventory of a dead product line is worth keeping.

The other reason is the shape of the constraint. Range anxiety was a real
objection to electric cars in 1890 and it is a real objection in 2026, not
because battery chemistry stood still but because the anxiety was never about
kWh — it was about trip planning and how much uncertainty a person will
tolerate before they stop trusting the thing. Some constraints live in the
human, and a better engine does not touch them.

Dictation has several of those. *Which words should I check?* is one. *How do I
tell the machine "type this" from "do this"?* is another. A model at 3% WER
does not answer either question; it just changes how often you ask.

**And the industry cannot ship most of this, for structural reasons rather than
ignorance.** Every mechanism below is per-user. Enrollment is friction in a
signup funnel. Learning from your documents is a privacy review. Correction
feedback is per-user state nobody wants to hold. Dragon sold a box that ran on
your machine and learned only you; a service with a million seats structurally
cannot. That gap is the whole opportunity for a single-user tool, and it is why
this is arbitrage rather than nostalgia.

The impressive part was never the bones. It was the landing.

---

## The inventory

| # | Mechanism | Goal | Status here |
|---|---|---|---|
| 1 | Enrollment / speaker adaptation | Fit the model to *this* voice | Substrate exists (corpus capture); nothing built |
| 2 | Vocabulary from your own writing | Bias decoding toward words you actually use | Phonetic dictionary is a shadow of it; boost tree is the mechanism |
| 3 | Correction as training signal | Every fix makes the next one less likely | Not wired — corrections are discarded |
| 4 | Deterministic formatting | Numbers, dates, currency, punctuation, without a model | Partly built (`postprocess.rs`, `Profile::Long`) |
| 5 | Confidence surfacing / N-best | Tell the human *where to look* | Confidence recovered, not surfaced (**D.0**) |
| 6 | Explicit modes + constrained grammar | Sidestep intent classification entirely | Mode toggle built; command grammar deferred |
| 7 | Audio path validation | Refuse to work badly in silence | Nothing — and we just found a real defect (**A.1**) |
| 8 | Addressable prior text | Fix what was said without touching the keyboard | Nothing (`design-dictation-as-control.md`) |

---

### 1. Enrollment / speaker adaptation

**Goal.** Stop being a general-purpose listener and become a listener for one
person.

**Then.** You read training text aloud — twenty minutes in the early versions —
and the system adapted its acoustic model to your voice. This was table stakes;
without it accuracy was unusable.

**Now.** Abandoned industry-wide, because large models generalise. But
*generalising* is what you need when you have many speakers. The trade was
never free — it was the right trade for a product with users, and the wrong one
for a tool with a user.

**Here.** Nothing built. `observe.corpus = true` now banks every utterance as
audio plus transcript, which is the substrate. What is missing is a method:
fine-tuning Parakeet is a real project, not a weekend. Worth revisiting once
the corpus has volume, and worth being honest that this is the hardest item on
the list.

---

### 2. Vocabulary from your own writing

**Goal.** The words you are about to say are not a random draw from English.
They are drawn from what you have already written.

**Then.** Dragon scanned your existing documents and adapted its language model
to them — your jargon, your names, your phrasing. For a domain user this was
the difference between unusable and good.

**Now.** Almost nobody does it. It is a privacy conversation and a per-user
storage cost, and end-to-end neural models have no clean seam to inject it at.
But a transducer *does* have that seam: decode-time context biasing over the
joint's logits.

**Here.** The phonetic dictionary is a pale version — a post-hoc whole-token
swap against a hand-written list. The real mechanism is **E.1**, murmure's
Aho-Corasick boost tree, which biases the decode itself. The step nobody has
taken is generating the boost list *from a corpus of the user's prose* rather
than by hand. For a writer with years of it, this is the biggest unexploited
win on the board.

---

### 3. Correction as training signal

**Goal.** A proofreading pass is expensive human attention. Spend it once.

**Then.** The correction dialog fed back into both the language model and the
acoustic adaptation. Dragon's manual pushed hard on *correct, don't retype* —
retyping fixed the document and taught the system nothing.

**Now.** Essentially gone from local tools. Cloud services do something like it
in aggregate, which helps the average user and not you.

**Here.** Not wired. Every correction is currently discarded. The corpus's
`reference` field is a manual, one-way record. The nearest live thing is
**C.3**: a misrouted note is an unlabelled signal that a name was mangled.
Closing this loop properly — a correction updating the dictionary or the boost
list automatically — is a small feature with a large compounding return, and it
is the one Dragon leaned on hardest.

---

### 4. Deterministic formatting

**Goal.** Spoken form in, written form out, predictably.

**Then.** Extensive rule engines: numbers, dates, times, currency, phone
numbers, addresses, punctuation-by-name. Rules, not statistics. Fast,
inspectable, and identical every time.

**Now.** The fashion is to hand this to a small LLM — s1-mini is a 0.6 B Qwen3
doing exactly this job. That buys false-start and self-correction repair, which
no rule can do. It costs determinism, ~460 MB, and an LLM in the path that can
invent.

**Here.** Partly built and split across two places. `postprocess.rs` handles
fillers, spoken punctuation and casing; `mode.rs`'s `Profile::Long` handles
contractions for Granite's bare output. The exclusions in `Profile::Long` are
the honest map of where rules run out: `I have` → `I've` needs part-of-speech,
`let us` → `let's` breaks on `let us know`, and numerals depend on whether it is
a count, a date or a version. That deterministic ceiling is deliberately kept
as the baseline any learned normaliser has to beat.

---

### 5. Confidence surfacing / N-best

**Goal.** Do not make the human scan the whole line. Point at the two words
worth checking.

**Then.** Uncertain words were marked, and the correction interface offered a
ranked alternates list from the decoder's own lattice. The machine said *I am
not sure about this one, and here are my other guesses.*

**Now.** Largely gone. Modern dictation UIs present a flat wall of text with no
indication of where the model was guessing — arguably a regression, because the
signal still exists inside every decoder and is simply thrown away at the API
boundary. `parakeet-rs` doing exactly that is what sent this repo's plan toward
the wrong engine for a night.

**Here.** Half done, and this is the highest-value cheap item. Per-token
confidence is recovered in `parakeet_ort.rs` and is sharply discriminative: on
the bench clip every correct word scored 0.99–1.00 while the two wrong ones,
`Meg` and `Calendar.`, scored 0.393 and 0.570. **D.0** surfaces the
sub-threshold words in the notification and the smoke log. That converts a
whole-sentence proofread into a two-word glance and needs no model change at
all. The N-best half — offering alternates from the lattice — is further off but
follows from the same signal.

---

### 6. Explicit modes and a constrained grammar

**Goal.** Distinguish "type this" from "do this" without guessing.

**Then.** Dictation Mode, Command Mode, Numbers Mode, Spell Mode. Explicit,
user-controlled, with a formal grammar for commands. They did not classify
intent from free text, because with their acoustic models that would have been
suicide.

**Now.** The fashion is intent classification, which is harder, less
predictable, and fails in a way that eats your document instead of ignoring you.

**Here.** The mode toggle is built — three modes on a hotkey, refused unless
idle. The command grammar is **E.4**, deliberately deferred. Worth noting: the
kill criterion on the command channel is *"if false positives on prose exceed
2%, ship push-to-talk-with-a-modifier instead."* That is Dragon's answer,
arrived at independently from first principles. When a modern design's failure
mode lands on the 1997 solution, the 1997 solution was probably not naive.

---

### 7. Audio path validation

**Goal.** Fail loudly on bad input instead of quietly transcribing mush.

**Then.** A microphone setup wizard measured levels and signal-to-noise and
told you when your hardware or your room was the problem, rather than blaming
the model.

**Now.** Basically absent. Tools accept whatever the OS hands them.

**Here.** Nothing, and this repo just proved the point on itself: `audio.rs`
resamples 44100 → 16000 by linear interpolation with no anti-aliasing filter,
so 12 kHz content arrives 2.2 dB down and folds onto 4 kHz — the middle of the
speech band. That is **A.1/A.2**, and it went unnoticed for months because
nothing was watching the audio path. `--doctor` is the natural home for a
check: device rate, channel handling, resampler quality, clipping.

---

### 8. Addressable prior text

**Goal.** Repair what you just said by speaking, without reaching for the
keyboard.

**Then.** Select-and-Say — the app tracked the text it had produced, so you
could say *select "the Meg Calendar"* and replace it by voice, with the
correction feeding mechanism 3.

**Now.** Almost entirely lost. Dictation tools inject text into someone else's
buffer and immediately forget it, which makes voice-driven repair impossible by
construction.

**Here.** Nothing. `design-dictation-as-control.md` sketches the shape and
correctly identifies intent classification as the load-bearing gap. Note the
dependency Dragon understood and the sketch does not stress: **Select-and-Say
requires owning a text buffer.** transcrust injects and forgets. That is a
bigger architectural change than the grammar it sits behind.

---

## What does not survive

Being honest, so this reads as an inventory rather than a shrine:

- **The decoder internals.** GMM-HMM acoustic modelling is obsolete. Nothing to
  recover there.
- **Discrete-word input.** Pausing. Between. Every. Word. was a constraint of
  the era, not a design choice worth revisiting.
- **Rigid grammars for *dictation*.** Fine for commands, hopeless for prose.
- **Mandatory twenty-minute enrollment as a first-run experience.** The
  adaptation is worth having; making it a wall between the user and the product
  is not. Passive enrollment from ordinary use is the modern shape, which is
  what corpus capture quietly enables.

## How to use this board

The four with the best return-per-line, in order: **5** (point at the words),
**7** (validate the audio path), **4** (rules before models), **3** (close the
correction loop). Those are all small and all compose with a good model.

**2** is the big one and a real project. **1** and **8** are the hard ones — one
needs a training pipeline, the other needs to own a buffer.

Tracked as tasks in `TASKBOARD-next.md`; this document is the *why* behind them.
