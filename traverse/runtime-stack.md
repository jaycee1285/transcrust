# Transcrust Runtime Stack

## Scope
- Root Rust CLI app in `src/`
- Granite Speech 5 470m TurboCTC, exported to ONNX at
  `~/repos/granite-speech-5.0-470m-turboctc` (**that directory no longer exists
  — only the installed int8 graph remains; the export scripts referenced below
  are gone and would need rebuilding**) (fp32 ~1.9 GB, dynamic-int8 ~551 MB;
  transcrust prefers int8). That repo's `ONNX-TRANSCRUST.md` is the graph contract
  — input/output names and shapes, the pad-to-512 rule, and the frontend spec that
  `src/granite.rs` implements. Its `export_onnx.py` / `quantize_onnx.py` /
  `validate_onnx.py` reproduce the export.
- That directory is **not** on transcrust's default search path. Symlink it into
  `~/.local/share/transcrust/models/` to get it into the tray switcher, or pin it
  with `TRANSCRUST_MODEL_PATH` / `model.path`.
## Model Discovery And Engine Switching
- `src/model.rs::discover_models` returns **every** installed model, best-first.
  `find_model_path` is now just its head. `--doctor` prints the same list.
- Discovery is **family-keyed and greedy**, so new quantisations and point
  releases drop in without a code change:
  - A directory is a candidate when its name *starts with* a family word —
    `FAMILY_KEYWORDS = ["parakeet", "granite"]`. The name only nominates;
    `model_kind` confirms by contents and has the final say.
  - Inside a candidate, `pick_onnx` takes the best-quantised graph by
    `quant_rank`: int8 → int4 → fp32. Parakeet wants `encoder*.onnx` +
    `decoder_joint*.onnx` + `vocab.txt`; Granite wants any `*.onnx` that is not
    `encoder*`/`decoder*`, plus `tokenizer.json`. `*.onnx.data` sidecars are
    weights, not graphs, and never match.
  - This fixed a latent bug: fp32 Parakeet (`encoder-model.onnx`, what
    `--download-model parakeet-tdt-0.6b` writes) was previously undetectable
    because only the int8/int4 filenames were enumerated.
- Ordering is Parakeet, then Granite (`ModelKind::rank`), so an unpinned
  install resolves exactly as it did before the switcher existed.
- **The tray owns activation, the main loop owns the swap.** `src/tray.rs`
  renders a `RadioGroup` of `discover_models` labels and sends the chosen index
  over an `mpsc` channel; `main.rs`'s select loop is the only place that builds
  a `TranscriptionService`. The tray never touches a model.
- **The radio must move inside the `select` callback.** ksni calls it from
  `update_immediately`, which re-renders the menu from `menu()` the moment the
  callback returns — so a callback that only sends the request repaints the
  *previous* engine and looks exactly like the click was ignored. Optimistic
  update, then correction: the main loop echoes the truth back on a `watch`
  channel, and **every path that declines a switch must send on it** (not idle,
  bad index, load failure) or the menu will claim an engine that never loaded.
  `tray.rs`'s `selecting_an_engine_moves_the_radio_immediately` pins this.
- A switch is **refused unless the app is Idle** — swapping the service under a
  running job would strand the audio receiver its worker is draining. The user
  gets a "Busy" notification, since a silent snap-back reads as a broken menu.
- The outgoing worker is not force-unloaded; it releases its model on its own
  idle timeout (`observe.idle_timeout_secs`, default 60s). Expect both models
  resident in RAM for up to that long after a switch.
- The submenu is hidden when fewer than two models are installed.
- **The job timeout belongs to the engine, not to `main`.**
  `TranscriptionService::timeout()` is per-engine rather than a global
  constant. Both current engines answer far inside 45s (worst measured:
  Parakeet 11.7s for 60s of audio); the seam exists for a future mode that adds
  a learned normalisation pass and needs its own ceiling.

## SNI Tray Contract
The reference host is `~/repos/ferritebar` (`src/modules/tray.rs`, the
`system-tray` crate). Its icon resolution is the contract every SNI host
implements some version of:

```rust
if icon_name.filter(|n| IconTheme::has_icon(n)) { set_icon_name(...) }
else if let Some(pixmaps) = icon_pixmap { /* blit ARGB32 */ }
else { set_icon_name("application-x-executable-symbolic") }   // silent
```

- **`IconName` is a lookup in the *host's* theme, not ours.** A name the theme
  lacks is not an error — the host substitutes a generic placeholder and the app
  is never told. This is why tray icons are so often wrong or missing.
  `emblem-ok-symbolic` was exactly that: a plausible freedesktop name that
  Adwaita does not ship, so Injecting and Complete rendered as a generic cog.
  Verify a name exists before using it:
  `find -L /run/current-system/sw/share/icons/Adwaita -name '<name>.*'`
- **Publish both.** `icon_name` stays primary because hosts recolor themed
  symbolic icons to match the panel and a bitmap cannot. `src/trayicon.rs`
  supplies `icon_pixmap` as the floor for whatever the host's theme is missing:
  SDF-drawn glyphs at 22px and 44px, rasterized once into a `LazyLock`
  (`Tray::icon_pixmap` is called on every property refresh — never rasterize per
  call). Bytes are `[A, R, G, B]`, **straight alpha, not premultiplied** — the
  host feeds them to a `Pixbuf`, so premultiplied data renders as a dark halo.
- **ksni already handles registration.** It requests
  `org.kde.StatusNotifierItem-<pid>-1`, calls `RegisterStatusNotifierItem`, and
  re-registers on the watcher's `NameOwnerChanged`, so it survives the bar
  restarting and starting before the bar. `watcher_offine()` defaults to `true`,
  which keeps the service alive when no watcher exists yet. A manual `gdbus`
  re-register loop used to sit in `run_tray`; it was racing a registration that
  had already succeeded. Removed after confirming the item still reaches the
  watcher without it.
- To inspect the live bus:
  `gdbus call --session --dest org.kde.StatusNotifierWatcher --object-path /StatusNotifierWatcher --method org.freedesktop.DBus.Properties.Get org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems`
  then `--dest org.kde.StatusNotifierItem-<pid>-1 --object-path /StatusNotifierItem`
  with `…Properties.Get org.kde.StatusNotifierItem IconPixmap` to confirm the
  bitmap is actually on the wire. **Rebuild with `cargo build` first** —
  `cargo test` does not refresh `target/debug/transcrust`, and a stale binary
  reports an empty `IconPixmap`.
- TDT runtime from the crates.io `parakeet-rs` 0.3.5 crate (used as-shipped, not vendored).
  Functionality is layered on top of the crate, not forked into it.
- User model root at `~/.local/share/transcrust/models/` (int8 preferred, int4 supported)

## Authority
- `src/main.rs`: CLI entrypoints, smoke mode, tray startup, quit flow
- `src/model.rs`: model resolution and expected on-disk layout (int8/int4)
- `src/parakeet.rs`: service boundary, worker startup, direct ORT preflight
- `src/parakeet_ort.rs`: direct-drive Parakeet — three graphs, TDT greedy decode, per-token confidence
- `src/mode.rs`: modes as (model, profile); the `— Long` repair profile
- `src/dictionary.rs` + `src/postprocess.rs`: post-transcription pipeline (see contract below)
- TDT greedy decode (incl. duration-head frame-skip) lives in the `parakeet-rs` crate's
  `model_tdt.rs`; transcrust uses it as-is — see Mutation Notes for the decode decision.
- **The crate is why Parakeet looks like it has no confidence signal.** It runs
  the decode internally and returns a `String`, discarding the joint's vocab
  logits. Those logits are in `decoder_joint-model.onnx` and driving the two
  sessions directly with `ort` — the pattern `granite.rs` and `vibevoice.rs`
  already use — yields per-token probability, word confidence and timestamps
  from the files already on disk. `murmure` does exactly this and does not
  depend on the crate at all. `Parakeet-v3.md` §5 has the detail;
  `TASKBOARD-next.md` Phase B is the work.

## Observations
- The original failure surface was not “bad model files”; it was an ORT/runtime mismatch.
- `ort 2.0.0-rc.12` plus the Nix-provided runtime wedged before model open.
- A matched `ort 2.0.0-rc.10` stack loaded `nemo128`, Whisper, and the Parakeet encoder/decoder normally.
- The custom ORT logger callback also caused probe crashes and is intentionally not used now.

## Post-Processing Contract
- The post-transcription pipeline lives in `src/postprocess.rs::fix_transcription`,
  called once from `main.rs::run_transcription_pipeline` after the worker returns text.
- Order: course-correction → repetition cleaning → filler removal → spoken
  punctuation → **phonetic dictionary** (`src/dictionary.rs`) → deterministic
  capitalization (`capitalize_sentences`, last).
- **Engine-agnostic by construction.** `TranscriptionService` (`src/transcription.rs`)
  is the only seam between the engines; every arm returns raw trimmed text and
  none does any word, phrase, or vocabulary work of its own. There is exactly
  one `fix_transcription` call on the live path, so all three engines get
  identical correction, filler, punctuation, dictionary, and casing treatment.
  Anything engine-specific belongs *before* that seam, not inside postprocess.
  The `— Long` profile (`mode::apply_profile`) is exactly that: it runs before
  the seam and hands the shared pipeline ordinary prose.
- The engines hand it differently shaped raw text — Parakeet TDT emits its
  own casing and punctuation, Granite's CTC head emits bare lowercase with none,
  which the `— Long` mode profile partly repairs before the shared seam.
  `postprocess.rs`'s `handles_granite_shaped_bare_lowercase_output` and
  `handles_parakeet_shaped_cased_punctuated_output` pin both shapes to the same
  result; keep that pair green when touching the pipeline.
- Granite's `tokenizer.json` is **ByteLevel BPE** (GPT-style `Ġ`, not SentencePiece
  `▁`) and does carry a `decoder` block, so `Tokenizer::decode(&ids, true)` returns
  normally spaced words that the `\b`-based passes can chew. Blank is id 0
  (`<|blank|>`, flagged special); `greedy_ctc_ids` drops it explicitly *and*
  `decode` skips specials. Verified end-to-end via `--granite-smoke`.
- Vocabulary is the phonetic dictionary and nothing else. No engine does
  decode-time biasing: Parakeet's `vocab.txt` and the other two engines'
  `tokenizer.json` are model vocabularies, not user vocabularies.
  `~/.config/transcrust/dictionary.txt` is the single user-facing vocabulary
  surface for both.
- The dictionary is the ported murmure leg: `rphonetic` with the `embedded_bm`
  feature (Beider-Morse rules compiled into the binary, no resource dir). It runs
  English-only via plain `encode()` — no language set, no French. It does a
  whole-token phonetic swap against `~/.config/transcrust/dictionary.txt`, and is
  a no-op when that file is absent.
- **Harper is intentionally not in this pipeline.** `harper-core` is still a
  dependency and its known-good config survives as dead code in `postprocess.rs`,
  but it is not called. Reason: Harper is a probabilistic grammar layer that
  re-ranks tokens toward general English *before* the dictionary can claim them —
  it was splitting `tori` into `tor i`, starving the phonetic corrector. The app's
  job is exact non-standard vocabulary; a general-English re-ranker is the wrong
  shape at this stage, and murmure (the source pipeline) never put grammar here
  either. Removing Harper unblocked `tori -> Tauri`.
- Consequence: no capitalization/punctuation grammar. Casing is left as the model
  emitted it. If that becomes a problem, the answer is a small purpose-built
  deterministic pass (or a toggled local post-processor), not re-adding Harper.

## Mutation Notes
- `parakeet-rs` is consumed from crates.io (0.3.5), not vendored. The earlier vendored
  TDT-only copy has been replaced by the published crate; the intent is to build on the
  crate and add functionality outside it rather than fork it.
- The crate's TDT decode keeps the duration head and frame-skips (`model_tdt.rs`). murmure's
  no-skip decode (leg 2) was reviewed and deferred — it would require forking the crate, and
  the payoff is second-order for short dictation. See `murmure.md` and `Smoke-Human-2026-06-08.md`.
- Int8 is preferred for normal use; int4 is supported (working local layout). Mixed fp32
  directories are no longer the target operator path.
- `--smoke` is the permanent high-observability path and should stay available even after UX cleanup.
- The authoritative operator-facing app now lives at the repo root; the earlier investigation subprojects were removed after the working runtime was promoted.
- Local-share model resolution accepts both canonical int8 filenames and the older `encoder-int8` / `decoder_joint-int8` layout so the user model cache remains valid without renaming.

## Removed: VibeVoice ASR Streaming 1.5B
Evaluated 2026-09-05 and removed 2026-09-06. It lost on every axis that matters
here: 1.8 GB against Parakeet's 391 MB, RTF 1.1-1.6 against 0.18, and no
measurable output-quality advantage. Its real strengths — CJK, speaker
attribution, 60-minute single-pass tracking — are not what this app does, and
Parakeet v3 already covers 25 European languages natively.

Full weights inventory, time-to-length tables and the postmortem live at
`~/syncthing/vibevoice-asr-15/`. The two findings worth carrying forward are
recorded in `Parakeet-v3.md`: an autoregressive decoder's cost scales with how
much was *said*, not clip length; and a traced ONNX graph can declare a dynamic
axis it does not honour.

`tools/vibevoice-export/` is retained as the ONNX export template — it is the
starting point for the Granite re-export that `TASKBOARD-next.md`
lists as blocked.
