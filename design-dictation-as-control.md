# Dictation as a control channel — gap map

Where the 2026-08-31 session's sketches sit against the actual problem.

## What exists today

Every utterance is content. There is no other possibility in the code.

```mermaid
flowchart LR
    A[audio] --> B[ASR<br/>Parakeet or Granite]
    B --> C[raw transcript]
    C --> D[fix_transcription]
    D --> E[inject]

    classDef have fill:#2d6a4f,stroke:#95d5b2,color:#fff
    class A,B,C,D,E have
```

One path, no branch. `fix_transcription` is a text→text function; it cannot
decline to return text, so nothing upstream can decide an utterance was not
meant to be typed.

## What the problem actually needs

```mermaid
flowchart TD
    A[audio] --> B[ASR]
    B --> C[raw transcript]
    C --> D{what is this for?}

    D -->|content| E[fix_transcription]
    E --> F[inject]

    D -->|command| G[parse intent + payload]
    G --> H[mutate app state]
    H --> I[confirm, inject nothing]

    D -->|correction| J[learn a mapping]
    J --> K[alias / vocabulary store]
    K -.->|applied next utterance| E

    classDef have fill:#2d6a4f,stroke:#95d5b2,color:#fff
    classDef sketched fill:#7c5e10,stroke:#e9c46a,color:#fff
    classDef missing fill:#7f1d1d,stroke:#fca5a5,color:#fff

    class A,B,C,E,F have
    class G,H,I,J,K sketched
    class D missing
```

Green is built. Amber was designed this session. Red is untouched.

## The load-bearing gap is the diamond

Everything amber is mechanism — a grammar, a store, a write path. All of it is
straightforward, roughly 300 lines, no new dependencies.

The red node is a *decision*, and none of the sketches make it. What was
proposed is a prefix match:

```mermaid
flowchart LR
    C[raw transcript] --> P{first token<br/>phonetically ≈<br/>repair / mark / undo?}
    P -->|yes| G[command]
    P -->|no| E[content]

    classDef sketched fill:#7c5e10,stroke:#e9c46a,color:#fff
    class C,P,G,E sketched
```

That is a heuristic standing in for classification. It works for the utterances
it was designed against and fails on:

- a command phrased naturally — *"can you make concept mean transfusion"*
- a command that does not lead with the trigger
- prose that happens to open with the trigger word

`Smoke-Human-vocabulary.md` §1.3 and §1.4 exist precisely to measure how badly.
They are the viability cases, not the correctness cases.

## The second gap: scope

"Dictation of feature changes" is larger than vocabulary. The sketches only
reach the narrowest corner of it.

```mermaid
flowchart TD
    S[spoken change] --> V[vocabulary<br/>alias, dictionary entry]
    S --> C[app config<br/>hotkey, engine, mode]
    S --> B[app behavior<br/>new rule, new pass]

    V --> V1[sketched: repair / mark<br/>+ shell oracle + retry diff]
    C --> C1[not designed<br/>tray radio exists, no voice path]
    B --> B1[not designed<br/>this is agent territory]

    classDef sketched fill:#7c5e10,stroke:#e9c46a,color:#fff
    classDef missing fill:#7f1d1d,stroke:#fca5a5,color:#fff
    class S,V,V1 sketched
    class C,C1,B,B1 missing
```

Vocabulary is the easy corner because the target is a flat key→value store that
already exists on disk. Config is harder — the tray switcher proves the state
can be mutated at runtime, but nothing routes speech to it. Behavior change is a
different category of problem entirely.

## What the session actually established

Not a step toward voice control. A demonstration that the surfaces hold:

- one post-processing call site, engine-agnostic by construction
- a model-discovery layer that takes a structurally different engine without a
  parallel pipeline
- a tray whose menu state is now correct end to end, including nested items
- measured signal sources: Granite exposes CTC logits, Parakeet exposes none,
  engine disagreement needs neither

  > **Corrected 2026-09-06.** "Parakeet exposes none" describes the
  > `parakeet-rs` crate, which runs the decode internally and returns a
  > `String`. The joint network's vocab logits are in
  > `decoder_joint-model.onnx`; driving the two sessions directly with `ort`
  > yields per-token probability, word confidence and timestamps from the files
  > already on disk. `murmure-reference.md` records the shape (the clone
  > itself was removed). This error sent the command-channel plan toward Granite; see
  > `TASKBOARD.md` and `Parakeet-v3.md` §5.

Those are the preconditions for building the diamond. They are not the diamond.

## Honest ranking of what is unsolved

1. **Intent classification.** The prefix heuristic is a placeholder. Everything
   else is downstream of it and cheap by comparison.
2. **Scope beyond vocabulary.** Config-by-voice has an existing mutation path
   (the tray switcher) and no route to it.
3. **WER.** Granite and Parakeet are close enough that neither is the
   bottleneck. Improving transcription does not move this problem.
