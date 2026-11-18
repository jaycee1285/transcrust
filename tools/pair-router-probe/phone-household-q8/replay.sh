#!/usr/bin/env bash
# Run from the repository: nix develop -c bash tools/pair-router-probe/phone-household-q8/replay.sh
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
reports=${1:-$HOME/syncthing/mopvox/phonon-trial/results}
out=${2:-$(mktemp -d /tmp/transcrust-phone-q8.XXXXXX)}
mkdir -p "$out"
out=$(realpath "$out")
vocab=$HOME/syncthing/john-vocab.md
cp "$vocab" "$out/vocabulary.md"
cp "$HOME/syncthing/mopvox/phonon-trial/audio/phonon-household.reference.md" "$out/reference.md"
sha256sum "$vocab" > "$out/vocabulary.sha256"
git rev-parse HEAD > "$out/revision.txt"
model=${TRANSCRUST_DEC_MODEL:-$HOME/.local/share/transcrust/models/d1-omni-600M-q8}
cp "$model/manifest.json" "$out/model-manifest.json"
cargo build --quiet
for engine in parakeet phonon; do
    case $engine in
        parakeet) source=$reports/2026-10-28T21-39-08.206338Z-parakeet-ae192609.json ;;
        phonon) source=$reports/2026-10-28T21-39-32.837596Z-phonon-1899aa16.json ;;
    esac
    jq -e '.status == "complete" and (.passes | length == 2) and (.passes[0].raw == .passes[1].raw)' "$source" > /dev/null
    sha256sum "$source" >> "$out/phone-reports.sha256"
    jq '{engine,stamp,audio_file,audio_sha256,audio_seconds,passes:[.passes[]|{pass,raw,decode_ms,score}]}' "$source" > "$out/$engine-phone.json"
    raw=$(jq -r '.passes[0].raw' "$source")
    jq -r '.passes[0].raw' "$source" > "$out/$engine-raw.txt"
    /run/current-system/sw/bin/time -f '{"wall_secs":%e,"peak_rss_kib":%M}' -o "$out/$engine-process.json" \
        "$root/target/debug/transcrust" --fix "$raw" --dec > "$out/$engine-output.txt" 2> "$out/$engine-stderr.txt"
    rg -F "shared vocabulary $vocab" "$out/$engine-stderr.txt" > /dev/null
    tail -n 1 logs/decisions.jsonl > "$out/$engine-decision.json"
    jq -e --arg raw "$raw" '.source == "--fix" and .raw == $raw and .error == null and (.results|length) == (.candidates|length)' "$out/$engine-decision.json" > /dev/null
done
cmp "$vocab" "$out/vocabulary.md"
rustc tools/pair-router-probe/phone-household-q8/score.rs -o "$out/score"
"$out/score" "$out/reference.md" "$out/parakeet-raw.txt" "$out/parakeet-output.txt" \
    "$out/phonon-raw.txt" "$out/phonon-output.txt" > "$out/scores.tsv"
printf 'Replay evidence: %s\n' "$out"
