//! Opt-in local vocabulary adjudication. Candidate offsets belong to the original
//! cleaned transcript; the resident native model returns bounded choices in one batch.
pub mod chain;
pub mod d1;
#[allow(dead_code)]
pub mod decision_tensor;
pub mod grammar;
pub mod split;
#[allow(dead_code)]
pub mod ultra;
use ort::session::RunOptions;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use transcrust_core::{vocabulary, Events};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Backend {
    #[default]
    Ultra,
    D1,
}
impl Backend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ultra => "UltraBERT INT4",
            Self::D1 => "D1 Q8",
        }
    }
    pub fn arguments(args: Vec<String>) -> Result<(Vec<String>, Self, bool), String> {
        let mut out = Vec::new();
        let mut backend = Self::default();
        let mut selected = false;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if arg == "--dec-backend" {
                if selected {
                    return Err("--dec-backend specified twice".into());
                }
                backend = match args.next().as_deref() {
                    Some("ultra") => Self::Ultra,
                    Some("d1") => Self::D1,
                    _ => return Err("--dec-backend requires ultra or d1".into()),
                };
                selected = true;
            } else {
                out.push(arg);
            }
        }
        Ok((out, backend, selected))
    }
}
enum Worker {
    Ultra(ultra::Ultra),
    D1(d1::Model),
}
impl Worker {
    #[cfg(test)]
    fn load(backend: Backend) -> Result<Self, String> {
        let path = model_path(backend);
        Self::load_at(backend, &path)
    }
    fn load_at(backend: Backend, path: &std::path::Path) -> Result<Self, String> {
        match backend {
            Backend::Ultra => {
                if !path.join("ultra-q4.json").is_file() {
                    return Err(format!("UltraBERT INT4 bundle missing at {}; set TRANSCRUST_DEC_MODEL or export with tools/schema-probe/int4.sh",path.display()));
                }
                if crate::decision_tensor::file_hash(&path.join("tokenizer.json"))
                    .map_err(|e| e.to_string())?
                    != "0bf32c50efb4e2307f75c357231c4cfa3802e484da742621b7eab4be60ee568a"
                {
                    return Err("UltraBERT tokenizer checksum mismatch".into());
                }
                Ok(Self::Ultra(
                    ultra::Ultra::load(&path).map_err(|e| e.to_string())?,
                ))
            }
            Backend::D1 => Ok(Self::D1(d1::Model::load(&path)?)),
        }
    }
    fn infer(
        &mut self,
        questions: &[d1::Question],
        options: &RunOptions,
        cancel: &AtomicBool,
    ) -> Result<(Vec<d1::Score>, f64), String> {
        match self {
            Self::D1(m) => m.infer(questions, options),
            Self::Ultra(m) => {
                let start = Instant::now();
                let requests = questions
                    .iter()
                    .map(|q| (q.state.clone(), q.instructions.clone(), q.options.to_vec()))
                    .collect::<Vec<_>>();
                let counts = requests
                    .iter()
                    .map(|(s, i, o)| m.tokens(s, i, o).map(|t| t.len()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;
                let p = m
                    .batch_cancellable(&requests, cancel)
                    .map_err(|e| e.to_string())?;
                let scores = p
                    .into_iter()
                    .zip(counts)
                    .map(|(p, token_count)| {
                        let probabilities: [f64; 3] = p
                            .try_into()
                            .map_err(|_| "UltraBERT decision width mismatch")?;
                        Ok(d1::Score {
                            probabilities,
                            token_count,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok((scores, start.elapsed().as_secs_f64() * 1000.))
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Term {
    canonical: String,
    description: String,
    ordinary: String,
    heard: Vec<String>,
    homophones: Vec<String>,
    distinct: Vec<String>,
    kind: String,
}

fn parse(text: &str) -> Vec<Term> {
    let mut terms = Vec::new();
    let mut current: Option<Term> = None;
    for line in text.lines().chain(std::iter::once("# end")) {
        if line.starts_with('#') {
            if let Some(term) = current.take() {
                if !term.canonical.is_empty() && !matches!(term.kind.as_str(), "habit" | "task") {
                    terms.push(term);
                }
            }
            if let Some(name) = line.strip_prefix("## ") {
                current = Some(Term {
                    canonical: name.trim().into(),
                    ..Default::default()
                });
            }
        } else if let Some(term) = &mut current {
            let line = line.trim().trim_start_matches(['-', '*']).trim();
            if let Some(value) = line.strip_prefix("description:") {
                term.description = value.trim().into();
            } else if let Some(value) = line.strip_prefix("ordinary:") {
                term.ordinary = value.trim().into();
            } else if let Some((value, spellings)) = line
                .strip_prefix("heard:")
                .map(|value| (value, &mut term.heard))
                .or_else(|| {
                    line.strip_prefix("homophones:")
                        .map(|value| (value, &mut term.homophones))
                })
                .or_else(|| {
                    line.strip_prefix("distinct:")
                        .map(|value| (value, &mut term.distinct))
                })
            {
                spellings.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                );
            } else if let Some(value) = line.strip_prefix("kind:") {
                term.kind = value.trim().to_lowercase();
            }
        }
    }
    terms
}

pub fn vocab_path() -> PathBuf {
    vocabulary::path()
}

fn vocabulary_at(path: &std::path::Path) -> Result<Vec<Term>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut terms = parse(&text);
    for term in &mut terms {
        if term.description.is_empty() {
            term.description = format!("the user's vocabulary term {}", term.canonical);
        }
        if term.ordinary.is_empty() {
            term.ordinary = "ordinary English or a different referent, rather than the supplied vocabulary name".into();
        }
        for spellings in [&mut term.heard, &mut term.homophones, &mut term.distinct] {
            spellings.retain(|s| s.chars().filter(|c| c.is_alphanumeric()).count() >= 2);
        }
    }
    Ok(terms)
}

#[derive(Clone, Debug, Serialize)]
struct Candidate {
    start: usize,
    end: usize,
    heard: String,
    canonical: String,
    state: String,
    question: Value,
    /// The «»-marked current sentence and the glossary description, for the rule chain.
    #[serde(skip)]
    marked: String,
    #[serde(skip)]
    description: String,
    #[serde(skip)]
    ordinary_spelling: Option<String>,
    #[serde(skip)]
    distinct: bool,
    /// A sounds-like candidate (`grammar.rs`): `Some(Some(reason))` the grammar licenses
    /// the name, `Some(None)` it doesn't (M10: keep). Never sent to the model.
    #[serde(skip)]
    sounds_like: Option<Option<String>>,
}

// The current sentence only (John, 2026-11-18). A neighbouring sentence made every
// model name an ordinary "lightning" after a Cars sentence; see
// decision-model-results.md, "Rules first on real audio".
// Offsets are UTF-8 byte offsets, so applying edits never splits a code point.
fn context_range(text: &str, start: usize, end: usize) -> (usize, usize) {
    let mut spans = Vec::new();
    let mut from = 0;
    for (index, ch) in text.char_indices() {
        if !matches!(ch, '.' | '?' | '!' | '\n') {
            continue;
        }
        if ch == '.' {
            let word = text[..index]
                .split_whitespace()
                .next_back()
                .unwrap_or("")
                .to_lowercase();
            if ["mr", "mrs", "ms", "dr", "jr", "sr", "st", "prof"].contains(&word.as_str()) {
                continue;
            }
            if text[..index]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_digit())
                && text[index + 1..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
            {
                continue;
            }
        }
        let to = index + ch.len_utf8();
        if !text[from..to].trim().is_empty() {
            spans.push((from, to));
        }
        from = to;
    }
    if from < text.len() {
        spans.push((from, text.len()));
    }
    spans
        .into_iter()
        .find(|&(a, b)| a <= start && b >= end)
        .unwrap_or((0, text.len()))
}

#[cfg(test)]
fn candidates(text: &str, terms: &[Term]) -> Result<Vec<Candidate>, String> {
    candidates_for(text, terms, Backend::Ultra)
}
fn candidates_for(text: &str, terms: &[Term], backend: Backend) -> Result<Vec<Candidate>, String> {
    let mut glossary = "Vocabulary glossary:\n| Term | Description |\n|---|---|\n".to_string();
    for term in terms {
        glossary.push_str(&format!(
            "| {} | {} |\n",
            term.canonical.replace('|', "\\|"),
            term.description.replace('|', "\\|")
        ));
    }
    let mut found = Vec::new();
    for term in terms {
        let mut spellings: Vec<_> = term
            .heard
            .iter()
            .chain(&term.homophones)
            .chain(&term.distinct)
            .collect();
        spellings.push(&term.canonical);
        spellings.sort_by_key(|s| std::cmp::Reverse(s.len()));
        spellings.dedup();
        let pattern = spellings
            .iter()
            .map(|s| {
                s.split_whitespace()
                    .map(regex::escape)
                    .collect::<Vec<_>>()
                    .join(r"[\p{Zs}\t]+")
            })
            .collect::<Vec<_>>()
            .join("|");
        let matcher = RegexBuilder::new(&format!(r"\b(?:{pattern})\b"))
            .case_insensitive(true)
            .build()
            .map_err(|e| e.to_string())?;
        for hit in matcher.find_iter(text) {
            // Preserve ordinary sentence capitalization, and reject legacy
            // grammar aliases such as and→in without mutating the glossary.
            if (term
                .canonical
                .chars()
                .next()
                .is_some_and(char::is_lowercase)
                && hit.as_str().eq_ignore_ascii_case(&term.canonical))
                || transcrust_core::correction::grammar_rewrite(
                    &transcrust_core::correction::Pair {
                        heard: hit.as_str().into(),
                        canonical: term.canonical.clone(),
                    },
                )
            {
                continue;
            }
            let (a, b) = context_range(text, hit.start(), hit.end());
            let marked = format!(
                "{}«{}»{}",
                &text[a..hit.start()],
                hit.as_str(),
                &text[hit.end()..b]
            );
            let state = match backend {
                Backend::Ultra => marked.trim().to_string(),
                Backend::D1 => format!("{glossary}\nDictation:\n{}", marked.trim()),
            };
            // Only restore a declared ordinary spelling for a single canonical word.
            // Preserve sentence-initial capitalization when the name appeared there.
            let ordinary_spelling = match term.homophones.as_slice() {
                [word]
                    if hit.as_str() == term.canonical
                        && !term.canonical.contains(char::is_whitespace)
                        && !word.contains(char::is_whitespace) =>
                {
                    Some(
                        if !text[a..hit.start()].chars().any(char::is_alphanumeric) {
                            let mut chars = word.chars();
                            chars.next().unwrap().to_uppercase().chain(chars).collect()
                        } else {
                            word.clone()
                        },
                    )
                }
                _ => None,
            };
            let distinct = term.distinct.iter().any(|spelling| {
                spelling
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .eq(hit.as_str().split_whitespace().map(str::to_lowercase))
            });
            found.push(Candidate {
                start: hit.start(),
                end: hit.end(),
                heard: hit.as_str().into(),
                canonical: term.canonical.clone(),
                state,
                ordinary_spelling,
                distinct,
                sounds_like: None,
                marked: marked.trim().to_string(),
                description: term.description.clone(),
                question: json!({
                    "type": "choice",
                    "instructions": "What does the marked span refer to in this spoken sentence?",
                    "criteria": {
                        "keep": format!("It is {}", term.ordinary),
                        "name": format!("It is {}, {}", term.canonical, term.description),
                        "abstain": "insufficient evidence"
                    }
                }),
            });
        }
    }
    // Sounds-like candidates where no listed spelling matched: grammar decides once.
    let names: Vec<String> = terms.iter().map(|t| t.canonical.clone()).collect();
    for (a, b, canonical) in grammar::sounds_like_spans(text, &names) {
        if found.iter().any(|c| a < c.end && c.start < b) {
            continue;
        }
        let Some(term) = terms.iter().find(|t| t.canonical == canonical) else { continue };
        let (s, e) = context_range(text, a, b);
        let lower_words = |t: &str| -> Vec<String> {
            t.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '’'))
                .filter(|w| !w.is_empty())
                .map(str::to_lowercase)
                .collect()
        };
        let heard = &text[a..b];
        let verdict = grammar::licenses_name(heard, &lower_words(&text[s..a]), &lower_words(&text[b..e]), &names, &canonical);
        // M10: without a licence the transcript stays, so there is nothing to record.
        if verdict.is_none() {
            continue;
        }
        let marked = format!("{}«{}»{}", &text[s..a], heard, &text[b..e]);
        found.push(Candidate {
            start: a,
            end: b,
            heard: heard.into(),
            canonical: canonical.clone(),
            state: marked.trim().to_string(),
            ordinary_spelling: None,
            distinct: false,
            sounds_like: Some(verdict),
            marked: marked.trim().to_string(),
            description: term.description.clone(),
            question: Value::Null,
        });
    }
    // A spelled-out canonical phrase takes precedence over a word inside it,
    // such as "McQueen" inside "Lightning McQueen".
    let found: Vec<_> = found
        .iter()
        .filter(|shorter| {
            !found.iter().any(|phrase| {
                phrase.start <= shorter.start
                    && phrase.end >= shorter.end
                    && (phrase.start != shorter.start || phrase.end != shorter.end)
                    && phrase.heard.eq_ignore_ascii_case(&phrase.canonical)
            })
        })
        .collect();
    // Other overlapping alternatives are ambiguous.
    let mut unique: Vec<Candidate> = found
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            !found
                .iter()
                .enumerate()
                .any(|(j, b)| i != &j && a.start < b.end && b.start < a.end)
        })
        .map(|(_, c)| (*c).clone())
        .collect();
    unique.sort_by_key(|c| c.start);
    Ok(unique)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Choice {
    Keep,
    Name,
    Abstain,
}
impl Choice {
    fn label(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Name => "name",
            Self::Abstain => "abstain",
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
struct Verdict {
    choice: Choice,
    probabilities: [f64; 3],
    token_count: usize,
    latency_ms: f64,
}

/// Who decided one occurrence, and why. `source` is the rule name, "prior" or "model".
#[derive(Clone, Debug, Serialize)]
struct Routed {
    choice: Option<Choice>,
    source: String,
    reason: String,
}
/// Run the deterministic chain on every occurrence. `choice: None` means Pass: the model
/// decides. Name/Keep are final; Prior is left as heard (Keep) until a per-speaker prior exists.
fn route(candidates: &[Candidate]) -> Vec<Routed> {
    candidates
        .iter()
        .map(|c| {
            if let Some(verdict) = &c.sounds_like {
                return match verdict {
                    Some(reason) => Routed {
                        choice: Some(Choice::Name),
                        source: "sounds-like + grammar".into(),
                        reason: reason.clone(),
                    },
                    None => Routed {
                        choice: Some(Choice::Keep),
                        source: "sounds-like, undetermined".into(),
                        reason: format!(
                            "{:?} shares consonants with {:?}, but nothing in the grammar licenses the name; kept as heard",
                            c.heard, c.canonical
                        ),
                    },
                };
            }
            if c.distinct {
                return Routed {
                    choice: Some(Choice::Name),
                    source: "distinct alias".into(),
                    reason: format!(
                        "the glossary lists {:?} as a distinct spelling of {:?}",
                        c.heard, c.canonical
                    ),
                };
            }
            let d = chain::decide(&c.marked, &c.canonical, &c.description);
            let (choice, source) = match d.label {
                chain::Label::Name => (Some(Choice::Name), d.rule.unwrap_or("rule").to_string()),
                chain::Label::Keep => (Some(Choice::Keep), d.rule.unwrap_or("rule").to_string()),
                chain::Label::Prior => (Some(Choice::Keep), "prior".to_string()),
                chain::Label::Pass => (None, "model".to_string()),
            };
            Routed {
                choice,
                source,
                reason: d.reason,
            }
        })
        .collect()
}
/// Indices of the occurrences the model must see.
fn model_bound(routes: &[Routed]) -> Vec<usize> {
    routes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.choice.is_none())
        .map(|(i, _)| i)
        .collect()
}
/// Fold the model's answers (one per model-bound occurrence, in order) into the routes.
fn merge(routes: &mut [Routed], answers: &[Verdict]) -> Result<(), String> {
    let bound = model_bound(routes);
    if bound.len() != answers.len() {
        return Err("decision count mismatch".into());
    }
    for result in answers {
        if result
            .probabilities
            .iter()
            .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
            || (result.probabilities.iter().sum::<f64>() - 1.0).abs() > 0.001
        {
            return Err("invalid decision probabilities".into());
        }
    }
    for (i, verdict) in bound.into_iter().zip(answers) {
        routes[i].choice = Some(verdict.choice);
        routes[i].reason = format!(
            "model: {}, P(name)={:.3}",
            verdict.choice.label(),
            verdict.probabilities[1]
        );
    }
    Ok(())
}

fn replacement<'a>(candidate: &'a Candidate, route: &Routed) -> Option<&'a str> {
    let spelling = match route.choice {
        Some(Choice::Name) => Some(candidate.canonical.as_str()),
        Some(Choice::Keep) if route.source != "prior" => candidate.ordinary_spelling.as_deref(),
        _ => None,
    };
    spelling.filter(|spelling| *spelling != candidate.heard)
}

fn apply_routes(text: &str, candidates: &[Candidate], routes: &[Routed]) -> String {
    let mut out = text.to_string();
    for (candidate, route) in candidates.iter().zip(routes).rev() {
        if let Some(spelling) = replacement(candidate, route) {
            out.replace_range(candidate.start..candidate.end, spelling);
        }
    }
    out
}

pub fn model_path(backend: Backend) -> PathBuf {
    let installed = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join(match backend {
            Backend::Ultra => "transcrust/models/ultrabert-380m-curation-q4",
            Backend::D1 => "transcrust/models/d1-omni-600M-q8",
        });
    std::env::var_os("TRANSCRUST_DEC_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if installed
                .join(match backend {
                    Backend::Ultra => "ultra-q4.json",
                    Backend::D1 => "manifest.json",
                })
                .exists()
            {
                installed
            } else {
                match backend {
                    Backend::Ultra => installed,
                    Backend::D1 => PathBuf::from("/tmp/transcrust-d1-omni-600M-q8"),
                }
            }
        })
}

pub struct Decision {
    model: Arc<Mutex<Option<Worker>>>,
    disabled: AtomicBool,
    backend: Backend,
    vocabulary_path: PathBuf,
}
pub struct Outcome {
    pub text: String,
    pub audit: Value,
}
impl Decision {
    pub async fn start(observer: &dyn Events, backend: Backend) -> Self {
        Self::start_at(observer, backend, model_path(backend), vocab_path()).await
    }
    /// Explicit paths for Android/Apple; no desktop installation assumptions.
    pub async fn start_at(
        observer: &dyn Events,
        backend: Backend,
        model_path: PathBuf,
        vocabulary_path: PathBuf,
    ) -> Self {
        let started = Instant::now();
        let cancel = Arc::new(AtomicBool::new(false));
        let task_cancel = cancel.clone();
        let load = tokio::task::spawn_blocking(move || {
            let mut model = Worker::load_at(backend, &model_path)?;
            model.infer(
                &[d1::Question {
                    state: "Ready.".into(),
                    instructions: "Choose the topic.".into(),
                    options: [
                        "ordinary text".into(),
                        "a name".into(),
                        "insufficient evidence".into(),
                    ],
                }],
                &RunOptions::new().map_err(|e| e.to_string())?,
                &task_cancel,
            )?;
            Ok::<_, String>(model)
        });
        let loaded = match tokio::time::timeout(Duration::from_secs(30), load).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => {
                cancel.store(true, Ordering::Relaxed);
                Err("decision model startup timeout".into())
            }
        };
        let model = match loaded {
            Ok(worker) => {
                observer.phase(
                    "dec",
                    &format!(
                        "{} ready in {:.2}s; batched decisions; shared vocabulary {}",
                        backend.label(),
                        started.elapsed().as_secs_f64(),
                        vocabulary_path.display()
                    ),
                );
                Some(worker)
            }
            Err(e) => {
                observer.error("dec", &format!("{e}; using vocabulary rules until restart"));
                None
            }
        };
        Self {
            model: Arc::new(Mutex::new(model)),
            disabled: AtomicBool::new(false),
            backend,
            vocabulary_path,
        }
    }
    async fn request(
        &self,
        candidates: &[Candidate],
        cancel: Arc<AtomicBool>,
    ) -> Result<Vec<Verdict>, String> {
        if self.disabled.load(Ordering::Relaxed) {
            return Err("decision model unavailable; restart transcrust to retry".into());
        }
        let questions = candidates
            .iter()
            .map(|c| d1::Question {
                state: c.state.clone(),
                instructions: c.question["instructions"].as_str().unwrap().into(),
                options: ["keep", "name", "abstain"]
                    .map(|k| c.question["criteria"][k].as_str().unwrap().into()),
            })
            .collect::<Vec<_>>();
        let options = Arc::new(RunOptions::new().map_err(|e| e.to_string())?);
        let task_options = options.clone();
        let task_cancel = cancel.clone();
        let model = self.model.clone();
        let task = tokio::task::spawn_blocking(move || {
            let mut guard = model.lock().map_err(|e| e.to_string())?;
            let result = guard
                .as_mut()
                .ok_or("decision model unavailable; restart transcrust to retry")?
                .infer(&questions, &task_options, &task_cancel);
            if result.is_err() && !task_cancel.load(Ordering::Relaxed) {
                *guard = None;
            }
            if task_cancel.load(Ordering::Relaxed) {
                Err("decision cancelled".into())
            } else {
                result
            }
        });
        // Known footgun: one fixed deadline covers the entire take's batch.
        // Expiry disables model decisions until reopen, even for healthy long
        // speech. See traverse/decision-worker.md before changing this policy.
        let answer = match tokio::time::timeout(Duration::from_secs(15), task).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => {
                let _ = options.terminate();
                cancel.store(true, Ordering::Relaxed);
                Err(
                    "15s decision budget exceeded; native inference terminated until restart"
                        .into(),
                )
            }
        };
        match answer {
            Ok((scores, batch_ms)) => Ok(scores
                .into_iter()
                .map(|score| {
                    let mut best = 0;
                    for i in 1..3 {
                        if score.probabilities[i] > score.probabilities[best] {
                            best = i;
                        }
                    }
                    Verdict {
                        choice: match best {
                            0 => Choice::Keep,
                            1 => Choice::Name,
                            _ => Choice::Abstain,
                        },
                        probabilities: score.probabilities,
                        token_count: score.token_count,
                        latency_ms: batch_ms,
                    }
                })
                .collect()),
            Err(e) => {
                if e != "decision cancelled" {
                    self.disabled.store(true, Ordering::Relaxed);
                }
                Err(e)
            }
        }
    }
    pub async fn adjudicate(&self, text: &str, observer: &dyn Events) -> Outcome {
        self.adjudicate_with_cancel(text, observer, Arc::new(AtomicBool::new(false)))
            .await
    }
    /// Ultra checks at layer boundaries. D1 may finish its current graph run.
    /// Cancellation discards model answers. Native adapters can discard the entire take.
    pub async fn adjudicate_with_cancel(
        &self,
        text: &str,
        observer: &dyn Events,
        cancel: Arc<AtomicBool>,
    ) -> Outcome {
        let started = Instant::now();
        let input = text;
        let mut jobs = Vec::new();
        let mut results = Vec::new();
        let mut routes: Vec<Routed> = Vec::new();
        let mut error: Option<String> = None;
        let vocabulary = if cancel.load(Ordering::Relaxed) {
            Err("decision cancelled".into())
        } else {
            vocabulary_at(&self.vocabulary_path)
        };
        // Fused registered names are split before matching; candidate offsets in the
        // audit belong to `split_text`.
        let (working, splits) = match &vocabulary {
            Ok(terms) => split::Names::new(terms.iter().map(|t| t.canonical.as_str())).repair(text),
            Err(_) => (text.to_string(), Vec::new()),
        };
        for s in &splits {
            observer.phase("dec.split", &format!("{:?} -> {:?}: {}", s.heard, s.replacement, s.reason));
        }
        let text = working.as_str();
        let work = vocabulary.and_then(|terms| candidates_for(text, &terms, self.backend));
        match work {
            Err(e) => error = Some(e),
            Ok(found) if found.is_empty() => {}
            Ok(found) => {
                jobs = found;
                routes = route(&jobs);
                let bound: Vec<Candidate> = model_bound(&routes)
                    .into_iter()
                    .map(|i| jobs[i].clone())
                    .collect();
                // The model is only invoked when at least one occurrence is Pass.
                let answered = if bound.is_empty() {
                    Ok(Vec::new())
                } else {
                    self.request(&bound, cancel.clone()).await
                };
                match answered {
                    Ok(_) if cancel.load(Ordering::Relaxed) => {
                        error = Some("decision cancelled".into())
                    }
                    Ok(answers) => match merge(&mut routes, &answers) {
                        Ok(()) => results = answers,
                        Err(e) => error = Some(e),
                    },
                    Err(e) => error = Some(e),
                }
            }
        }
        let delivered = apply_routes(text, &jobs, &routes);
        let elapsed = started.elapsed().as_secs_f64();
        if let Some(e) = &error {
            for route in routes.iter_mut().filter(|route| route.choice.is_none()) {
                route.reason = format!("{e}; left as heard");
            }
            observer.error("dec", &format!("{e}; retained rule decisions"));
        }
        observer.phase(
            "dec",
            &format!(
                "{} candidates, {} replacements, {:.0} ms",
                jobs.len(),
                jobs.iter()
                    .zip(&routes)
                    .filter(|(c, r)| replacement(c, r).is_some())
                    .count(),
                elapsed * 1000.0
            ),
        );
        let mut verdicts = results.iter();
        let mut decisions = Vec::new();
        for (candidate, route) in jobs.iter().zip(&routes) {
            let model = if route.source == "model" {
                verdicts.next()
            } else {
                None
            };
            let edit = replacement(candidate, route);
            observer.phase(
                "dec.choice",
                &format!(
                    "{:?} -> {:?}: {}; source {}; {}{}",
                    candidate.heard,
                    edit.unwrap_or(&candidate.heard),
                    route.choice.map_or("none", Choice::label),
                    route.source,
                    route.reason,
                    model.map_or(String::new(), |v| format!("; batch {:.0}ms", v.latency_ms))
                ),
            );
            decisions.push(json!({"heard":candidate.heard,"canonical":candidate.canonical,"choice":route.choice,"source":route.source,"reason":route.reason,"replacement":edit}));
        }
        let audit = json!({"backend":self.backend.label(),"input":input,"split_text":text,"splits":splits,"delivered":delivered,"decision_secs":elapsed,"candidates":jobs,"decisions":decisions,"results":results,"error":error});
        Outcome {
            text: delivered,
            audit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_grammar_aliases_do_not_rewrite_sentence_words_or_capitalization() {
        let terms =
            parse("## in\nheard: and\n## and\nheard: in\n## it\nheard: I\n## Flo\nheard: flow\n");
        for backend in [Backend::Ultra, Backend::D1] {
            let jobs = candidates_for("It is in the flow and I agree.", &terms, backend).unwrap();
            assert_eq!(
                jobs.iter().map(|c| c.heard.as_str()).collect::<Vec<_>>(),
                vec!["flow"]
            );
            assert_eq!(jobs[0].canonical, "Flo");
        }
    }
    fn terms() -> Vec<Term> {
        parse("## Gambol\ndescription: pet dog\nheard: gamble\n## Flo\ndescription: Cars character\nheard: flow\n")
    }
    #[test]
    fn ultra_is_primary_and_selector_does_not_leak_into_fix_text() {
        let input = vec![
            "transcrust",
            "--fix",
            "My dog Gamble.",
            "--dec-backend",
            "d1",
            "--dec",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let (args, backend, selected) = Backend::arguments(input).unwrap();
        assert!(selected);
        assert_eq!(backend, Backend::D1);
        assert_eq!(args, vec!["transcrust", "--fix", "My dog Gamble.", "--dec"]);
        assert_eq!(Backend::default(), Backend::Ultra);
        for args in [
            vec!["app", "--dec-backend"],
            vec!["app", "--dec-backend", "nope"],
        ] {
            assert!(Backend::arguments(args.into_iter().map(String::from).collect()).is_err());
        }
    }
    #[test]
    fn ultra_gets_only_context_but_keeps_the_candidates_description() {
        let text = "My dog Gamble is here. Water has flow.";
        let terms = terms();
        let u = candidates_for(text, &terms, Backend::Ultra).unwrap();
        let d = candidates_for(text, &terms, Backend::D1).unwrap();
        assert_eq!(u.len(), d.len());
        assert_eq!(u[0].state, "My dog «Gamble» is here.");
        assert!(!u[0].state.contains("Vocabulary glossary"));
        assert!(d[0].state.starts_with("Vocabulary glossary:"));
        assert_eq!(u[0].question, d[0].question);
        assert!(u[0].question["criteria"]["name"]
            .as_str()
            .unwrap()
            .contains("pet dog"));
    }
    #[test]
    #[ignore = "requires the local INT4 model; verifies the promoted inference against frozen research outputs"]
    fn ultra_native_matches_research_and_cancels() {
        let cases: Value = serde_json::from_reader(
            std::fs::File::open(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tools/pair-router-probe/ultra-int4-2026-11-01/q4/stripped.json"
            ))
            .unwrap(),
        )
        .unwrap();
        let mut worker = Worker::load(Backend::Ultra).unwrap();
        let run = RunOptions::new().unwrap();
        let cancel = AtomicBool::new(false);
        for source in ["parakeet", "phonon"] {
            for arm in ["single", "two"] {
                let rows: Vec<_> = cases["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["case"]["source"] == source && r["case"]["arm"] == arm)
                    .collect();
                let qs = rows
                    .iter()
                    .map(|r| {
                        let c = &r["case"];
                        d1::Question {
                            state: c["state"].as_str().unwrap().into(),
                            instructions: c["candidate"]["question"]["instructions"]
                                .as_str()
                                .unwrap()
                                .into(),
                            options: ["keep", "name", "abstain"].map(|k| {
                                c["candidate"]["question"]["criteria"][k]
                                    .as_str()
                                    .unwrap()
                                    .into()
                            }),
                        }
                    })
                    .collect::<Vec<_>>();
                let (scores, _) = worker.infer(&qs, &run, &cancel).unwrap();
                for (score, row) in scores.iter().zip(rows) {
                    for i in 0..3 {
                        assert!(
                            (score.probabilities[i] - row["probabilities"][i].as_f64().unwrap())
                                .abs()
                                < 1e-5
                        );
                    }
                }
                cancel.store(true, Ordering::Relaxed);
                assert!(worker
                    .infer(&qs, &run, &cancel)
                    .unwrap_err()
                    .contains("cancelled"));
                cancel.store(false, Ordering::Relaxed);
                if source == "phonon" && arm == "two" {
                    let started = Instant::now();
                    std::thread::scope(|scope| {
                        let flag = &cancel;
                        scope.spawn(move || {
                            std::thread::sleep(Duration::from_millis(25));
                            flag.store(true, Ordering::Relaxed);
                        });
                        assert!(worker
                            .infer(&qs, &run, &cancel)
                            .unwrap_err()
                            .contains("cancelled"));
                    });
                    assert!(
                        started.elapsed() < Duration::from_secs(5),
                        "cancellation did not stop at a layer boundary"
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the installed INT4 bundle; exercises native-adapter cancellation and recovery"]
    fn cancelled_utterance_preserves_text_and_model_for_next_take() {
        let path =
            std::env::temp_dir().join(format!("transcrust-cancel-{}.md", std::process::id()));
        std::fs::write(
            &path,
            "## Gambol\ndescription: pet dog\nheard: gamble\nordinary: a gambling verb\n## Chili\nheard: chili\n",
        )
        .unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let model = rt.block_on(Decision::start_at(
            &transcrust_core::SilentEvents,
            Backend::Ultra,
            model_path(Backend::Ultra),
            path.clone(),
        ));
        let flag = Arc::new(AtomicBool::new(false));
        let signal = flag.clone();
        let cancel = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            signal.store(true, Ordering::Relaxed);
        });
        let input = "My dog Gamble is an Australian shepherd. We made chili for dinner.";
        let cancelled =
            rt.block_on(model.adjudicate_with_cancel(input, &transcrust_core::SilentEvents, flag));
        cancel.join().unwrap();
        assert_eq!(
            cancelled.text,
            "My dog Gambol is an Australian shepherd. We made chili for dinner."
        );
        assert!(cancelled.audit["error"]
            .as_str()
            .unwrap()
            .contains("cancel"));
        let next = rt.block_on(model.adjudicate(
            "My dog Gamble is an Australian shepherd.",
            &transcrust_core::SilentEvents,
        ));
        assert_eq!(next.text, "My dog Gambol is an Australian shepherd.");
        assert!(next.audit["error"].is_null());
        std::fs::remove_file(path).unwrap();
    }
    /// Phone take a137dfff-5c5e-4df3-a87e-9551e4aff16d, Parakeet Ultra whole-take raw
    /// (2026-11-19), household child's name replaced with Rowan.
    const NELLAND_TAKE: &str = "Nelland Rowan took gamble to the park this morning. Gamble chased a ball while Rowan talked about Lightning McQueen. Nell said we should never gamble with the grocery money. We watched the water flow under the bridge, then Rowan asked whether Flo would have lunch ready for meter new paragraph back at home Louis and Bingo were on television. Chili helped the children while Bandit looked for a toy. We made chili for dinner and Nell said the kitchen felt chilly. Later we played bingo at the table. The story had a bandit who stole a bag of coins. New paragraph Rowan chose cars again after dinner. Mader drove to see flow and lightning McQueen waited outside a flash of lightning hit the window. It does matter whether the door is closed when rain comes in a called gamble inside and Rowan put the little tow truck beside his race car.";
    const NELLAND_VOCAB: &str = "## Gambol\ndescription: pet dog\nheard: gamble\nordinary: a gambling verb\n## Nell\n## Rowan\n## Flo\n## Mater\n## Bluey\n## Bingo\n## Chili\n## Bandit\n## Lightning McQueen\n## McQueen\n";

    fn nelland_pipeline(model: PathBuf) -> Outcome {
        let path =
            std::env::temp_dir().join(format!("transcrust-nelland-{}.md", std::process::id()));
        std::fs::write(&path, NELLAND_VOCAB).unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let decision = rt.block_on(Decision::start_at(
            &transcrust_core::SilentEvents,
            Backend::Ultra,
            model,
            path.clone(),
        ));
        let outcome =
            rt.block_on(decision.adjudicate(NELLAND_TAKE, &transcrust_core::SilentEvents));
        std::fs::remove_file(path).unwrap();
        outcome
    }

    fn assert_nelland_split(outcome: &Outcome) {
        assert!(
            outcome.text.starts_with("Nell and Rowan took "),
            "{}",
            outcome.text
        );
        assert!(!outcome.text.contains("Nelland"));
        assert_eq!(outcome.audit["input"], NELLAND_TAKE);
        let splits = outcome.audit["splits"].as_array().unwrap();
        assert_eq!(splits.len(), 1, "{splits:?}");
        assert_eq!(splits[0]["heard"], "Nelland");
        assert_eq!(splits[0]["replacement"], "Nell and");
    }

    /// The whole adjudicate path (vocabulary, split, candidates, rules, apply) with no
    /// model: model-bound occurrences stay as heard, the split still lands.
    #[test]
    fn nelland_take_is_split_through_the_pipeline_without_a_model() {
        let outcome = nelland_pipeline(std::env::temp_dir().join("transcrust-no-model"));
        assert_nelland_split(&outcome);
    }

    #[test]
    #[ignore = "requires the installed INT4 bundle; the Nelland take through the live decision path"]
    fn nelland_take_is_split_through_the_pipeline_with_ultra() {
        let outcome = nelland_pipeline(model_path(Backend::Ultra));
        assert!(outcome.audit["error"].is_null(), "{}", outcome.audit["error"]);
        assert_nelland_split(&outcome);
        println!("delivered: {}", outcome.text);
    }

    /// 2026-11-20 benches: names missed as real words get a sounds-like candidate, and
    /// only grammar can license the name. Everything else stays as heard (M10).
    #[test]
    fn sounds_like_candidates_need_grammar() {
        let terms = parse(
            "## Mater\ndescription: tow truck\n## Bluey\ndescription: blue heeler\n## Bingo\n\
             description: Bluey's sister\n## Todoroki\ndescription: race car\n## Shu Todoroki\nheard: shoe to rookie\n",
        );
        let fixed = |text: &str| {
            let (jobs, routes) = routed(text, &terms);
            for (job, route) in jobs.iter().zip(&routes) {
                assert!(job.sounds_like.is_none() || route.choice.is_some(), "{text}: sounds-like reached the model");
            }
            apply_routes(text, &jobs, &routes)
        };
        // M5: preposition + bare singular count noun.
        assert_eq!(fixed("Rowan asked whether lunch was ready for meter."), "Rowan asked whether lunch was ready for Mater.");
        // Coordination with a registered name (the joiner case).
        assert_eq!(fixed("Bingo and Blue Wee played a game."), "Bingo and Bluey played a game.");
        // Not an English word.
        assert_eq!(fixed("Shu Totoroki raced past them."), "Shu Todoroki raced past them.");
        // M10: grammar licenses the ordinary word, so nothing changes.
        for text in [
            "It does matter whether the door is closed.",
            "The meter was broken.",
            "We read the meter after lunch.",
            "The water ran blue all day.",
            // bench 1, 2026-11-20: "a ball" shares Bluey's skeleton and sits beside
            // "and Rowan", but a determiner makes it a complete noun phrase.
            "Gambol chased a ball and Bingo laughed.",
        ] {
            assert_eq!(fixed(text), text);
        }
    }

    #[test]
    fn targets_canonical_occurrences_without_substrings() {
        let text = "Gambol gambled. I gamble; water flows. The flow is slow.";
        let found = candidates(text, &terms()).unwrap();
        assert_eq!(
            found.iter().map(|c| c.heard.as_str()).collect::<Vec<_>>(),
            vec!["Gambol", "gamble", "flow"]
        );
        assert!(!found[0].state.contains("The flow is slow"));
    }
    #[test]
    fn utf8_offsets_two_sentences_and_independent_replacements() {
        let text = "Éloïse likes my dog Gamble.I gamble with cards. Water has flow.";
        let found = candidates(text, &terms()).unwrap();
        assert_eq!(found.len(), 3);
        // One sentence only: neither neighbour reaches the model.
        assert!(found[1].state.contains("I «gamble» with cards."));
        assert!(!found[1].state.contains("Éloïse"));
        assert!(!found[1].state.contains("Water"));
        let result = |choice| Verdict {
            choice,
            probabilities: [0.1, 0.8, 0.1],
            token_count: 80,
            latency_ms: 1.0,
        };
        let mut routes = vec![
            Routed {
                choice: None,
                source: "model".into(),
                reason: String::new(),
            };
            found.len()
        ];
        merge(
            &mut routes,
            &[
                result(Choice::Name),
                result(Choice::Keep),
                result(Choice::Abstain),
            ],
        )
        .unwrap();
        assert_eq!(
            apply_routes(text, &found, &routes),
            "Éloïse likes my dog Gambol.I gamble with cards. Water has flow."
        );
    }
    #[test]
    fn overlap_is_not_arbitrarily_resolved_and_initial_is_not_a_candidate() {
        let mut terms = terms();
        terms.push(Term {
            canonical: "Gambleton".into(),
            heard: vec!["gamble".into()],
            ..Default::default()
        });
        terms.push(Term {
            canonical: "Rowan".into(),
            heard: vec!["Rowen".into()],
            ..Default::default()
        });
        assert!(candidates("I gamble. My son O is here.", &terms)
            .unwrap()
            .is_empty());
    }
    #[test]
    fn reload_parser_preserves_descriptions_and_excludes_tasks() {
        let terms=parse("## Rowan\ndescription: son\nordinary: other person\nheard: Rowen, Roan\n## Cigs\nkind: habit\nheard: six\n");
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].heard, vec!["Rowen", "Roan"]);
        assert_eq!(terms[0].description, "son");
    }
    #[test]
    fn shared_file_edits_and_deletions_take_effect_without_starter_fallback() {
        let path =
            std::env::temp_dir().join(format!("transcrust-dec-reload-{}.md", std::process::id()));
        std::fs::write(&path, "## Gambol\nkind: person\ndescription: pet dog\nheard: gamble\n## Cigs\nkind: habit\nheard: six\n").unwrap();
        assert_eq!(
            candidates("I gamble. Set six.", &vocabulary_at(&path).unwrap())
                .unwrap()
                .len(),
            1
        );
        std::fs::write(&path, "## Gambol\nkind: person\ndescription: pet dog\n").unwrap();
        assert!(candidates("I gamble.", &vocabulary_at(&path).unwrap())
            .unwrap()
            .is_empty());
        std::fs::write(&path, "").unwrap();
        assert!(vocabulary_at(&path).unwrap().is_empty());
        std::fs::remove_file(&path).unwrap();
        assert!(vocabulary_at(&path).is_err());
    }

    #[test]
    fn abbreviations_and_decimals_stay_in_the_sentence() {
        let text = "Dr. Smith saw 2.5 dogs and I gamble. Then we leave.";
        let found = candidates(text, &terms()).unwrap();
        assert!(found[0]
            .state
            .contains("Dr. Smith saw 2.5 dogs and I «gamble»."));
        assert!(!found[0].state.contains("Then we leave"));
    }
    #[test]
    fn phrase_alias_preserves_explicit_line_breaks() {
        let terms = parse("## transcrust\ndescription: software project\nheard: trans crust\n");
        assert_eq!(candidates("Use trans\tcrust.", &terms).unwrap().len(), 1);
        assert!(candidates("Use trans\ncrust.", &terms).unwrap().is_empty());
    }

    #[test]
    fn incomplete_or_invalid_reply_cannot_partially_edit_text() {
        let text = "My dog Gamble barked. We made chili for dinner.";
        let mut terms = terms();
        terms.extend(parse("## Chili\nheard: chili"));
        let (jobs, mut routes) = routed(text, &terms);
        assert_eq!(model_bound(&routes), vec![1]);
        assert!(merge(&mut routes, &[]).is_err());
        let bad = Verdict {
            choice: Choice::Name,
            probabilities: [f64::NAN, 1.0, 0.0],
            token_count: 1,
            latency_ms: 0.0,
        };
        assert!(merge(&mut routes, &[bad]).is_err());
        assert_eq!(
            apply_routes(text, &jobs, &routes),
            "My dog Gambol barked. We made chili for dinner."
        );
    }

    fn routed(text: &str, terms: &[Term]) -> (Vec<Candidate>, Vec<Routed>) {
        let jobs = candidates(text, terms).unwrap();
        let routes = route(&jobs);
        (jobs, routes)
    }
    #[test]
    fn rules_decide_everything_so_no_model_call_is_needed() {
        let terms = terms();
        let (jobs, routes) = routed("I like to gamble. My dog Gamble barked.", &terms);
        assert_eq!(jobs.len(), 2);
        assert!(model_bound(&routes).is_empty());
        assert_eq!(routes[0].choice, Some(Choice::Keep));
        assert!(routes[0].source.starts_with('2'), "{:?}", routes[0]);
        assert_eq!(routes[1].choice, Some(Choice::Name));
        assert!(!routes[1].reason.is_empty());
        let mut done = routes.clone();
        merge(&mut done, &[]).unwrap();
        assert_eq!(
            apply_routes("I like to gamble. My dog Gamble barked.", &jobs, &done),
            "I like to gamble. My dog Gambol barked."
        );
    }
    #[test]
    fn lone_word_is_left_as_heard_with_prior_reason() {
        let terms = parse("## Bingo\ndescription: Bluey's 4 year old sister\nheard: bingo\n");
        for text in ["Bingo!", "bingo!"] {
            let (jobs, routes) = routed(text, &terms);
            assert_eq!(jobs.len(), 1);
            assert!(model_bound(&routes).is_empty());
            assert_eq!(routes[0].source, "prior");
            assert_eq!(routes[0].choice, Some(Choice::Keep));
            assert!(routes[0].reason.contains("speaker prior"));
            assert_eq!(apply_routes(text, &jobs, &routes), text);
        }
    }

    #[test]
    fn ordinary_homophones_are_candidates_and_can_restore_case() {
        let terms = parse("## Lightning McQueen\ndescription: Cars movie character\nhomophones: lightning\n## McQueen\n## Chili\nhomophones: chili");
        let text = "Lightning McQueen won. The lightning was loud. We made Chili for dinner.";
        let (jobs, mut routes) = routed(text, &terms);
        assert_eq!(
            jobs.iter().map(|c| c.heard.as_str()).collect::<Vec<_>>(),
            ["Lightning McQueen", "lightning", "Chili"]
        );
        assert_eq!(routes[1].choice, Some(Choice::Keep));
        // Rule 0 (2026-11-20): an exact capitalized dictionary spelling mid-sentence is never
        // lowercased. On the bloey corpus the model's lowercasings were 13 wrong, 0 right.
        assert_eq!(routes[2].source, "0 capitalization");
        for route in &mut routes {
            route.choice.get_or_insert(Choice::Keep);
        }
        assert_eq!(
            apply_routes(text, &jobs, &routes),
            "Lightning McQueen won. The lightning was loud. We made Chili for dinner."
        );
    }

    #[test]
    fn distinct_alias_repairs_each_occurrence_and_ignores_unlisted_words() {
        let terms = parse("## Svelte\nkind: term\ndistinct: svelt\nhomophones: svelte");
        let text = "I used svelt for the UI. The svelt compiler worked. I played Atari.";
        let (jobs, routes) = routed(text, &terms);
        assert_eq!(jobs.len(), 2);
        assert!(model_bound(&routes).is_empty());
        assert!(routes.iter().all(|route| route.source == "distinct alias"));
        assert_eq!(
            apply_routes(text, &jobs, &routes),
            "I used Svelte for the UI. The Svelte compiler worked. I played Atari."
        );
    }
    #[test]
    fn pass_goes_to_the_model_batch_with_the_unchanged_state() {
        let terms = parse("## Bingo\ndescription: Bluey's 4 year old sister\nheard: bingo\n## Gambol\ndescription: pet dog\nheard: gamble\n");
        let text = "We saw bingo at the park. I like to gamble.";
        let (jobs, mut routes) = routed(text, &terms);
        assert_eq!(jobs.len(), 2);
        assert_eq!(model_bound(&routes), vec![0]);
        assert_eq!(routes[1].choice, Some(Choice::Keep));
        assert_eq!(jobs[0].state, "We saw «bingo» at the park.");
        let verdict = Verdict {
            choice: Choice::Keep,
            probabilities: [0.9, 0.05, 0.05],
            token_count: 5,
            latency_ms: 1.0,
        };
        assert!(merge(&mut routes.clone(), &[]).is_err());
        merge(&mut routes, &[verdict]).unwrap();
        assert!(routes[0].reason.starts_with("model:"));
    }
}
