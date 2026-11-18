//! Engine graph layouts; platform installation/discovery stays in the CLI.
use std::path::{Path, PathBuf};
#[cfg(test)]
pub(crate) fn test_nemotron_filterbank() -> Option<PathBuf> {
    let root = dirs::data_dir()?.join("transcrust/models");
    std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("nemotron"))
                && has_nemotron_model(p)
        })
        .map(|p| p.join("filterbank.bin"))
}
const PARAKEET_VOCAB_FILE: &str = "vocab.txt";
const NEMOTRON_TOKENS_FILE: &str = "tokens.txt";
const NEMOTRON_FILTERBANK_FILE: &str = "filterbank.bin";

/// Quantisation preference when a directory ships several variants of the same
/// graph: int8 first (the working default), then int4, then unquantised.
pub fn quant_rank(file_name: &str) -> u8 {
    let lower = file_name.to_ascii_lowercase();
    if lower.contains("int8") {
        0
    } else if lower.contains("int4") {
        1
    } else {
        2
    }
}

/// Every `*.onnx` graph directly inside `dir`. External-weight sidecars
/// (`*.onnx.data`) are not graphs and never match.
pub fn onnx_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("onnx")
        })
        .collect()
}

/// Greedily pick the best-quantised graph in `dir` whose file name satisfies
/// `matches`. This is what lets one family cover its int8, int4, and fp32
/// layouts without enumerating every filename combination.
pub fn pick_onnx(dir: &Path, matches: impl Fn(&str) -> bool) -> Option<PathBuf> {
    onnx_files(dir)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(&matches)
        })
        .min_by_key(|path| {
            let name = file_name_of(path).to_string();
            (quant_rank(&name), name)
        })
}

pub fn file_name_of(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
}

pub fn parakeet_encoder_path(path: &Path) -> Option<PathBuf> {
    pick_onnx(path, |name| name.starts_with("encoder"))
}

pub fn parakeet_decoder_path(path: &Path) -> Option<PathBuf> {
    pick_onnx(path, |name| {
        name.starts_with("decoder_joint") || name.starts_with("decoder-joint")
    })
}

pub fn has_parakeet_model(path: &Path) -> bool {
    parakeet_encoder_path(path).is_some()
        && parakeet_decoder_path(path).is_some()
        && path.join(PARAKEET_VOCAB_FILE).is_file()
}

/// Nemotron needs four things, and the two that identify it are the frontend
/// and the tokens file.
///
/// Gate 3 is why `filterbank.bin` is mandatory rather than optional: Parakeet's
/// `nemo128.onnx` applies NeMo's `normalize: per_feature` and Nemotron's config
/// says `normalize: null`. Feeding the wrong mel does not error — it produces
/// fluent, confident, invented English. A directory without its own filterbank
/// is therefore not a usable Nemotron, however many graphs it has.
pub fn has_nemotron_model(path: &Path) -> bool {
    nemotron_encoder_path(path).is_some()
        && nemotron_decoder_path(path).is_some()
        && path.join(NEMOTRON_TOKENS_FILE).is_file()
        && path.join(NEMOTRON_FILTERBANK_FILE).is_file()
}

/// Either export naming convention for the same two graphs.
pub fn nemotron_encoder_path(path: &Path) -> Option<PathBuf> {
    pick_onnx(path, |name| name.starts_with("encoder"))
}

/// The decoder and joint are fused in every Nemotron export seen, whichever
/// of the two names it uses.
pub fn nemotron_decoder_path(path: &Path) -> Option<PathBuf> {
    pick_onnx(path, |name| {
        name.starts_with("decoder_model") || name.starts_with("decoder_joint")
    })
}

pub fn parakeet_direct_graphs(path: &Path) -> Option<ParakeetDirectGraphs> {
    Some(ParakeetDirectGraphs {
        preprocessor: pick_onnx(path, |name| name.starts_with("nemo128"))?,
        encoder: parakeet_encoder_path(path)?,
        decoder_joint: parakeet_decoder_path(path)?,
    })
}

/// Whether this directory can be driven directly for per-token confidence and
/// timestamps, rather than through `parakeet-rs`.
pub fn has_parakeet_direct(path: &Path) -> bool {
    has_parakeet_model(path) && parakeet_direct_graphs(path).is_some()
}

/// The three graphs the direct-drive Parakeet path needs.
///
/// `nemo128.onnx` is NeMo's own preprocessor exported as a graph. It is absent
/// from older installs, which is exactly why this returns `Option`: a directory
/// without it stays a perfectly good `parakeet-rs` model and simply does not
/// offer the direct path.
pub struct ParakeetDirectGraphs {
    pub preprocessor: PathBuf,
    pub encoder: PathBuf,
    pub decoder_joint: PathBuf,
}
