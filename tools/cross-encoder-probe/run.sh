#!/usr/bin/env bash
# From the repository: nix develop -c bash tools/cross-encoder-probe/run.sh
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
out=${1:-$(mktemp -d /tmp/transcrust-cross-encoder.XXXXXX)}
mkdir -p "$out"
out=$(realpath "$out")
for model in deberta bge; do
    directory=$root/target/pair-probe/$model
    mkdir -p "$directory"
    if [[ $model == deberta ]]; then
        repository=cross-encoder/nli-deberta-v3-small
        revision=fa2804872c3b4bd748f38c0185cc85775361e735
        graph=onnx/model_quint8_avx2.onnx
    else
        repository=Xenova/bge-reranker-base
        revision=280bcc27a84e0b898c251e06fddb25171bd9b101
        graph=onnx/model_quantized.onnx
    fi
    for file in config.json tokenizer.json "$graph"; do
        destination=$directory/$file
        if [[ $file == onnx/* ]]; then destination=$directory/model.onnx; fi
        if [[ ! -s $destination ]]; then
            printf 'Downloading %s/%s\n' "$repository" "$file"
            curl -fL "https://huggingface.co/$repository/resolve/$revision/$file" -o "$destination.partial"
            mv "$destination.partial" "$destination"
        fi
    done
    cp "$directory/config.json" "$out/$model-config.json"
    sha256sum "$directory/model.onnx" "$directory/tokenizer.json" "$directory/config.json" > "$out/$model-artifacts.sha256"
done
cargo build --release --bin pair-probe
target/release/pair-probe prepare "$out/cases.json"
git rev-parse HEAD > "$out/git-revision.txt"
sha256sum tools/cross-encoder-probe/main.rs tools/cross-encoder-probe/run.sh Cargo.lock "$out/cases.json" \
    tools/pair-router-probe/bluey-expanded-suite.json \
    tools/pair-router-probe/bluey-q8-expanded-glossary/results.json \
    tools/pair-router-probe/phone-household-q8/evidence/parakeet-decision.json \
    tools/pair-router-probe/phone-household-q8/evidence/phonon-decision.json > "$out/inputs.sha256"
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
for model in deberta bge; do
    /run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$model-process.json" \
        target/release/pair-probe run "$model" "target/pair-probe/$model" "$out/cases.json" "$out/$model-results.json" \
        2> >(tee "$out/$model-stderr.txt" >&2)
    for engine in parakeet phonon; do
        for arm in single two; do
            for mode in forced policy; do
                jq -r --arg source "$engine" --arg arm "$arm" --arg mode "$mode" \
                    '.household[] | select(.source==$source and .arm==$arm and .mode==$mode) | .delivered' \
                    "$out/$model-results.json" > "$out/$model-$engine-$arm-$mode.txt"
            done
        done
    done
    "$out/score" tools/pair-router-probe/phone-household-q8/evidence/reference.md \
        "$out/$model-parakeet-"*.txt "$out/$model-phonon-"*.txt > "$out/$model-phone-scores.tsv"
    jq '{model,load_secs,policy,validation,held_out:[.metrics[]|select(.split=="eval")]}' "$out/$model-results.json"
done
printf 'Comparison evidence: %s\n' "$out"
