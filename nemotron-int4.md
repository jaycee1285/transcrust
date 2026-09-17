# Nemotron int4 @ 1120 ms — and dropping the Parakeet int8 option

Plan to replace the installed Nemotron Speech Streaming EN int8 export (560 ms chunks,
876 MB on disk) with the int4 1120 ms export described in
`~/syncthing/nemotron-speech-en-0.6b-onnx-1120ms-int4-README.md` (~469 MB: encoder int4
MatMulNBits, fused decoder fp16), and to remove the Parakeet int8 download option.
Written 2026-09-15 from a read of `src/nemotron.rs`, `src/model.rs`, `src/main.rs`,
`design-long-form-routes.md` and `keyword-boost-prd.md`. The int4 files are not
installed yet.

## Status (2026-09-15)

- **Installed** at `~/.local/share/transcrust/models/nemotron-speech-en-0.6b-onnx-1120ms-int4/`
  (dir name carries `nemotron` for discovery and `int4` for the label).
- **`filterbank.bin` and `preprocessor.config` copied from the int8 directory**, byte-identical
  (131,584 / 136 bytes). `tokens.txt` is identical across the two exports.
- **Probed** with the installed `transcrust --probe-onnx`: every encoder and decoder input and
  output is **Float32** — risk 2 is closed, no casts needed. All probe cases load (encoder
  2.3 s, decoder 35 ms).
- **Names match the code.** `Layout::detect` resolves `audio_signal` (3-D, 128 mel),
  `length`, `outputs`, `encoded_lengths`, `cache_last_channel_next(_len)`,
  `cache_last_time_next`; dynamic cache axes fall back to the canonical 70 / 8. The decoder's
  `encoder_outputs` / `targets` / `target_length` / `input_states_1/2` match `joint_step`.
- **The directory carries a 448 MB `.git`** (LFS objects duplicating the weights); the model
  files are ~469 MB. Safe to delete if the clone isn't needed.
- **Do not select it in the tray yet.** The live chunk is still 56 frames. The graph's axes
  are dynamic, so 560 ms chunks through the 1120 ms export will run without an error and
  degrade quietly. Step 3 has to land first.

## Status (2026-09-17)

- **Step 3 landed.** `Tuning::for_export` reads `config.json` → `<n>ms` in the dir name →
  56; the live worker uses it and logs `Nemotron model loaded, 1120 ms chunks`.
- **Step 4 not needed** (probe: all float32).
- **Gate (Record-2, live streaming path):** int8 @ 560 ms 256 chunks / 425 tokens; int4 @
  1120 ms 128 chunks / 422 tokens; 3.57% word difference, 4 spans — `transcrestwork` →
  `transcrust work` (better), `parakeet` → `tarrachet` (worse), `APT track` → `at track
  start`, `NEL` → `Nell`. Outputs in `logs/margins-int4-gate/`.
- **int8 moved, not deleted,** to `~/.local/share/transcrust/retired-models/`. Delete it
  (876 MB) once the int4 toggle passes a human dictation.
- **Parakeet int8 download option removed;** int4 is the `--download-model` default.
- **Menu:** one mode per engine — `Parakeet TDT (int4)` hold, `Nemotron Toggle (int4)`,
  `Granite Speech 5 Toggle` (repair profile). Granite's raw hold entry is gone.

## Already in place

- **Chunk size is already a knob.** `Tuning.chunk_frames` (`src/nemotron.rs`) exists
  and the live loop reads it (`let chunk = self.tuning.chunk_frames`), but only the test
  harness sets it (`MARGIN_RUNS='a=<dir>@560;b=<dir>@1120'`) — that's where
  `keyword-boost-prd.md`'s 560 vs 1120 ms numbers came from. The live default is
  `CHUNK_MEL_FRAMES = 56` (560 ms).
- **The int4 export's shape matches what transcrust expects.** `encoder_model.onnx`,
  `decoder_model.onnx`, `tokens.txt`; cache shapes `cache_last_channel[1,24,70,1024]` /
  `cache_last_time[1,24,1024,8]` (same `CHANNEL_CACHE = 70`, `TIME_CACHE = 8`); 128-mel
  input; fused decoder+joint with `input_states_1/2[2,1,640]`; blank id 1024.
- **Quant label comes from the directory name.** `model_variant` falls back to the
  directory for Nemotron, so a directory named `…-1120ms-int4` labels as int4.

## Risks

1. **No `filterbank.bin` ships with the int4 export, and transcrust refuses to load
   Nemotron without one, on purpose.** `has_nemotron_model` requires it; `model.rs` notes
   that without the export's own filterbank the result was "fluent, confident, invented
   English" (Gate 3). Both exports come from the same base model
   (`nvidia/nemotron-speech-streaming-en-0.6b`) with the same preprocessor (128 mel, n_fft
   512, hop 160, win 400, preemph 0.97, Slaney), so copying `filterbank.bin` and
   `preprocessor.config` from the int8 directory should be correct — but that is an
   assumption the repo's own rules say to verify, not trust. The existing
   batch-vs-streaming frontend test plus a transcript comparison settles it.
2. **The decoder is fp16; `nemotron.rs` has no fp16 handling.** The README says I/O is
   "unchanged from the FP16 export". If that export keeps float32 inputs and outputs
   (common), nothing changes. If its tensors are float16, `joint_step` needs casts
   (~30–50 lines). `--probe-onnx` answers it in a second (Gate 1 in
   `design-long-form-routes.md`).

## Wiring it

1. **Install.** Put the files in
   `~/.local/share/transcrust/models/nemotron-speech-streaming-en-0.6b-1120ms-int4/`, and
   copy `filterbank.bin` + `preprocessor.config` in from the int8 directory.
2. **Probe.** `transcrust --probe-onnx` on both graphs: confirm tensor names and dtypes
   (encoder `audio_signal[1,128,121]` = 112 new + 9 pre-encode frames).
3. **Take the chunk size from the export, not a constant.** The probe shows the encoder's
   `audio_signal` is `[-1, 128, -1]` — dynamic — so the graph cannot say 121. Use, in
   order: `encoder.chunk_mel_frames` from the export's `config.json` when present (the
   int8 export has it: 56); else a `(\d+)ms` in the directory name (`…-1120ms-int4` → 112);
   else `CHUNK_MEL_FRAMES`. Set `Tuning.chunk_frames` from that at load. ~25 lines, plus a
   test for each source.
4. **fp16 casts** in the decoder path, only if step 2 says so.
5. **Gate it before switching.** Frontend tests, `--bench`, and `--wav` on the corpus:
   int8 @ 560 ms vs int4 @ 1120 ms. Only then delete the int8 directory (876 MB).

## Dropping the Parakeet int8 option

Parakeet int8 is not installed (only `parakeet-tdt-0.6b-v3-int4` is), so this is download
and default code only:

- Delete from `src/model.rs`: `PARAKEET_INT8_MODELS_BASE`, `PARAKEET_TDT_INT8_FILES`,
  `DEFAULT_PARAKEET_INT8_DIR`, the `("tdt-0.6b-int8", …)` entry, and the `-int8` branch
  of the download path.
- `src/main.rs` `--help`: drop the `parakeet-tdt-0.6b-int8 … (default)` line; make int4
  the default download, which is what actually runs.
- Revisit `preferred_int8_model_dir` / `required_int8_files` (doctor output) — named for
  int8 but used generically; rename or point at int4.
- **Keep `quant_rank`'s int8-first preference.** Granite (int8) and the current
  Nemotron directory depend on it; removing it silently changes which graph they load.
- Update picker test labels and any docs that mention the int8 option
  (`config.example.toml`, `Parakeet-v3.md`).

~40 lines, mostly deletions.

## Size

One normal session. Short if the probe shows float32 decoder I/O; longer if it needs
fp16 casts. Start with the probe once the files are in place.
