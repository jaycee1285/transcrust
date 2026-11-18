use super::common::Error;
pub use super::common::{file_hash as hash, ULTRA_SOURCE_HASH as SOURCE_HASH};
use candle_core::{
    quantized::{GgmlDType, QTensor},
    Device, Tensor,
};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::FileExt,
    path::Path,
};
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::quantized::QStorage;
    use std::borrow::Cow;
    #[test]
    fn q4_rows_survive_storage_roundtrip() -> Result<(), Error> {
        let values = (0..96)
            .map(|i| ((i as f32) * 0.13).sin() * 3.)
            .collect::<Vec<_>>();
        let original = Tensor::from_slice(&values, (3, 32), &Device::Cpu)?;
        let q = QTensor::quantize(&original, GgmlDType::Q4_0)?;
        let packed = q.data()?.into_owned();
        assert_eq!(packed.len(), 54);
        let decoded = q.dequantize(&Device::Cpu)?.to_vec2::<f32>()?;
        for (i, row) in packed.chunks_exact(18).enumerate() {
            let storage = QStorage::from_data(Cow::Borrowed(row), &Device::Cpu, GgmlDType::Q4_0)?;
            let got = QTensor::new(storage, (1, 32))?
                .dequantize(&Device::Cpu)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert_eq!(got, decoded[i]);
        }
        let error = values
            .iter()
            .zip(decoded.iter().flatten())
            .map(|(a, b)| (a - b).abs())
            .fold(0f32, f32::max);
        assert!(error < 0.4, "unexpected Q4 error {error}");
        Ok(())
    }
}
pub fn export(src: &Path, dst: &Path) -> Result<(), Error> {
    if hash(&src.join("model.safetensors"))? != SOURCE_HASH {
        return Err("UltraBERT source checksum mismatch".into());
    }
    if dst.join("ultra-q4.json").exists() || dst.join("weights.q4").exists() {
        return Err("output artifact already exists; choose a new directory".into());
    }
    fs::create_dir_all(dst)?;
    let mut input = File::open(src.join("model.safetensors"))?;
    let mut size = [0; 8];
    input.read_exact(&mut size)?;
    let size = u64::from_le_bytes(size);
    let mut header = vec![0; size as usize];
    input.read_exact(&mut header)?;
    let header: Value = serde_json::from_slice(&header)?;
    let partial = dst.join("weights.q4.partial");
    let mut output = File::create(&partial)?;
    let mut entries = serde_json::Map::new();
    let mut offset = 0u64;
    for (name, info) in header.as_object().ok_or("invalid source header")? {
        if name == "__metadata__" {
            continue;
        }
        if info["dtype"] != "F32" {
            return Err("expected released F32 weights".into());
        }
        let shape: Vec<usize> = serde_json::from_value(info["shape"].clone())?;
        let a = info["data_offsets"][0].as_u64().unwrap();
        let b = info["data_offsets"][1].as_u64().unwrap();
        let mut raw = vec![0; (b - a) as usize];
        input.read_exact_at(&mut raw, 8 + size + a)?;
        let packed = name.starts_with("model.encoder_model.") && shape.len() == 2;
        let dtype;
        if packed {
            let values = raw
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                .collect::<Vec<_>>();
            let t = Tensor::from_vec(values, shape.clone(), &Device::Cpu)?;
            let q = QTensor::quantize(&t, GgmlDType::Q4_0)?;
            let data = q.data()?;
            output.write_all(&data)?;
            entries.insert(
                name.clone(),
                json!({"offset":offset,"bytes":data.len(),"shape":shape,"dtype":"q4_0"}),
            );
            offset += data.len() as u64;
            dtype = "Q4_0";
        } else {
            output.write_all(&raw)?;
            entries.insert(
                name.clone(),
                json!({"offset":offset,"bytes":raw.len(),"shape":shape,"dtype":"f32"}),
            );
            offset += raw.len() as u64;
            dtype = "F32";
        }
        eprintln!("export {dtype}: {name}");
    }
    output.sync_all()?;
    drop(output);
    let sha = hash(&partial)?;
    fs::rename(partial, dst.join("weights.q4"))?;
    fs::copy(src.join("tokenizer.json"), dst.join("tokenizer.json"))?;
    let manifest = json!({"format":"ultra-q4-v1","source_sha256":SOURCE_HASH,"sha256":sha,"bytes":offset,"tensors":entries,"quantization":"GGML Q4_0, 32 elements / 18 bytes; encoder matrices and embeddings only; head/norms F32"});
    fs::write(
        dst.join("ultra-q4.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    eprintln!("export complete: {offset} bytes");
    Ok(())
}
