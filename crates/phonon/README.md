# Native Rust Phonon 2.5 English

Actual trained Phonon 2.5 graph and original packed weights, with Rust frontend,
encoder and greedy TDT decoder. No C, Python or ONNX inference runtime. Desktop
worker/capture integration lives in `../../src/phonon.rs`; mobile/Apple target
builds and exposing this engine through `transcrust-ffi` remain pending.

## Reference and memory behavior

Weights: `FermionResearch/Phonon-2.5-English`, revision
`da200358d638886bc96c758c672ad5568828b119`. Use the archive's config, which
differs from the repository's top-level config. The loader verifies the pinned
container SHA256 and configuration. Original container: 177,479,389 bytes.

Arithmetic/allocation oracle: `eschmidbauer/phonon-2-c`, revision
`c4b860f6ad745369d6f21599d6e23d381c1fcc7a`. See NOTICE and reference license.
The original container stays memory mapped. Encoder weights unpack into small
reusable panels (at most 256 KiB per worker, four workers), with MR4/NR16 exact
FP32 FMA accumulation. Subsampling follows C buffer lifetimes and in-place
bias/ReLU. Decoder tables expand to FP32 (69,813,760 bytes), as in C.

Measured 2026-11-04 on the reference i5-1345U, public 6.015s fixture
`5105-28240-0004.wav`:

| Implementation | Inference | Peak RSS |
|---|---|---|
| Rust exact, two passes | 1.336 / 1.309s | 280,968 KiB (~274 MiB), `/proc` VmHWM |
| C exact, four threads, cache off, one pass | 1.11s | 329,472 KiB (~322 MiB) |

These are standalone ASR processes; desktop UI and the decision model add RAM.
Three public clips match C transcripts on both Rust passes. All 24 encoder
blocks were compared; final encoder max absolute error 0.00000322, relative
L2 error 0.00000112. Local evidence remains under `/tmp/phonon25/`; private
household audio/transcripts are not fixtures in Git. No mobile memory claim
follows from these x86 measurements.

## Run

From the repository root, with the installed native model symlink:

```sh
TRANSCRUST_MODEL_PATH="$HOME/.local/share/transcrust/models/phonon-2.5-english-native" nix develop -c cargo run --release -- --smoke --dec
```

Exit any existing daemon first. Tap Print to start, tap again to stop. The
resident worker defaults to a six-second window cap. The root starts
Phonon on microphone press and tees chunks to it while preserving an exact
corpus copy. Delivery waits for release. These are not trained streaming caches.
Inspect delivered text, delay and correct-last behavior. `--smoke` banks the
device-rate WAV and raw/delivered transcript JSON under
`~/.local/share/transcrust/corpus/` for replay. First desktop human smoke failed
quality. The saved November 6 take matches C on every live window; see
[diagnosis and replay](../../tools/phonon-probe/smoke-2026-11-06.md). The active test unit is
`transcrust-native-phonon.service`.

For the 12-second experiment, set `TRANSCRUST_PHONON_WINDOW_SECS=12` before
launch. Supported caps are 6, 12, 18, 24 and 30 seconds. The six-second minimum
for pause cuts and three-second forced-cut lookback stay unchanged. A 12-second
cap therefore does not impose fixed boundaries every twelve seconds. The
500-word reference is [here](../../tools/phonon-probe/phoneme-script-500.md),
also copied to `~/syncthing/phonon-12s-phoneme-smoke.md`. Read the marked body
at normal pace; banking keeps one complete WAV for same-audio 6/12 replay.

Standalone public WAV probe, two passes with RSS and optional tensor trace:

```sh
nix develop -c cargo run --release -p transcrust-phonon --bin phonon-native -- MODEL_DIR PUBLIC_16K_WAV
```

Kernel tail/FMA and bounded/lossless segmentation checks:

```sh
nix develop -c cargo test --release -p transcrust-phonon --lib
```
