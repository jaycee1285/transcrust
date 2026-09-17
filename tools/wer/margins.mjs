// Are the words two runs disagree on the words the model was least sure of?
//
// Input: two per-word score tables written by `nemotron.rs`'s ignored
// `margin_study` test (index, word, p1, margin). p1 is the least confident
// lettered token's probability; margin is p1 minus the runner-up's.
//
//   node tools/wer/margins.mjs a.tsv b.tsv [gate]
//
// Words are normalised exactly as compare.mjs does, aligned by edit distance,
// and every word on each side is marked agree (matched an equal word) or
// disagree (substituted, inserted or deleted).
import { readFileSync } from "node:fs";

const [aF, bF, gateArg] = process.argv.slice(2);
const gate = Number(gateArg ?? 0.75);
const norm = (s) => s.toLowerCase().replace(/[’']/g, "'").replace(/[^a-z0-9' ]+/g, " ").split(/\s+/).filter(Boolean);

const load = (f) =>
  readFileSync(f, "utf8").trim().split("\n").slice(1).flatMap((line) => {
    const [, word, p1, margin] = line.split("\t");
    return norm(word).map((w) => ({ w, p1: Number(p1), margin: Number(margin) }));
  });

const a = load(aF), b = load(bF);
const n = a.length, m = b.length;
const d = Array.from({ length: n + 1 }, (_, i) => { const r = new Int32Array(m + 1); r[0] = i; return r; });
for (let j = 0; j <= m; j++) d[0][j] = j;
for (let i = 1; i <= n; i++)
  for (let j = 1; j <= m; j++)
    d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1].w === b[j - 1].w ? 0 : 1));

const aOk = new Array(n).fill(false), bOk = new Array(m).fill(false);
for (let i = n, j = m; i > 0 || j > 0; ) {
  if (i > 0 && j > 0 && a[i - 1].w === b[j - 1].w && d[i][j] === d[i - 1][j - 1]) { aOk[i - 1] = bOk[j - 1] = true; i--; j--; }
  else if (i > 0 && j > 0 && d[i][j] === d[i - 1][j - 1] + 1) { i--; j--; }
  else if (i > 0 && d[i][j] === d[i - 1][j] + 1) i--;
  else j--;
}

const median = (xs) => {
  const s = [...xs].sort((x, y) => x - y);
  if (!s.length) return NaN;
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
};
// Chance a disagreeing word scores lower than an agreeing one (Mann-Whitney AUC).
// 0.5: scores say nothing about disagreement. 1.0: every disagreement is less
// certain than every agreement.
const auc = (lo, hi) => {
  if (!lo.length || !hi.length) return NaN;
  let wins = 0;
  for (const x of lo) for (const y of hi) wins += x < y ? 1 : x === y ? 0.5 : 0;
  return wins / (lo.length * hi.length);
};

const f = (x) => (Number.isNaN(x) ? "  n/a" : x.toFixed(3));
function report(label, file, words, ok) {
  const agree = words.filter((_, i) => ok[i]);
  const differ = words.filter((_, i) => !ok[i]);
  console.log(`${label} ${file}: ${words.length} words, ${differ.length} disagree`);
  for (const key of ["p1", "margin"]) {
    const ag = agree.map((w) => w[key]), di = differ.map((w) => w[key]);
    console.log(`  ${key.padEnd(6)} median agree ${f(median(ag))}  disagree ${f(median(di))}  AUC ${f(auc(di, ag))}`);
  }
  const flagged = words.filter((w) => w.p1 < gate).length;
  const caught = differ.filter((w) => w.p1 < gate).length;
  console.log(`  gate p1 < ${gate}: flags ${flagged}/${words.length} words, catches ${caught}/${differ.length} disagreements`);
  console.log(`  disagreeing: ${differ.map((w) => `${w.w}(${w.p1.toFixed(2)})`).join(" ")}\n`);
}

report("A", aF, a, aOk);
report("B", bF, b, bOk);
