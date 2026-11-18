#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:-$(mktemp -d /tmp/transcrust-d1-context.XXXXXX)}
mkdir -p "$out"
out=$(realpath "$out")
fixture=tools/pair-router-probe/cross-encoder-q8/cases.json
cargo build --release --bin d1-context-probe
sha256sum "$fixture" crates/decision/src/d1.rs tools/d1-context-probe/main.rs Cargo.lock > "$out/inputs.sha256"
/run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/process.json" \
    target/release/d1-context-probe "$fixture" "$out/results.json" 2> >(tee "$out/stderr.txt" >&2)
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
for source in parakeet phonon; do
    for arm in single two; do
        jq -r --arg source "$source" --arg arm "$arm" '.household[]|select(.source==$source and .arm==$arm)|.delivered' \
            "$out/results.json" > "$out/$source-$arm.txt"
    done
done
"$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
    "$out/parakeet-"*.txt "$out/phonon-"*.txt > "$out/phone-scores.tsv"
jq '{metrics:[.metrics[]|select(.split=="eval")],household}' "$out/results.json"
printf 'D1 context evidence: %s\n' "$out"
