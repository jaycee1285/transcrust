import { readFileSync } from "node:fs";
const vttToText = (raw) => raw.split("\n").filter(l => l.includes("<c>")).map(l => l.replace(/<[^>]*>/g,"").trim()).filter(Boolean).join(" ");
const norm = (s) => s.toLowerCase().replace(/[’']/g,"'").replace(/[^a-z0-9' ]+/g," ").split(/\s+/).filter(Boolean);
const ref = norm(vttToText(readFileSync("aimodels.en.vtt","utf8")));
const hyp = norm(readFileSync("aimodels.md","utf8").replace(/^---[\s\S]*?\n---\n/,"").replace(/^#.*$/m,""));
// Anchor on runs of 6 matching words, report the mismatched spans between anchors.
const K = 6;
const key = (a,i)=>a.slice(i,i+K).join(" ");
const map = new Map();
for (let i=0;i<hyp.length-K;i++) if(!map.has(key(hyp,i))) map.set(key(hyp,i), i);
let ri=0, hi=0; const spans=[];
while (ri < ref.length-K) {
  const k = key(ref,ri), h = map.get(k);
  if (h !== undefined && h >= hi) {
    if (ri>0 && (ref.slice(hi===0?0:hi, h).join(" ") !== ref.slice(0,0).join(" "))) {
      const r = ref.slice(spans.at(-1)?.rEnd ?? 0, ri).join(" ");
      const y = hyp.slice(spans.at(-1)?.hEnd ?? 0, h).join(" ");
      if (r !== y) spans.push({r,y,rEnd:ri+K,hEnd:h+K}); else spans.push({r:null,rEnd:ri+K,hEnd:h+K});
    }
    ri += K; hi = h + K;
  } else ri++;
}
const real = spans.filter(s=>s.r!==null && s.r.length && s.y.length && s.r!==s.y);
console.log(`${real.length} disagreeing spans\n`);
for (const s of real.slice(0,25)) console.log(`yt : ${s.r}\ntr : ${s.y}\n`);
