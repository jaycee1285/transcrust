//! Offline MNN-Pocket-0.3B-Dec native probe (keep / name / abstain). No app or backend changes.
//! Native transliteration of the pinned `inference.py` (revision dfbe0aed...): the
//! state/question serializer, a Qwen3 backbone with the LoRA adapter merged in F32, and
//! the FP32 pointer head. One question per request, so the reference's branch mask is
//! plain causal and its per-branch position reset equals the running index.
use candle_core::{safetensors::MmapedSafetensors, DType, Device, Result, Tensor, D};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use tokenizers::Tokenizer;

type Err = Box<dyn std::error::Error>;
type Row = (Vec<u32>, usize, Vec<usize>);
const NAMES: [&str; 3] = ["keep", "name", "abstain"];
// Order in inference.py SPECIAL_TOKENS: state, question, option, close, decide.
const SPECIAL: [&str; 5] = ["<|fim_prefix|>", "<|fim_middle|>", "<|box_start|>", "<|box_end|>", "<|fim_suffix|>"];
const LAYERS: usize = 28;
const HEADS: usize = 14;
const KV: usize = 2;
const HD: usize = 64;
const EPS: f64 = 1e-6;
const THETA: f64 = 1_000_000.0;

struct Layer {
    q: Tensor, k: Tensor, v: Tensor, o: Tensor,
    gate: Tensor, up: Tensor, down: Tensor,
    ln1: Tensor, ln2: Tensor, qn: Tensor, kn: Tensor,
}

struct Model {
    embed: Tensor, // BF16 [vocab, 896]; rows gathered then widened
    layers: Vec<Layer>,
    norm: Tensor,
    hq: Tensor, hqb: Tensor, hk: Tensor, hkb: Tensor,
    tok: Tokenizer,
    ids: [u32; 5],
    temperature: f64,
}

fn f32w(t: Tensor) -> Result<Tensor> { t.to_dtype(DType::F32) }

impl Model {
    fn load(dir: &Path) -> std::result::Result<Self, Err> {
        let dev = Device::Cpu;
        let cfg: Value = serde_json::from_slice(&fs::read(dir.join("decision_config.json"))?)?;
        let tok = Tokenizer::from_file(dir.join("base/tokenizer.json")).map_err(|e| e.to_string())?;
        let mut ids = [0u32; 5];
        for (i, name) in SPECIAL.iter().enumerate() {
            let want = cfg["special_tokens"][*name].as_u64().ok_or("missing special token")? as u32;
            let got = tok.token_to_id(name).ok_or("tokenizer lacks control token")?;
            if got != want { return Err(format!("control token {name}: tokenizer {got} != config {want}").into()); }
            ids[i] = got;
        }
        let temperature = serde_json::from_slice::<Value>(&fs::read(dir.join("temperature.json"))?)?["temperature"]
            .as_f64().ok_or("temperature")?;
        let base = unsafe { MmapedSafetensors::new(dir.join("base/model.safetensors"))? };
        let lora = unsafe { MmapedSafetensors::new(dir.join("adapter/adapter_model.safetensors"))? };
        let head = unsafe { MmapedSafetensors::new(dir.join("decision_head.safetensors"))? };
        let scale = 32.0 / 16.0; // lora_alpha / r from adapter_config.json
        // W' = W + scale * B A, computed in F32; stored pre-transposed [in, out].
        let merged = |name: &str| -> Result<Tensor> {
            let w = f32w(base.load(&format!("model.layers.{name}.weight"), &dev)?)?;
            let a = lora.load(&format!("base_model.model.model.layers.{name}.lora_A.weight"), &dev)?;
            let b = lora.load(&format!("base_model.model.model.layers.{name}.lora_B.weight"), &dev)?;
            let delta = (b.matmul(&a)? * scale)?;
            (w + delta)?.t()?.contiguous()
        };
        let vec = |name: &str| -> Result<Tensor> { f32w(base.load(name, &dev)?) };
        let mut layers = Vec::new();
        for i in 0..LAYERS {
            let p = |s: &str| format!("{i}.{s}");
            layers.push(Layer {
                q: merged(&p("self_attn.q_proj"))?, k: merged(&p("self_attn.k_proj"))?,
                v: merged(&p("self_attn.v_proj"))?, o: merged(&p("self_attn.o_proj"))?,
                gate: merged(&p("mlp.gate_proj"))?, up: merged(&p("mlp.up_proj"))?, down: merged(&p("mlp.down_proj"))?,
                ln1: vec(&format!("model.layers.{i}.input_layernorm.weight"))?,
                ln2: vec(&format!("model.layers.{i}.post_attention_layernorm.weight"))?,
                qn: vec(&format!("model.layers.{i}.self_attn.q_norm.weight"))?,
                kn: vec(&format!("model.layers.{i}.self_attn.k_norm.weight"))?,
            });
        }
        let hl = |n: &str| head.load(n, &dev);
        Ok(Model {
            embed: base.load("model.embed_tokens.weight", &dev)?,
            layers,
            norm: vec("model.norm.weight")?,
            hq: hl("q.weight")?.t()?.contiguous()?, hqb: hl("q.bias")?,
            hk: hl("k.weight")?.t()?.contiguous()?, hkb: hl("k.bias")?,
            tok, ids, temperature,
        })
    }

    fn tokens(&self, text: &str) -> Vec<u32> {
        self.tok.encode(escape(text), false).expect("tokenize").get_ids().to_vec()
    }

    /// state + one choice branch. Returns (ids, decide position, option-end positions).
    fn encode(&self, state: &str, instructions: &str, options: &[(String, String)]) -> Row {
        let [st, qu, op, cl, de] = self.ids;
        let mut ids = vec![st];
        ids.extend(self.tokens(state));
        ids.push(qu);
        ids.extend(self.tokens(instructions));
        let mut ends = Vec::new();
        for (key, desc) in options {
            ids.push(op);
            let text = if desc.is_empty() { key.clone() } else { format!("{key}: {desc}") };
            ids.extend(self.tokens(&text));
            ids.push(cl);
            ends.push(ids.len() - 1);
        }
        ids.push(de);
        let decide = ids.len() - 1;
        (ids, decide, ends)
    }

    fn rms(&self, x: &Tensor, w: &Tensor) -> Result<Tensor> {
        let var = x.sqr()?.mean_keepdim(D::Minus1)?;
        x.broadcast_div(&(var + EPS)?.sqrt()?)?.broadcast_mul(w)
    }

    fn rope(&self, x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
        // x [heads, T, 64]; half-split rotation (HF rotate_half).
        let x1 = x.narrow(2, 0, HD / 2)?;
        let x2 = x.narrow(2, HD / 2, HD / 2)?;
        let a = (x1.broadcast_mul(cos)? - x2.broadcast_mul(sin)?)?;
        let b = (x2.broadcast_mul(cos)? + x1.broadcast_mul(sin)?)?;
        Tensor::cat(&[a, b], 2)
    }

    /// Final-norm hidden states [T, 896].
    fn hidden(&self, ids: &[u32]) -> Result<Tensor> {
        let dev = Device::Cpu;
        let t = ids.len();
        let idx = Tensor::new(ids, &dev)?;
        let mut x = f32w(self.embed.index_select(&idx, 0)?)?;
        let inv: Vec<f64> = (0..HD / 2).map(|i| 1.0 / THETA.powf(2.0 * i as f64 / HD as f64)).collect();
        let mut cos = Vec::new();
        let mut sin = Vec::new();
        for p in 0..t {
            for f in &inv { let a = p as f64 * f; cos.push(a.cos() as f32); sin.push(a.sin() as f32); }
        }
        let cos = Tensor::from_vec(cos, (t, HD / 2), &dev)?;
        let sin = Tensor::from_vec(sin, (t, HD / 2), &dev)?;
        let mut m = vec![0f32; t * t];
        for i in 0..t { for j in i + 1..t { m[i * t + j] = -1e30; } }
        let mask = Tensor::from_vec(m, (t, t), &dev)?;
        for l in &self.layers {
            let h = self.rms(&x, &l.ln1)?;
            let heads = |w: &Tensor, n: usize| -> Result<Tensor> { h.matmul(w)?.reshape((t, n, HD)) };
            let q = self.rms(&heads(&l.q, HEADS)?, &l.qn)?.transpose(0, 1)?.contiguous()?;
            let k = self.rms(&heads(&l.k, KV)?, &l.kn)?.transpose(0, 1)?.contiguous()?;
            let v = heads(&l.v, KV)?.transpose(0, 1)?.contiguous()?;
            let q = self.rope(&q, &cos, &sin)?;
            let k = self.rope(&k, &cos, &sin)?;
            let rep = |z: Tensor| -> Result<Tensor> {
                z.unsqueeze(1)?.expand((KV, HEADS / KV, t, HD))?.reshape((HEADS, t, HD))
            };
            let (k, v) = (rep(k)?, rep(v)?);
            let s = (q.matmul(&k.transpose(1, 2)?.contiguous()?)? * (1.0 / (HD as f64).sqrt()))?.broadcast_add(&mask)?;
            let mx = s.max_keepdim(D::Minus1)?;
            let e = s.broadcast_sub(&mx)?.exp()?;
            let p = e.broadcast_div(&e.sum_keepdim(D::Minus1)?)?;
            let a = p.matmul(&v)?.transpose(0, 1)?.contiguous()?.reshape((t, HEADS * HD))?;
            x = (x + a.matmul(&l.o)?)?;
            let h = self.rms(&x, &l.ln2)?;
            let g = h.matmul(&l.gate)?;
            let silu = g.clone().broadcast_div(&((g.neg()?.exp()? + 1.0)?))?;
            x = (x + (silu * h.matmul(&l.up)?)?.matmul(&l.down)?)?;
        }
        self.rms(&x, &self.norm)
    }

    /// Raw pointer logits over the options (before temperature).
    fn logits(&self, row: &Row) -> Result<Vec<f32>> {
        let h = self.hidden(&row.0)?;
        let dec = h.narrow(0, row.1, 1)?;
        let q = dec.matmul(&self.hq)?.broadcast_add(&self.hqb)?; // [1,256]
        let pos: Vec<u32> = row.2.iter().map(|&e| e as u32).collect();
        let opt = h.index_select(&Tensor::new(pos.as_slice(), &Device::Cpu)?, 0)?;
        let k = opt.matmul(&self.hk)?.broadcast_add(&self.hkb)?; // [n,256]
        let s = (k.broadcast_mul(&q)?.sum(D::Minus1)? * (1.0 / 256f64.sqrt()))?;
        s.to_vec1::<f32>()
    }

    fn decide(&self, row: &Row) -> std::result::Result<(Vec<f32>, Vec<f64>), Err> {
        let logits = self.logits(row)?;
        if logits.iter().any(|v| !v.is_finite()) { return Err("non-finite logits".into()); }
        let mx = logits.iter().cloned().fold(f32::MIN, f32::max) as f64;
        let w: Vec<f64> = logits.iter().map(|&v| ((v as f64 - mx) / self.temperature).exp()).collect();
        let total: f64 = w.iter().sum();
        Ok((logits, w.iter().map(|v| v / total).collect()))
    }
}

/// `<|NAME|>` in user text becomes `<¦NAME¦>` exactly like inference.py's CONTROL_TOKEN.sub.
fn escape(text: &str) -> String {
    let b: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '<' && i + 1 < b.len() && b[i + 1] == '|' {
            let mut j = i + 2;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '_') { j += 1; }
            if j > i + 2 && j + 1 < b.len() && b[j] == '|' && b[j + 1] == '>' {
                out.push_str("<¦");
                out.extend(&b[i + 2..j]);
                out.push_str("¦>");
                i = j + 2;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Local window: up to `n` whitespace words either side of the «marked» span, never crossing a
/// sentence boundary (. ! ? or newline), so no neighbouring sentence leaks in.
fn window(state: &str, n: usize) -> Option<(String, String)> {
    let a = state.find('«')?;
    let b = a + state[a..].find('»')? + '»'.len_utf8();
    let marked = &state[a..b];
    let before = &state[..a];
    let cut = before.rfind(|c| matches!(c, '.' | '!' | '?' | '\n')).map(|i| i + 1).unwrap_or(0);
    let words: Vec<&str> = before[cut..].split_whitespace().collect();
    let lead = words[words.len().saturating_sub(n)..].join(" ");
    let after = &state[b..];
    let mut tail = String::new();
    let mut ended = false;
    for (i, w) in after.split_whitespace().enumerate() {
        if i >= n { break; }
        if !tail.is_empty() { tail.push(' '); }
        match w.find(|c| matches!(c, '.' | '!' | '?')) {
            Some(p) => { tail.push_str(&w[..p + 1]); ended = true; break; }
            None => tail.push_str(w),
        }
    }
    let _ = ended;
    let glued = after.chars().next().map_or(false, |c| !c.is_whitespace());
    let mut out = String::new();
    if !lead.is_empty() { out.push_str(&lead); out.push(' '); }
    out.push_str(marked);
    if !tail.is_empty() { if !glued { out.push(' '); } out.push_str(&tail); }
    Some((out, marked.trim_matches(|c| c == '«' || c == '»').to_string()))
}

fn best(p: &[f64]) -> usize {
    (1..p.len()).fold(0, |b, i| if p[i] > p[b] { i } else { b })
}

fn main() -> std::result::Result<(), Err> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 3 {
        return Err("usage: pocket-probe MODEL_DIR FROZEN_ROWS.json OUT.json  (rows from vela-probe expanded/full json)".into());
    }
    let start = Instant::now();
    let model = Model::load(Path::new(&a[0]))?;
    let load = start.elapsed().as_secs_f64();
    eprintln!("Pocket loaded in {load:.2}s");
    let input: Value = serde_json::from_slice(&fs::read(&a[1])?)?;
    let rows_in = input["rows"].as_array().ok_or("rows")?;
    let batches_in = input["batches"].as_array().ok_or("batches")?;
    let enc = |r: &Value| -> std::result::Result<Row, Err> {
        let q = &r["question"];
        let opts: Vec<(String, String)> = NAMES.iter()
            .map(|k| (k.to_string(), q["criteria"][k].as_str().unwrap_or("").to_string())).collect();
        let mut state = r["state"].as_str().ok_or("state")?.to_string();
        let mut instr = q["instructions"].as_str().ok_or("instr")?.to_string();
        if let Some(n) = std::env::var("POCKET_WINDOW").ok().and_then(|v| v.parse::<usize>().ok()) {
            let (w, span) = window(&state, n).ok_or("no marked span")?;
            state = w;
            if std::env::var("POCKET_FRAME").is_ok() {
                instr = format!("Is \"{span}\" used here as a name or as the ordinary word?");
            }
        } else if std::env::var("POCKET_FRAME").is_ok() {
            let (_, span) = window(&state, 0).ok_or("no marked span")?;
            instr = format!("Is \"{span}\" used here as a name or as the ordinary word?");
        }
        Ok(model.encode(&state, &instr, &opts))
    };
    // Warm-up on the first row, untimed (allocator and gemm thread pool).
    model.decide(&enc(&rows_in[0])?)?;
    let mut results = Vec::new();
    let mut batches = Vec::new();
    let mut at = 0;
    for b in batches_in {
        let n = b["candidates"].as_u64().unwrap() as usize;
        let group = &rows_in[at..at + n];
        at += n;
        let encoded: Vec<Row> = group.iter().map(&enc).collect::<std::result::Result<_, _>>()?;
        let now = Instant::now();
        let mut outs = Vec::new();
        for e in &encoded { outs.push(model.decide(e)?); }
        let seconds = now.elapsed().as_secs_f64();
        eprintln!("Pocket {} {}: {n} candidates, {seconds:.3}s", b["source"], b["arm"]);
        batches.push(json!({"source":b["source"],"arm":b["arm"],"candidates":n,"seconds":seconds,
            "token_counts":encoded.iter().map(|e|e.0.len()).collect::<Vec<_>>()}));
        for ((r, e), (logits, p)) in group.iter().zip(&encoded).zip(outs) {
            let choice = NAMES[best(&p)];
            results.push(json!({"case":r["case"],"state":r["state"],"question":r["question"],"choice":choice,"decoded":model.tok.decode(&e.0[..e.1.min(e.0.len())], false).unwrap_or_default(),
                "probabilities":p,"logits":logits,"token_ids":e.0,"decide":e.1,"option_ends":e.2}));
        }
    }
    assert_eq!(at, rows_in.len(), "batch grouping must cover all rows");
    // Same edit application as vela-probe: marked spans replaced from the end of the transcript.
    let out = &a[2];
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            let path = format!("tools/pair-router-probe/phone-household-q8/evidence/{source}-decision.json");
            let evidence: Value = serde_json::from_slice(&fs::read(&path)?)?;
            let mut text = evidence["input"].as_str().ok_or("evidence input")?.to_string();
            let mut edits: Vec<&Value> = results.iter()
                .filter(|r| r["case"]["source"] == source && r["case"]["arm"] == arm && r["choice"] == "name").collect();
            edits.sort_by_key(|r| std::cmp::Reverse(r["case"]["candidate"]["start"].as_u64().unwrap()));
            for r in edits {
                let c = &r["case"]["candidate"];
                let (s, e) = (c["start"].as_u64().unwrap() as usize, c["end"].as_u64().unwrap() as usize);
                if text.get(s..e) != c["heard"].as_str() { return Err("occurrence span mismatch".into()); }
                text.replace_range(s..e, c["canonical"].as_str().unwrap());
            }
            fs::write(format!("{out}.{source}-{arm}.txt"), text)?;
        }
    }
    let mut metrics = Vec::new();
    for split in ["phone", "dev", "eval", "diagnostic"] {
        for arm in ["single", "two"] {
            let rows: Vec<&Value> = results.iter().filter(|r| r["case"]["split"] == split && r["case"]["arm"] == arm).collect();
            if rows.is_empty() { continue; }
            let cnt = |f: &dyn Fn(&&Value) -> bool| rows.iter().filter(|r| f(r)).count();
            metrics.push(json!({"split":split,"arm":arm,"n":rows.len(),
                "repair_total":cnt(&|r| r["case"]["want"]=="name"),
                "repairs":cnt(&|r| r["case"]["want"]=="name"&&r["choice"]=="name"),
                "preserve_total":cnt(&|r| r["case"]["want"]!="name"),
                "unwanted_edits":cnt(&|r| r["case"]["want"]!="name"&&r["choice"]=="name"),
                "abstentions":cnt(&|r| r["choice"]=="abstain")}));
        }
    }
    fs::write(out, serde_json::to_vec_pretty(&json!({"model":"MNN-Pocket-0.3B-Dec","revision":"dfbe0aed3abd4caed175b883a4b462ed35184fad",
        "temperature":model.temperature,"input":a[1],"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),
        "batches":batches,"rows":results,"metrics":metrics}))?)?;
    Ok(())
}
