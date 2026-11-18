//! CPU transliteration of decision.js's pinned packed Q8 d3 engine.
//! Persistent matrices stay Q8; one operator's F32/F16-rounded weight scratch
//! is dequantized at a time. No expanded trunk, shared-prefix approximation or media.
use candle_core::{
    quantized::{GgmlDType, QStorage, QTensor},
    DType, Device, Tensor, D,
};
use half::f16;
use rayon::prelude::*;
use serde_json::Value;
use std::{
    borrow::Cow, collections::HashMap, fs::File, ops::Range, os::unix::fs::FileExt, path::Path,
};
pub type Error = Box<dyn std::error::Error>;
fn r(x: f32) -> f32 {
    f16::from_f32(x).to_f32()
}
fn silu(x: f32) -> f32 {
    x / (1. + (-x).exp())
}
fn bytes(file: &File, offset: u64, n: usize) -> Result<Vec<u8>, Error> {
    let mut b = vec![0; n];
    file.read_exact_at(&mut b, offset)?;
    Ok(b)
}
fn repack(codes: &[u8], scales: &[u8]) -> Vec<u8> {
    assert_eq!(codes.len() % 32, 0);
    assert_eq!(scales.len(), codes.len() / 16);
    let mut packed = vec![0; codes.len() / 32 * 34];
    for (i, block) in packed.chunks_exact_mut(34).enumerate() {
        block[..2].copy_from_slice(&scales[i * 2..i * 2 + 2]);
        // decision.js stores offset-u8 (zero = 128); GGML stores signed i8.
        for (dst, src) in block[2..].iter_mut().zip(&codes[i * 32..i * 32 + 32]) {
            *dst = src.wrapping_sub(128);
        }
    }
    packed
}
pub struct Model {
    file: File,
    manifest: Value,
    weights: HashMap<String, QTensor>,
    aux: HashMap<String, Vec<f32>>,
    pub packed_bytes: usize,
}
impl Model {
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let manifest: Value = serde_json::from_slice(&std::fs::read(dir.join("manifest.json"))?)?;
        assert_eq!(manifest["bytes"], 802817536);
        assert_eq!(manifest["format"], "d1.js/1");
        assert_eq!(manifest["config"]["hidden"], 1024);
        assert_eq!(manifest["config"]["linear_heads"], 16);
        assert_eq!(manifest["config"]["temperature"], 1.0);
        let file = File::open(dir.join("weights.bin"))?;
        assert_eq!(file.metadata()?.len(), 802817536);
        let mut weights = HashMap::new();
        let mut aux = HashMap::new();
        let mut packed_bytes = 0;
        for (name, info) in manifest["tensors"].as_object().unwrap() {
            if name == "embed" {
                continue;
            } // pread just the needed rows; do not expand the 254M embedding entries
            let data = bytes(
                &file,
                info["offset"].as_u64().unwrap(),
                info["bytes"].as_u64().unwrap() as usize,
            )?;
            if info["dtype"] == "q8" {
                let scales = bytes(
                    &file,
                    info["scales"].as_u64().unwrap(),
                    info["scales_bytes"].as_u64().unwrap() as usize,
                )?;
                let packed = repack(&data, &scales);
                packed_bytes += packed.len();
                let shape: Vec<_> = info["shape"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_u64().unwrap() as usize)
                    .collect();
                // Candle 0.9.2's raw loader copies through a borrowed typed
                // slice. Keep the bytes alive until that copy has completed.
                let storage =
                    QStorage::from_data(Cow::Borrowed(&packed), &Device::Cpu, GgmlDType::Q8_0)?;
                weights.insert(name.clone(), QTensor::new(storage, shape)?);
            } else {
                let values = if info["dtype"] == "f16" {
                    data.chunks_exact(2)
                        .map(|b| f16::from_le_bytes([b[0], b[1]]).to_f32())
                        .collect()
                } else {
                    assert_eq!(info["dtype"], "f32");
                    data.chunks_exact(4)
                        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                        .collect()
                };
                aux.insert(name.clone(), values);
            }
        }
        Ok(Self {
            file,
            manifest,
            weights,
            aux,
            packed_bytes,
        })
    }
    fn linear(&self, x: &[f32], rows: usize, key: &str) -> Result<Vec<f32>, Error> {
        let q = &self.weights[key];
        let (n, k) = q.shape().dims2()?;
        assert_eq!(x.len(), rows * k);
        // Match reference portable kernels: Q8 code * scale is rounded to F16
        // before FP32 dot accumulation. Scratch dies before the next operator.
        let w = q
            .dequantize(&Device::Cpu)?
            .to_dtype(DType::F16)?
            .to_dtype(DType::F32)?;
        let input = Tensor::from_slice(x, (rows, k), &Device::Cpu)?;
        let mut out = input.matmul(&w.t()?)?.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(out.len(), rows * n);
        out.iter_mut().for_each(|v| *v = r(*v));
        Ok(out)
    }
    fn embeddings(&self, ids: &[u32]) -> Result<Vec<f32>, Error> {
        let info = &self.manifest["tensors"]["embed"];
        let mut result = Vec::with_capacity(ids.len() * 1024);
        for id in ids {
            if *id >= 248320 {
                return Err("token id out of range".into());
            }
            let codes = bytes(
                &self.file,
                info["offset"].as_u64().unwrap() + *id as u64 * 1024,
                1024,
            )?;
            let scale = bytes(
                &self.file,
                info["scales"].as_u64().unwrap() + *id as u64 * 64,
                64,
            )?;
            for (i, b) in codes.iter().enumerate() {
                let d = f16::from_le_bytes([scale[(i / 32) * 2], scale[(i / 32) * 2 + 1]]).to_f32();
                result.push(r((*b as f32 - 128.) * d));
            }
        }
        Ok(result)
    }
    fn norm(&self, x: &mut [f32], residual: Option<&[f32]>, key: &str) -> Vec<f32> {
        let weight = &self.aux[key];
        assert_eq!(weight.len(), 1024);
        let mut out = vec![0.; x.len()];
        for row in 0..x.len() / 1024 {
            let mut ss = 0.;
            for d in 0..1024 {
                let at = row * 1024 + d;
                let v = x[at] + residual.map_or(0., |y| y[at]);
                ss += v * v;
                x[at] = r(v);
            }
            let inv = 1. / (ss / 1024. + 1e-6).sqrt();
            for d in 0..1024 {
                let at = row * 1024 + d;
                out[at] = r(x[at] * inv * weight[d]);
            }
        }
        out
    }
    fn delta(&self, z: &[f32], spans: &[Range<usize>], p: &str) -> Vec<f32> {
        let conv = &self.aux[&format!("{p}conv")];
        let alog = &self.aux[&format!("{p}A_log")];
        let dt = &self.aux[&format!("{p}dt_bias")];
        let norm = &self.aux[&format!("{p}gnorm")];
        let pieces: Vec<_> = (0..spans.len() * 16)
            .into_par_iter()
            .map(|job| {
                let span = &spans[job / 16];
                let head = job % 16;
                let mut state = vec![0f32; 128 * 128];
                let mut out = vec![0.; span.len() * 128];
                let mut q = [0f32; 128];
                let mut k = [0f32; 128];
                let mut v = [0f32; 128];
                let mut y = [0f32; 128];
                for (local, t) in span.clone().enumerate() {
                    for part in 0..3 {
                        for d in 0..128 {
                            let ch = part * 2048 + head * 128 + d;
                            let mut value = 0.;
                            for tap in 0..4 {
                                let back = 3 - tap;
                                if local >= back {
                                    value += z[(t - back) * 8320 + ch] * conv[ch * 4 + tap];
                                }
                            }
                            let value = r(silu(value));
                            match part {
                                0 => q[d] = value,
                                1 => k[d] = value,
                                _ => v[d] = value,
                            }
                        }
                    }
                    let qnorm =
                        1. / (q.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt() / 128f32.sqrt();
                    let knorm = 1. / (k.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt();
                    for d in 0..128 {
                        q[d] *= qnorm;
                        k[d] *= knorm;
                    }
                    let a = z[t * 8320 + 8208 + head] + dt[head];
                    let sp = if a > 20. { a } else { (1. + a.exp()).ln() };
                    let decay = (-alog[head].exp() * sp).exp();
                    let beta = 1. / (1. + (-z[t * 8320 + 8192 + head]).exp());
                    delta_step(&mut state, &q, &k, &v, decay, beta, &mut y);
                    let inv = 1. / (y.iter().map(|v| v * v).sum::<f32>() / 128. + 1e-6).sqrt();
                    for d in 0..128 {
                        out[local * 128 + d] =
                            r(y[d] * inv * norm[d] * silu(z[t * 8320 + 6144 + head * 128 + d]));
                    }
                }
                (span.start, head, out)
            })
            .collect();
        let mut out = vec![0.; z.len() / 8320 * 2048];
        for (start, head, values) in pieces {
            for (local, row) in values.chunks_exact(128).enumerate() {
                out[(start + local) * 2048 + head * 128..(start + local) * 2048 + (head + 1) * 128]
                    .copy_from_slice(row);
            }
        }
        out
    }
    fn attention(&self, z: &mut [f32], spans: &[Range<usize>], p: &str) -> Result<Vec<f32>, Error> {
        let qn = &self.aux[&format!("{p}q_norm")];
        let kn = &self.aux[&format!("{p}k_norm")];
        for span in spans {
            for (position, t) in span.clone().enumerate() {
                for h in 0..10 {
                    let base = t * 5120 + h * 256;
                    let weight = if h < 8 { qn } else { kn };
                    let inv = 1.
                        / (z[base..base + 256].iter().map(|v| v * v).sum::<f32>() / 256. + 1e-6)
                            .sqrt();
                    for d in 0..32 {
                        let a = z[base + d] * inv * weight[d];
                        let b = z[base + d + 32] * inv * weight[d + 32];
                        let angle = position as f32 * 10000000f32.powf(-((2 * d) as f32) / 64.);
                        let (s, c) = angle.sin_cos();
                        z[base + d] = r(a * c - b * s);
                        z[base + d + 32] = r(b * c + a * s);
                    }
                    for d in 64..256 {
                        z[base + d] = r(z[base + d] * inv * weight[d]);
                    }
                }
            }
        }
        let mut out = vec![0.; z.len() / 5120 * 2048];
        for span in spans {
            let l = span.len();
            let mut q = Vec::with_capacity(8 * l * 256);
            let mut k = Vec::with_capacity(8 * l * 256);
            let mut v = Vec::with_capacity(8 * l * 256);
            for h in 0..8 {
                for t in span.clone() {
                    q.extend_from_slice(&z[t * 5120 + h * 256..t * 5120 + (h + 1) * 256]);
                    k.extend_from_slice(
                        &z[t * 5120 + 2048 + (h / 4) * 256..t * 5120 + 2048 + (h / 4 + 1) * 256],
                    );
                    v.extend_from_slice(
                        &z[t * 5120 + 2560 + (h / 4) * 256..t * 5120 + 2560 + (h / 4 + 1) * 256],
                    );
                }
            }
            let q = Tensor::from_vec(q, (8, l, 256), &Device::Cpu)?;
            let k = Tensor::from_vec(k, (8, l, 256), &Device::Cpu)?;
            let v = Tensor::from_vec(v, (8, l, 256), &Device::Cpu)?;
            let logits = (q.matmul(&k.transpose(1, 2)?.contiguous()?)? / 16.)?;
            let exp = logits
                .broadcast_sub(&logits.max_keepdim(D::Minus1)?)?
                .exp()?;
            let result = exp
                .broadcast_div(&exp.sum_keepdim(D::Minus1)?)?
                .matmul(&v)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            for h in 0..8 {
                for (i, t) in span.clone().enumerate() {
                    for d in 0..256 {
                        out[t * 2048 + h * 256 + d] = r(result[(h * l + i) * 256 + d]
                            / (1. + (-z[t * 5120 + 3072 + h * 256 + d]).exp()));
                    }
                }
            }
        }
        Ok(out)
    }
    pub fn infer(
        &self,
        sequences: &[Vec<u32>],
        options: &[usize],
        debug: bool,
    ) -> Result<(Vec<Vec<f64>>, Value), Error> {
        assert_eq!(sequences.len(), options.len());
        let mut ids = Vec::new();
        let mut spans = Vec::new();
        for seq in sequences {
            if seq.is_empty() {
                return Err("empty sequence".into());
            }
            let start = ids.len();
            ids.extend_from_slice(seq);
            spans.push(start..ids.len());
        }
        if ids.len() > 8192 {
            return Err("native probe exceeds 8192 packed tokens".into());
        }
        let t = ids.len();
        let mut x = self.embeddings(&ids)?;
        let mut residual: Option<Vec<f32>> = None;
        let mut snapshots = serde_json::Map::new();
        for layer in 0..24 {
            let p = format!("layers.{layer}.");
            let normed = self.norm(&mut x, residual.as_deref(), &format!("{p}input_norm"));
            if debug {
                snapshots.insert(
                    layer.to_string(),
                    serde_json::json!([&x[..1024], &x[(t - 1) * 1024..]]),
                );
            }
            let mixed = if layer % 4 != 3 {
                let z = self.linear(&normed, t, &format!("{p}in"))?;
                self.linear(&self.delta(&z, &spans, &p), t, &format!("{p}out"))?
            } else {
                let mut z = self.linear(&normed, t, &format!("{p}qkvg"))?;
                self.linear(&self.attention(&mut z, &spans, &p)?, t, &format!("{p}o"))?
            };
            let normed = self.norm(&mut x, Some(&mixed), &format!("{p}post_norm"));
            let z = self.linear(&normed, t, &format!("{p}w13"))?;
            let mut h = vec![0.; t * 3584];
            for row in 0..t {
                for d in 0..3584 {
                    h[row * 3584 + d] = r(silu(z[row * 7168 + d]) * z[row * 7168 + 3584 + d]);
                }
            }
            residual = Some(self.linear(&h, t, &format!("{p}w2"))?);
            if debug {
                eprintln!("d3 layer {layer}");
            }
        }
        let final_hidden = self.norm(&mut x, residual.as_deref(), "final_norm");
        if debug {
            snapshots.insert(
                "final".into(),
                serde_json::json!([&final_hidden[..1024], &final_hidden[(t - 1) * 1024..]]),
            );
        }
        let readout = &self.aux["readout"];
        let mut result = Vec::new();
        for (span, n) in spans.iter().zip(options) {
            if *n < 2 || *n > 255 {
                return Err("unsupported option count".into());
            }
            let last = &final_hidden[(span.end - 1) * 1024..span.end * 1024];
            let logits: Vec<f32> = (0..*n)
                .map(|c| {
                    last.iter()
                        .zip(&readout[c * 1024..(c + 1) * 1024])
                        .map(|(a, b)| a * b)
                        .sum()
                })
                .collect();
            if !logits.iter().all(|x| x.is_finite()) {
                return Err("non-finite d3 logits".into());
            }
            let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
            let exp: Vec<_> = logits.iter().map(|v| (*v as f64 - max).exp()).collect();
            let sum: f64 = exp.iter().sum();
            result.push(exp.into_iter().map(|v| v / sum).collect());
        }
        Ok((result, Value::Object(snapshots)))
    }
}
fn delta_step(
    state: &mut [f32],
    q: &[f32; 128],
    k: &[f32; 128],
    v: &[f32; 128],
    decay: f32,
    beta: f32,
    out: &mut [f32; 128],
) {
    let kq: f32 = k.iter().zip(q).map(|(a, b)| a * b).sum();
    for row in 0..128 {
        let s = &mut state[row * 128..(row + 1) * 128];
        let mut dotk = 0.;
        let mut dotq = 0.;
        for d in 0..128 {
            dotk += s[d] * k[d];
            dotq += s[d] * q[d];
        }
        let delta = beta * (v[row] - decay * dotk);
        out[row] = decay * dotq + kq * delta;
        for d in 0..128 {
            s[d] = decay * s[d] + k[d] * delta;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn block_repacking_converts_offset_u8_and_preserves_half_scales() -> Result<(), Error> {
        let codes: Vec<_> = (-16..16).map(|v| (v + 128) as u8).collect();
        let scale = f16::from_f32(0.5).to_le_bytes();
        let packed = repack(&codes, &scale);
        let q = QTensor::new(
            QStorage::from_data(Cow::Borrowed(&packed), &Device::Cpu, GgmlDType::Q8_0)?,
            (1, 32),
        )?;
        let got = q
            .dequantize(&Device::Cpu)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(got, (-16..16).map(|v| v as f32 * 0.5).collect::<Vec<_>>());
        Ok(())
    }
    #[test]
    fn delta_rule_updates_the_value_rows_and_decays_previous_state() {
        let mut state = vec![0.; 128 * 128];
        let mut q = [0.; 128];
        q[0] = 1.;
        let k = q;
        let mut v = [0.; 128];
        v[3] = 2.;
        let mut out = [0.; 128];
        delta_step(&mut state, &q, &k, &v, 1., 0.5, &mut out);
        assert_eq!(out[3], 1.);
        assert_eq!(state[3 * 128], 1.);
        delta_step(&mut state, &q, &k, &[0.; 128], 0.5, 0., &mut out);
        assert_eq!(out[3], 0.5);
        assert_eq!(state[3 * 128], 0.5);
    }
}
