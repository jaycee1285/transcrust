# Model registry

Maintained 2026-11-19. Read this before model selection, downloads, update
checks, quantization or cleanup. `AGENTS.md` includes it in the agent read order.
`TASKBOARD.md` owns scheduling and acceptance. The linked experiment reports
contain measurements. Update this registry whenever a model is introduced,
tested, promoted, rejected, relocated or removed.

**Zed migration, 2026-11-13:** installed Q4 and Parakeet hashes verified unchanged;
Parakeet/Nemotron/Granite, UltraBERT and D1 ran on the AMD laptop. The optional
UltraBERT FP32 reference cache listed below is absent on Zed; recreate only for
research/requantization. No weights were fetched or changed in this migration.
See [device verification and smoke](migration-2026-11-13.md).

This inventory covers model repositories explicitly referenced in this repo,
the supplied model cards and our model discussions. Related base models and
format variants are retained below with their limited status. Dataset, Space,
organization and documentation URLs are separate resources.

## Update watchlist

Check these during a model-review session or the model sweep before closeout.
Use Git metadata and small README/config/source blobs first. Compare actual
weight pointers, tokenizer, trained head and prompt code with the last reviewed
or tested revision. Record the check date, old/new revisions and what changed.
A documentation or SDK change can leave the checkpoint identical. Changed
checkpoints need the same frozen correction/ASR comparisons before promotion.

| Repository | Last upstream check | Review baseline and next check |
|---|---|---|
| [fastino/GLiDE-0.4B](https://huggingface.co/fastino/GLiDE-0.4B) | 2026-11-19, HEAD `6bda7ab91aeb36dcad6d27828a8871e548c7fc71` | Same HEAD as November 10 review. Next decision-model trial in the research queue. [Review](decision-model-review-2026-11-10-glide.md). |
| [convaiinnovations/laya](https://huggingface.co/convaiinnovations/laya) | 2026-11-19, HEAD `fd4f9dbcf4fde222fd43174bdd4c3b08b789e4e1` | Compared with `85cc74ddae6109077ebe2853a9b3077fb6606a78`. Only README changed. English, multilingual and typed-decisions weight hashes, tokenizers, configs and code match. Local correction testing remains open. |
| [moondream/parakeet-ultra](https://huggingface.co/moondream/parakeet-ultra) | 2026-11-19, HEAD `73175eb7aeb0d82f1e2a6b53b3aabc10a90bcd0b` | Same upstream HEAD. Source of the installed local INT4 export. The old exporter recorded no exact source revision. |

For a quick upstream check, run `git ls-remote <repository URL> HEAD`. A changed
HEAD is a prompt to inspect changes. Keep the installed/tested revision separate
from today's upstream HEAD. Regular review here is an agent workflow, with
dates recorded in this file.

## Installed model identities and locations

November14 local pipeline replay: nine saved WAVs ran through both current
Parakeet and Nemotron with primary UltraBERT Q4. Weights/revisions unchanged;
candidate policy now excludes ordinary function-word swaps and lowercase-only
case edits. Proper-name repair retained. [Results and limits](pipeline-2026-11-14.md).
The November 19 inventory and cleanup candidates are in [the cleanup ledger](decision-model-cleanup.md).

November 19 same-WAV phone study used installed Nemotron INT4, Parakeet Ultra
INT4, retained community Parakeet v3 INT4 and UltraBERT Q4. Whole/6/12 s and
repeated-opening arms preserve those identities. Weights are unchanged.
[Results and exact-input evidence](~/syncthing/transcrust-work/window-study/README.md).

### Parakeet Ultra is installed under the legacy v3 directory name

The active Parakeet directory is
`~/.local/share/transcrust/models/parakeet-tdt-0.6b-v3-int4`, a symlink to
`~/syncthing/models/parakeet-tdt-0.6b-v3-int4`. Its encoder is our local
**moondream/parakeet-ultra INT4 export**, built September 25. On November 11,
embedded ONNX stack traces identify `tools/ultra-export/export_ultra.py` and
`kestrel.models.parakeet_tdt`. That exporter sets
`SOURCE_MODEL = "moondream/parakeet-ultra"`. Mopvox's September 25 notes also
identify Ultra in this directory. [Exporter and measurements](tools/ultra-export/README.md).

The app's directory name and `src/model.rs` download URL still describe the
earlier NVIDIA v3/community export. Treat the installed payload and downloader
source as distinct identities. Preserve this distinction when reinstalling or
rebuilding. The preprocessor and vocabulary were reused from the v3 layout.

Installed artifact SHA256 values, checked 2026-11-11:

| Artifact | SHA256 |
|---|---|
| `encoder-model.int4.onnx` | `b4d10153243c706b4a760d7dda404309c7c602fe40daa09a7758748f92c76860` |
| `decoder_joint-model.int8.onnx` | `50ec6c71febb25f723db5cdd58fdbdf33f1ca6dc709e3d923361df7357dbcc93` |
| `nemo128.onnx` | `a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f` |

### Other installed assets

| Asset | Actual role and location | Provenance / outstanding detail |
|---|---|---|
| UltraBERT curation Q4 | Primary `--dec`, installed at `~/.local/share/transcrust/models/ultrabert-380m-curation-q4`. A verified copy under `~/syncthing/models/ultrabert-380m-curation-q4` serves the Mopvox JNI test. | Source revision `c6a1fa055d36aeb3349b9a54537a5bc3f585d77d`. Packed SHA256 `75ffd6cdf6c147ecaceab1878e388701d4feadcf69bfc8bd88a59cbd69ab8869`. Research callers use the installed path or `TRANSCRUST_DEC_MODEL`. |
| UltraBERT FP32 reference | Expected at `~/.cache/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d`. Absent on Zed, rechecked November 19. | Source SHA256 `67ac518c172934138f5d0bb06aa34523c9d9dfb6261f5dac3c860ab3f3542e91`. Recreate for comparisons with `nix develop -c bash tools/schema-probe/run.sh ultra`, or requantize with `nix develop -c bash tools/schema-probe/int4.sh`. `TRANSCRUST_ULTRA_REFERENCE` overrides the cache path. |
| D1 text Q8 | Selectable `--dec-backend d1`, installed at `~/.local/share/transcrust/models/d1-omni-600M-q8`. | Local manifest pins LiquidAI `02b55d7076f15129e59ab3f94783f32c4b088674` and ONNX community export `4ebc1b97bf1477485371c79d1cf8d5e4e8eebfa0`, with file hashes. |
| Nemotron INT4, 1120ms | Shared desktop/phone asset at `~/syncthing/models/nemotron-speech-en-0.6b-onnx-1120ms-int4`, symlinked into desktop models. | Installed README identifies the NVIDIA base and shameez FP16 export. Exact published INT4 repository/revision remains a provenance gap. Keep this separate from the retired local 560ms quantization. |
| Granite TurboCTC INT8 | Installed at `~/.local/share/transcrust/models/granite-speech-5.0-470m-turboctc`. | Original export project was removed. Local config has vocab16384, but exact source revision is unrecorded. Preserve graph/tokenizer together. Current upstream and old local measurements need separate identities. |
| Phonon 2.5 original packed | `~/syncthing/models/phonon-2.5-english-native`, with desktop symlink. | `SOURCE.json` pins revision `da200358d638886bc96c758c672ad5568828b119`, archive/container/config hashes. Desktop evaluation retired. Preserve for mobile/Apple work. |
| Phonon 2.5 converted C layout | Android asset at `~/syncthing/models/phonon-2.5-english-c`. | Same pinned original checkpoint, Rust conversion and converted-weight hashes in `SOURCE.json`. Phone C runtime is distinct from the native Rust implementation. |
| Whistle | Historical phone trial assets under `~/syncthing/models/whistle`. | Replaced by Phonon 2.5 in the live IME. Shared files may still support research. |
| s1-mini | Optional text normalization model under `~/.local/share/transcrust/models/s1-mini-gguf`. | Used by `--normalise` and live long-form polish. [Actual GGUF source](https://huggingface.co/superwhisper/s1-mini-GGUF). |
| Parakeet community v3 rollback | `~/syncthing/models/paraback/parakeet-tdt-0.6b-v3-int4`. | Encoder and decoder SHA256 match the [efederici community export](https://huggingface.co/efederici/parakeet-tdt-0.6b-v3-onnx-int4). The `nemo128.onnx` hash matches active Parakeet Ultra. No current loader path found. |

## ASR sources, exports and historical candidates

| Hugging Face repository | Relationship / disposition |
|---|---|
| [nvidia/parakeet-tdt-0.6b-v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) | Original Parakeet architecture/base lineage and historical v3 comparison. Ultra is post-trained from this family. |
| [istupakov/parakeet-tdt-0.6b-v3-onnx](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx) | Historical graph export and shared `nemo128.onnx` source. |
| [efederici/parakeet-tdt-0.6b-v3-onnx-int4](https://huggingface.co/efederici/parakeet-tdt-0.6b-v3-onnx-int4) | Current downloader's community v3 INT4 source. Installed encoder has since been replaced by the local Ultra export. |
| [moondream/parakeet-redux](https://huggingface.co/moondream/parakeet-redux) | Ternary sibling of Ultra. Related reference, separate weights/runtime route. |
| [mldecode/parakeet-ultra-onnx-int8](https://huggingface.co/mldecode/parakeet-ultra-onnx-int8) | Exporter notes describe incompatible sherpa split layout for our existing loader. |
| [eschmidbauer/parakeet-redux-onnx](https://huggingface.co/eschmidbauer/parakeet-redux-onnx) | Tracing-wrapper reference used for the local Ultra export. |
| [nvidia/nemotron-speech-streaming-en-0.6b](https://huggingface.co/nvidia/nemotron-speech-streaming-en-0.6b) | Base of the installed streaming model. |
| [shameez/nemotron-speech-streaming-en-0.6b-onnx-1120ms](https://huggingface.co/shameez/nemotron-speech-streaming-en-0.6b-onnx-1120ms) | FP16 export cited by the installed INT4 asset's README. |
| [danielbodart/nemotron-speech-600m-onnx](https://huggingface.co/danielbodart/nemotron-speech-600m-onnx) | FP32 source for the retired local 560ms quantization in mopvox's tools. |
| [ibm-granite/granite-speech-5.0-470m-turboctc](https://huggingface.co/ibm-granite/granite-speech-5.0-470m-turboctc) | Upstream model family of installed Granite. November11 card advertises an October21 multilingual/punctuation update. Exact installed revision still needs recovery. |
| [ibm-granite/granite-speech-5.0-470m-turboctc-nc](https://huggingface.co/ibm-granite/granite-speech-5.0-470m-turboctc-nc) | Related separately licensed checkpoint. Keep separate from the installed variant. |
| [FermionResearch/Phonon-2](https://huggingface.co/FermionResearch/Phonon-2) | Earlier mobile experiment and format-conversion reference. |
| [FermionResearch/Phonon-2.5-English](https://huggingface.co/FermionResearch/Phonon-2.5-English) | Actual 2.5 checkpoint used by native Rust and current mopvox C assets. |
| [eschmidbauer/phonon-2-c](https://huggingface.co/eschmidbauer/phonon-2-c) | Converted-weight alternative discussed before building the local pinned conversion. Engine reference code is on GitHub. |
| [Cactus-Compute/whistle](https://huggingface.co/Cactus-Compute/whistle) | Retired live IME mode, historical mobile comparison. |
| [microsoft/VibeVoice-ASR-Streaming-1.5B](https://huggingface.co/microsoft/VibeVoice-ASR-Streaming-1.5B) | Evaluated and removed. Export research remains in `tools/vibevoice-export/`. Previously cited external postmortem directory was absent at November11 inventory. |
| [superwhisper/s1-mini](https://huggingface.co/superwhisper/s1-mini) | Optional readability/normalization route, separate from acoustic transcription. |

The newly proposed Nemotron checkpoint still has no supplied URL/path. Do not
substitute a guessed checkpoint for that pending request.

## Decision models and correction experiments

GLiDE and Pocket explore D1's bounded-decision role with different architectures.
Keep model-specific rendering, tokenization, heads and calibration tied to the
checkpoint. Frozen repair/preservation cases are shared across comparisons.

| Hugging Face repository | Status and evidence |
|---|---|
| [davanstrien/ultrabert-380m-curation](https://huggingface.co/davanstrien/ultrabert-380m-curation) | Current primary, native Q4. [Quantization results](tools/pair-router-probe/ultra-int4-2026-11-01/README.md). |
| [LiquidAI/d1-omni-600M](https://huggingface.co/LiquidAI/d1-omni-600M) | Accepted desktop Q8 baseline, text-only inference. [Results index](decision-model-results.md). |
| [onnx-community/d1-omni-600M-ONNX](https://huggingface.co/onnx-community/d1-omni-600M-ONNX) | Actual Q8/Q4 graph source for D1 trials. Installed Q8 manifest above pins the export. |
| [taobao-mnn/MNN-Pocket-0.3B-Dec](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Dec) | Tested November 18 and parked. Native F32 produced fewer repairs and more unwanted edits than UltraBERT. Pinned weights at `~/.cache/transcrust/pocket-dec`. [Runner, revision and results](tools/pocket-probe/README.md). |
| [hindsight-ai/hs-decide-1-flash](https://huggingface.co/hindsight-ai/hs-decide-1-flash) | Queued after GLiDE. Hybrid Qwen3.5 typed model. Card/source review only. |
| [baselabs/joey-1](https://huggingface.co/baselabs/joey-1) | Later larger baseline. Card/source review only. |
| [Mincofficial/Minico-M1-1105](https://huggingface.co/Mincofficial/Minico-M1-1105) | Queued separate bounded-scoring research formulation. Generative LFM2 model. |
| [doofz/THX-01](https://huggingface.co/doofz/THX-01) | Native provisional test failed to improve household results. Reference parity remains open. Weights removed. [Probe](tools/thx-probe/README.md). |
| [cross-encoder/nli-deberta-v3-small](https://huggingface.co/cross-encoder/nli-deberta-v3-small) | Paired NLI formulation tested, preservation failed. Weights removed. [Comparison](tools/pair-router-probe/cross-encoder-q8/README.md). |
| [BAAI/bge-reranker-base](https://huggingface.co/BAAI/bge-reranker-base) | Original reranker family. Tested ranking formulation failed preservation. |
| [Xenova/bge-reranker-base](https://huggingface.co/Xenova/bge-reranker-base) | Actual Q8 ONNX export used by the reranker runner. Weights removed. |
| [mrm8488/embeddinggemma-2-ft-typed-decision](https://huggingface.co/mrm8488/embeddinggemma-2-ft-typed-decision) | Preservation failed. November8 revision comparison found identical weights/code and only an added evaluation YAML. [Native results](tools/pair-router-probe/schema-native-2026-10-31/README.md). |
| [Jacqkues/embeddinggemma-2-typed-decision-decisionjs](https://huggingface.co/Jacqkues/embeddinggemma-2-typed-decision-decisionjs) | Actual packed Q8 bundle used in that experiment. Weights removed. |
| [vllm-sr/d3-lite](https://huggingface.co/vllm-sr/d3-lite) | Native decision probe failed ordinary-word preservation. |
| [Jacqkues/d3-lite-decisionjs](https://huggingface.co/Jacqkues/d3-lite-decisionjs) | Actual packed Q8 bundle used by the d3 runner. Weights removed. [Probe](tools/d3-probe/README.md). |
| [vllm-sr/vela3-fast](https://huggingface.co/vllm-sr/vela3-fast) | Tested, mixed repair/preservation tradeoff and larger RAM. Parked. |
| [altertable/vela3-fast-onnx-int8](https://huggingface.co/altertable/vela3-fast-onnx-int8) | Actual ONNX export for Vela's native ORT comparison. Weights removed. [Results](tools/pair-router-probe/vela3-onnx-2026-11-01/README.md). |
| [FluidInference/clef-text-0.6b](https://huggingface.co/FluidInference/clef-text-0.6b) | Card reviewed, local typed-head/Apple evaluation still open. |
| [Cactus-Compute/needle2](https://huggingface.co/Cactus-Compute/needle2) | Earlier proposal. Actual saved tests used Needle3, with a separate GitHub/local bundle source. Preserve version distinction. |
| [convaiinnovations/laya-multilingual](https://huggingface.co/convaiinnovations/laya-multilingual) | Related current Laya checkpoint. Card reviewed as family metadata on November11. No local correction score. |
| [convaiinnovations/laya-typed-decisions](https://huggingface.co/convaiinnovations/laya-typed-decisions) | Related Laya checkpoint fine-tuned on typed workflows. Distinct from the default English root. No local correction score. |

GLiDE and Laya root links and current watch revisions are at the top of this
file. Private offline single-family use is John's confirmed scope. The prior
license discussion is settled for the GLiDE trial. Preserve each model's license
and provenance with its artifacts.

## Related bases, formats and card-linked references

These links appeared in supplied cards or local references. Their presence
records lineage and alternatives. Actual experiments are identified above.

| Hugging Face repository | Relationship |
|---|---|
| [dmis-lab/UltraBERT-380M](https://huggingface.co/dmis-lab/UltraBERT-380M) | Backbone named by the UltraBERT curation card. |
| [knowledgator/gliclass-v4-edge](https://huggingface.co/knowledgator/gliclass-v4-edge) | Related comparator in that card. |
| [LiquidAI/d1-2B](https://huggingface.co/LiquidAI/d1-2B) | Related comparator in the UltraBERT card. |
| [google/embeddinggemma-2](https://huggingface.co/google/embeddinggemma-2) | Base named by the tested EmbeddingGemma typed-decision card. |
| [Qwen/Qwen3.5-2B](https://huggingface.co/Qwen/Qwen3.5-2B) | Joey base. |
| [Qwen/Qwen3.5-0.8B](https://huggingface.co/Qwen/Qwen3.5-0.8B) | Hindsight base. |
| [Qwen/Qwen3-4B-Instruct-2507](https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507) | Comparator in the saved UltraBERT card. |
| [baselabs/elephant-1](https://huggingface.co/baselabs/elephant-1) | Larger Joey sibling mentioned in its card. |
| [baselabs/mammoth-1](https://huggingface.co/baselabs/mammoth-1) | Larger Joey sibling mentioned in its card. |
| [taobao-mnn/MNN-Pocket-0.3B](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B) | Pocket decision model's bundled base. |
| [taobao-mnn/MNN-Pocket-0.3B-MNN](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-MNN) | QAT INT4 base format linked in Pocket's card. The decision adapter/head still need to be retained. |
| [taobao-mnn/MNN-Pocket-0.3B-GGUF](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-GGUF) | Base-model GGUF variant. |
| [taobao-mnn/MNN-Pocket-0.3B-DFlash2](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-DFlash2) | Speculative-decoding draft model. |
| [taobao-mnn/MNN-Pocket-0.3B-Base](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Base) | Pretraining checkpoint. |
| [taobao-mnn/MNN-Pocket-0.3B-SFT](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-SFT) | SFT checkpoint. |
| [LiquidAI/LFM2.5-350M](https://huggingface.co/LiquidAI/LFM2.5-350M) | Minico base. |
| [LiquidAI/LFM2.5-1.2B-Thinking](https://huggingface.co/LiquidAI/LFM2.5-1.2B-Thinking) | Teacher named in Minico's training description. |
| [LiquidAI/LFM2.5-1.2B-Instruct](https://huggingface.co/LiquidAI/LFM2.5-1.2B-Instruct) | Teacher named in Minico's training description. |
| [Mincofficial/Minico-M1-Preview](https://huggingface.co/Mincofficial/Minico-M1-Preview) | Previous Minico release. |
| [Mincofficial/Minico-M1-1105-GGUF](https://huggingface.co/Mincofficial/Minico-M1-1105-GGUF) | Linked quantized format. |
| [Mincofficial/Minico-M1-1105-MLX](https://huggingface.co/Mincofficial/Minico-M1-1105-MLX) | Linked Apple format. |
| [Mincofficial/Minico-M1-1105-ONNX](https://huggingface.co/Mincofficial/Minico-M1-1105-ONNX) | Linked ONNX format. |

## References without a recorded Hugging Face checkpoint URL

Needle3 used a GitHub/local runtime bundle. OpenJev/Verdict used the local
`~/repos/dz/jev-options/rlcd-151m` bundle. API-only reference runs used
`typesafe/jev-1.13` and `cloudflare/clef-flash` on OpenRouter. Exact Hugging Face
checkpoint URLs are unrecorded for those references. Keep these gaps explicit.

## Model sweep and future additions

For every new model, add its exact repository URL and role here before fetching
weights. Record source revision, artifact precision/format, license, local home,
trial result and next action. Give trial weights a named temporary/cache home
and installed weights the directory the app discovers. The former UltraBERT
assets under `target/schema-probe/` were relocated November 11. The working
executable and launchers live under `~/.local/libexec/transcrust/`, independently
of Cargo's disposable build artifacts.

Before a sweep, inspect runtime and research callers. Keep results, fixtures,
checksums and rerun commands. Record proposed keep/move/delete paths and sizes
in `decision-model-cleanup.md`, agree dispositions with John, then verify moved
live assets and run the human smoke. Use this registry to avoid repeating a
rejected checkpoint unless weights, implementation, formulation or test scope
changed. [Existing cleanup record](decision-model-cleanup.md).

Coverage was checked against all literal Hugging Face model URLs in the repo,
including scripts, JSON metadata and saved cards, with build outputs, runtime
logs and Git internals excluded. Named repository IDs from downloader scripts,
installed manifests and the discussion were added separately. Related-only
entries retain their limited status rather than borrowing another model's test.
