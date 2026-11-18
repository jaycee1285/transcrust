use candle_core::{
    quantized::{GgmlDType, QStorage, QTensor},
    Device, Tensor, D,
};
use half::f16;
use serde_json::Value;
use std::{
    borrow::Cow, collections::HashMap, fs::File, io::Read, ops::Range, os::unix::fs::FileExt,
    path::Path,
};
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub const ULTRA_SOURCE_HASH: &str =
    "67ac518c172934138f5d0bb06aa34523c9d9dfb6261f5dac3c860ab3f3542e91";
pub fn file_hash(path: &Path) -> Result<String, Error> {
    use sha2::{Digest, Sha256};
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub struct Weights {
    pub dense: HashMap<String, Tensor>,
    pub quant: HashMap<String, QTensor>,
    pub file: File,
    pub manifest: Value,
}
fn bytes(f: &File, start: u64, n: usize) -> Result<Vec<u8>, Error> {
    let mut v = vec![0; n];
    f.read_exact_at(&mut v, start)?;
    Ok(v)
}
fn tensor(data: &[u8], dtype: &str, shape: Vec<usize>) -> Result<Tensor, Error> {
    let v: Vec<f32> = match dtype {
        "f32" | "F32" => data
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect(),
        "f16" | "F16" => data
            .chunks_exact(2)
            .map(|b| f16::from_le_bytes(b.try_into().unwrap()).to_f32())
            .collect(),
        "BF16" => data
            .chunks_exact(2)
            .map(|b| half::bf16::from_le_bytes(b.try_into().unwrap()).to_f32())
            .collect(),
        _ => return Err(format!("unsupported dtype {dtype}").into()),
    };
    Ok(Tensor::from_vec(v, shape, &Device::Cpu)?)
}
impl Weights {
    pub fn ultra_q4(dir: &Path) -> Result<Self, Error> {
        let manifest: Value = serde_json::from_slice(&std::fs::read(dir.join("ultra-q4.json"))?)?;
        if manifest["format"] != "ultra-q4-v1" || manifest["source_sha256"] != ULTRA_SOURCE_HASH {
            return Err("invalid UltraBERT Q4 manifest".into());
        }
        if file_hash(&dir.join("weights.q4"))?
            != manifest["sha256"]
                .as_str()
                .ok_or("missing packed checksum")?
        {
            return Err("Q4 weight checksum mismatch".into());
        }
        let file = File::open(dir.join("weights.q4"))?;
        if file.metadata()?.len() != manifest["bytes"].as_u64().ok_or("missing packed size")? {
            return Err("packed size mismatch".into());
        }
        let mut dense = HashMap::new();
        let mut quant = HashMap::new();
        for (name, info) in manifest["tensors"].as_object().ok_or("missing tensors")? {
            if name == "model.encoder_model.embed_tokens.weight" {
                continue;
            }
            let raw = bytes(
                &file,
                info["offset"].as_u64().unwrap(),
                info["bytes"].as_u64().unwrap() as usize,
            )?;
            let shape: Vec<usize> = serde_json::from_value(info["shape"].clone())?;
            match info["dtype"].as_str() {
                Some("q4_0") => {
                    let storage =
                        QStorage::from_data(Cow::Borrowed(&raw), &Device::Cpu, GgmlDType::Q4_0)?;
                    quant.insert(name.clone(), QTensor::new(storage, shape)?);
                }
                Some("f32") => {
                    dense.insert(name.clone(), tensor(&raw, "f32", shape)?);
                }
                _ => return Err("unknown Q4 artifact dtype".into()),
            }
        }
        Ok(Self {
            dense,
            quant,
            file,
            manifest,
        })
    }
    pub fn ultra_embed(&self, ids: &[u32]) -> Result<Tensor, Error> {
        let key = "model.encoder_model.embed_tokens.weight";
        if let Some(t) = self.dense.get(key) {
            return Ok(t.index_select(&Tensor::new(ids, &Device::Cpu)?, 0)?);
        }
        let info = &self.manifest["tensors"][key];
        let width = info["shape"][1].as_u64().ok_or("missing embedding width")? as usize;
        let rows = info["shape"][0].as_u64().unwrap();
        let row_bytes = width / 32 * 18;
        let offset = info["offset"].as_u64().unwrap();
        let mut cache = HashMap::new();
        let mut values = Vec::with_capacity(ids.len() * width);
        for id in ids {
            if *id as u64 >= rows {
                return Err("embedding token outside table".into());
            }
            if !cache.contains_key(id) {
                let raw = bytes(
                    &self.file,
                    offset + *id as u64 * row_bytes as u64,
                    row_bytes,
                )?;
                let storage =
                    QStorage::from_data(Cow::Borrowed(&raw), &Device::Cpu, GgmlDType::Q4_0)?;
                let row = QTensor::new(storage, (1, width))?
                    .dequantize(&Device::Cpu)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                cache.insert(*id, row);
            }
            values.extend_from_slice(&cache[id]);
        }
        Ok(Tensor::from_vec(values, (ids.len(), width), &Device::Cpu)?)
    }
    pub fn packed(dir: &Path) -> Result<Self, Error> {
        let manifest: Value = serde_json::from_slice(&std::fs::read(dir.join("manifest.json"))?)?;
        let file = File::open(dir.join("weights.bin"))?;
        assert_eq!(file.metadata()?.len(), manifest["bytes"].as_u64().unwrap());
        let mut dense = HashMap::new();
        let mut quant = HashMap::new();
        for section in ["tensors", "host"] {
            for (name, info) in manifest[section].as_object().unwrap() {
                if name == "embed" {
                    continue;
                }
                let data = bytes(
                    &file,
                    info["offset"].as_u64().unwrap(),
                    info["bytes"].as_u64().unwrap() as usize,
                )?;
                let shape: Vec<_> = info["shape"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_u64().unwrap() as usize)
                    .collect();
                if info["dtype"] == "q8" {
                    let scales = bytes(
                        &file,
                        info["scales"].as_u64().unwrap(),
                        info["scales_bytes"].as_u64().unwrap() as usize,
                    )?;
                    let mut packed = vec![0; data.len() / 32 * 34];
                    for (i, block) in packed.chunks_exact_mut(34).enumerate() {
                        block[..2].copy_from_slice(&scales[i * 2..i * 2 + 2]);
                        for (d, s) in block[2..].iter_mut().zip(&data[i * 32..i * 32 + 32]) {
                            *d = s.wrapping_sub(128);
                        }
                    }
                    let store =
                        QStorage::from_data(Cow::Borrowed(&packed), &Device::Cpu, GgmlDType::Q8_0)?;
                    quant.insert(name.clone(), QTensor::new(store, shape)?);
                } else {
                    dense.insert(
                        name.clone(),
                        tensor(&data, info["dtype"].as_str().unwrap(), shape)?,
                    );
                }
            }
        }
        Ok(Self {
            dense,
            quant,
            file,
            manifest,
        })
    }
    pub fn safetensors(dir: &Path) -> Result<Self, Error> {
        let mut file = File::open(dir.join("model.safetensors"))?;
        let mut n = [0; 8];
        file.read_exact(&mut n)?;
        let n = u64::from_le_bytes(n);
        let header: Value = serde_json::from_slice(&bytes(&file, 8, n as usize)?)?;
        let mut dense = HashMap::new();
        for (name, info) in header.as_object().unwrap() {
            if name == "__metadata__" {
                continue;
            }
            let start = info["data_offsets"][0].as_u64().unwrap();
            let end = info["data_offsets"][1].as_u64().unwrap();
            let shape: Vec<_> = info["shape"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap() as usize)
                .collect();
            dense.insert(
                name.clone(),
                tensor(
                    &bytes(&file, 8 + n + start, (end - start) as usize)?,
                    info["dtype"].as_str().unwrap(),
                    shape,
                )?,
            );
        }
        Ok(Self {
            dense,
            quant: HashMap::new(),
            file,
            manifest: header,
        })
    }
    pub fn linear(&self, x: &Tensor, key: &str, bias: Option<&str>) -> Result<Tensor, Error> {
        // FP32 compute throughout: EmbeddingGemma explicitly forbids F16 inference.
        let scratch;
        if let Some(q) = self.quant.get(key) {
            scratch = q.dequantize(&Device::Cpu)?;
        } else {
            scratch = self.dense[key].clone();
        }
        let y = x.matmul(&scratch.t()?)?;
        Ok(if let Some(b) = bias {
            y.broadcast_add(&self.dense[b])?
        } else {
            y
        })
    }
    pub fn affine(&self, x: &Tensor, p: &str) -> Result<Tensor, Error> {
        self.linear(x, &format!("{p}.weight"), Some(&format!("{p}.bias")))
    }
    pub fn rms(&self, x: &Tensor, key: &str) -> Result<Tensor, Error> {
        Ok(unit_rms(x)?.broadcast_mul(&self.dense[key])?)
    }
    pub fn ln(&self, x: &Tensor, p: &str) -> Result<Tensor, Error> {
        let c = x.broadcast_sub(&x.mean_keepdim(D::Minus1)?)?;
        let v = (c.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?.sqrt()?;
        Ok(c.broadcast_div(&v)?
            .broadcast_mul(&self.dense[&format!("{p}.weight")])?
            .broadcast_add(&self.dense[&format!("{p}.bias")])?)
    }
    pub fn embed_packed(&self, ids: &[u32]) -> Result<Tensor, Error> {
        let info = &self.manifest["tensors"]["embed"];
        let width = info["shape"][1].as_u64().unwrap() as usize;
        let mut values = Vec::with_capacity(ids.len() * width);
        for id in ids {
            assert!((*id as u64) < info["shape"][0].as_u64().unwrap());
            let raw = bytes(
                &self.file,
                info["offset"].as_u64().unwrap() + *id as u64 * width as u64,
                width,
            )?;
            let s = bytes(
                &self.file,
                info["scales"].as_u64().unwrap() + *id as u64 * (width / 16) as u64,
                width / 16,
            )?;
            for (i, b) in raw.iter().enumerate() {
                let scale = f16::from_le_bytes([s[i / 32 * 2], s[i / 32 * 2 + 1]]).to_f32();
                values.push((*b as f32 - 128.) * scale);
            }
        }
        Ok(Tensor::from_vec(values, (ids.len(), width), &Device::Cpu)?)
    }
}
pub fn unit_rms(x: &Tensor) -> Result<Tensor, Error> {
    Ok(x.broadcast_div(&(x.sqr()?.mean_keepdim(D::Minus1)? + 1e-6)?.sqrt()?)?)
}
pub fn gelu(x: &Tensor) -> Result<Tensor, Error> {
    Ok((x * 0.5)?.broadcast_mul(
        &(((x + (x.sqr()?.mul(x)? * 0.044715)?)? * 0.7978845608028654)?.tanh()? + 1.)?,
    )?)
}
pub fn l2(x: &Tensor) -> Result<Tensor, Error> {
    let n = x
        .sqr()?
        .sum_keepdim(D::Minus1)?
        .sqrt()?
        .clamp(1e-12, f32::MAX as f64)?;
    Ok(x.broadcast_div(&n)?)
}
fn rope(x: &Tensor, theta: f32) -> Result<Tensor, Error> {
    let (_, l, d) = x.dims3()?;
    let mut angles = Vec::with_capacity(l * d);
    for t in 0..l {
        for i in 0..d {
            angles.push(t as f32 * theta.powf(-((2 * (i % (d / 2))) as f32) / d as f32));
        }
    }
    let a = Tensor::from_vec(angles, (1, l, d), &Device::Cpu)?;
    let half = Tensor::cat(
        &[&x.narrow(2, d / 2, d / 2)?.neg()?, &x.narrow(2, 0, d / 2)?],
        2,
    )?;
    Ok((x.broadcast_mul(&a.cos()?)? + half.broadcast_mul(&a.sin()?)?)?)
}
pub fn attention(
    q: &Tensor,
    k: &Tensor,
    v: &Tensor,
    spans: &[Range<usize>],
    heads: usize,
    kv: usize,
    d: usize,
    theta: f32,
    window: Option<usize>,
    scale: f64,
) -> Result<Tensor, Error> {
    let mut outputs = Vec::new();
    let idx = Tensor::new(
        (0..heads)
            .map(|h| (h / (heads / kv)) as u32)
            .collect::<Vec<_>>()
            .as_slice(),
        &Device::Cpu,
    )?;
    for span in spans {
        let l = span.len();
        let q = rope(
            &q.narrow(0, span.start, l)?
                .reshape((l, heads, d))?
                .permute((1, 0, 2))?
                .contiguous()?,
            theta,
        )?;
        let k = rope(
            &k.narrow(0, span.start, l)?
                .reshape((l, kv, d))?
                .permute((1, 0, 2))?
                .contiguous()?,
            theta,
        )?
        .index_select(&idx, 0)?;
        let v = v
            .narrow(0, span.start, l)?
            .reshape((l, kv, d))?
            .permute((1, 0, 2))?
            .contiguous()?
            .index_select(&idx, 0)?;
        let mut logits = (q.matmul(&k.transpose(1, 2)?.contiguous()?)? * scale)?;
        if let Some(w) = window {
            let mask: Vec<f32> = (0..l)
                .flat_map(|i| {
                    (0..l).map(move |j| {
                        if i.abs_diff(j) <= w {
                            0.
                        } else {
                            f32::NEG_INFINITY
                        }
                    })
                })
                .collect();
            logits = logits.broadcast_add(&Tensor::from_vec(mask, (1, l, l), &Device::Cpu)?)?;
        }
        let e = logits
            .broadcast_sub(&logits.max_keepdim(D::Minus1)?)?
            .exp()?;
        let p = e.broadcast_div(&e.sum_keepdim(D::Minus1)?)?;
        outputs.push(
            p.matmul(&v)?
                .transpose(0, 1)?
                .contiguous()?
                .reshape((l, heads * d))?,
        );
    }
    Ok(Tensor::cat(&outputs, 0)?)
}
pub fn softmax(logits: &[f32], temperature: f64) -> Vec<f64> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let v: Vec<_> = logits
        .iter()
        .map(|x| ((*x as f64 - max) / temperature).exp())
        .collect();
    let sum: f64 = v.iter().sum();
    v.into_iter().map(|x| x / sum).collect()
}
pub fn best(p: &[f64]) -> usize {
    let mut i = 0;
    for j in 1..p.len() {
        if p[j] > p[i] {
            i = j;
        }
    }
    i
}
