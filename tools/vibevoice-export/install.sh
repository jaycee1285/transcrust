#!/usr/bin/env bash
# Build the VibeVoice ASR ONNX graphs from the upstream checkpoint and install
# them where transcrust looks for models.
#
# There is no published ONNX export of this checkpoint, so unlike Parakeet
# there is nothing for `--download-model` to fetch: the graphs are built here.
# Everything is incremental — rerunning skips work that is already done.
#
#   ./install.sh                 build int4 (~1.9 GB installed) and install
#   ./install.sh --mode int8     build int8 instead (~2.6 GB, marginally better)
#   ./install.sh --keep-build    leave the intermediate fp32 graphs in ./build
#
# Peak usage while building: ~13 GB of RAM+swap and ~15 GB of disk.
set -euo pipefail

cd "$(dirname "$0")"

MODE=int4
KEEP_BUILD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --keep-build) KEEP_BUILD=1; shift ;;
    -h|--help) sed -n '2,14p' "$0"; exit 0 ;;
    *) echo "unknown flag: $1" >&2; exit 1 ;;
  esac
done

case "$MODE" in
  int4|int8) ;;
  *) echo "--mode must be int4 or int8" >&2; exit 1 ;;
esac

DEST="${XDG_DATA_HOME:-$HOME/.local/share}/transcrust/models/vibevoice-asr-streaming-1.5b-$MODE"
PYTHON=".venv/bin/python"

if ! command -v uv >/dev/null 2>&1; then
  echo "uv is not on PATH. Install it, or run this inside 'nix develop .' from this directory." >&2
  exit 1
fi

echo "==> python toolchain"
[ -x "$PYTHON" ] || uv venv --python 3.12 .venv
uv pip install --quiet --python "$PYTHON" -r requirements.txt

echo "==> export (downloads ~5.6 GB from Hugging Face on first run)"
[ -f build/speech/speech_encoder.onnx ] || "$PYTHON" export_speech_encoder.py --out build/speech/speech_encoder.onnx
[ -f build/decoder/decoder.onnx ]       || "$PYTHON" export_decoder.py       --out build/decoder/decoder.onnx

echo "==> quantise"
mkdir -p build/q
# The embedding table is written straight out at per-row int8 by its exporter;
# 4-bit rows would cost more accuracy than the 117 MB is worth.
[ -f build/q/embed_tokens.int8.onnx ]    || "$PYTHON" export_embed.py --out build/q/embed_tokens.int8.onnx
[ -f "build/q/speech_encoder.$MODE.onnx" ] || "$PYTHON" quantize_onnx.py --mode "$MODE" \
  --src build/speech/speech_encoder.onnx --dst "build/q/speech_encoder.$MODE.onnx"
[ -f "build/q/decoder.$MODE.onnx" ]        || "$PYTHON" quantize_onnx.py --mode "$MODE" \
  --src build/decoder/decoder.onnx        --dst "build/q/decoder.$MODE.onnx"

echo "==> install to $DEST"
mkdir -p "$DEST"
for name in "speech_encoder.$MODE.onnx" "embed_tokens.int8.onnx" "decoder.$MODE.onnx"; do
  cp -f "build/q/$name" "$DEST/$name"
  [ -f "build/q/$name.data" ] && cp -f "build/q/$name.data" "$DEST/$name.data"
done
"$PYTHON" - "$DEST" <<'PY'
import shutil, sys
from pathlib import Path
from common import checkpoint_dir
shutil.copyfile(checkpoint_dir() / "tokenizer.json", Path(sys.argv[1]) / "tokenizer.json")
PY

if [ "$KEEP_BUILD" -eq 0 ]; then
  echo "==> dropping intermediate fp32 graphs (pass --keep-build to keep them)"
  rm -rf build/speech build/decoder
fi

echo
du -sh "$DEST"
echo
echo "Installed. Verify with:"
echo "  nix develop -c cargo run -- --doctor"
echo "  nix develop -c cargo run -- --vibevoice-smoke $DEST"
echo "Remove with:"
echo "  ./tools/vibevoice-export/uninstall.sh"
