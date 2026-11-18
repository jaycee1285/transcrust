#!/usr/bin/env bash
# nix develop -c bash tools/thx-probe/run.sh
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
model=${THX_MODEL_DIR:-$root/target/thx-01}
out=${1:-$(mktemp -d /tmp/transcrust-thx-probe.XXXXXX)}
mkdir -p "$model/tokenizer" "$out"
out=$(realpath "$out")
base=https://huggingface.co/doofz/THX-01/resolve/b323afcef149f9288a9d0daef4badd67b9b87c61
for file in model.safetensors encoder/config.json rl_agent_config.json tokenizer/tokenizer.json tokenizer/tokenizer_config.json LICENSE NOTICE; do
    destination=$model/$file
    if [[ $file == encoder/config.json ]]; then destination=$model/encoder-config.json; fi
    if [[ ! -s $destination ]]; then
        printf 'Downloading %s\n' "$file"
        curl -fL "$base/$file" -o "$destination.partial"
        mv "$destination.partial" "$destination"
    fi
done
cargo build --release --features thx-probe --bin thx-probe
export RAYON_NUM_THREADS=${RAYON_NUM_THREADS:-4}
/run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/process.json" \
    target/release/thx-probe "$model" "$out/results.json" \
    tools/pair-router-probe/phone-household-q8/evidence/parakeet-decision.json \
    tools/pair-router-probe/phone-household-q8/evidence/phonon-decision.json \
    tools/pair-router-probe/bluey-q8-expanded-glossary/results.json \
    2> >(tee "$out/stderr.txt" >&2)
jq -r '.sources[] | select(.delivered != null) | .source, .delivered' "$out/results.json"
printf 'THX evidence: %s\n' "$out"
