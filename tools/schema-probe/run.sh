#!/usr/bin/env bash
# Native CPU probes. First run downloads ~1.9 GB for both models.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
selection=${1:-both}
out=${2:-$(mktemp -d /tmp/transcrust-schema.XXXXXX)}
mkdir -p "$out"
out=$(realpath "$out")
case "$selection" in
  both) models=(eg2 ultra);;
  eg2|ultra) models=("$selection");;
  *) printf 'Usage: %s [both|eg2|ultra] [output-directory]\n' "$0" >&2; exit 2;;
esac
cargo build --release --features schema-probe --bin schema-probe
export RAYON_NUM_THREADS=4
jq -r '"Original Phonon input:\n" + .input' tools/pair-router-probe/phone-household-q8/evidence/phonon-decision.json
for model in "${models[@]}"; do
  if [[ $model == ultra ]]; then
    dir=${TRANSCRUST_ULTRA_REFERENCE:-${XDG_CACHE_HOME:-$HOME/.cache}/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d}
  else
    dir=${XDG_CACHE_HOME:-$HOME/.cache}/transcrust/model-trials/embeddinggemma-2-e7a9e1fe811e00be92484ae463f78909b0981c2c
  fi
  mkdir -p "$dir" "$out/$model"
  if [[ $model == eg2 ]]; then
    base=https://huggingface.co/Jacqkues/embeddinggemma-2-typed-decision-decisionjs/resolve/e7a9e1fe811e00be92484ae463f78909b0981c2c
    files=(weights.bin manifest.json tokenizer.json reference.json)
    fixtures=tools/schema-probe/fixtures
  else
    base=https://huggingface.co/davanstrien/ultrabert-380m-curation/resolve/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d
    files=(model.safetensors tokenizer.json)
    fixtures=tools/schema-probe/fixtures/ultra-public.json
  fi
  for file in "${files[@]}"; do
    if [[ ! -s $dir/$file ]]; then
      curl -fL "$base/$file" -o "$dir/$file.partial"
      mv "$dir/$file.partial" "$dir/$file"
    fi
  done
  printf '%s: checking public reference outputs\n' "$model"
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$model/parity-process.json" \
    target/release/schema-probe "$model" parity "$dir" "$fixtures" "$out/$model/parity.json" \
    2> >(tee "$out/$model/parity-stderr.txt" >&2)
  printf '%s: frozen household corrections\n' "$model"
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$model/correction-process.json" \
    target/release/schema-probe "$model" evaluate "$dir" "$out/$model/parity.json" \
    tools/pair-router-probe/cross-encoder-q8/cases.json "$out/$model/corrections.json" \
    2> >(tee "$out/$model/correction-stderr.txt" >&2)
  score_bin=$out/score
  rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$score_bin"
  "$score_bin" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
    "$out/$model/corrections.json."*.txt > "$out/$model/phone-scores.tsv"
  cat "$out/$model/phone-scores.tsv"
  printf '%s: Phonon, two-sentence full-glossary output:\n' "$model"
  cat "$out/$model/corrections.json.phonon-two.txt"
  printf '\n'
  sha256sum tools/schema-probe/*.rs crates/decision/src/ultra.rs crates/decision/src/decision_tensor.rs tools/schema-probe/fixtures/*.json Cargo.lock > "$out/$model/provenance.sha256"
done
bash tools/schema-probe/ablate.sh "$out" "$selection"
printf 'Native correction evidence: %s\n' "$out"
