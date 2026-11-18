//! Phonon frontend: constant padding, symmetric Hann, Slaney mel, Bessel norm.
//! Separate from Nemotron's unnormalised, reflect-padded streaming frontend.
use crate::Result;
use rayon::prelude::*;
const FFT: usize = 512;
const MELS: usize = 128;
fn fft(x: &mut [f32; 1024]) {
    let mut j = 0;
    for i in 1..FFT {
        let mut bit = FFT >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1
        }
        j ^= bit;
        if i < j {
            x.swap(2 * i, 2 * j);
            x.swap(2 * i + 1, 2 * j + 1);
        }
    }
    let mut len = 2;
    while len <= FFT {
        let angle = -2.0 * std::f64::consts::PI / len as f64;
        let (wr, wi) = (angle.cos() as f32, angle.sin() as f32);
        for i in (0..FFT).step_by(len) {
            let (mut cr, mut ci) = (1f32, 0f32);
            for j in 0..len / 2 {
                let (u, v) = (i + j, i + j + len / 2);
                let tr = x[2 * v] * cr - x[2 * v + 1] * ci;
                let ti = x[2 * v] * ci + x[2 * v + 1] * cr;
                x[2 * v] = x[2 * u] - tr;
                x[2 * v + 1] = x[2 * u + 1] - ti;
                x[2 * u] += tr;
                x[2 * u + 1] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        len *= 2;
    }
}
pub fn mel(audio: &[f32]) -> Result<(Vec<f32>, usize)> {
    let frames = audio.len() / 160;
    if frames < 2 {
        return Err("Phonon requires at least 20 ms of audio".into());
    }
    if audio.iter().any(|v| !v.is_finite()) {
        return Err("non-finite audio".into());
    }
    let mut signal = vec![0f32; audio.len() + 512];
    signal[256] = audio[0];
    for i in 1..audio.len() {
        signal[256 + i] = audio[i] - 0.97 * audio[i - 1];
    }
    let mut window = [0f32; 512];
    for i in 0..400 {
        window[56 + i] = (0.5 - 0.5 * (2. * std::f64::consts::PI * i as f64 / 399.).cos()) as f32;
    }
    let hi = 15. + 27. * 8f64.ln() / 6.4f64.ln();
    let hz: Vec<f64> = (0..130)
        .map(|i| {
            let m = hi * i as f64 / 129.;
            if m < 15. {
                m * 200. / 3.
            } else {
                1000. * ((m - 15.) * 6.4f64.ln() / 27.).exp()
            }
        })
        .collect();
    let mut bank = vec![0f32; 128 * 257];
    for m in 0..128 {
        for k in 0..257 {
            let f = 16000. * k as f64 / 512.;
            let (lo, mid, hi) = (hz[m], hz[m + 1], hz[m + 2]);
            let v = if f >= lo && f <= mid {
                (f - lo) / (mid - lo)
            } else if f > mid && f <= hi {
                (hi - f) / (hi - mid)
            } else {
                0.
            };
            bank[m * 257 + k] = (v * 2. / (hi - lo)) as f32;
        }
    }
    let mut out = vec![0f32; frames * MELS];
    out.par_chunks_mut(MELS)
        .enumerate()
        .for_each(|(frame, row)| {
            let mut x = [0f32; 1024];
            for i in 0..512 {
                x[2 * i] = signal[frame * 160 + i] * window[i];
            }
            fft(&mut x);
            let mut power = [0f32; 257];
            for k in 0..257 {
                power[k] = x[2 * k] * x[2 * k] + x[2 * k + 1] * x[2 * k + 1];
            }
            for m in 0..MELS {
                let mut sum = 0.;
                for k in 0..257 {
                    sum += bank[m * 257 + k] * power[k];
                }
                row[m] = (sum + 5.960_464_5e-8).ln();
            }
        });
    let mut means = [0f64; 128];
    let mut vars = [0f64; 128];
    for row in out.chunks_exact(128) {
        for m in 0..128 {
            means[m] += row[m] as f64;
        }
    }
    for mean in &mut means {
        *mean /= frames as f64;
    }
    for row in out.chunks_exact(128) {
        for m in 0..128 {
            vars[m] += (row[m] as f64 - means[m]).powi(2);
        }
    }
    let mut inv = [0f32; 128];
    for m in 0..128 {
        inv[m] = 1. / ((vars[m] / (frames - 1) as f64).sqrt() as f32 + 1e-5);
    }
    for row in out.chunks_exact_mut(128) {
        for m in 0..128 {
            row[m] = (row[m] - means[m] as f32) * inv[m];
        }
    }
    Ok((out, frames))
}
