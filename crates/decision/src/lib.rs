//! Opt-in local vocabulary adjudication. Candidate offsets belong to the original
//! cleaned transcript; the resident native model returns bounded choices in one batch.
pub mod d1;
#[allow(dead_code)]
pub mod decision_tensor;
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
use transcrust_core::{dictionary, Events};

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
            } else if let Some(value) = line.strip_prefix("heard:") {
                term.heard.extend(
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
    dictionary::dictionary_path()
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
        term.heard
            .retain(|s| s.chars().filter(|c| c.is_alphanumeric()).count() >= 2);
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
}

// Include the previous sentence, or the following one for the first sentence.
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
    let Some(i) = spans.iter().position(|&(a, b)| a <= start && b >= end) else {
        return (0, text.len());
    };
    if i > 0 {
        (spans[i - 1].0, spans[i].1)
    } else {
        (spans[i].0, spans.get(i + 1).unwrap_or(&spans[i]).1)
    }
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
        let mut spellings = term.heard.clone();
        spellings.push(term.canonical.clone());
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
            if hit.as_str() == term.canonical {
                continue;
            }
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
            found.push(Candidate { start: hit.start(), end: hit.end(), heard: hit.as_str().into(), canonical: term.canonical.clone(), state,
                question: json!({"type":"choice", "instructions":"What does the marked span refer to in this spoken sentence?",
                    "criteria":{"keep":format!("It is {}", term.ordinary), "name":format!("It is {}, {}",term.canonical,term.description), "abstain":"insufficient evidence"}}) });
        }
    }
    // Overlapping alternatives are ambiguous candidate generation, not a model decision.
    let mut unique: Vec<_> = found
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            !found
                .iter()
                .enumerate()
                .any(|(j, b)| i != &j && a.start < b.end && b.start < a.end)
        })
        .map(|(_, c)| c.clone())
        .collect();
    unique.sort_by_key(|c| c.start);
    Ok(unique)
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Choice {
    Keep,
    Name,
    Abstain,
}
#[derive(Debug, Deserialize, Serialize)]
struct Verdict {
    choice: Choice,
    probabilities: [f64; 3],
    token_count: usize,
    latency_ms: f64,
}

fn apply(text: &str, candidates: &[Candidate], results: &[Verdict]) -> Result<String, String> {
    if candidates.len() != results.len() {
        return Err("decision count mismatch".into());
    }
    for result in results {
        if result
            .probabilities
            .iter()
            .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
            || (result.probabilities.iter().sum::<f64>() - 1.0).abs() > 0.001
        {
            return Err("invalid decision probabilities".into());
        }
    }
    let mut out = text.to_string();
    for (candidate, result) in candidates.iter().zip(results).rev() {
        if result.choice == Choice::Name {
            out.replace_range(candidate.start..candidate.end, &candidate.canonical);
        }
    }
    Ok(out)
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
                        "{} ready in {:.2}s; batched decisions; decoder boosting off; shared vocabulary {}",
                        backend.label(),
                        started.elapsed().as_secs_f64(),
                        vocabulary_path.display()
                    ),
                );
                Some(worker)
            }
            Err(e) => {
                observer.error(
                    "dec",
                    &format!("{e}; delivering without vocabulary changes until restart"),
                );
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
    /// A native adapter can discard in-flight work; Ultra checks at layer boundaries.
    /// D1 may finish its current graph run; cancelled output is never applied.
    pub async fn adjudicate_with_cancel(
        &self,
        text: &str,
        observer: &dyn Events,
        cancel: Arc<AtomicBool>,
    ) -> Outcome {
        let started = Instant::now();
        let mut jobs = Vec::new();
        let mut results = Vec::new();
        let mut delivered = text.to_string();
        let mut error: Option<String> = None;
        let work = if cancel.load(Ordering::Relaxed) {
            Err("decision cancelled".into())
        } else {
            vocabulary_at(&self.vocabulary_path)
                .and_then(|terms| candidates_for(text, &terms, self.backend))
        };
        match work {
            Err(e) => error = Some(e),
            Ok(found) if found.is_empty() => {}
            Ok(found) => {
                jobs = found;
                match self.request(&jobs, cancel.clone()).await {
                    Ok(_) if cancel.load(Ordering::Relaxed) => {
                        error = Some("decision cancelled".into())
                    }
                    Ok(answers) => match apply(text, &jobs, &answers) {
                        Ok(out) => {
                            delivered = out;
                            results = answers;
                        }
                        Err(e) => error = Some(e),
                    },
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }
        }
        let elapsed = started.elapsed().as_secs_f64();
        if let Some(e) = &error {
            observer.error("dec", &format!("{e}; transcript preserved"));
        }
        observer.phase(
            "dec",
            &format!(
                "{} candidates, {} replacements, {:.0} ms",
                jobs.len(),
                results.iter().filter(|r| r.choice == Choice::Name).count(),
                elapsed * 1000.0
            ),
        );
        for (candidate, verdict) in jobs.iter().zip(&results) {
            observer.phase(
                "dec.choice",
                &format!(
                    "{:?} -> {:?}: {:?}; P(name)={:.3}; batch {:.0}ms",
                    candidate.heard,
                    candidate.canonical,
                    verdict.choice,
                    verdict.probabilities[1],
                    verdict.latency_ms
                ),
            );
        }
        let audit = json!({"backend":self.backend.label(),"input":text,"delivered":delivered,"decision_secs":elapsed,"candidates":jobs,"results":results,"error":error});
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
        assert_eq!(u[0].state, "My dog «Gamble» is here. Water has flow.");
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
            "## Gambol\ndescription: pet dog\nheard: gamble\nordinary: a gambling verb\n",
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
        let input = "My dog Gamble is an Australian shepherd. Gamble loves her frisbee.";
        let cancelled =
            rt.block_on(model.adjudicate_with_cancel(input, &transcrust_core::SilentEvents, flag));
        cancel.join().unwrap();
        assert_eq!(cancelled.text, input);
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
    #[test]
    fn targets_occurrences_without_substrings_or_existing_canonical_names() {
        let text = "Gambol gambled. I gamble; water flows. The flow is slow.";
        let found = candidates(text, &terms()).unwrap();
        assert_eq!(
            found.iter().map(|c| c.heard.as_str()).collect::<Vec<_>>(),
            vec!["gamble", "flow"]
        );
        assert!(!found[0].state.contains("The flow is slow"));
    }
    #[test]
    fn utf8_offsets_two_sentences_and_independent_replacements() {
        let text = "Éloïse likes my dog Gamble.I gamble with cards. Water has flow.";
        let found = candidates(text, &terms()).unwrap();
        assert_eq!(found.len(), 3);
        assert!(found[1].state.contains("Éloïse"));
        assert!(!found[1].state.contains("Water"));
        let result = |choice| Verdict {
            choice,
            probabilities: [0.1, 0.8, 0.1],
            token_count: 80,
            latency_ms: 1.0,
        };
        assert_eq!(
            apply(
                text,
                &found,
                &[
                    result(Choice::Name),
                    result(Choice::Keep),
                    result(Choice::Abstain)
                ]
            )
            .unwrap(),
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
            heard: vec!["Rowan".into()],
            ..Default::default()
        });
        assert!(candidates("I gamble. My son O is here.", &terms)
            .unwrap()
            .is_empty());
    }
    #[test]
    fn reload_parser_preserves_descriptions_and_excludes_tasks() {
        let terms=parse("## Rowan\ndescription: son\nordinary: other person\nheard: Rowan, Rowan\n## Cigs\nkind: habit\nheard: six\n");
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].heard, vec!["Rowan", "Rowan"]);
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
        let text = "Dr. Nell saw 2.5 dogs. I gamble. Then we leave.";
        let found = candidates(text, &terms()).unwrap();
        assert!(found[0].state.contains("Dr. Nell saw 2.5 dogs."));
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
        let found = candidates("I gamble.", &terms()).unwrap();
        assert!(apply("I gamble.", &found, &[]).is_err());
        let bad = Verdict {
            choice: Choice::Name,
            probabilities: [f64::NAN, 1.0, 0.0],
            token_count: 1,
            latency_ms: 0.0,
        };
        assert!(apply("I gamble.", &found, &[bad]).is_err());
    }
}
