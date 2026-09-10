# WER tooling

Built 2026-09-09 to answer Phase A and B. Kept because the measurements in
`TASKBOARD.md` are not reproducible without it.

There is no ground truth for John's audio, so these compare against two
substitutes: yt-dlp's auto-captions for YouTube material, and one transcrust
run against another.

```sh
# WER of a --wav transcript against yt-dlp auto-captions
node tools/wer/wer.mjs captions.en.vtt transcript.md

# Two transcripts against each other, with the differing spans printed
node tools/wer/compare.mjs a.md b.md

# Anchored span diff, for characterising *which* words moved
node tools/wer/spans.mjs        # edit the paths at the bottom
```

**Reading the numbers.** Auto-captions are not truth — sampling the
disagreements on a tech talk showed most were yt-dlp's (`codeex`, `grock`,
`octa`, `kimmy`). Treat a WER against them as a ceiling on the real error rate,
and treat the *ratio* between two engines as the reliable signal.

Getting captions:

```sh
yt-dlp --write-auto-subs --sub-langs "en-orig,en" --sub-format vtt \
  --skip-download --no-simulate -o "src.%(ext)s" "$URL"
```

`--sub-lang en` silently returns nothing when the original track is `en-orig`,
and `--print` implies `--simulate`, so `--no-simulate` is required to get files.
Auto-VTT repeats each cue's tail; filter by physical line, not cue block:

```sh
rg '<c>' src.en.vtt | sed 's/<[^>]*>//g; s/^[[:space:]]*//' > captions.txt
```
