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
const PARAKEET_VOCAB_FILE: &str = "vocab.txt";
const GRANITE_TOKENIZER_FILE: &str = "tokenizer.json";
const NEMOTRON_TOKENS_FILE: &str = "tokens.txt";
const NEMOTRON_FILTERBANK_FILE: &str = "filterbank.bin";

/// Discovery is keyed on the family word at the *top* of a directory name.
/// Every supported model belongs to one of these families, so a new
/// quantisation or point release drops in without touching this file. The
/// directory name only nominates a candidate — [`model_kind`] inspects the
/// contents and has the final say.
const FAMILY_KEYWORDS: &[&str] = &["parakeet", "granite", "nemotron"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKind {
    Parakeet,
    Granite,
    /// Nemotron Speech Streaming EN 0.6B — cache-aware streaming FastConformer
    /// with an RNN-T decoder. Route 4 of `design-long-form-routes.md`: toggled
    /// capture at `Profile::Raw`, because it emits punctuation and casing
    /// natively and a repair pass on it would be all downside.
    Nemotron,
}

impl ModelKind {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Parakeet => "Parakeet TDT",
            Self::Granite => "Granite Speech 5 TurboCTC",
            Self::Nemotron => "Nemotron Speech Streaming EN",
        }
    }

    /// Parakeet sorts first, so an unpinned install keeps resolving to Parakeet
    /// exactly as it did before the tray switcher existed.
    fn rank(self) -> u8 {
        match self {
            Self::Parakeet => 0,
            // Ahead of Granite: it is a dictation engine, and Granite is now
            // principally a `--wav` skimmer.
            Self::Nemotron => 1,
            Self::Granite => 2,
        }
    }
}

/// One usable model directory found on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledModel {
    pub kind: ModelKind,
    pub path: PathBuf,
    /// Tray-facing name, e.g. `Parakeet TDT (int4)`.
    pub label: String,
}

pub fn model_kind(path: &Path) -> Option<ModelKind> {
    // Nemotron first: its directory carries `encoder*` and `decoder*` graphs, so
    // it must be ruled in before the two families whose tests look at those
    // same prefixes. It is distinguished by shipping its own frontend
    // (`filterbank.bin`) and a `tokens.txt` rather than Parakeet's `vocab.txt`.
    if has_nemotron_model(path) {
        Some(ModelKind::Nemotron)
    } else if has_granite_model(path) {
        Some(ModelKind::Granite)
    } else if has_parakeet_model(path) {
        Some(ModelKind::Parakeet)
    } else {
        None
    }
}

pub fn find_model_path(config_path: Option<&str>) -> Option<PathBuf> {
    discover_models(config_path)
        .into_iter()
        .next()
        .map(|model| model.path)
}

/// Every usable model on disk, best-first. An explicit `config_path` override
/// takes the first slot; after that each search directory contributes its
/// finds, in `ModelKind::rank` order. The tray switcher renders this list, and
/// `find_model_path` takes its head.
pub fn discover_models(config_path: Option<&str>) -> Vec<InstalledModel> {
    let mut found: Vec<InstalledModel> = Vec::new();

    if let Some(raw) = config_path {
        let path = PathBuf::from(raw);
        if let Some(kind) = model_kind(&path) {
            push_unique(&mut found, describe(kind, path));
        }
    }

    for base in default_model_dirs() {
        let mut in_base: Vec<InstalledModel> = Vec::new();

        if let Some(kind) = model_kind(&base) {
            in_base.push(describe(kind, base.clone()));
        }

        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.filter_map(|entry| entry.ok()) {
                let path = entry.path();
                if !path.is_dir() || !has_family_keyword(&path) {
                    continue;
                }
                if let Some(kind) = model_kind(&path) {
                    in_base.push(describe(kind, path));
                }
            }
        }

        in_base.sort_by(|left, right| {
            left.kind
                .rank()
                .cmp(&right.kind.rank())
                .then_with(|| left.path.cmp(&right.path))
        });
        for model in in_base {
            push_unique(&mut found, model);
        }
    }

    found
}

fn describe(kind: ModelKind, path: PathBuf) -> InstalledModel {
    let label = format!("{} ({})", kind.display_name(), model_variant(kind, &path));
    InstalledModel { kind, path, label }
}

fn push_unique(found: &mut Vec<InstalledModel>, model: InstalledModel) {
    if !found.iter().any(|existing| existing.path == model.path) {
        found.push(model);
    }
}

fn has_family_keyword(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    FAMILY_KEYWORDS
        .iter()
        .any(|keyword| lower.starts_with(keyword))
}

pub fn default_model_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let Some(data_dir) = dirs::data_dir() else {
        return dirs;
    };

    dirs.push(data_dir.join("transcrust").join("models"));

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("parakeet-tdt-0.6b-v3-onnx"));
        dirs.push(cwd.join("granite-speech-5.0-470m-turboctc"));
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
    // Probe the graphs this directory would actually load, not a hardcoded
    // filename list that goes stale on every re-quantisation.
    let mut candidates: Vec<PathBuf> = Vec::new();
    match model_kind(path) {
        Some(ModelKind::Parakeet) => {
            candidates.extend(parakeet_encoder_path(path));
            candidates.extend(parakeet_decoder_path(path));
            candidates.push(path.join(PARAKEET_VOCAB_FILE));
        }
        Some(ModelKind::Granite) => {
            candidates.extend(granite_onnx_path(path));
            candidates.push(path.join(GRANITE_TOKENIZER_FILE));
        }
        Some(ModelKind::Nemotron) => {
            candidates.extend(nemotron_encoder_path(path));
            candidates.extend(nemotron_decoder_path(path));
            candidates.push(path.join(NEMOTRON_TOKENS_FILE));
            // Probed like a graph because gate 3 makes it load-bearing.
            candidates.push(path.join(NEMOTRON_FILTERBANK_FILE));
        }
        None => {}
    }

    let mut results = Vec::new();

    for candidate in candidates {
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

/// Quantisation preference when a directory ships several variants of the same
/// graph: int8 first (the working default), then int4, then unquantised.
fn quant_rank(file_name: &str) -> u8 {
    let lower = file_name.to_ascii_lowercase();
    if lower.contains("int8") {
        0
    } else if lower.contains("int4") {
        1
    } else {
        2
    }
}

fn variant_label(file_name: &str) -> &'static str {
    match quant_rank(file_name) {
        0 => "int8",
        1 => "int4",
        _ => "fp32",
    }
}

/// Every `*.onnx` graph directly inside `dir`. External-weight sidecars
/// (`*.onnx.data`) are not graphs and never match.
fn onnx_files(dir: &Path) -> Vec<PathBuf> {
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
fn pick_onnx(dir: &Path, matches: impl Fn(&str) -> bool) -> Option<PathBuf> {
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

fn file_name_of(path: &Path) -> &str {
    path.file_name().and_then(|name| name.to_str()).unwrap_or("")
}

/// Quantisation of the graph that would actually be loaded from `path`.
fn model_variant(kind: ModelKind, path: &Path) -> &'static str {
    let graph = match kind {
        ModelKind::Parakeet => parakeet_encoder_path(path),
        ModelKind::Granite => granite_onnx_path(path),
        ModelKind::Nemotron => nemotron_encoder_path(path),
    };
    // Nemotron exports put the precision in the *directory* name
    // (`fp32/`, `int8-dynamic/`) and leave the graph called plain
    // `encoder_model.onnx`, so keying on the filename alone labels an int8
    // build "fp32". Fall back to the directory when the file is silent.
    if kind == ModelKind::Nemotron {
        if let Some(dir) = path.file_name().and_then(|n| n.to_str()) {
            let from_dir = variant_label(dir);
            if from_dir != "fp32" {
                return from_dir;
            }
        }
    }
    graph
        .as_deref()
        .map(|graph| variant_label(file_name_of(graph)))
        .unwrap_or("unknown")
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

pub fn has_granite_model(path: &Path) -> bool {
    granite_onnx_path(path).is_some() && path.join(GRANITE_TOKENIZER_FILE).is_file()
}

/// Granite ships a single graph per directory, so take any `*.onnx` that is not
/// half of a Parakeet encoder/decoder pair. Names are not pinned to a point
/// release: a re-export or a fresh quantisation is found without a code change.
pub fn granite_onnx_path(path: &Path) -> Option<PathBuf> {
    pick_onnx(path, |name| {
        !name.starts_with("encoder") && !name.starts_with("decoder")
    })
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

pub fn preferred_int8_model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("transcrust")
        .join("models")
        .join(DEFAULT_PARAKEET_INT8_DIR)
}

pub fn required_int8_files() -> &'static [&'static str] {
    &[
        "encoder*.onnx (int8 preferred, then int4, then fp32)",
        "decoder_joint*.onnx",
        "vocab.txt",
    ]
}

pub fn required_granite_files() -> &'static [&'static str] {
    &[
        "*.onnx, excluding encoder*/decoder* (int8 preferred, then int4, then fp32)",
        "tokenizer.json",
    ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A scratch directory that removes itself. Avoids a dev-dependency for the
    /// handful of on-disk layout cases below.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "transcrust-model-test-{}-{unique}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("failed to create temp dir");
            Self(path)
        }

        fn dir(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::create_dir_all(&path).expect("failed to create model dir");
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn touch(dir: &Path, names: &[&str]) {
        for name in names {
            std::fs::write(dir.join(name), b"").expect("failed to write fixture file");
        }
    }

    #[test]
    fn parakeet_detected_across_quantisations() {
        let temp = TempDir::new();
        for encoder in [
            "encoder-model.int8.onnx",
            "encoder-model.int4.onnx",
            // fp32, the layout `--download-model parakeet-tdt-0.6b` produces.
            "encoder-model.onnx",
            // The older flat naming the model cache may still hold.
            "encoder-int8.onnx",
        ] {
            let dir = temp.dir(encoder);
            touch(&dir, &[encoder, "decoder_joint-model.int8.onnx", "vocab.txt"]);
            assert_eq!(
                model_kind(&dir),
                Some(ModelKind::Parakeet),
                "expected {encoder} to be recognised as Parakeet"
            );
        }
    }

    #[test]
    fn granite_detected_without_pinning_the_release_name() {
        let temp = TempDir::new();
        let dir = temp.dir("granite-whatever-comes-next");
        touch(&dir, &["some-future-export.onnx", "tokenizer.json"]);
        assert_eq!(model_kind(&dir), Some(ModelKind::Granite));
        assert_eq!(
            granite_onnx_path(&dir).as_deref().map(file_name_of),
            Some("some-future-export.onnx")
        );
    }

    #[test]
    fn greedy_pick_prefers_int8_then_int4_then_fp32() {
        let temp = TempDir::new();
        let dir = temp.dir("granite-speech-5.0-470m-turboctc");
        touch(&dir, &["g.onnx", "tokenizer.json"]);
        assert_eq!(model_variant(ModelKind::Granite, &dir), "fp32");

        touch(&dir, &["g.int4.onnx"]);
        assert_eq!(model_variant(ModelKind::Granite, &dir), "int4");

        touch(&dir, &["g.int8.onnx"]);
        assert_eq!(model_variant(ModelKind::Granite, &dir), "int8");
        assert_eq!(
            granite_onnx_path(&dir).as_deref().map(file_name_of),
            Some("g.int8.onnx")
        );
    }

    #[test]
    fn external_weight_sidecars_are_not_graphs() {
        let temp = TempDir::new();
        let dir = temp.dir("parakeet-fp32");
        touch(
            &dir,
            &[
                "encoder-model.onnx",
                "encoder-model.onnx.data",
                "decoder_joint-model.onnx",
                "vocab.txt",
            ],
        );
        assert_eq!(
            parakeet_encoder_path(&dir).as_deref().map(file_name_of),
            Some("encoder-model.onnx")
        );
    }

    #[test]
    fn incomplete_directories_are_not_models() {
        let temp = TempDir::new();
        let no_tokenizer = temp.dir("granite-no-tokenizer");
        touch(&no_tokenizer, &["granite.int8.onnx"]);
        assert_eq!(model_kind(&no_tokenizer), None);

        let no_vocab = temp.dir("parakeet-no-vocab");
        touch(
            &no_vocab,
            &["encoder-model.int8.onnx", "decoder_joint-model.int8.onnx"],
        );
        assert_eq!(model_kind(&no_vocab), None);
    }

    #[test]
    fn family_keyword_nominates_candidates() {
        let temp = TempDir::new();
        assert!(has_family_keyword(&temp.dir("parakeet-tdt-0.6b-v3-int4")));
        assert!(has_family_keyword(&temp.dir("Granite-Speech-5.0")));
        assert!(!has_family_keyword(&temp.dir("whisper-large-v3")));
        // The keyword has to lead: this is not a Granite directory.
        assert!(!has_family_keyword(&temp.dir("my-granite-backup")));
    }

    /// Parakeet sorts ahead of Granite so an unpinned install keeps
    /// resolving to Parakeet.
    #[test]
    fn parakeet_sorts_ahead_of_granite() {
        assert!(ModelKind::Parakeet.rank() < ModelKind::Granite.rank());
    }
}
