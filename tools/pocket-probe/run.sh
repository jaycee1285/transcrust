#!/usr/bin/env bash
# MNN-Pocket-0.3B-Dec native probe. Usage: bash tools/pocket-probe/run.sh [OUT_DIR]
# Downloads only the pinned model (~754 MB) into ~/.cache/transcrust/pocket-dec; runs offline after that.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:-tools/pocket-probe/results}
mkdir -p "$out"; out=$(realpath "$out")
model=${POCKET_MODEL_DIR:-$HOME/.cache/transcrust/pocket-dec}
rev=dfbe0aed3abd4caed175b883a4b462ed35184fad
base=https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Dec/resolve/$rev
mkdir -p "$model/base" "$model/adapter"
for f in LICENSE.txt README.md inference.py decision_config.json decision_head.safetensors temperature.json \
         adapter/adapter_config.json adapter/adapter_model.safetensors base/config.json base/tokenizer.json base/model.safetensors; do
  [[ -s $model/$f ]] || { curl -fL "$base/$f" -o "$model/$f.partial" && mv "$model/$f.partial" "$model/$f"; }
done
(cd "$model" && sha256sum -c "$OLDPWD/tools/pocket-probe/SHA256SUMS")
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$HOME/.cache/transcrust/pocket-target}
export RAYON_NUM_THREADS=${RAYON_NUM_THREADS:-4}
(cd tools/pocket-probe && cargo build --release --offline 2>/dev/null || cargo build --release)
bin=$CARGO_TARGET_DIR/release/pocket-probe
fz=tools/pair-router-probe/vela3-onnx-2026-11-01/fast
run() { # name rows-json [env...]
  local name=$1 rows=$2; shift 2
  env "$@" python3 -I tools/pocket-probe/rss.py "$out/$name-process.json" "$bin" "$model" "$rows" "$out/$name.json" 2> "$out/$name-stderr.txt"
}
run stripped   $fz/expanded.json POCKET_X=1          # headline: stripped glossary, all 132 frozen rows
run full       $fz/full.json     POCKET_X=1          # full glossary, 32 phone rows
run win3       $fz/expanded.json POCKET_WINDOW=3
run win3frame  $fz/expanded.json POCKET_WINDOW=3 POCKET_FRAME=1
run frame      $fz/expanded.json POCKET_FRAME=1
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
for n in stripped full win3 win3frame frame; do
  "$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md "$out/$n.json."*.txt > "$out/$n-scores.tsv"
done
nix develop "path:$PWD#pair-probe" -c python3 -I tools/pocket-probe/parity.py "$model" "$out/stripped.json" 6 | tee "$out/parity.txt"
python3 -I tools/pocket-probe/report.py "$out" | tee "$out/report.txt"
rm -f "$out/score"
echo "Pocket evidence: $out"
