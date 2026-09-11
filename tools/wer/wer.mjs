import { readFileSync } from "node:fs";

// YouTube auto-VTT is a rolling two-line window: each cue repeats the previous
// cue's tail. Keeping only lines with inline <c> word timings, then stripping
// them, recovers the spoken stream once.
function vttToText(raw) {
  const out = [];
  for (const line of raw.split("\n")) {
    if (!line.includes("<c>")) continue;
    const text = line.replace(/<[^>]*>/g, "").trim();
    if (text) out.push(text);
  }
  return out.join(" ");
}

const norm = (s) =>
  s
    .toLowerCase()
    .replace(/[’']/g, "'")
    .replace(/[^a-z0-9' ]+/g, " ")
    .split(/\s+/)
    .filter(Boolean);

function wer(ref, hyp) {
  const m = ref.length, n = hyp.length;
  let prev = new Int32Array(n + 1);
  let cur = new Int32Array(n + 1);
  for (let j = 0; j <= n; j++) prev[j] = j;
  for (let i = 1; i <= m; i++) {
    cur[0] = i;
    for (let j = 1; j <= n; j++) {
      const cost = ref[i - 1] === hyp[j - 1] ? 0 : 1;
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + cost);
    }
    [prev, cur] = [cur, prev];
  }
  return prev[n] / m;
}

const [vttPath, mdPath] = process.argv.slice(2);
const refText = vttToText(readFileSync(vttPath, "utf8"));
const md = readFileSync(mdPath, "utf8").replace(/^---[\s\S]*?\n---\n/, "").replace(/^#.*$/m, "");
const ref = norm(refText);
const hyp = norm(md);
console.log(`reference (yt-dlp auto): ${ref.length} words`);
console.log(`hypothesis (transcrust): ${hyp.length} words`);
console.log(`WER vs auto-captions:    ${(wer(ref, hyp) * 100).toFixed(2)}%`);
console.log(`\nref  head: ${ref.slice(0, 40).join(" ")}`);
console.log(`hyp  head: ${hyp.slice(0, 40).join(" ")}`);
