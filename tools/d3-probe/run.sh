#!/usr/bin/env bash
# Native reference validation, then correction testing. First run downloads ~803 MB.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:-$(mktemp -d /tmp/transcrust-d3.XXXXXX)}
scope=${2:-household}
model=${D3_MODEL_DIR:-$PWD/target/d3-lite}
fixtures=$model/fixtures
mkdir -p "$out" "$fixtures"
out=$(realpath "$out")
base=https://huggingface.co/Jacqkues/d3-lite-decisionjs/resolve/d4d54cab63bf0b1493cb23d0fd838fd253766251
for file in weights.bin manifest.json tokenizer.json reference.json LICENSE NOTICE; do
    if [[ ! -s $model/$file ]]; then
        curl -fL "$base/$file" -o "$model/$file.partial"
        mv "$model/$file.partial" "$model/$file"
    fi
done
reference=https://raw.githubusercontent.com/Jacqkues/decision.js/4782abfd297fe067d965ab7f587a449b7b0cad0a/test/fixtures
if [[ ! -s $fixtures/test-fixtures-d3-prompts.json ]]; then
    curl -fL "$reference/d3-prompts.json" -o "$fixtures/test-fixtures-d3-prompts.json.partial"
    mv "$fixtures/test-fixtures-d3-prompts.json.partial" "$fixtures/test-fixtures-d3-prompts.json"
fi
if [[ ! -s $fixtures/requests.json ]]; then
    curl -fL "$reference/prompts.json" -o "$fixtures/requests.json.partial"
    mv "$fixtures/requests.json.partial" "$fixtures/requests.json"
fi
cargo build --release --features d3-probe --bin d3-probe
export RAYON_NUM_THREADS=4
sha256sum "$model/weights.bin" "$model/manifest.json" "$model/tokenizer.json" "$model/reference.json" \
    "$fixtures/test-fixtures-d3-prompts.json" "$fixtures/requests.json" \
    tools/d3-probe/main.rs tools/d3-probe/model.rs Cargo.lock > "$out/provenance.sha256"
/run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/parity-process.json" \
    target/release/d3-probe parity "$model" "$fixtures" "$out/parity.json" \
    2> >(tee "$out/parity-stderr.txt" >&2)
jq -e '.gate_pass == true' "$out/parity.json" > /dev/null
/run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/correction-process.json" \
    target/release/d3-probe evaluate "$model" tools/pair-router-probe/cross-encoder-q8/cases.json \
    "$out/corrections.json" "$out/parity.json" "$scope" \
    2> >(tee "$out/correction-stderr.txt" >&2)
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
for source in parakeet phonon; do
    for arm in single two; do
        jq -r --arg source "$source" --arg arm "$arm" '.household[]|select(.source==$source and .arm==$arm)|.delivered' \
            "$out/corrections.json" > "$out/$source-$arm.txt"
    done
done
"$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
    "$out/parakeet-"*.txt "$out/phonon-"*.txt > "$out/phone-scores.tsv"
jq '{scope,household,metrics}' "$out/corrections.json"
printf 'Native d3 evidence: %s\n' "$out"
