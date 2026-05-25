use std::path::{Path, PathBuf};

const PARAKEET_MODELS_BASE: &str =
    "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main";
const PARAKEET_INT8_MODELS_BASE: &str =
    "https://huggingface.co/nasedkinpv/parakeet-tdt-0.6b-v3-onnx-int8/resolve/main";
const PARAKEET_INT4_MODELS_BASE: &str =
    "https://huggingface.co/efederici/parakeet-tdt-0.6b-v3-onnx-int4/resolve/main";

const PARAKEET_MODELS: &[(&str, &str)] = &[
    ("tdt-0.6b", "parakeet-tdt-0.6b-v3"),
    ("tdt-0.6b-int8", "parakeet-tdt-0.6b-v3-int8"),
    ("tdt-0.6b-int4", "parakeet-tdt-0.6b-v3-int4"),
];

const PARAKEET_TDT_FILES: &[&str] = &[
    "encoder-model.onnx",
    "encoder-model.onnx.data",
    "decoder_joint-model.onnx",
    "vocab.txt",
];

const PARAKEET_TDT_INT8_FILES: &[(&str, &str)] = &[
    ("encoder-model.int8.onnx", "encoder-model.int8.onnx"),
    ("decoder_joint-model.int8.onnx", "decoder_joint-model.int8.onnx"),
    ("vocab.txt", "vocab.txt"),
];

const PARAKEET_TDT_INT4_FILES: &[(&str, &str)] = &[
    ("encoder-model.int4.onnx", "encoder-model.int4.onnx"),
    ("decoder_joint-model.int8.onnx", "decoder_joint-model.int8.onnx"),
    ("vocab.txt", "vocab.txt"),
];

const DEFAULT_PARAKEET_INT8_DIR: &str = "parakeet-tdt-0.6b-v3-int8";
const DEFAULT_PARAKEET_INT4_DIR: &str = "parakeet-tdt-0.6b-v3-int4";

pub fn find_model_path(config_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = config_path {
        let path = PathBuf::from(p);
        if path.exists() && has_parakeet_model(&path) {
            return Some(path);
        }
    }

    for dir in default_model_dirs() {
        if let Some(path) = find_parakeet_model_in(&dir) {
            return Some(path);
        }
    }

    None
}

pub fn default_model_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let Some(data_dir) = dirs::data_dir() else {
        return dirs;
    };

    dirs.push(data_dir.join("transcrust").join("models"));

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("parakeet-tdt-0.6b-v3-onnx"));
    }

    dirs
}

pub fn explain_search_paths() -> Vec<String> {
    default_model_dirs()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect()
}

pub fn probe_model_files(path: &Path) -> Vec<String> {
    let candidates = [
        "vocab.txt",
        "encoder-model.int8.onnx",
        "encoder-model.int4.onnx",
        "decoder_joint-model.int8.onnx",
        "config.json",
    ];

    let mut results = Vec::new();

    for name in candidates {
        let candidate = path.join(name);
        if !candidate.exists() {
            continue;
        }

        match std::fs::File::open(&candidate) {
            Ok(mut file) => {
                use std::io::Read;

                let mut buf = [0u8; 16];
                match file.read(&mut buf) {
                    Ok(bytes) => {
                        let size = candidate.metadata().map(|m| m.len()).unwrap_or(0);
                        results.push(format!(
                            "readable: {} ({} bytes, read {} bytes)",
                            candidate.display(),
                            size,
                            bytes
                        ));
                    }
                    Err(e) => {
                        results.push(format!("unreadable: {} ({e})", candidate.display()));
                    }
                }
            }
            Err(e) => {
                results.push(format!("unopenable: {} ({e})", candidate.display()));
            }
        }
    }

    results
}

pub fn has_parakeet_model(path: &Path) -> bool {
    let has_encoder = [
        "encoder-model.int8.onnx",
        "encoder-model.int4.onnx",
        "encoder-int8.onnx",
    ]
    .iter()
    .any(|name| path.join(name).is_file());
    let has_decoder = ["decoder_joint-model.int8.onnx", "decoder_joint-int8.onnx"]
        .iter()
        .any(|name| path.join(name).is_file());
    let has_vocab = path.join("vocab.txt").is_file();

    has_encoder && has_decoder && has_vocab
}

pub fn preferred_int8_model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("transcrust")
        .join("models")
        .join(DEFAULT_PARAKEET_INT8_DIR)
}

pub fn required_int8_files() -> &'static [&'static str] {
    &[
        "encoder-model.int8.onnx or encoder-model.int4.onnx or encoder-int8.onnx",
        "decoder_joint-model.int8.onnx or decoder_joint-int8.onnx",
        "vocab.txt",
    ]
}

fn find_parakeet_model_in(base_dir: &Path) -> Option<PathBuf> {
    if base_dir.is_dir() && has_parakeet_model(base_dir) {
        return Some(base_dir.to_path_buf());
    }

    for dir_name in [DEFAULT_PARAKEET_INT8_DIR, DEFAULT_PARAKEET_INT4_DIR] {
        let path = base_dir.join(dir_name);
        if path.is_dir() && has_parakeet_model(&path) {
            return Some(path);
        }
    }

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .map(|name| {
                        name.starts_with("parakeet")
                            && (name.contains("int8") || name.contains("int4"))
                    })
                    .unwrap_or(false)
                && has_parakeet_model(&path)
            {
                return Some(path);
            }
        }
    }

    None
}

pub async fn download_model(
    model_name: Option<&str>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let name = model_name.unwrap_or("parakeet-tdt-0.6b-int8");

    let target = dirs::data_dir()
        .expect("No XDG data directory")
        .join("transcrust")
        .join("models");
    std::fs::create_dir_all(&target)?;

    let parakeet_name = name.strip_prefix("parakeet-").unwrap_or(name);
    let (_, dir_name) = PARAKEET_MODELS
        .iter()
        .find(|(n, _)| *n == parakeet_name)
        .ok_or_else(|| {
            format!(
                "Unknown parakeet model: {parakeet_name}. Available: {}",
                PARAKEET_MODELS
                    .iter()
                    .map(|(n, _)| format!("parakeet-{n}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let dest = target.join(dir_name);

    if dest.is_dir() && has_parakeet_model(&dest) {
        return Ok(dest);
    }

    std::fs::create_dir_all(&dest)?;

    if parakeet_name.ends_with("-int4") {
        download_pair_list(&dest, PARAKEET_INT4_MODELS_BASE, PARAKEET_TDT_INT4_FILES).await?;
    } else if parakeet_name.ends_with("-int8") {
        download_pair_list(&dest, PARAKEET_INT8_MODELS_BASE, PARAKEET_TDT_INT8_FILES).await?;
    } else {
        for file_name in PARAKEET_TDT_FILES {
            let file_dest = dest.join(file_name);
            if file_dest.is_file() {
                continue;
            }
            let url = format!("{PARAKEET_MODELS_BASE}/{file_name}");

            let status = tokio::process::Command::new("curl")
                .args(["-L", "--progress-bar", "-o"])
                .arg(&file_dest)
                .arg(&url)
                .status()
                .await?;

            if !status.success() {
                std::fs::remove_file(&file_dest).ok();
                return Err(format!("Download failed for {file_name}").into());
            }
        }
    }

    Ok(dest)
}

async fn download_pair_list(
    dest: &Path,
    base_url: &str,
    files: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    for (remote_name, local_name) in files {
        let file_dest = dest.join(local_name);
        if file_dest.is_file() {
            continue;
        }
        let url = format!("{base_url}/{remote_name}");

        let status = tokio::process::Command::new("curl")
            .args(["-L", "--progress-bar", "-o"])
            .arg(&file_dest)
            .arg(&url)
            .status()
            .await?;

        if !status.success() {
            std::fs::remove_file(&file_dest).ok();
            return Err(format!("Download failed for {local_name}").into());
        }
    }
    Ok(())
}
