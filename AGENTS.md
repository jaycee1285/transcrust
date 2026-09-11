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

## State as of 2026-09-06

Two engines: **Parakeet TDT 0.6B v3** (default, int4) and **Granite Speech 5
470m TurboCTC** (int8). Three *modes* — modes are `(model, profile)` pairs, so
`Granite — Long` is the same directory with a repair profile.

VibeVoice was evaluated and removed the same night. Postmortem, with weights
inventory and timing tables, at **`~/syncthing/vibevoice-asr-15/`** — outside
the repo, so it will not be found by grep.

Recently built and worth knowing about:

- `--bench <WAV...>` — times every mode over the same clips, through the live
  path. Fixtures in `tools/bench-clips/`.
- `observe.corpus = true` — banks every dictation as WAV + JSON under
  `~/.local/share/transcrust/corpus/`. That directory has its own README.
- `src/parakeet_ort.rs` — Parakeet driven directly through `ort`, recovering the
  per-token confidence `parakeet-rs` discards. **Built, tested, and not yet
  wired into the live path.** Resolving that is Phase B.

## Three facts that are easy to get wrong

1. **Parakeet exposes confidence.** The claim that it does not describes the
   `parakeet-rs` crate boundary, not the model. The joint's vocab logits are in
   `decoder_joint-model.onnx`. This error cost a night and sent a plan toward
   the wrong engine.
2. **A traced ONNX graph can declare a dynamic axis it does not honour.** Bit us
   twice — the VibeVoice encoder returned the traced frame count for any input,
   and Granite's pad-to-512 is a hard constraint despite a dynamic declaration.
   Verify shapes by running the graph, not by reading its metadata.
3. **The audio path is suspect and unfixed.** `audio.rs` resamples 44100 → 16000
   by linear interpolation with no anti-aliasing filter. It confounds any
   accuracy measurement taken before Phase A.

## House style

FOSS, minimal abstraction, local files plus Syncthing, no SQLite unless asked.
Touch only what the seam requires. Do not pin the app to a reference's
dependencies just because the reference used them. Hand work back as one
copy-pasteable `nix develop -c ...` command and wait for the human.
