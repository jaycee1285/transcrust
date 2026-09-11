#!/usr/bin/env bash
# Remove everything install.sh created. Prints what it will delete and asks
# first, because the Hugging Face checkpoint alone is a 5.6 GB re-download.
#
#   ./uninstall.sh          remove the installed model(s)
#   ./uninstall.sh --all    also remove the build tree, the venv and the
#                           Hugging Face checkpoint cache
#   ./uninstall.sh --yes    do not ask
set -euo pipefail

cd "$(dirname "$0")"

ALL=0
ASSUME_YES=0
while [ $# -gt 0 ]; do
  case "$1" in
    --all) ALL=1; shift ;;
    --yes|-y) ASSUME_YES=1; shift ;;
    -h|--help) sed -n '2,9p' "$0"; exit 0 ;;
    *) echo "unknown flag: $1" >&2; exit 1 ;;
  esac
done

MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/transcrust/models"
HF_CACHE="${HF_HOME:-$HOME/.cache/huggingface}/hub/models--microsoft--VibeVoice-ASR-Streaming-1.5B"

TARGETS=()
for candidate in "$MODELS_DIR"/vibevoice-asr-streaming-1.5b-*; do
  [ -d "$candidate" ] && TARGETS+=("$candidate")
done
if [ "$ALL" -eq 1 ]; then
  [ -d build ] && TARGETS+=("$PWD/build")
  [ -d .venv ] && TARGETS+=("$PWD/.venv")
  [ -d "$HF_CACHE" ] && TARGETS+=("$HF_CACHE")
fi

if [ ${#TARGETS[@]} -eq 0 ]; then
  echo "Nothing to remove."
  exit 0
fi

echo "Will delete:"
for target in "${TARGETS[@]}"; do
  printf '  %-8s %s\n' "$(du -sh "$target" 2>/dev/null | cut -f1)" "$target"
done

if [ "$ASSUME_YES" -eq 0 ]; then
  read -r -p "Proceed? [y/N] " answer
  case "$answer" in
    y|Y|yes|YES) ;;
    *) echo "Aborted."; exit 1 ;;
  esac
fi

for target in "${TARGETS[@]}"; do
  rm -rf "$target"
  echo "removed $target"
done

if [ "$ALL" -eq 0 ]; then
  echo
  echo "The build tree, the venv and the 5.6 GB Hugging Face checkpoint are still here."
  echo "Pass --all to remove those too."
fi
