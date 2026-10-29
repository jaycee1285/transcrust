# transcrust — start here

Push-to-talk voice input for a keyboard-driven Wayland desktop. One user, one
machine, no network. Written by John (content strategist, not a career
engineer); assume the architecture notes are load-bearing and the constraints
are deliberate.

## Read in this order

| | |
|---|---|
| `traverse/runtime-stack.md` | how the thing is built and why. The authority on architecture. |
| `TASKBOARD.md` | what to do next, in order, with kill criteria |
| `Dragon-Mechanisms.md` | the research north star — what pre-neural dictation solved and which of it is still load-bearing |
| `Parakeet-v3.md` | why the default engine is the right one for on-device control. Reference doc, not a task list. |
| `murmure-reference.md` | notes from an AGPL reference app that was read and removed — calibrated constants, and the licence caveat before copying any of it |
| `design-dictation-as-control.md` | the unbuilt half: dictation as a command channel |
| `design-long-form-routes.md` | the four long-form dictation routes, their acceptance gates, and the fuzzel picker |

## State reconciled 2026-10-28

Three engine families: **Parakeet TDT** (default, int4; Ultra supported),
**Granite Speech 5 470m TurboCTC** (int8), and **Nemotron Speech Streaming EN**
(int4, toggle capture). Modes carry model, profile and capture policy.
`Granite — Long` uses the same directory with a repair profile.

VibeVoice was evaluated and removed the same night. Postmortem, with weights
inventory and timing tables, at **`~/syncthing/vibevoice-asr-15/`** — outside
the repo, so it will not be found by grep.

Recently built and worth knowing about:

- `--bench <WAV...>` — times every mode over the same clips, through the live
  path. Fixtures in `tools/bench-clips/`.
- `observe.corpus = true` — banks every dictation as WAV + JSON under
  `~/.local/share/transcrust/corpus/`. That directory has its own README.
- `src/parakeet_ort.rs` — Parakeet driven directly through `ort`, recovering the
  per-token confidence `parakeet-rs` discards. **Wired into the live path**,
  with a crate fallback if the direct graph set is absent or broken.
- `--pick` opens fuzzel on a running daemon; startup does not open the picker.
- Correct last dictation: `--correct`, optional `--correct-chord C-A-c`, and a
  tray entry open an egui editor. Replace/Copy and Keep/Drop vocabulary proposals
  are wired; **the GUI and GTK palette passed John's 2026-10-06 smoke**.
  Learned vocabulary accuracy is a separate, still-open gate. See `design-correct-last.md` and
  `Smoke-Human-transcrust.md`. Apple's review-before-paste contract remains in
  `~/repos/swiftype/PRD.md`; the crate split and parity gates are still pending.
- `--dec` is implemented in native Rust (`src/decision.rs`, `src/d1.rs`), using
  resident D1 Q8 and one padded batch for all candidate occurrences. No runtime
  Python. John accepted desktop native Q8 as-is for now on 2026-10-29;
  mobile latency/RAM remain separate gates. Both descriptions and learned aliases
  belong in `~/syncthing/john-vocab.md` (or `[vocabulary].path`).
- Phonon is a **mopvox Android trial**, not a transcrust engine. Phone findings
  and remaining correction/Apple work are summarized at the top of `TASKBOARD.md`.
  Its current-state table and ordered next steps outrank historical phase prose.

## Three facts that are easy to get wrong

1. **Parakeet exposes confidence.** The claim that it does not describes the
   `parakeet-rs` crate boundary, not the model. The joint's vocab logits are in
   `decoder_joint-model.onnx`. This error cost a night and sent a plan toward
   the wrong engine.
2. **A traced ONNX graph can declare a dynamic axis it does not honour.** Bit us
   twice — the VibeVoice encoder returned the traced frame count for any input,
   and Granite's pad-to-512 is a hard constraint despite a dynamic declaration.
   Verify shapes by running the graph, not by reading its metadata.
3. **The resampler was fixed, not proven to improve accuracy.** `audio.rs` uses
   a band-limited polyphase decimator. A.2 did not demonstrate an accuracy win
   on codec-limited speech; real microphone replay remains open.

## House style

FOSS, minimal abstraction, local files plus Syncthing, no SQLite unless asked.
Touch only what the seam requires. Do not pin the app to a reference's
dependencies just because the reference used them. Hand work back as one
copy-pasteable `nix develop -c ...` command and wait for the human.
