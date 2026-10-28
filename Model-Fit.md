# Model fit: pick the family, then the member, for the job

Which speech model does which job, why, and how they combine into a transcription
pipeline that runs entirely on the device. Written 2026-09-26.

Provenance: **[J]** John's method or judgment · **[M]** measured in this repo or its
siblings · **[C]** model card, not reproduced here · **[?]** claimed, not yet measured.

---

## 1. The selection method **[J]**

1. **Pick the family by its architecture, then the member by the job.** Check for a
   duration head before the WER table (`Parakeet-v3.md` §2).
2. **Take the first English-only member.** A model trained for one language
   spends its capacity on that language.
   - **Exception, Parakeet:** there is no English-only member that anyone
     quantizes, so it's v3 (multilingual), or Ultra, which is v3 trained further.
   - **Nemotron Speech Streaming EN** is its own English model, not a derivative of
     the multilingual Nemotron 3.5.
3. **Check what the model is built on before you check what it claims.** A model
   inherits its base. VibeVoice's own card says it "inherits any biases, errors, or
   omissions produced by its base model (specifically, Qwen2.5 1.5b in this
   release)". Microsoft speech models are usually built on Qwen, so check that first.
4. **Prefer models that are best at one thing over models that add features.**
   Microsoft's speech models tend to add capabilities without being best at any of
   them. VibeVoice is out of every route here, including speaker labelling.

---

## 2. Job map

| Job | Model | Why this one | Evidence |
|---|---|---|---|
| **Short dictation** (hold-to-talk, < 20 s) | **Parakeet Ultra int4** (0.6B, TDT) | best English accuracy of the Parakeet family, and the best under noise; the duration head keeps decoding cheap | **[M]** noise-drift study 2026-09-25; phone wait ~0.8 s worst case at 6 s windows |
| **Long drafting, word by word** | **Nemotron Speech Streaming EN int4** (0.6B, 1120 ms chunks) | encodes while you talk; the wait is about one chunk, however long you speak; beats Granite on ordinary conversational speech **[J]** | **[M]** 0.1–0.2 s wait on the phone (mopvox TASKBOARD) |
| **Long recorded transcripts, compliance-bound** (legal, medical, SOC 2) | **Granite Speech 5 470M TurboCTC** + an **organisation-wide vocabulary file** + a **normalisation pass** | fastest long-form batch here; CTC, so it can't generate text the audio doesn't contain; the vocabulary file carries the firm's or health system's terms **[J]** | **[M]** 36:35 of audio in 2:17 (Parakeet: 7:37); `Profile::Long` 13.71% → 10.11% WER on contractions. **[?]** accuracy on legal or medical vocabulary |
| **Who spoke when**, for those transcripts | **Nemotron 3 Diarization** (100M) | per-speaker activity at 10 ms resolution, up to 8 speakers, streaming or offline; word-level speaker tags with a transcript | **[C]** see §4. **[?]** not yet run here |

---

## 3. The pipeline is the claim

Models change every quarter. The shape doesn't: **[J]**

```text
audio ─► transcribe ─► attribute speakers ─► correct against the org's terms ─► normalise ─► review
         (Granite)     (Nemotron 3 Diar.)    (vocabulary file,                   (casing,
                                              `heard:` pairs,                     punctuation,
                                              decode-time boost)                  possessives)
```

- **Every stage is swappable and measured separately.** A better diarizer or a
  better recogniser replaces one box.
- **Everything runs on the machine.** No audio or text leaves it. For a law firm or
  a hospital that isn't a feature, it's the precondition.
- **Neither model generates text:** Granite is CTC, and the diarizer only labels
  time spans. Neither can invent a sentence into silence, and inventing a sentence is
  the failure that matters in a deposition or a chart. Whisper-class models can
  hallucinate into silence (`Parakeet-v3.md` §3).
- **The vocabulary file is organisation-wide.** One file per firm or health system
  carries case names, drug names, parties and terms, edited by people, not retrained.
- **The transcrust demos show the pattern works end to end.** The simple vocabulary
  file, and Granite with s1-mini normalisation (TASKBOARD D.4), are demonstrations of
  the pattern, not a personal preference.

---

## 4. Nemotron 3 Diarization, from its card **[C]**

Source: `~/repos/diarization-README.md` (released 2026-09-23, OpenMDW 1.1 licence,
commercial use allowed).

- **Architecture:** 31-layer Transformer encoder (RoPE), 80 ms encoder frames
  upsampled to 10 ms output; initialised from a NEST self-supervised checkpoint.
  **100M parameters.**
- **Speakers:** up to 8, channels ordered by first arrival. Labels are generic
  (`speaker1`, start, end); mapping them to names is a human step.
- **Latency:** one checkpoint covers everything from an 80 ms buffer (0.32 s is the
  lowest recommended) to an offline-style 30.4 s buffer. Chunked inference has no
  maximum duration.
- **Local runtime:** NeMo-Speech.cpp, a native C++ CLI:
  `nemo-speech diarize meeting.wav`, or `nemo-speech transcribe meeting.wav --diarize --json`
  for word-level speaker tags. An integration guide for streaming ASR ships with it.
- **Accuracy (DER, lower is better; offline 30.4 s configuration against its own 4-speaker predecessor):**

  | Set | Previous model | Nemotron 3 Diarization |
  |---|---|---|
  | DIHARD III, full | 19.09 | 12.73 |
  | CALLHOME part 2, full | 10.32 | 9.10 |
  | AMI test, headset mix | 15.81 | 9.25 |
  | NOTSOFAR1, headset mix | 21.77 | 6.77 |

  The card stresses that these use forced-alignment reference labels, and that
  numbers scored against other labels aren't comparable.
- **Speed:** RTFx 1,340 (batch 1, eager) offline, measured on an NVIDIA RTX PRO 5000
  GPU in BF16. **[?]** CPU speed isn't stated; measure it on the target machine.

---

## 5. Why trust the design **[J]**

John wrote legal content for law firms for ten years, more than 600 pieces, and knows
from partners' reactions what the reader of a legal document does with a wrong word.
He has personal experience of what a record means to the person it's about. The
failure the pipeline is designed against, an invented or misattributed sentence in a
record, comes from that, not from a benchmark.

*(John: decide how much of this goes public before this section is lifted anywhere.)*

---

## 6. Open, and what would settle it

- **[?] Granite on legal and medical vocabulary.** Run it once on a public long-form
  legal or medical recording with a small domain vocabulary file; score the terms,
  not overall WER.
- **[?] Nemotron 3 Diarization on CPU:** RTFx and memory on the laptop, through
  NeMo-Speech.cpp.
- **[?] Alignment of diarization and transcript:** how Granite's word timings map
  onto 10 ms speaker spans, and what happens at overlapping speech.

## 7. Docs this corrects

- **`Parakeet-v3.md` §8** says VibeVoice-class models are "the right one" for
  speaker labels in medical, legal and meetings work. Superseded: Nemotron 3
  Diarization with Granite.
- **`design-long-form-routes.md`** describes Granite as "demoted to research".
  For *dictation* that holds; for long compliance-bound transcripts it's the route
  of choice. See §2.
