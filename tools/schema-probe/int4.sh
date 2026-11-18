#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:-$(mktemp -d /tmp/transcrust-ultra-int4.XXXXXX)}
mkdir -p "$out"
out=$(realpath "$out")
src=${TRANSCRUST_ULTRA_REFERENCE:-${XDG_CACHE_HOME:-$HOME/.cache}/transcrust/model-references/ultrabert-380m-curation/c6a1fa055d36aeb3349b9a54537a5bc3f585d77d}
packed=${TRANSCRUST_DEC_MODEL:-${XDG_DATA_HOME:-$HOME/.local/share}/transcrust/models/ultrabert-380m-curation-q4}
if [[ ! -s $src/model.safetensors || ! -s $src/tokenizer.json ]]; then
  printf 'Download the pinned FP32 model first: nix develop -c bash tools/schema-probe/run.sh ultra\n' >&2
  exit 1
fi
cargo build --release --features schema-probe --bin schema-probe
export RAYON_NUM_THREADS=4
if [[ ! -s $packed/ultra-q4.json ]]; then
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/export-process.json" \
    target/release/schema-probe quantize-ultra "$src" "$packed" \
    2> >(tee "$out/export-stderr.txt" >&2)
fi
cp "$packed/ultra-q4.json" "$out/quantization.json"
jq 'map(select(.source != "synthetic") | .state = (.state | split("\n\nDictation:\n") |
  if length != 2 then error("unexpected frozen state") else .[1] end) |
  .candidate.state = .state)' tools/pair-router-probe/cross-encoder-q8/cases.json > "$out/no-glossary-cases.json"
for precision in fp32 q4; do
  mkdir -p "$out/$precision"
  if [[ $precision == fp32 ]]; then
    model=$src
    reference=()
  else
    model=$packed
    reference=("$out/fp32/public.json")
  fi
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$precision/public-process.json" \
    target/release/schema-probe ultra parity "$model" tools/schema-probe/fixtures/ultra-public.json \
    "$out/$precision/public.json" "${reference[@]}" \
    2> >(tee "$out/$precision/public-stderr.txt" >&2)
  for scope in full stripped; do
    if [[ $scope == full ]]; then
      cases=tools/pair-router-probe/cross-encoder-q8/cases.json
    else
      cases=$out/no-glossary-cases.json
    fi
    command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$precision/$scope-process.json" \
      target/release/schema-probe ultra evaluate "$model" "$out/$precision/public.json" "$cases" \
      "$out/$precision/$scope.json" 2> >(tee "$out/$precision/$scope-stderr.txt" >&2)
    rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
    "$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
      "$out/$precision/$scope.json."*.txt > "$out/$precision/$scope-scores.tsv"
    cat "$out/$precision/$scope-scores.tsv"
    printf '%s / %s: Phonon with neighboring context:\n' "$precision" "$scope"
    cat "$out/$precision/$scope.json.phonon-two.txt"
    printf '\n'
  done
done
cargo build --release --bin d1-context-probe
mkdir -p "$out/d1"
jq 'map(select(.source != "synthetic"))' tools/pair-router-probe/cross-encoder-q8/cases.json > "$out/d1/cases.json"
command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/d1/process.json" \
  target/release/d1-context-probe "$out/d1/cases.json" "$out/d1/results.json" \
  2> >(tee "$out/d1/stderr.txt" >&2)
for source in parakeet phonon; do
  for arm in single two; do
    jq -r --arg source "$source" --arg arm "$arm" \
      '.household[]|select(.source==$source and .arm==$arm)|.delivered' \
      "$out/d1/results.json" > "$out/d1/$source-$arm.txt"
  done
done
"$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
  "$out/d1/parakeet-"*.txt "$out/d1/phonon-"*.txt > "$out/d1/scores.tsv"
for scope in full stripped; do
  jq -s '[range(0; (.[0].rows|length)) as $i |
    .[0].rows[$i] as $a | .[1].rows[$i] as $b |
    if $a.case != $b.case then error("comparison inputs differ") else
    {id:$a.case.id,source:$a.case.source,arm:$a.case.arm,heard:$a.case.heard,
     fp32:$a.choice,q4:$b.choice,same:($a.choice==$b.choice),
     probability_drift:([range(0;3) as $j | ($a.probabilities[$j]-$b.probabilities[$j])|fabs]|max)} end] |
    {count:length,agrees:(map(select(.same))|length),max_probability_drift:(map(.probability_drift)|max),rows:.}' \
    "$out/fp32/$scope.json" "$out/q4/$scope.json" > "$out/$scope-comparison.json"
  jq '{count,agrees,max_probability_drift,changes:[.rows[]|select(.same|not)]}' "$out/$scope-comparison.json"
done
jq -s '[range(0;(.[0].rows|length)) as $i |
  .[0].rows[$i].native as $a | .[1].rows[$i].native as $b |
  {id:.[0].rows[$i].id,max_probability_drift:([range(0;($a|length)) as $j |
    ($a[$j]-$b[$j])|fabs]|max)}] |
  {max_probability_drift:(map(.max_probability_drift)|max),rows:.}' \
  "$out/fp32/public.json" "$out/q4/public.json" > "$out/public-comparison.json"
jq -s '{d1_peak_rss_kib:.[0].peak_rss_kib,q4_full_peak_rss_kib:.[1].peak_rss_kib,
  q4_stripped_peak_rss_kib:.[2].peak_rss_kib,
  stripped_rss_reduction_percent:(100*(1-.[2].peak_rss_kib/.[0].peak_rss_kib)),
  full_rss_reduction_percent:(100*(1-.[1].peak_rss_kib/.[0].peak_rss_kib))}' \
  "$out/d1/process.json" "$out/q4/full-process.json" "$out/q4/stripped-process.json" > "$out/resources.json"
cat "$out/resources.json"
sha256sum tools/schema-probe/*.rs tools/schema-probe/*.sh tools/schema-probe/fixtures/*.json \
  tools/pair-router-probe/cross-encoder-q8/cases.json crates/decision/src/d1.rs crates/decision/src/ultra.rs crates/decision/src/decision_tensor.rs tools/d1-context-probe/main.rs Cargo.lock > "$out/provenance.sha256"
printf 'INT4 evidence: %s\n' "$out"
