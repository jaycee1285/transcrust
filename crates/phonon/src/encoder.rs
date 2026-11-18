use crate::{Result, Weights};
use candle_core::{Device, Tensor, D};
use rayon::prelude::*;
use std::{fs, path::Path};
const D_MODEL: usize = 1024;
fn dump(dir: Option<&Path>, name: &str, t: &Tensor) -> Result<()> {
    if let Some(dir) = dir {
        fs::create_dir_all(dir)?;
        let v = t.flatten_all()?.to_vec1::<f32>()?;
        fs::write(
            dir.join(format!("{name}.f32")),
            v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>(),
        )?;
    }
    Ok(())
}
fn norm(x: &Tensor, w: &Weights, name: &str) -> Result<Tensor> {
    let mu = x.mean_keepdim(D::Minus1)?;
    let z = x.broadcast_sub(&mu)?;
    let std = (z.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?.sqrt()?;
    Ok(z.broadcast_div(&std)?
        .broadcast_mul(&w.tensor(&format!("{name}.weight"))?)?
        .broadcast_add(&w.tensor(&format!("{name}.bias"))?)?)
}
fn silu(x: &Tensor) -> Result<Tensor> {
    Ok(x.broadcast_div(&(x.neg()?.exp()? + 1.)?)?)
}
fn linear(w: &Weights, x: &Tensor, module: &str) -> Result<Tensor> {
    w.linear(x, &format!("{module}.weight"), None)
}
struct Plane {values:Vec<f32>,height:usize,width:usize}
// Same allocation and channel layout as C's conv3x3s2_full/dw_worker. Bias
// and ReLU update the output itself, avoiding whole-output tensor copies.
fn spatial(w:&Weights,x:&Plane,layer:usize,depthwise:bool,relu:bool)->Result<Plane> {
    let p=format!("encoder.subsampling.layers.{layer}");let kernel=w.values(&format!("{p}.weight"))?;let bias=w.values(&format!("{p}.bias"))?;
    let (height,width)=(x.height.div_ceil(2),x.width.div_ceil(2));let mut values=vec![0f32;256*height*width];
    values.par_chunks_mut(height*width).enumerate().for_each(|(channel,out)| {
        let base=if depthwise{channel*x.height*x.width}else{0};
        for oh in 0..height{for ow in 0..width{
            let mut sum=bias[channel];
            for kh in 0..3{let ih=2*oh as isize-1+kh as isize;if ih<0||ih>=x.height as isize{continue}
                for kw in 0..3{let iw=2*ow as isize-1+kw as isize;if iw>=0&&iw<x.width as isize {
                    sum=x.values[base+ih as usize*x.width+iw as usize].mul_add(kernel[channel*9+kh*3+kw],sum);
                }}
            }
            out[oh*width+ow]=if relu{sum.max(0.)}else{sum};
        }}
    });
    Ok(Plane{values,height,width})
}
fn pointwise(w:&Weights,x:Plane,layer:usize)->Result<Plane>{
    let p=format!("encoder.subsampling.layers.{layer}");let spatial=x.height*x.width;
    let input=Tensor::from_vec(x.values,(256,spatial),&Device::Cpu)?;
    let weight=w.tensor(&format!("{p}.weight"))?.reshape((256,256))?;
    let y=weight.matmul(&input)?;let mut values=y.flatten_all()?.to_vec1::<f32>()?;
    let bias=w.values(&format!("{p}.bias"))?;
    for (channel,row) in values.chunks_exact_mut(spatial).enumerate(){for v in row{*v=(*v+bias[channel]).max(0.);}}
    Ok(Plane{values,height:x.height,width:x.width})
}
fn subsample(w: &Weights, mel: Vec<f32>, frames: usize) -> Result<Tensor> {
    let input=Plane{values:mel,height:frames,width:128};
    let s1=spatial(w,&input,0,false,true)?;drop(input);
    let s2_dw=spatial(w,&s1,2,true,false)?;drop(s1);
    let s2=pointwise(w,s2_dw,3)?;
    let s3_dw=spatial(w,&s2,5,true,false)?;drop(s2);
    let s3=pointwise(w,s3_dw,6)?;let t=s3.height;
    let mut flat=vec![0f32;t*4096];
    for channel in 0..256{for row in 0..t{flat[row*4096+channel*16..row*4096+(channel+1)*16].copy_from_slice(&s3.values[(channel*t+row)*16..(channel*t+row+1)*16]);}}
    drop(s3);let x=Tensor::from_vec(flat,(t,4096),&Device::Cpu)?;
    w.linear(
        &x,
        "encoder.subsampling.linear.weight",
        Some("encoder.subsampling.linear.bias"),
    )
}
fn feed_forward(w: &Weights, x: Tensor, p: &str, which: usize) -> Result<Tensor> {
    let h = norm(&x, w, &format!("{p}.norm_feed_forward{which}"))?;
    let h = linear(w, &h, &format!("{p}.feed_forward{which}.linear1"))?;
    let h = linear(w, &silu(&h)?, &format!("{p}.feed_forward{which}.linear2"))?;
    Ok((x + (h * 0.5)?)?)
}
fn attention(w: &Weights, x: Tensor, p: &str, pe: &Tensor) -> Result<Tensor> {
    let t = x.dim(0)?;
    let l = 2 * t - 1;
    let h = norm(&x, w, &format!("{p}.norm_self_att"))?;
    let project = |name: &str| -> Result<Tensor> {
        Ok(linear(w, &h, &format!("{p}.self_attn.{name}"))?
            .reshape((t, 8, 128))?
            .transpose(0, 1)?
            .contiguous()?)
    };
    let q = project("q_proj")?;
    let k = project("k_proj")?;
    let v = project("v_proj")?;
    let qu = q.broadcast_add(
        &w.tensor(&format!("{p}.self_attn.bias_u"))?
            .reshape((8, 1, 128))?,
    )?;
    let qv = q.broadcast_add(
        &w.tensor(&format!("{p}.self_attn.bias_v"))?
            .reshape((8, 1, 128))?,
    )?;
    let pos = linear(w, pe, &format!("{p}.self_attn.relative_k_proj"))?
        .reshape((l, 8, 128))?
        .transpose(0, 1)?
        .contiguous()?;
    let ac = qu
        .matmul(&k.transpose(1, 2)?)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let bd = qv
        .matmul(&pos.transpose(1, 2)?)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let mut scores = vec![0f32; 8 * t * t];
    for head in 0..8 {
        for i in 0..t {
            let row = &mut scores[(head * t + i) * t..(head * t + i + 1) * t];
            for j in 0..t {
                row[j] = (ac[(head * t + i) * t + j] + bd[(head * t + i) * l + t - 1 - i + j])
                    / (128f32).sqrt();
            }
            let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let mut sum = 0.;
            for s in row.iter_mut() {
                *s = (*s - max).exp();
                sum += *s;
            }
            for s in row.iter_mut() {
                *s /= sum;
            }
        }
    }
    let scores = Tensor::from_vec(scores, (8, t, t), &Device::Cpu)?;
    let a = scores
        .matmul(&v)?
        .transpose(0, 1)?
        .contiguous()?
        .reshape((t, 1024))?;
    Ok((x + linear(w, &a, &format!("{p}.self_attn.o_proj"))?)?)
}
fn convolution(w: &Weights, x: Tensor, p: &str) -> Result<Tensor> {
    let t = x.dim(0)?;
    let n = norm(&x, w, &format!("{p}.norm_conv"))?;
    let projected = linear(w, &n, &format!("{p}.conv.pointwise_conv1"))?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let mut gated = vec![0f32; t * 1024];
    for i in 0..t {
        for d in 0..1024 {
            gated[i * 1024 + d] = projected[i * 2048 + d]
                / (1. + (-projected[i * 2048 + 1024 + d]).clamp(-87., 87.).exp());
        }
    }
    let dw = w.values(&format!("{p}.conv.depthwise_conv.weight"))?;
    let gamma = w.values(&format!("{p}.conv.norm.weight"))?;
    let beta = w.values(&format!("{p}.conv.norm.bias"))?;
    let mu = w.values(&format!("{p}.conv.norm.running_mean"))?;
    let var = w.values(&format!("{p}.conv.norm.running_var"))?;
    let mut out = vec![0f32; t * 1024];
    for d in 0..1024 {
        let scale = gamma[d] as f64 / (var[d] as f64 + 1e-5).sqrt();
        let bias = (beta[d] as f64 - mu[d] as f64 * scale) as f32;
        let kernel: Vec<f32> = (0..9)
            .map(|k| (dw[d * 9 + k] as f64 * scale) as f32)
            .collect();
        for i in 0..t {
            let mut y = bias;
            for k in 0..9 {
                let frame = i as isize + k as isize - 4;
                if frame >= 0 && (frame as usize) < t {
                    y += gated[frame as usize * 1024 + d] * kernel[k];
                }
            }
            out[i * 1024 + d] = y / (1. + (-y).clamp(-87., 87.).exp());
        }
    }
    let out = Tensor::from_vec(out, (t, 1024), &Device::Cpu)?;
    Ok((x + linear(w, &out, &format!("{p}.conv.pointwise_conv2"))?)?)
}
pub fn encode(w: &Weights, mel: Vec<f32>, frames: usize, trace: Option<&Path>) -> Result<Vec<f32>> {
    let profile = std::env::var_os("PHONON_PROFILE").is_some();
    let start = std::time::Instant::now();
    let mut x = subsample(w, mel, frames)?;
    if profile {
        eprintln!("subsample {:.3}s", start.elapsed().as_secs_f64());
    }
    dump(trace, "subsample", &x)?;
    let t = x.dim(0)?;
    let l = 2 * t - 1;
    let mut pos = vec![0f32; l * D_MODEL];
    for j in 0..l {
        for i in 0..D_MODEL / 2 {
            let inv = 1f32 / 10000f32.powf((2 * i) as f32 / D_MODEL as f32);
            let phase = inv * (t as i32 - 1 - j as i32) as f32;
            pos[j * D_MODEL + 2 * i] = phase.sin();
            pos[j * D_MODEL + 2 * i + 1] = phase.cos();
        }
    }
    let pe = Tensor::from_vec(pos, (l, D_MODEL), &Device::Cpu)?;
    for i in 0..24 {
        let p = format!("encoder.layers.{i}");
        let start = std::time::Instant::now();
        x = feed_forward(w, x, &p, 1)?;
        let ff1 = start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        x = attention(w, x, &p, &pe)?;
        let attn = start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        x = convolution(w, x, &p)?;
        let conv = start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        x = feed_forward(w, x, &p, 2)?;
        let ff2 = start.elapsed().as_secs_f64();
        if profile {
            eprintln!("block {i}: ff1 {ff1:.3}s attn {attn:.3}s conv {conv:.3}s ff2 {ff2:.3}s");
        }
        x = norm(&x, w, &format!("{p}.norm_out"))?;
        dump(trace, &format!("block-{i:02}"), &x)?;
    }
    let out = w.linear(
        &x,
        "encoder_projector.weight",
        Some("encoder_projector.bias"),
    )?;
    dump(trace, "encoder", &out)?;
    Ok(out.flatten_all()?.to_vec1::<f32>()?)
}
