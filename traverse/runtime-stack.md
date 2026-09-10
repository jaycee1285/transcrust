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
- `src/wav.rs`: `--wav` — offline files through the live seam, plus the windowing
- `src/audio.rs`: capture, and the polyphase resampler every engine feeds through
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

## The Resampler
- `audio.rs::resample` is a **band-limited polyphase decimator** as of
  2026-09-09, replacing linear interpolation that had no anti-aliasing filter at
  all. Cutoff 0.45 × the lower Nyquist, 16 sinc zero crossings, Blackman window.
- **Linear interpolation is a filter, just a terrible one** — a two-tap average
  whose first null sits at the input rate. On 44100 Hz capture it attenuated
  12 kHz by 2.1 dB and folded it onto 4 kHz, mid speech band.
- The bank is precomputed per rate pair. 44100→16000 reduces by gcd 100 to
  **160 phases**, so the inner loop is a dot product with no `sin` in it. Without
  that the naive form calls `sin` once per tap — 98 taps per output sample — and
  the cost stops being ignorable. Measured **18 ms per 10-second clip**.
- **Do not read this as a quality win.** A.2 in `TASKBOARD-next.md` measured it
  on 18 minutes of real speech: 0.87% of words changed, and no reference could
  tell which version was better (7.43% vs 7.39% WER against auto-captions). It
  removes a confound; it is not itself an improvement, and nothing downstream
  should cite it as one. The open question is whether real microphone audio,
  which carries the 10-14 kHz energy a lossy codec has already discarded,
  behaves differently.
- Tests pin the *defect*, not the implementation: `alias_band_is_rejected`
  demands ≥40 dB at 10/12/14 kHz, where the replaced code sat at −1.5/−2.1/−2.8 dB.
  Any future rewrite has to clear the same bar.

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

## Offline Files (`--wav`)
- **`--wav` is how you test the live engine without a microphone.** It drives
  the same `parakeet.rs` worker the hotkey drives, so an engine change is
  verifiable end to end from a file — B.3's direct-driver swap and its crate
  fallback were both confirmed this way, on a machine with no dictation
  happening. What `--wav` does *not* exercise is capture and injection, which is
  exactly where the two bugs of 2026-09-10 lived. Use it for engine work; use a
  human smoke for anything touching the edges.
- Reference recording: `~/syncthing/Record-2.wav` — 2:23, 44.1 kHz mono, John's
  voice on his phone, deliberately enumerating his habits, tasks and two example
  commands. The A.2 spectral result, the D.0 confidence hit-rate and the
  Moonshine comparison all come from it. `tools/wer/` holds the comparison
  scripts.
- `src/wav.rs` is the offline twin of the hotkey path, and adds exactly one
  thing the live path does not need: **windowing**. Everything else — engine
  selection via `discover_modes`, the `transcribe(observer, rx, rate)` seam,
  `apply_profile` then `fix_transcription` — is the live path's, unchanged.
  It never injects; it writes `<name>.md` beside the WAV with a YAML header
  carrying engine, duration, wall time and RTF. `--mode <substring>` picks the
  engine; the head of `discover_modes` is the default.
- **Parakeet does not degrade past a long window, it throws.** At 600s in one
  call the encoder dies in ORT: `Add node /layers.0/self_attn/Add_2 … Attempting
  to broadcast an axis by a dimension other than 1. 2501 by 7501` — a positional
  table sized for ~2500 frames meeting 7500. 300s still runs. So the ceiling is
  a cliff between 300s and 600s, not a slope, and windowing is what keeps the
  app off it. Its RTF also worsens with window length well before the cliff
  (45s → 0.18, 90s → 0.20, 180s → 0.26, 300s → 0.32), so short windows are
  faster *and* safer.
- **Granite improves with length, and memory is what stops it.** It is CTC —
  pure forward pass, no decode loop — so length costs it nothing per second.
  `Parakeet-v3.md` §1 has the cost model: `ceil(dur / 10.24s) × 0.53s`, a hard
  512-frame quantum. That quantum is a *floor*, so RTF falls as the partial
  final block amortises: 0.070 at 45s, 0.068 at 90s, 0.066 at 180s, 0.064 at
  600s, asymptotic by about three minutes. A ~9% gain — small beside Parakeet's
  78% degradation over the same range, which is the real story: **the gap widens
  with every second of window.**
- What caps Granite is memory, at roughly **19 MB of peak RSS per extra second
  of window**. One 1200s call peaks at **10.5 GB**; the same audio in windows
  peaks at 1.6-1.9 GB. On a 16 GB laptop that is the whole argument.
- Net: **one 60s constant serves both engines for two unrelated reasons** —
  Parakeet because long windows are slower and eventually fatal, Granite because
  long windows are expensive. 60s sits past the knee of Granite's amortisation
  and well under Parakeet's cliff. If a per-engine budget is ever wanted,
  `TranscriptionService::timeout()` is the idiom to copy.
- **Cut in a pause, do not overlap-and-stitch.** Windows are cut at the quietest
  20ms frame within ±7s of each 45s boundary. The alternative — fixed windows
  with overlap, joined by text-level dedup — needs a heuristic on every seam,
  and a wrong guess there silently deletes real words. `wav.rs`'s
  `windows_tile_the_clip_without_gaps_or_overlap` pins that windows tile the
  clip exactly: no sample dropped, none heard twice.
- The worker's idle timeout is floored at 300s here. Windows land back to back,
  so nothing is ever idle; the floor only stops a reload between files.

### Measured 2026-09-09 — 36:35 YouTube talk
| Mode | Window | Wall | RTF | Peak RSS | WER vs auto-captions |
|---|---|---|---|---|---|
| Parakeet TDT (int4) | 45s | 7:37 | 0.208 | 1.36 GB | **4.86%** |
| Parakeet TDT (int4) | 60s | — | 0.20 | 1.21 GB | — |
| Granite TurboCTC (int8) | 45s | 2:17 | 0.062 | 1.60 GB | 13.71% |
| Granite — Long | 45s | 2:19 | 0.063 | 1.60 GB | 10.11% |
| Granite — Long | **60s** | 2:24 | 0.065 | **1.75 GB** | 10.15% |

Going 45s → 60s drops 52 windows to 39 and leaves WER and RTF unmoved, for
150 MB. That is the whole trade: **longer windows buy fewer seams, not speed.**

- **`— Long` is now measured, and it earns its place**: 13.71% → 10.11% on the
  same audio, a 26% relative cut, entirely from restoring contractions. This is
  the deterministic baseline the mode exists to provide. What it cannot reach is
  the possessive: Granite emits `today is sponsor` and `when is the last time`,
  and `today is` → `today's` is not safely reversible (`today is Tuesday`), the
  same wall `mode.rs` already documents for `I have`.
- **Granite trades away exactly the words worth transcribing.** Its WER is 2×
  Parakeet's, and the excess lands on proper nouns rather than function words.
  Counting the same terms across both transcripts: `Claude Code` 5 → 1,
  `Kimi` 13 → 5, `Grok` 13 → 9, `browserbase` 4 → 2, `Okta` 1 → 0. Granite wrote
  `kimmy`, `kimmyk 3`, `kimik 3`, `clcode`, `cl code`, `cloud code` where
  Parakeet wrote `Kimi K3` and `Claude Code`. Granite also emits no punctuation
  at all, so a long transcript arrives as one run-on per window.
- Consequence for callers: **use Granite when you want the audio skimmed cheaply,
  Parakeet when the nouns are the payload.** Anything that mines a transcript for
  names, products or numbers wants Parakeet and the extra five minutes.
- These mis-hearings are precisely the case `dictionary.rs` was ported for —
  `kimmy` → `Kimi` is a textbook Beider-Morse collision. A populated
  `~/.config/transcrust/dictionary.txt` would recover much of Granite's proper-noun
  gap, at the cost of knowing the vocabulary in advance. Untested here.
- **The shared pipeline is a dictation pipeline, and `--wav` inherits that.**
  Filler removal deletes a speaker's real `you know`, and the spoken-punctuation
  pass turns a literal spoken "question mark" into `?`. Both are correct when
  you are dictating and wrong when you are transcribing someone else. On the
  clip above it cost ~10 words in 7569 (0.13%), so it is not the WER driver —
  but on a rambling speaker it would be. If that becomes a problem the answer is
  a `Verbatim` profile in `mode.rs`, not a special case inside `postprocess`.
- The WER figures measure agreement with **yt-dlp's auto-captions, not truth**.
  Sampling Parakeet's disagreements, most are yt-dlp's errors: it wrote `codeex`,
  `grock`, `octa`, `kimmy`, `browser base`. Read 4.86% as a ceiling on Parakeet's
  real error rate, and the Parakeet-to-Granite ratio as the reliable signal.

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
