#!/usr/bin/env bun
// corpus-progress: minutes of transcrust corpus audio against a goal.
//
//   corpus-progress [goal-minutes] [--print]
//
// Minutes come from each WAV's own header, not its size: transcrust banks
// float32 at the device rate, so the same minute is twice the bytes of 16-bit.
// Default goal 120 minutes. Sends a notify-send unless --print is given; always
// prints the same line.
import { readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const CORPUS =
  process.env.TRANSCRUST_CORPUS ??
  join(process.env.XDG_DATA_HOME ?? join(homedir(), ".local/share"), "transcrust", "corpus");

const args = process.argv.slice(2);
const printOnly = args.includes("--print");
const goalArg = args.find((a) => /^\d+$/.test(a));
const goal = goalArg ? Number(goalArg) : 120;

// Walk RIFF chunks for fmt's byte rate and data's length. Reads only the first
// 64 KB; the data chunk header sits well inside that.
async function seconds(path: string): Promise<number> {
  const size = statSync(path).size;
  const head = new DataView(await Bun.file(path).slice(0, 65536).arrayBuffer());
  const tag = (at: number) => String.fromCharCode(head.getUint8(at), head.getUint8(at + 1), head.getUint8(at + 2), head.getUint8(at + 3));
  if (head.byteLength < 12 || tag(0) !== "RIFF" || tag(8) !== "WAVE") return 0;
  let byteRate = 0;
  for (let at = 12; at + 8 <= head.byteLength; ) {
    const id = tag(at);
    const length = head.getUint32(at + 4, true);
    if (id === "fmt " && at + 20 <= head.byteLength) byteRate = head.getUint32(at + 16, true);
    if (id === "data") {
      // An unfinalised writer leaves the length wrong; the file size is the truth.
      const bytes = Math.min(length, size - (at + 8));
      return byteRate > 0 ? bytes / byteRate : 0;
    }
    at += 8 + length + (length % 2);
  }
  return 0;
}

let total = 0;
try {
  for (const file of readdirSync(CORPUS)) {
    if (file.toLowerCase().endsWith(".wav")) total += await seconds(join(CORPUS, file));
  }
} catch {}

const line = `${Math.floor(total / 60)} completed towards ${goal} goal`;
console.log(line);
if (!printOnly) Bun.spawn(["notify-send", "transcrust corpus", `${line} (minutes)`]);
