#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:-$(mktemp -d /tmp/transcrust-vela.XXXXXX)}
surface=${2:-phone}
mkdir -p "$out" target/vela3-fast
out=$(realpath "$out")
model=$PWD/target/vela3-fast
base=https://huggingface.co/altertable/vela3-fast-onnx-int8/resolve/9afe1776af402e34ade77a62d05105d0ffe50e26
for file in model.onnx tokenizer.json; do
  if [[ ! -s $model/$file ]]; then
    curl -fL "$base/model/$file" -o "$model/$file.partial"
    mv "$model/$file.partial" "$model/$file"
  fi
done
cargo build --release --bin vela-probe
rustc tools/pair-router-probe/phone-household-q8/score.rs -o target/vela3-fast/score
for mode in fast low; do
  mkdir -p "$out/$mode"
  printf 'Vela %s: public example and packing checks\n' "$mode"
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$mode/public-process.json" \
    target/release/vela-probe public "$model" "$mode" "$out/$mode/public.json" \
    2> >(tee "$out/$mode/public-stderr.txt" >&2)
  for scope in full stripped; do
    command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$mode/$scope-process.json" \
      target/release/vela-probe evaluate "$model" "$mode" "$out/$mode/public.json" "$scope" "$surface" \
      "$out/$mode/$scope.json" 2> >(tee "$out/$mode/$scope-stderr.txt" >&2)
    target/vela3-fast/score tools/pair-router-probe/phone-household-q8/evidence/reference.md \
      "$out/$mode/$scope.json."*.txt > "$out/$mode/$scope-scores.tsv"
    cat "$out/$mode/$scope-scores.tsv"
    printf 'Vela %s/%s: Phonon with neighboring context\n' "$mode" "$scope"
    cat "$out/$mode/$scope.json.phonon-two.txt"
    printf '\n'
  done
done
if [[ $surface == phone ]]; then
  printf 'Vela fast/stripped: expanded frozen surface\n'
  command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/fast/expanded-process.json" \
    target/release/vela-probe evaluate "$model" fast "$out/fast/public.json" stripped all \
    "$out/fast/expanded.json" 2> >(tee "$out/fast/expanded-stderr.txt" >&2)
fi
if [[ $surface == phone ]]; then baseline_input=$out/fast/expanded.json; else baseline_input=$out/fast/stripped.json; fi
cargo build --release --bin ultra-text-baseline
command time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/ultra-baseline-process.json" \
  target/release/ultra-text-baseline "$baseline_input" "$out/ultra-baseline.json" \
  2> >(tee "$out/ultra-baseline-stderr.txt" >&2)
jq -s '[range(0;(.[0].rows|length)) as $i | .[0].rows[$i] as $v | .[1].rows[$i] as $u |
  if $v.case!=$u.case or $v.state!=$u.state or $v.question!=$u.question then error("comparison input mismatch") else
  {id:$v.case.id,source:$v.case.source,arm:$v.case.arm,split:$v.case.split,want:$v.case.want,
   heard:$v.case.heard,state:$v.state,vela:$v.choice,ultra:$u.choice} end] |
  {count:length,changes:map(select(.vela!=.ultra))}' \
  "$baseline_input" "$out/ultra-baseline.json" > "$out/paired.json"
if [[ -s tools/pair-router-probe/ultra-int4-2026-11-01/q4/stripped.json ]]; then
  jq -s '[.[0].rows[] as $r | select($r.case.source!="synthetic") | .[1].rows[] |
    select(.case.id==$r.case.id and .case.arm==$r.case.arm) |
    {same:(.choice==$r.choice),same_state:(.state==$r.case.state),
     delta:([range(0;3) as $i | (.probabilities[$i]-$r.probabilities[$i])|fabs]|max)}] |
    {n:length,choices_match:(map(select(.same))|length),all_states_match:all(.same_state),max_delta:(map(.delta)|max)}' \
    tools/pair-router-probe/ultra-int4-2026-11-01/q4/stripped.json "$out/ultra-baseline.json" > "$out/baseline-phone-check.json"
fi
jq '{model:"Vela3-fast stripped",metrics:[.metrics[]|select(.split=="eval")]}' "$baseline_input"
jq '{model,metrics:[.metrics[]|select(.split=="eval")]}' "$out/ultra-baseline.json"
sha256sum tools/vela-probe/*.rs tools/vela-probe/run.sh tools/vela-probe/fixtures/public.json crates/decision/src/ultra.rs crates/decision/src/decision_tensor.rs \
  tools/pair-router-probe/cross-encoder-q8/cases.json \
  tools/pair-router-probe/bluey-q8-expanded-glossary/results.json Cargo.lock > "$out/provenance.sha256"
printf 'Vela evidence: %s\n' "$out"
