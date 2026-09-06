# murmure — reference notes

`murmure` (github.com/Kieirra/murmure, **AGPL-3.0-or-later**, read at commit
`8c7a07c`) is a Tauri dictation app that drives the same Parakeet ONNX exports
transcrust does. It was cloned here as a conversation piece and removed on
2026-09-06; these are the notes that outlived it, so nothing in
`TASKBOARD-next.md` dangles.

## The licence question, before anyone copies anything

murmure is **AGPL-3.0-or-later**. transcrust has **no licence file at all**.

An earlier draft of the taskboard described porting `boost_tree.rs` as *"a copy,
not a port."* That was the wrong instinct and the wrong word. Lifting 295 lines
verbatim from an AGPL project would make transcrust a derivative work and pull
it under AGPL, including that licence's network clause. That may be perfectly
fine — it is John's call — but it should be a decision, not something that
happens because a file got pasted.

The clean path, and the one the board now specifies: **implement from the
design, not from the source.** The algorithm (a weighted Aho-Corasick automaton
with backoff, gating logit boosts by top-K rank) is a published technique —
murmure's own comments credit NeMo's GPU-PB, `context_graph_universal.py` and
`boosting_graph_batched.py`. The tuned constants below are measurements, and
they are recorded here so the implementation can be independent of the source
that suggested them.

Not legal advice — but a licence mismatch is worth noticing before building on
it rather than after.

## What it does that transcrust does not

- **Drives the Parakeet ONNX directly.** No `parakeet-rs`. Three sessions:
  `nemo128` (preprocessor), `encoder-model`, `decoder_joint-model`. This is the
  finding that mattered most — it proved Parakeet's per-token logits are
  reachable, which `parakeet-rs` hides. Already reimplemented independently in
  `src/parakeet_ort.rs` against the ONNX graph contract.
- **Per-token probability and word confidence**, minimum over sub-tokens
  ("weakest link"), punctuation-only tokens excluded from the minimum.
- **Decode-time phrase boosting** over the joint's non-blank logits, gated by
  top-K rank, with a divergence guard that falls back to the unboosted decode.
- **Confidence-gated fuzzy correction** — words the model was sure of are never
  fuzzy-matched against the dictionary.
- **Bigram segmentation repair** — joins split dictionary words
  (`app image` → `AppImage`).

## Calibrated constants

Measurements, recorded so an independent implementation starts from a known-good
point instead of guessing.

### Phrase boosting

| constant | value | meaning |
|---|---:|---|
| `CONTEXT_SCORE` | 1.0 | score for the first token of a phrase |
| `DEPTH_SCALING` | 2.0 | later tokens score `CONTEXT_SCORE × 2 + ln(i+1)` |
| `BOOST_ALPHA_MAX` / `MIN` | 3.5 / 1.0 | fusion weight bounds |
| `BOOST_TOP_K` | 5 | gate at phrase start — in greedy decoding only near-misses are recoverable |
| `BOOST_TOP_K_DEEP` | 20 | relaxed gate once a match is engaged |
| `BOOST_DEEP_DEPTH` | 3 | tokens in before the gate relaxes |
| `GUARD_MIN_TOKENS` | 24 | shorter utterances are not guarded |
| `GUARD_DIVERGENCE` | 0.35 | normalised token edit distance above which the unboosted decode wins |

Alpha decays with vocabulary size — more words means more first-tokens armed at
the root, so the volume comes down:

```
alpha(n) = (3.5 − log10(n / 5)).clamp(1.0, 3.5)
# n=5 → 3.5,  n=50 → 2.5,  n=500 → 1.5,  n≥5000 → 1.0
```

Backoff weight is negative and reimburses boost accumulated along a branch the
decode then abandons; a *completed* phrase keeps its boost (backoff 0).

### Post-correction

| constant | value | meaning |
|---|---:|---|
| `POSTCORR_CONF_THRESHOLD` | 0.45 | at or above this, never fuzzy-correct |
| `POSTCORR_MAX_DICT_WORDS` | 100 | above this, fuzzy off entirely; exact casing restore stays |
| `POSTCORR_MIN_LEN` | 5 | shorter words are not fuzzy candidates |
| `POSTCORR_LONG_LEN` | 8 | at or above, allow edit distance 2 instead of 1 |
| `POSTCORR_BIGRAM_LONG_LEN` | 12 | joined words this long may absorb 3 edits |

### TDT decode

| constant | value |
|---|---:|
| `MAX_TOKENS_PER_STEP` | 10 |
| `SUBSAMPLING_FACTOR` | 8 |

## Two false friends, if adapting anything

- **Tokenizer.** murmure targets Parakeet's SentencePiece/Metaspace vocabulary,
  where the boundary marker is prepended automatically and a word must *not* be
  space-prefixed. Granite is ByteLevel BPE, where the opposite holds and the
  segmentation differs outright: `syntocinon` → `sy n to cin on` (5 tokens) but
  ` syntocinon` → `Ġsyn to cin on` (4).
- **Token id 0.** murmure treats it as unknown. In Granite it is `<|blank|>`,
  the CTC blank. ByteLevel BPE has no unknown token at all.

## Getting it back

```sh
git clone https://github.com/Kieirra/murmure   # read at 8c7a07c
```
