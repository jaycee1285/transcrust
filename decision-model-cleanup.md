# Decision-model cache cleanup

## November14 pipeline build artifacts

No model payload downloaded or relocated. Audio replays read original corpus
WAVs; outputs and frozen glossary are under ignored `logs/pipeline-20261114`
(about0.4MiB). Temporary config/observer logs: `/tmp/transcrust-pipeline-20261114`.
`target/release` snapshot:3,095,691,264 allocated bytes after app and regression
builds, including standalone ORT feature variants. Recreating it costs the
documented Cargo test/build commands; it holds no live installed model bundle.
No debug tree created. Do not clear the build tree as part of a feature handoff
without agreeing that rebuild cost.

Rollback copy `~/.local/libexec/transcrust/transcrust-before-pipeline-20261114`
is35,159,968 bytes. Retain until human acceptance of the new pipeline; this is
the preceding working executable, not another weight cache. Installed current
binary/source/evidence are recorded in `pipeline-2026-11-14.md`.

## November 11 sweep

John approved relocation, preserving the executable, clearing build outputs,
an app-only rebuild and verification. All five machine steps are complete;
live microphone/editor acceptance remains John's check.

The Q4 bundle is installed at
`~/.local/share/transcrust/models/ultrabert-380m-curation-q4` and the FP32
reference at
`~/.cache/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d`.
Both hashes match their manifests/source constants. Existing research scripts
and the Vela baseline now resolve those paths. The application no longer uses
the build-tree Ultra fallback.

Preserved executable SHA256 is
`90bcb78fbc33445827ee7ac0fdbd96b2cad626d8d0c84f840f32631aae9baaa3`,
at `~/.local/libexec/transcrust/transcrust-before-cleanup-20261111`.
The active executable path is `~/.local/libexec/transcrust/transcrust` and
`launch-smoke` / `launch-before-cleanup` in the same directory capture the Nix
environment, working directory and flags. No active daemon existed at the
start of this sweep. The preserved app loaded installed Q4 with default
resolution, repaired Gamble→Gambol and ran the Parakeet WAV seam successfully.

Removed Cargo output before the fresh build:

| Directory | Allocated bytes |
|---|---:|
| `target/debug` | 7,263,883,264 |
| `target/release` | 5,027,307,520 |
| emptied `target/schema-probe` | 700,416 |

Total removed 12,291,891,200 bytes, approximately 11.45 GiB. Repo size immediately
after removal was 48,717,824 bytes, with model payloads now outside the repo.
`release.sh` uses `cargo build --release --offline -p transcrust --bin transcrust`.

The fresh app-only release build passed. Its executable and installed copy both
hash to `2ae7008cb62b03decdeb8def1d0f1bede3bc99989971935c78fe598e4a14aedc`.
Normal CLI checks match the preserved executable exactly: Gamble→Gambol through
`--fix --dec`, and the transcript body from `chat-05s.wav --mode parakeet`.
Evidence is in ignored `logs/cleanup-20261111/`; the reproducible audio fixture
remains `tools/bench-clips/chat-05s.wav`.

After rebuilding, the repository occupies 1,411,751,936 allocated bytes
(1.31 GiB), down from 14,108,475,392 bytes (13.14 GiB). Fresh `target/release`
occupies 1,362,989,056 bytes (1.27 GiB). No debug/schema build tree remains.
This is a repository reduction of 12,696,723,456 bytes (11.82 GiB), not all
freed disk: Q4 (235,134,976 bytes) and FP32 (1,532,755,968 bytes) were relocated
and retained. The FP32 cache can be reconstructed from the pinned revision
using `tools/schema-probe/run.sh ultra`; Q4 can be regenerated with
`tools/schema-probe/int4.sh`. Removing either requires a deliberate decision.

The stable launcher runs as `transcrust-cleanup-smoke.service`, with
`--dec --smoke` and default Parakeet. Startup, installed Ultra loading, audio
banking and StatusNotifierWatcher registration were verified. After stopping
that unit, it can be started with
`~/.local/libexec/transcrust/launch-smoke`. Hold Print, speak into an editor,
release, then use correct-last from the tray. Human acceptance is pending.
The requested temporary `11-11.md` is removed after durable records are updated.

### Original proposal, now executed above

Pocket and GLiDE are card reviews and queued trials. Their weights have yet
to be fetched. Assign each trial a named cache directory before its first run.

| Path | Size | Disposition to review |
|---|---:|---|
| `target/schema-probe/ultra-q4/` | 225 MiB | Install in `~/.local/share/transcrust/models/ultrabert-380m-curation-q4`, verify normal `--dec` loading, then retire the build-tree copy. The schema and Vela comparison runners also refer to the old location and need to resolve the installed bundle for their reruns. |
| `target/schema-probe/ultra/` | 1.5 GiB | Decide whether to keep the FP32 reference in a model cache or fetch it on demand. `tools/schema-probe/run.sh ultra` pins the source revision, and `tools/schema-probe/int4.sh` uses the reference for quantization and FP32 comparisons. |
| `target/debug/` | 6.8 GiB | Rebuildable Cargo output. Clearing it costs a fresh debug/test build. |
| `target/release/` | 4.7 GiB | Rebuildable Cargo output, including the executable used by the test daemon. Retain a working executable through any cleanup and verify the replacement before retiring its old path. |

Keep the installed D1 bundle, the Syncthing model sources used by desktop or
phone, and the saved fixtures and results. The proposal above was approved and
executed; its old paths describe the starting inventory only.

## Historical November 2 sweep

The following retention paths and source layout describe November 2, before
crate extraction and the November 11 moves above. They are not current loader
instructions; the latest sweep and Models.md own current paths.

Inventory checked 2026-11-02. **Approved and applied 2026-11-02.** John explicitly
approved deleting the five listed caches. This pass removed downloaded data
for rejected or parked decision-model trials: **3.10 GiB** of allocated files.
Saved results, model comparisons, fixtures and research runners remain available.

## Removed

| Directory | Allocated disk space | Original purpose | Effect of removal |
|---|---:|---|---|
| `target/thx-01/` | 646.8 MiB | THX weights/config/tokenizer for the provisional trial | THX reruns download the pinned bundle again |
| `target/pair-probe/` | 1,005.4 MiB | DeBERTa Q8, its FP32 reference, and BGE Q8 | NLI/reranker reruns download their artifacts again |
| `target/d3-lite/` | 780.4 MiB | Packed Q8 d3 weights and public reference fixtures | d3 reruns download the pinned bundle again |
| `target/schema-probe/eg2/` | 310.1 MiB | EmbeddingGemma Q8 weights/tokenizer/reference outputs | EG2 reruns download the pinned bundle again |
| `target/vela3-fast/` | 435.5 MiB | ONNX Vela comparison, tokenizer and generated scoring executable | Vela remains a parked research option; reruns download the graph again and rebuild the scorer |

Total: **3,254,476 KiB = 3.10 GiB**, measured by `du`, not theoretical weight
sizes. None of these directories is referenced by the live application paths.
All five removals leave current dictation and the D1 backend usable.

## Retained

- **`target/schema-probe/ultra-q4/` — live primary weights.** The current
  development build resolves this directory if no installed Ultra bundle is
  present. Removing it would disable the primary decision backend.
- **`target/schema-probe/ultra/` — FP32 source/reference.** The quantization
  and paired FP32/INT4 runner requires it, even when the packed bundle exists.
- **D1's installed bundle and fallback caches.** D1 remains selectable and its
  installed path resolves outside the removed caches.
- **`tools/pair-router-probe/` — about 13 MiB of recorded evidence and fixtures.**
  Current model tests and the new model write-up depend on these files.
- **Research runners, model cards, licences and notices.** These preserve the
  tested implementations and allow rejected checkpoints to be revisited without
  reconstructing the experiment. Standalone THX/NLI/d3 runners total only about
  132 KiB; deleting them would break the reports' reproduction commands.
- **`src/ultra.rs`, `src/decision_tensor.rs`, `src/d1.rs` and their dependencies.**
  These implement the live models and are shared with the research tools.
- **Shared Needle/OpenJev bundles under other repositories.** They are not
  transcrust-owned disposable caches and may serve other apps.
- **General `target/debug/` and `target/release/` trees.** They contain the live
  binary and shared build artifacts, not exclusively rejected-model leftovers.

Do **not** run a blanket `cargo clean` or remove `target/schema-probe/`: the live
INT4 bundle is in that tree. This cleanup removed only the five listed caches.

## Verification

All five directories are absent. The live Q4 artifact and installed D1 manifest
remain present. The release UltraBERT text probe loads successfully and repairs
Gamble → Gambol (.46 s decision time). The explicit D1 backend also loads and
preserves the ordinary gambling verb (.79 s). No model rebuild/download was
needed for either retained backend. Live microphone/injection smoke remains
the human handoff below; the daemon was not restarted by these text checks.

After exiting the current tray daemon, the usual live smoke command is:

```sh
nix develop -c cargo run --release -- --dec --smoke
```

Approval order comes from
[ponytail-clean](/home/john/.agents/skills/ponytail-clean/SKILL.md):
“Propose. Do not apply.” John approved the exact proposed caches before removal.
No source/runtime changes or daemon restart were made in this cleanup.
