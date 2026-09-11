import { readFileSync } from "node:fs";
const body=(f)=>readFileSync(f,"utf8").replace(/^---[\s\S]*?\n---\n/,"").replace(/^#.*$/m,"");
const norm=(s)=>s.toLowerCase().replace(/[’']/g,"'").replace(/[^a-z0-9' ]+/g," ").split(/\s+/).filter(Boolean);
function wer(ref,hyp){const m=ref.length,n=hyp.length;let p=new Int32Array(n+1),c=new Int32Array(n+1);
 for(let j=0;j<=n;j++)p[j]=j;
 for(let i=1;i<=m;i++){c[0]=i;for(let j=1;j<=n;j++){const k=ref[i-1]===hyp[j-1]?0:1;
 c[j]=Math.min(p[j]+1,c[j-1]+1,p[j-1]+k);}[p,c]=[c,p];}return p[n]/m;}
const [aF,bF]=process.argv.slice(2);
const a=norm(body(aF)), b=norm(body(bF));
console.log(`${aF}: ${a.length} words`);
console.log(`${bF}: ${b.length} words`);
console.log(`word-level difference: ${(wer(a,b)*100).toFixed(2)}%`);
// show the differing spans
const K=5, map=new Map();
for(let i=0;i<b.length-K;i++){const k=b.slice(i,i+K).join(" ");if(!map.has(k))map.set(k,i);}
let ai=0,bi=0;const spans=[];let lastA=0,lastB=0;
while(ai<a.length-K){const k=a.slice(ai,ai+K).join(" "),h=map.get(k);
 if(h!==undefined&&h>=bi){const ra=a.slice(lastA,ai).join(" "),rb=b.slice(lastB,h).join(" ");
  if(ra!==rb) spans.push([ra,rb]); lastA=ai+K; lastB=h+K; ai+=K; bi=h+K;} else ai++;}
console.log(`\n${spans.length} differing spans:`);
for(const [x,y] of spans.slice(0,20)) console.log(`  old: ${x}\n  new: ${y}\n`);
