//! Native signed-int8 dot products. No saturating pairwise arithmetic.
use rayon::prelude::*;
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avxvnni")]
unsafe fn dot_vnni(a: &[i8], b: &[i8], weight_sum: i32) -> i32 {
    use std::arch::x86_64::*;
    let mut s0 = _mm256_setzero_si256();
    let mut s1 = s0;
    let sign = _mm256_set1_epi8(-128);
    let mut i = 0;
    while i + 64 <= a.len() {
        let a0 = _mm256_xor_si256(_mm256_loadu_si256(a.as_ptr().add(i).cast()), sign);
        let a1 = _mm256_xor_si256(_mm256_loadu_si256(a.as_ptr().add(i + 32).cast()), sign);
        s0 = _mm256_dpbusd_avx_epi32(s0, a0, _mm256_loadu_si256(b.as_ptr().add(i).cast()));
        s1 = _mm256_dpbusd_avx_epi32(s1, a1, _mm256_loadu_si256(b.as_ptr().add(i + 32).cast()));
        i += 64;
    }
    let mut lanes = [0i32; 8];
    _mm256_storeu_si256(lanes.as_mut_ptr().cast(), _mm256_add_epi32(s0, s1));
    let mut out = lanes.iter().sum::<i32>();
    while i < a.len() {
        out += (a[i] as i32 + 128) * b[i] as i32;
        i += 1;
    }
    out - 128 * weight_sum
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dot_avx(a: &[i8], b: &[i8]) -> i32 {
    use std::arch::x86_64::*;
    let mut s0 = _mm256_setzero_si256();
    let mut s1 = s0;
    let mut i = 0;
    while i + 32 <= a.len() {
        let a0 = _mm256_cvtepi8_epi16(_mm_loadu_si128(a.as_ptr().add(i).cast()));
        let b0 = _mm256_cvtepi8_epi16(_mm_loadu_si128(b.as_ptr().add(i).cast()));
        let a1 = _mm256_cvtepi8_epi16(_mm_loadu_si128(a.as_ptr().add(i + 16).cast()));
        let b1 = _mm256_cvtepi8_epi16(_mm_loadu_si128(b.as_ptr().add(i + 16).cast()));
        s0 = _mm256_add_epi32(s0, _mm256_madd_epi16(a0, b0));
        s1 = _mm256_add_epi32(s1, _mm256_madd_epi16(a1, b1));
        i += 32;
    }
    let mut lanes = [0i32; 8];
    _mm256_storeu_si256(lanes.as_mut_ptr().cast(), _mm256_add_epi32(s0, s1));
    let mut out = lanes.iter().sum::<i32>();
    while i < a.len() {
        out += a[i] as i32 * b[i] as i32;
        i += 1;
    }
    out
}
fn dot(a: &[i8], b: &[i8]) -> i32 {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avxvnni") {
        return unsafe { dot_vnni(a, b, b.iter().map(|&v| v as i32).sum()) };
    }
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        return unsafe { dot_avx(a, b) };
    }
    a.iter().zip(b).map(|(&a, &b)| a as i32 * b as i32).sum()
}
pub fn matmul(a: &[f32], m: usize, k: usize, w: &[i8], n: usize, scales: &[f32]) -> Vec<f32> {
    let mut aq = vec![0i8; a.len()];
    let mut ascale = vec![1f32; m];
    for row in 0..m {
        let source = &a[row * k..(row + 1) * k];
        let max = source.iter().fold(0f32, |a, &v| a.max(v.abs()));
        let scale = if max > 0. { max / 127. } else { 1. };
        ascale[row] = scale;
        for i in 0..k {
            aq[row * k + i] = (source[i] / scale).round_ties_even().clamp(-127., 127.) as i8;
        }
    }
    let mut transposed = vec![0f32; n * m];
    #[cfg(target_arch = "x86_64")]
    let vnni = std::is_x86_feature_detected!("avxvnni");
    transposed
        .par_chunks_exact_mut(m)
        .enumerate()
        .for_each(|(col, out)| {
            let weight = &w[col * k..(col + 1) * k];
            #[cfg(target_arch = "x86_64")]
            let weight_sum = if vnni {
                weight.iter().map(|&v| v as i32).sum()
            } else {
                0
            };
            for row in 0..m {
                #[cfg(target_arch = "x86_64")]
                let sum = if vnni {
                    unsafe { dot_vnni(&aq[row * k..(row + 1) * k], weight, weight_sum) }
                } else {
                    dot(&aq[row * k..(row + 1) * k], weight)
                };
                #[cfg(not(target_arch = "x86_64"))]
                let sum = dot(&aq[row * k..(row + 1) * k], weight);
                out[row] = sum as f32 * (ascale[row] * scales[col]);
            }
        });
    let mut out = vec![0f32; n * m];
    for row in 0..m {
        for col in 0..n {
            out[row * n + col] = transposed[col * m + row];
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simd_dot_preserves_full_signed_range_without_saturation() {
        for len in [17, 32, 64, 1024, 4096] {
            let a: Vec<_> = (0..len)
                .map(|i| if i % 3 == 0 { -127 } else { 127 })
                .collect();
            let b: Vec<_> = (0..len)
                .map(|i| if i % 7 == 0 { -127 } else { 127 })
                .collect();
            let expected: i32 = a.iter().zip(&b).map(|(&a, &b)| a as i32 * b as i32).sum();
            assert_eq!(dot(&a, &b), expected);
        }
    }
}
