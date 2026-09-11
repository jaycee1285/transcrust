# Human Smoke 2026-08-31 — Engine parity, tray, release

Session record. Nothing is committed in any repo; all of this is working-tree.

## What changed in this pass

### transcrust

- **Engine parity audited.** `postprocess::fix_transcription` is called exactly
  once, at `main.rs`, after the worker returns. `TranscriptionService` is the
  only seam and neither arm does word/phrase/vocabulary work of its own, so
  Parakeet and Granite get identical treatment by construction.
- **Dictionary casing bug fixed.** `map_word_with` returned the *heard* token on
  an exact case-insensitive hit, so `wayland` never became `Wayland`. Invisible
  while Parakeet supplied its own casing; Granite's bare lowercase exposed it.
- **`empty_dictionary_is_identity` was reading the real `~/.config` dictionary**
  and passing because of that bug. `correct`/`map_word` now take the corrector
  as a parameter so the tests are hermetic.
- **Family-keyed greedy model discovery** (`model.rs`). Candidates are
  directories whose name starts with `parakeet`/`granite`; contents confirm.
  Inside one, `quant_rank` prefers int8 → int4 → fp32. Fixed a latent bug: fp32
  Parakeet (`encoder-model.onnx`, what `--download-model parakeet-tdt-0.6b`
  writes) was previously undetectable.
- **Tray engine switcher.** `discover_models` feeds a `RadioGroup`; the tray
  sends an index over mpsc and `main.rs` is the only place that builds a
  service. Refused unless Idle.
- **Tray icons.** `emblem-ok-symbolic` does not exist in Adwaita, so Injecting
  and Complete rendered as a generic cog — now `object-select-symbolic`. Added
  `trayicon.rs`: SDF-drawn ARGB32 pixmaps at 22/44px as a theme-independent
  floor. Removed a `gdbus` re-registration hack that raced a registration ksni
  had already done.
- Docs: `traverse/runtime-stack.md` gained the post-processing contract,
  discovery/switching rules, and an SNI contract section.

57 tests pass. No new clippy warnings.

### ferritebar

- **Deleted a `tokio::time::interval(5s)` loop** that re-sent a full tray
  snapshot forever. Events arrive ~1.3ms after an item appears, so it added no
  arrival latency, but it cloned every ARGB payload, rebuilt textures, and
  logged per item per tick.

  **These two changes are coupled — do not revert one alone.** John observed the
  engine radio switching correctly against *old* ferritebar, which means the
  poll was doing real work: its `Snapshot` handler does `item.menu = entry.menu`,
  replacing the whole cached menu from the client every 5s and so papering over
  the dropped nested diffs below. Remove the poll without
  `apply_menu_diffs_deep` and nested menu state goes permanently stale instead
  of self-healing within five seconds.
- **`apply_menu_diffs_deep`.** `system_tray::data::apply_menu_diffs` walks only
  the top level, so diffs for nested items are dropped silently. This is why
  the engine radio "fell back" — the app switched correctly and the bar kept
  rendering a stale cache.
- Reconcile-not-clobber on snapshot menu state, icon fingerprinting, per-item
  popup dismissal, `MenuDiff` logging (previously the only unlogged event).

## Commands actually run

```sh
nix develop -c cargo test                      # transcrust, 57 pass
nix develop -c cargo build                     # both repos
./release.sh                                   # both repos, exit 0
nixos-rebuild build --flake ~/repos/config#Sed # exit 1, see Blocked
```

## Measured, not assumed

| | before | after |
|---|---|---|
| ferritebar log lines, 90s idle, 1 tray item | 36 | **0** |
| ferritebar CPU, same window | 0.88s | 0.64s |
| nested menu diffs applied | 0 of 2 | **2 of 2** |
| transcrust exec → SNI register | 56ms | unchanged |
| bar register → item pickup | 3.8ms | unchanged |

The last two matter for the "first launch sniggle": the bar is not the
bottleneck. 56ms is transcrust's own startup — ORT init, model discovery, and
probing 400MB of files all happen before `run_tray` is spawned.

## Released

Both clobbered onto GitHub `v0.1.0` and wired into `~/repos/config/tauri.nix`:

```
ferritebar  sha256-H4GNqSh7IUsbrjw4IwWmfcMt/pmEzeAwn+4Y5J79kJA=
transcrust  sha256:4c7210b2e46252b06129a09e58b6f94c955861ed0d86e503540f0b280a48e253
```

Both uploaded assets were prefetched and match the local tarballs. Both
derivations build; the shipped binaries were run and carry the fixes.

## Blocked

`nixos-rebuild switch` fails, **not** on the app hashes:

1. `undefined variable 'gitpulsar'` — resolved by changing it to
   `pkgs.unstable.gitpulsar` plus a `nixpkgs-unstable` bump (2026-08-26 →
   2026-08-31), since `gitpulsar` landed after the old pin. `flake.lock` backup
   was in the session scratchpad; `git checkout flake.lock` also reverts.
2. `seagoat-1.2.0` fails `deepmerge<3.0.0,>=2.0.0 not satisfied by version 3.0`.
   **Pre-existing** — its drv hash is byte-identical before and after the lock
   bump. It was masked by the gitpulsar eval error. Unstable's seagoat builds,
   but that change was reverted at John's instruction; this is his to resolve.

## Verified by John, in the session

- **The tray right-click menu renders and the engine switch works.** Confirmed
  by hand. I could not test this myself — `/dev/uinput` is
  `crw------- root root`, so no synthetic clicks — and the machine-side evidence
  only went as far as the DBusMenu data being correct (`toggle-state` moves,
  diffs apply 2/2).
- One nuance not separately confirmed: whether the radio **stays** moved when
  you reopen the menu. That is specifically what `apply_menu_diffs_deep` fixes,
  and the bar running during the check was the patched build, so it very likely
  holds — but the reopen was not called out explicitly. Worth ten seconds
  (§5 of `Smoke-Human-transcrust.md`).

## Not verified

- **Live A/B of both engines on the same speech.** Parity was proven
  structurally, on synthetic audio, and on both raw-text shapes via `--fix`.
  Never on one real utterance through both engines.
- Whether int4 Granite degrades recognition versus int8.

## Models

`~/.local/share/transcrust/models/granite-speech-5.0-470m-turboctc` was a
symlink into `~/repos`; it is now a real 527MB copy (int8 onnx, tokenizer.json,
config.json, preprocessor_config.json). The repo's two `.onnx` build artifacts
were then deleted, 8.9G → 6.6G. Regeneration intact: `model.safetensors` plus
`export_onnx.py` → `quantize_onnx.py` → `validate_onnx.py`.

## What John is testing next

`Smoke-Human-vocabulary.md` — acceptance criteria for two unbuilt features,
written before the code:

1. **Spoken command dispatch.** `repair <heard> <intended>` / `mark <term>` /
   `undo that`, routed *before* `fix_transcription` treats the transcript as
   content. Cases 1.3 (prose containing the trigger must not be eaten) and 1.4
   (a mangled trigger must still fire, via Beider-Morse on the trigger itself)
   are the two that decide viability; the rest are correctness.
2. **Shell oracle.** A bash hook turns `command not found: transnistrian` into
   an alias candidate, matched against `$PATH` + cwd + branches. Promotes after
   repeat sightings only.

Both write one artifact, `~/.config/transcrust/aliases.json`, applied ahead of
`dictionary::correct`. Prerequisite for debugging either: an `Aliases:` line in
`--doctor`.

## Design notes worth not re-deriving

- **Beider-Morse misses `transcrust`/`transnistrian`** — measured. It also
  misses `Wayland`/`waylund`. So aliases are not an edge case; the phonetic net
  is tighter than it looks and the alias map carries real weight.
- **Edit-distance + prefix caught `transnistrian → transcrust` at rank 1**,
  which phonetics cannot. The shell oracle wants a *different* matcher than the
  dictionary uses; they cover different failure shapes. Scratch harness was
  `vocab.sh` + `match.awk`, ~60 lines of awk, not preserved.
- **`Kubernetes` is in the authored dictionary but is not a command or
  directory**, so the shell oracle structurally cannot fix it. The two stores
  cover disjoint vocabulary; neither subsumes the other.
- **Parakeet exposes no confidence.** `TranscriptionResult { text, tokens }`,
  `TimedToken { text, start, end }` — timestamps only. Granite has raw logits
  in-process and `greedy_ctc_ids` already discards the max value, so a per-token
  margin is ~5 lines. Ensemble disagreement between the two engines is a signal
  that needs no confidence API from either.
- **s1-mini** (<https://huggingface.co/superwhisper/s1-mini>) subsumes five of six passes
  in `fix_transcription` and adds ITN, but not the phonetic dictionary. It is a
  0.6B Qwen3 finetune, GGUF Q4_K_M at 462MB — a second runtime unless exported
  to ONNX. Its trained input shape is lowercase and unpunctuated, which is
  exactly Granite CTC output. Ordering risk is the Harper lesson again: a
  learned normalizer will re-rank jargon toward general English unless the
  dictionary claims those tokens first.
