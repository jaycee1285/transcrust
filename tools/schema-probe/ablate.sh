#!/usr/bin/env bash
# Reuse independent gates, remove only the global glossary table from state.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=$(realpath "${1:?pass a completed run directory}")
selection=${2:-both}
case "$selection" in
  both) models=(eg2 ultra);;
  eg2|ultra) models=("$selection");;
  *) exit 2;;
esac
jq 'map(select(.source != "synthetic") |
  .state = (.state | split("\n\nDictation:\n") |
    if length != 2 then error("unexpected frozen state format") else .[1] end) |
  .candidate.state = .state)' tools/pair-router-probe/cross-encoder-q8/cases.json > "$out/no-glossary-cases.json"
export RAYON_NUM_THREADS=4
score_bin=$out/score
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$score_bin"
for model in "${models[@]}"; do
  if [[ $model == ultra ]]; then
    model_dir=${TRANSCRUST_ULTRA_REFERENCE:-${XDG_CACHE_HOME:-$HOME/.cache}/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d}
  else
    model_dir=${XDG_CACHE_HOME:-$HOME/.cache}/transcrust/model-trials/embeddinggemma-2-e7a9e1fe811e00be92484ae463f78909b0981c2c
  fi
  printf '%s: controlled glossary-removal ablation\n' "$model"
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$model/no-glossary-process.json" \
    target/release/schema-probe "$model" evaluate "$model_dir" \
    "$out/$model/parity.json" "$out/no-glossary-cases.json" "$out/$model/no-glossary.json" \
    2> >(tee "$out/$model/no-glossary-stderr.txt" >&2)
  "$score_bin" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
    "$out/$model/no-glossary.json."*.txt > "$out/$model/no-glossary-scores.tsv"
  cat "$out/$model/no-glossary-scores.tsv"
  printf '%s: Phonon, two-sentence output without glossary:\n' "$model"
  cat "$out/$model/no-glossary.json.phonon-two.txt"
  printf '\n'
done
