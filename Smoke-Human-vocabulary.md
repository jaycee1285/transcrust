# Human Smoke — Vocabulary Learning

Acceptance criteria for two unbuilt features, written before the code so the
grammar has to satisfy them rather than the other way round.

Both write one artifact, `~/.config/transcrust/aliases.json`, applied **before**
`dictionary::correct` — aliases exist for the pairs Beider-Morse cannot reach.

Every case is: run a command, speak, run a command, paste the output back.

---

## 0. Setup

Terminal A — daemon:

```sh
cd ~/repos/transcrust && nix develop -c cargo run --offline -- --smoke
```

Terminal B — evidence. Each case below gives you one command to run here; this
is the shell you paste from.

Baseline, run once before starting:

```sh
nix develop -c cargo run --offline -- --doctor | rg "Aliases|Phonetic dictionary"
```

Expected: an `Aliases:` line naming the file and a count, next to the existing
`Phonetic dictionary:` line. If aliases are invisible to `--doctor`, neither
feature is debuggable and everything below is guesswork.

Reset between runs if you want a clean slate:

```sh
cp ~/.config/transcrust/aliases.json{,.bak} 2>/dev/null; echo '{}' > ~/.config/transcrust/aliases.json
```

---

## 1. Spoken command dispatch

### 1.1 A command is consumed, not injected

Put the cursor in a text field you can watch. Say:

> repair concept transfusion

```sh
rg "transcription.raw|command\.|inject" logs/latest.log | tail -5; \
cat ~/.config/transcrust/aliases.json
```

Expected: a `command.repair` line, **no `inject` line**, the text field still
empty, a desktop notification naming both halves, and `concept -> transfusion`
in the JSON.

### 1.2 The alias applies to the next utterance

Without restarting the daemon, say:

> the concept is ready

```sh
rg "transcription.raw|transcription.postprocess" logs/latest.log | tail -2
```

Expected: raw contains `concept`, postprocess contains `transfusion`.

This is the case that catches a `LazyLock` alias store. If it only works after
a restart, the store loads once per process and needs `RwLock` or
reload-on-mtime.

### 1.3 Prose containing the trigger is NOT eaten

Say:

> we need to repair the concept before friday

```sh
rg "command\.|inject" logs/latest.log | tail -3
```

Expected: an `inject` line, **no `command.` line**, full sentence injected.

If this is consumed as a command, stop — the feature silently deletes sentences
and nothing else matters until the trigger is anchored and length-bounded.

### 1.4 A mangled trigger still fires

Say `repair` sloppily, then:

> repair widget gadget

```sh
rg "transcription.raw|command\." logs/latest.log | tail -2
```

Expected: raw shows the mangling (`prepare`, `we pair`, `repaired`) **and** a
`command.repair` line anyway. Paste the raw line either way — that's the record
of what the trigger matcher had to survive.

If it injected prose instead, the trigger is being string-compared. The
Beider-Morse code in `dictionary.rs` already solves this against a two-entry
corpus.

### 1.5 `mark` writes to the dictionary, not the alias map

> mark svelte

```sh
tail -3 ~/.config/transcrust/dictionary.txt; cat ~/.config/transcrust/aliases.json
```

Expected: `svelte` in `dictionary.txt`, alias map unchanged. A term that only
needs its phonetic neighbourhood covered should not become a hard alias.

### 1.6 Undo

Immediately after any command:

> undo that

```sh
cat ~/.config/transcrust/aliases.json
```

Expected: the previous rule gone, notification confirms.

Build this first. The first time 1.3 fails for real you will want it, and that
will not be a good moment to add it.

---

## 2. Shell oracle

Prerequisite: hook installed in `~/repos/config/home/bash/local.bash`, recording
command text plus exit status.

### 2.1 A failed command produces a candidate

In a fresh shell:

```sh
transnistrian --doctor; cd repose
```

```sh
cat ~/.config/transcrust/alias-candidates.jsonl
```

Expected: `transnistrian`/`transcrust` and `repose`/`repos`, each `count: 1`.
Both ranked #1 in the scratch harness, so ranking is not the risk — confirm the
token is the bare word and not the whole `command not found:` line.

### 2.2 One sighting does not promote

```sh
cat ~/.config/transcrust/aliases.json
```

Expected: unchanged. One observation is a typo until proven otherwise.

### 2.3 Repetition promotes

```sh
transnistrian --doctor; transnistrian --doctor; \
cat ~/.config/transcrust/aliases.json; \
nix develop -c cargo run --offline -- --doctor | rg Aliases
```

Expected: after the third sighting the pair moves from candidates into
`aliases.json` and the `--doctor` count goes up.

### 2.4 A genuine typo stays quarantined

```sh
gti status; cat ~/.config/transcrust/alias-candidates.jsonl | tail -2
```

Expected: `gti -> git` as a candidate but not promoted. It's a real mapping, so
candidacy is right — you just shouldn't have to run it three more times to
reject it.

### 2.5 Vocabulary boundary

```sh
kubernetis get pods; cat ~/.config/transcrust/alias-candidates.jsonl | tail -2
```

Expected: **no useful candidate**, and that is correct. `Kubernetes` is not a
command or a directory here; it lives only in the authored dictionary. The
shell oracle and the style file cover disjoint vocabulary.

A confident wrong match here means the accept threshold is too loose.

---

## 3. What to paste back

One line per case: case number, injected / consumed / wrong, plus the command
output above.

Then:

```sh
nix develop -c cargo run --offline -- --doctor | rg "Aliases|Phonetic"; \
wc -l ~/.config/transcrust/alias-candidates.jsonl
```

And in prose:

- did any prose get eaten in 1.3, and what was the utterance
- the raw transcript from 1.4
- after a normal day, how many candidates accumulated and how many were junk
- whether a command ever fired when you did not intend one

That last one is the only failure that loses text. It outweighs every miss.
