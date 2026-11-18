//! C reference's MR=4 / NR=16 blocked exact GEMM, transliterated to Rust.
//! B stays compressed; each worker reuses at most 4096*16 floats (256 KiB).
use crate::Result;
use rayon::prelude::*;
use std::cell::RefCell;
thread_local! {static B_PACK:RefCell<Vec<f32>>=const{RefCell::new(Vec::new())};}

#[cfg(target_arch="x86_64")]
#[target_feature(enable="avx2,fma")]
unsafe fn block_avx<const MR:usize>(a:&[f32],b:&[f32],out:&mut[f32],k:usize) {
    use std::arch::x86_64::*;
    let mut c0=[_mm256_setzero_ps();MR];let mut c1=c0;
    for kk in 0..k {
        let b0=_mm256_loadu_ps(b.as_ptr().add(kk*16));let b1=_mm256_loadu_ps(b.as_ptr().add(kk*16+8));
        for row in 0..MR {
            let x=_mm256_set1_ps(a[row*k+kk]);
            c0[row]=_mm256_fmadd_ps(x,b0,c0[row]);c1[row]=_mm256_fmadd_ps(x,b1,c1[row]);
        }
    }
    for row in 0..MR {
        _mm256_storeu_ps(out.as_mut_ptr().add(row*16),c0[row]);_mm256_storeu_ps(out.as_mut_ptr().add(row*16+8),c1[row]);
    }
}
fn panel(a:&[f32],b:&[f32],out:&mut[f32],m:usize,k:usize) {
    #[cfg(target_arch="x86_64")]
    if std::is_x86_feature_detected!("avx2")&&std::is_x86_feature_detected!("fma") {
        for base in (0..m).step_by(4) {
            let a=&a[base*k..];let y=&mut out[base*16..];
            unsafe{match (m-base).min(4){1=>block_avx::<1>(a,b,y,k),2=>block_avx::<2>(a,b,y,k),3=>block_avx::<3>(a,b,y,k),_=>block_avx::<4>(a,b,y,k)}}
        }return;
    }
    for row in 0..m {for col in 0..16 {let mut sum=0f32;for kk in 0..k {sum=a[row*k+kk].mul_add(b[kk*16+col],sum);}out[row*16+col]=sum;}}
}
pub fn linear<F>(a:&[f32],m:usize,k:usize,n:usize,pack:F)->Result<Vec<f32>>
where F:Fn(usize,&mut[f32])->Result<()>+Sync {
    if a.len()!=m*k||k==0||k>4096{return Err("packed GEMM geometry exceeds the Phonon scratch bound".into())}
    if m==0{return Ok(Vec::new())}
    let mut panels=vec![0f32;n.div_ceil(16)*m*16];
    panels.par_chunks_mut(m*16).enumerate().try_for_each(|(p,out)| {
        B_PACK.with(|slot| {
            let mut buf=slot.borrow_mut();buf.resize(k*16,0.);pack(p*16,&mut buf)?;
            panel(a,&buf,out,m,k);Ok::<(),crate::Error>(())
        })
    })?;
    let mut out=vec![0f32;m*n];
    for row in 0..m{for col in 0..n{out[row*n+col]=panels[(col/16)*m*16+row*16+col%16];}}
    Ok(out)
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn packed_panels_preserve_tail_rows_columns_and_accumulation(){
        for m in [1,3,4,7]{for n in [1,16,19,32]{let k=37;
            let a:Vec<f32>=(0..m*k).map(|i|(i as f32*0.13).sin()).collect();
            let b:Vec<f32>=(0..k*n).map(|i|(i as f32*0.17).cos()).collect();
            let actual=linear(&a,m,k,n,|col,panel|{for kk in 0..k{for j in 0..16{panel[kk*16+j]=if col+j<n{b[kk*n+col+j]}else{0.};}}Ok(())}).unwrap();
            for row in 0..m{for col in 0..n{let mut expected=0f32;for kk in 0..k{expected=a[row*k+kk].mul_add(b[kk*n+col],expected);}assert_eq!(actual[row*n+col],expected);}}
        }}
    }
}
