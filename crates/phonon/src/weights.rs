//! Actual Fermion container, not a Parakeet checkpoint or ONNX reconstruction.
use crate::{Error, Result};
use candle_core::{Device, Tensor};
use half::f16;
use memmap2::Mmap;
use rayon::prelude::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs::File, io::Read, ops::Range, path::Path};

pub const CONTAINER_SHA: &str = "c57119c0f04756932acc1d60d90506346e3708d69f00e654084acc7fe507fe27";
struct FiveMeta {
    scales: usize,
    row_flag: Vec<usize>,
}
struct Record {
    kind: String,
    shape: Vec<usize>,
    bytes: Range<usize>,
    five: Option<FiveMeta>,
}
pub struct Weights {
    data: Mmap,
    records: HashMap<String, Record>,
    dense: HashMap<String, Vec<f32>>,
}
fn half(b: &[u8], n: usize) -> f32 {
    f16::from_bits(u16::from_le_bytes(b[2 * n..2 * n + 2].try_into().unwrap())).to_f32()
}
const fn trit_table() -> [[u8; 5]; 256] {
    let mut table = [[0; 5]; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut x = byte;
        let mut i = 0;
        while i < 5 {
            table[byte][i] = (x % 3) as u8;
            x /= 3;
            i += 1;
        }
        byte += 1;
    }
    table
}
const TRITS: [[u8; 5]; 256] = trit_table();
impl Weights {
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join("model.fermion");
        let mut file = File::open(&path)?;
        let mut hash = Sha256::new();
        let mut buf = [0u8; 65536];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
        }
        if format!("{:x}", hash.finalize()) != CONTAINER_SHA {
            return Err("Phonon 2.5 English container checksum mismatch".into());
        }
        // Read-only mapping; the model bundle must remain immutable while loaded.
        let data = unsafe { Mmap::map(&file)? };
        let len = u64::from_le_bytes(data.get(..8).ok_or("short container")?.try_into()?) as usize;
        if len > 1024 * 1024 {
            return Err("container header too large".into());
        }
        let head: Value = serde_json::from_slice(data.get(8..8 + len).ok_or("short header")?)?;
        if head["format"] != "fermion-five-value-parakeet-v1" {
            return Err("unsupported Fermion format".into());
        }
        let mut at = 8 + len;
        let mut records = HashMap::new();
        for r in head["index"].as_array().ok_or("missing index")? {
            let n = r["b"].as_u64().ok_or("missing record size")? as usize;
            let end = at.checked_add(n).ok_or("record overflow")?;
            if end > data.len() {
                return Err("short payload".into());
            }
            let record = Record {
                kind: r["k"].as_str().ok_or("missing kind")?.into(),
                shape: r["shape"]
                    .as_array()
                    .ok_or("missing shape")?
                    .iter()
                    .map(|v| v.as_u64().map(|v| v as usize).ok_or("invalid dimension"))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
                bytes: at..end,
                five: None,
            };
            if records
                .insert(r["n"].as_str().ok_or("missing name")?.into(), record)
                .is_some()
            {
                return Err("duplicate tensor".into());
            }
            at = end;
        }
        if at != data.len() || records.len() != 723 {
            return Err("unexpected 2.5 container layout".into());
        }
        for record in records.values_mut().filter(|r| r.kind == "five_value") {
            let (rows, cols) = (record.shape[0], record.shape[1]);
            let rb = cols.div_ceil(5);
            let b = &data[record.bytes.clone()];
            let mut row_flag = Vec::with_capacity(rows);
            let mut nnz = 0;
            for row in 0..rows {
                row_flag.push(nnz);
                for byte in 0..rb {
                    nnz += TRITS[b[row * rb + byte] as usize][..(cols - byte * 5).min(5)]
                        .iter()
                        .filter(|&&s| s != 1)
                        .count();
                }
            }
            let scales = rows * rb + nnz.div_ceil(8);
            if scales + rows * 4 != b.len() {
                return Err("bad five-value payload".into());
            }
            record.five = Some(FiveMeta { scales, row_flag });
        }
        let mut w = Self {
            data,
            records,
            dense: HashMap::new(),
        };
        let names: Vec<_> = w
            .records
            .iter()
            .filter(|(name, r)| r.kind != "five_value" && name.as_str() != "encoder.subsampling.linear.weight" && name.as_str() != "encoder_projector.weight")
            .map(|(n, _)| n.clone())
            .collect();
        for name in names {
            let values = w.decode(&name)?;
            w.dense.insert(name, values);
        }
        Ok(w)
    }
    pub fn take(&mut self, name: &str) -> Result<Vec<f32>> {
        self.dense
            .remove(name)
            .ok_or_else(|| format!("dense tensor missing: {name}").into())
    }
    pub fn values(&self, name: &str) -> Result<&[f32]> {
        self.dense
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("dense tensor missing: {name}").into())
    }
    pub fn tensor(&self, name: &str) -> Result<Tensor> {
        let r = self.record(name)?;
        let values = match self.dense.get(name) {
            Some(v) => v.clone(),
            None => self.decode(name)?,
        };
        Ok(Tensor::from_vec(values, r.shape.as_slice(), &Device::Cpu)?)
    }
    pub fn linear(&self, x: &Tensor, name: &str, bias: Option<&str>) -> Result<Tensor> {
        let r = self.record(name)?;
        if matches!(r.kind.as_str(),"five_value"|"int6"|"int8") {
            let (n,k)=(r.shape[0],r.shape.iter().skip(1).product::<usize>());let m=x.dim(0)?;
            if x.dim(1)?!=k{return Err("packed GEMM shape mismatch".into())}
            let a=x.flatten_all()?.to_vec1::<f32>()?;
            let values=crate::kernel::linear(&a,m,k,n,|col,panel|self.pack_panel(r,col,panel))?;
            let y=Tensor::from_vec(values,(m,n),&Device::Cpu)?;
            return Ok(if let Some(b)=bias{y.broadcast_add(&self.tensor(b)?)?}else{y});
        }
        let w = self.tensor(name)?;
        let y = x.matmul(&w.reshape((w.dim(0)?, ()))?.t()?)?;
        Ok(if let Some(b) = bias {
            y.broadcast_add(&self.tensor(b)?)?
        } else {
            y
        })
    }
    fn pack_panel(&self,r:&Record,col:usize,panel:&mut[f32])->Result<()> {
        let (n,k)=(r.shape[0],r.shape.iter().skip(1).product::<usize>());let b=&self.data[r.bytes.clone()];
        panel.fill(0.);
        for j in 0..(n-col).min(16) {
            let row=col+j;
            if r.kind=="five_value" {
                let meta=r.five.as_ref().ok_or("missing five-value index")?;
                let rb=k.div_ceil(5);let flags=n*rb;let mut next=meta.row_flag[row];
                let mag=[half(&b[meta.scales..],row),half(&b[meta.scales+n*2..],row)];
                for byte in 0..rb {let digits=&TRITS[b[row*rb+byte] as usize];
                    for offset in 0..(k-byte*5).min(5){let sign=digits[offset] as i8-1;if sign!=0 {
                        let high=(b[flags+(next>>3)]>>(next&7))&1;next+=1;
                        panel[(byte*5+offset)*16+j]=sign as f32*mag[high as usize];
                    }}
                }
            } else {
                let body=b.len()-n*2;let scale=half(&b[body..],row);
                for kk in 0..k {
                    let i=row*k+kk;
                    let q=if r.kind=="int8"{b[i] as i8 as i32}else{
                        let bit=i*6;let byte=bit/8;let word=b[byte] as u16|((b.get(byte+1).copied().unwrap_or(0) as u16)<<8);
                        ((word>>(bit%8))&63) as i32-32
                    };
                    panel[kk*16+j]=q as f32*scale;
                }
            }
        }
        Ok(())
    }
    fn decode(&self, name: &str) -> Result<Vec<f32>> {
        let r = self.record(name)?;
        let b = &self.data[r.bytes.clone()];
        let size: usize = r.shape.iter().product();
        match r.kind.as_str() {
            "fp16" => {
                if b.len() != size * 2 {
                    return Err("fp16 length mismatch".into());
                }
                Ok((0..size).map(|i| half(b, i)).collect())
            }
            "int6" | "int8" => {
                let rows = r.shape[0];
                let cols = size / rows;
                let body = b.len().checked_sub(rows * 2).ok_or("missing scales")?;
                let bits = if r.kind == "int6" { 6 } else { 8 };
                if body != (size * bits).div_ceil(8) {
                    return Err(format!("int length mismatch: {name}").into());
                }
                Ok((0..size)
                    .map(|i| {
                        let q = if bits == 8 {
                            b[i] as i8 as i32
                        } else {
                            let bit = i * 6;
                            let j = bit / 8;
                            let x =
                                b[j] as u16 | ((b.get(j + 1).copied().unwrap_or(0) as u16) << 8);
                            ((x >> (bit % 8)) & 63) as i32 - 32
                        };
                        q as f32 * half(&b[body..], i / cols)
                    })
                    .collect())
            }
            "five_value" => {
                if r.shape.len() != 2 {
                    return Err("five-value matrix is not 2D".into());
                }
                let (rows, cols) = (r.shape[0], r.shape[1]);
                let rb = cols.div_ceil(5);
                let flags = rows * rb;
                // Row-wise base-3 signs, then bits for NONZERO entries only.
                let meta = r.five.as_ref().ok_or("missing five-value index")?;
                let scales = meta.scales;
                let mut values = vec![0f32; size];
                values
                    .par_chunks_exact_mut(cols)
                    .enumerate()
                    .for_each(|(row, output)| {
                        let mut nonzero = meta.row_flag[row];
                        let mag = [half(&b[scales..], row), half(&b[scales + rows * 2..], row)];
                        for byte in 0..rb {
                            let digits = &TRITS[b[row * rb + byte] as usize];
                            for col in 0..(cols - byte * 5).min(5) {
                                let s = digits[col];
                                if s != 1 {
                                    let high = (b[flags + (nonzero >> 3)] >> (nonzero & 7)) & 1;
                                    nonzero += 1;
                                    output[byte * 5 + col] =
                                        (s as i32 - 1) as f32 * mag[high as usize];
                                }
                            }
                        }
                    });
                Ok(values)
            }
            _ => Err(Error::from(format!("unsupported tensor kind {}", r.kind))),
        }
    }
    fn record(&self, name: &str) -> Result<&Record> {
        self.records
            .get(name)
            .or_else(|| {
                name.strip_suffix(".weight")
                    .and_then(|key| self.records.get(key))
            })
            .ok_or_else(|| format!("missing {name}").into())
    }
}
