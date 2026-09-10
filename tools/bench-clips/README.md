# Benchmark clips

Fixed corpus for `--bench`, so timing runs are comparable across sessions.

`chat-*.wav` are prefixes of the same 69 s interview at 1/2/3/4/5/10/20/40/60
seconds, resampled to 48 kHz PCM16.

**Caveat, found 2026-09-06:** this machine's capture device is **44100 Hz, 2ch**
(`--list-devices`), not 48 kHz. So these fixtures do *not* exercise the same
resample ratio a real dictation pays — 48000→16000 is a clean 3:1, while
44100→16000 is 2.75625:1 through `audio.rs`'s linear interpolation with no
anti-aliasing filter. That is the exact ratio that turned `recognition` into
`dilation` on the demo clip. Treat these as engine-vs-engine timing fixtures
only; anything about accuracy needs audio captured at the real device rate.
`demo3-hotwords.wav` is Mandarin/English code-switched, kept as the one clip
Parakeet and Granite both fail on for a reason other than noise.

```sh
nix develop -c cargo build --release
./target/release/transcrust --bench tools/bench-clips/chat-*.wav
```

The 1–4 s clips are the ones that matter: that is where the Parakeet/Granite
crossover sits (~3 s) and where command utterances live.

**These are timing fixtures, not a WER corpus.** There is no ground truth here.
`TASKBOARD.md` Phase C is the hand-corrected set, and it needs
John's own voice, not an interview clip.

Source: `demo/asr_demo/` of github.com/microsoft/VibeVoice (MIT).
