//! Phonon prediction network, trained joint head and duration-aware TDT loop.
use crate::{Result, Weights};
use rayon::prelude::*;
const H: usize = 640;
struct Dense {
    weights: Vec<f32>,
    bias: Vec<f32>,
    cols: usize,
}
impl Dense {
    fn new(w: &mut Weights, name: &str, cols: usize) -> Result<Self> {
        let weights = w.take(&format!("{name}.weight"))?;
        let bias = w.take(&format!("{name}.bias"))?;
        if weights.len() != bias.len() * cols {
            return Err(format!("bad dense geometry {name}").into());
        }
        Ok(Self {
            weights,
            bias,
            cols,
        })
    }
    fn apply(&self, x: &[f32]) -> Vec<f32> {
        self.weights
            .par_chunks_exact(self.cols)
            .zip(self.bias.par_iter())
            .map(|(w, &b)| dot(w, x) + b)
            .collect()
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn dot_avx(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    let mut s0 = _mm256_setzero_ps();
    let mut s1 = s0;
    let mut i = 0;
    while i + 16 <= a.len() {
        s0 = _mm256_fmadd_ps(
            _mm256_loadu_ps(a.as_ptr().add(i)),
            _mm256_loadu_ps(b.as_ptr().add(i)),
            s0,
        );
        s1 = _mm256_fmadd_ps(
            _mm256_loadu_ps(a.as_ptr().add(i + 8)),
            _mm256_loadu_ps(b.as_ptr().add(i + 8)),
            s1,
        );
        i += 16;
    }
    let mut sum = [0f32; 8];
    _mm256_storeu_ps(sum.as_mut_ptr(), _mm256_add_ps(s0, s1));
    let mut out = sum.iter().sum::<f32>();
    while i < a.len() {
        out += a[i] * b[i];
        i += 1;
    }
    out
}
fn dot(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma") {
        return unsafe { dot_avx(a, b) };
    }
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn sigmoid(x: f32) -> f32 {
    1. / (1. + (-x).clamp(-87., 87.).exp())
}
struct Lstm {
    ih: Vec<f32>,
    hh: Vec<f32>,
    bi: Vec<f32>,
    bh: Vec<f32>,
}
impl Lstm {
    fn step(&self, x: &[f32], h: &mut [f32], c: &mut [f32]) {
        let gates: Vec<f32> = (0..4 * H)
            .into_par_iter()
            .map(|r| {
                dot(&self.ih[r * H..(r + 1) * H], x)
                    + dot(&self.hh[r * H..(r + 1) * H], h)
                    + self.bi[r]
                    + self.bh[r]
            })
            .collect();
        for j in 0..H {
            c[j] = sigmoid(gates[H + j]) * c[j] + sigmoid(gates[j]) * gates[2 * H + j].tanh();
            h[j] = sigmoid(gates[3 * H + j]) * c[j].tanh();
        }
    }
}
pub struct Decoder {
    embedding: Vec<f32>,
    layers: Vec<Lstm>,
    project: Dense,
    head: Dense,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub id: usize,
    pub frame: usize,
    pub duration: usize,
}
impl Decoder {
    pub fn load(w: &mut Weights) -> Result<Self> {
        let embedding = w.take("decoder.embedding.weight")?;
        if embedding.len() != 8193 * H || embedding[8192 * H..].iter().any(|&v| v != 0.) {
            return Err("bad blank embedding".into());
        }
        let mut layers = Vec::new();
        for i in 0..2 {
            let ih = w.take(&format!("decoder.lstm.weight_ih_l{i}"))?;
            let hh = w.take(&format!("decoder.lstm.weight_hh_l{i}"))?;
            let bi = w.take(&format!("decoder.lstm.bias_ih_l{i}"))?;
            let bh = w.take(&format!("decoder.lstm.bias_hh_l{i}"))?;
            if ih.len() != 4 * H * H
                || hh.len() != 4 * H * H
                || bi.len() != 4 * H
                || bh.len() != 4 * H
            {
                return Err("bad LSTM geometry".into());
            }
            layers.push(Lstm { ih, hh, bi, bh });
        }
        Ok(Self {
            embedding,
            layers,
            project: Dense::new(w, "decoder.decoder_projector", H)?,
            head: Dense::new(w, "joint.head", H)?,
        })
    }
    fn step(&self, token: usize, h: &mut [Vec<f32>; 2], c: &mut [Vec<f32>; 2]) -> Vec<f32> {
        self.layers[0].step(
            &self.embedding[token * H..(token + 1) * H],
            &mut h[0],
            &mut c[0],
        );
        let (first, second) = h.split_at_mut(1);
        self.layers[1].step(&first[0], &mut second[0], &mut c[1]);
        self.project.apply(&h[1])
    }
    pub fn decode(&self, encoded: &[f32]) -> Result<Vec<Token>> {
        if encoded.len() % H != 0 {
            return Err("bad encoded shape".into());
        }
        let frames = encoded.len() / H;
        let mut h = [vec![0f32; H], vec![0f32; H]];
        let mut c = h.clone();
        let mut pred = self.step(8192, &mut h, &mut c);
        let mut tokens = Vec::new();
        let (mut t, mut symbols) = (0, 0);
        while t < frames {
            let hidden: Vec<f32> = encoded[t * H..(t + 1) * H]
                .iter()
                .zip(&pred)
                .map(|(a, b)| (a + b).max(0.))
                .collect();
            let logits = self.head.apply(&hidden);
            if logits.len() != 8198 || logits.iter().any(|v| !v.is_finite()) {
                return Err("invalid Phonon joint output".into());
            }
            let id = argmax(&logits[..8193]);
            let mut duration = argmax(&logits[8193..]);
            if id == 8192 {
                if duration == 0 {
                    duration = 1;
                }
            } else {
                tokens.push(Token {
                    id,
                    frame: t,
                    duration,
                });
                pred = self.step(id, &mut h, &mut c);
                symbols += 1;
                if duration == 0 && symbols >= 10 {
                    duration = 1;
                }
            }
            if duration != 0 {
                symbols = 0;
            }
            t += duration;
        }
        Ok(tokens)
    }
}
fn argmax(x: &[f32]) -> usize {
    let mut best = 0;
    for i in 1..x.len() {
        if x[i] > x[best] {
            best = i;
        }
    }
    best
}
