#!/usr/bin/env bun
// corpus-match: label a transcrust corpus clip with the text that was actually sent.
//
// Reads text on stdin. If it is a near-copy of a recent dictation whose JSON
// `reference` is still null, it writes that text into `reference`. Anything
// else is read, compared and forgotten: nothing about a non-match is written.
//
//   corpus-match                 agent user-prompt-submit hooks; exact matches
//                                count, because an unedited prompt was reviewed
//   corpus-match --source clip   clipwatch; exact matches are ignored, because
//                                transcrust copies every transcript itself
//
// stdin may be plain text, or a JSON object with a string `prompt` field (the
// shape Claude Code hands a UserPromptSubmit hook), so a hook can call this
// directly. Always exits 0 and prints nothing: a hook must never block a prompt.
//
// Store: `reference` in the clip's own `.json`, the field transcrust's corpus
// contract already defines. `matches.log` beside the clips is an audit trail of
// what this wrote (time, stem, source, difference), never a second store.
import { appendFileSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

// Mirrors transcrust's corpus::dir(). The env override exists for testing.
const CORPUS =
  process.env.TRANSCRUST_CORPUS ??
  join(process.env.XDG_DATA_HOME ?? join(homedir(), ".local/share"), "transcrust", "corpus");

const RECENT = 10;            // newest clips considered
const WINDOW_MINUTES = 30;    // a clip older than this can't be what you just sent
const LENGTH_SLACK = 0.3;     // word counts must agree within ±30% (min ±2 words)
const MAX_DIFFERENCE = 0.35;  // word edit distance / dictation length
const MAX_INPUT = 20_000;     // characters; a longer paste is never a dictation

const source = (() => {
  const i = process.argv.indexOf("--source");
  return i >= 0 ? process.argv[i + 1] ?? "hook" : "hook";
})();

// Same normalisation as tools/wer/compare.mjs.
const norm = (s: string) =>
  s.toLowerCase().replace(/[’']/g, "'").replace(/[^a-z0-9' ]+/g, " ").split(/\s+/).filter(Boolean);

function editDistance(a: string[], b: string[]): number {
  let prev = Array.from({ length: b.length + 1 }, (_, j) => j);
  for (let i = 1; i <= a.length; i++) {
    const cur = [i];
    for (let j = 1; j <= b.length; j++)
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
    prev = cur;
  }
  return prev[b.length];
}

// Stems are UTC stamps like 2026-09-06T17-19-20Z.
function stampToMs(stamp: string): number {
  const m = /^(\d{4}-\d{2}-\d{2})T(\d{2})-(\d{2})-(\d{2})Z$/.exec(stamp);
  return m ? Date.parse(`${m[1]}T${m[2]}:${m[3]}:${m[4]}Z`) : NaN;
}

function unwrap(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) return trimmed;
  try {
    const value = JSON.parse(trimmed);
    if (value && typeof value.prompt === "string") return value.prompt.trim();
  } catch {}
  return trimmed;
}

// Two hooks and the watcher can fire together; each clip must be claimed once.
function withLock(work: () => void) {
  const lock = join(CORPUS, ".corpus-match.lock");
  for (let attempt = 0; attempt < 40; attempt++) {
    try {
      mkdirSync(lock);
      try { work(); } finally { rmSync(lock, { recursive: true, force: true }); }
      return;
    } catch (error: any) {
      if (error?.code !== "EEXIST") return;
      try {
        if (Date.now() - statSync(lock).mtimeMs > 10_000) rmSync(lock, { recursive: true, force: true });
      } catch {}
      Bun.sleepSync(50);
    }
  }
}

async function main() {
  const raw = await Bun.stdin.text();
  if (!raw || raw.length > MAX_INPUT || !existsSync(CORPUS)) return;
  const text = unwrap(raw);
  const words = norm(text);
  if (!words.length) return;

  const now = Date.now();
  const files = readdirSync(CORPUS).filter((f) => f.endsWith(".json")).sort().slice(-RECENT);
  let best: { path: string; stem: string; difference: number; at: number } | null = null;

  for (const file of files) {
    const path = join(CORPUS, file);
    let entry: any;
    try { entry = JSON.parse(readFileSync(path, "utf8")); } catch { continue; }
    if (entry?.reference != null) continue; // already labelled, so already claimed

    const stem = file.slice(0, -".json".length);
    const at = stampToMs(typeof entry.recorded === "string" ? entry.recorded : stem);
    if (!Number.isFinite(at) || now - at > WINDOW_MINUTES * 60_000 || at - now > 60_000) continue;

    const target = norm(typeof entry.injected === "string" ? entry.injected : "");
    if (!target.length) continue;
    if (Math.abs(target.length - words.length) > Math.max(2, Math.ceil(target.length * LENGTH_SLACK))) continue;

    const distance = editDistance(target, words);
    const difference = distance / target.length;
    if (difference > MAX_DIFFERENCE) continue;
    if (distance === 0 && source === "clip") continue;

    if (!best || difference < best.difference || (difference === best.difference && at > best.at))
      best = { path, stem, difference, at };
  }
  if (!best) return;

  const chosen = best;
  withLock(() => {
    const entry = JSON.parse(readFileSync(chosen.path, "utf8"));
    if (entry.reference != null) return; // claimed while we were comparing
    entry.reference = text;
    const tmp = `${chosen.path}.tmp`;
    writeFileSync(tmp, JSON.stringify(entry, null, 2) + "\n");
    renameSync(tmp, chosen.path);
    appendFileSync(
      join(CORPUS, "matches.log"),
      `${new Date().toISOString()}\t${chosen.stem}\t${source}\t${chosen.difference.toFixed(3)}\n`,
    );
  });
}

try { await main(); } catch {}
process.exit(0);
