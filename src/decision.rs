//! Opt-in local vocabulary adjudication. Candidate offsets belong to the original
//! cleaned transcript; the resident native model returns bounded choices in one batch.
use crate::{d1, dictionary, observe::Observer};
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

fn vocabulary() -> Result<Vec<Term>, String> {
    vocabulary_at(&vocab_path())
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

fn candidates(text: &str, terms: &[Term]) -> Result<Vec<Candidate>, String> {
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
            let (a, b) = context_range(text, hit.start(), hit.end());
            let marked = format!(
                "{}«{}»{}",
                &text[a..hit.start()],
                hit.as_str(),
                &text[hit.end()..b]
            );
            let state = format!("{glossary}\nDictation:\n{}", marked.trim());
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

fn model_path() -> PathBuf {
    let installed = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("transcrust/models/d1-omni-600M-q8");
    std::env::var_os("TRANSCRUST_DEC_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if installed.join("manifest.json").exists() {
                installed
            } else {
                PathBuf::from("/tmp/transcrust-d1-omni-600M-q8")
            }
        })
}

pub struct Decision {
    model: Arc<Mutex<Option<d1::Model>>>,
    disabled: AtomicBool,
}
pub struct Outcome {
    pub text: String,
    pub audit: Value,
}
impl Decision {
    pub async fn start(observer: &Observer) -> Self {
        let started = Instant::now();
        let load = tokio::task::spawn_blocking(|| {
            let mut model = d1::Model::load(&model_path())?;
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
            )?;
            Ok::<_, String>(model)
        });
        let loaded = match tokio::time::timeout(Duration::from_secs(30), load).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("decision model startup timeout".into()),
        };
        let model = match loaded {
            Ok(worker) => {
                observer.phase(
                    "dec",
                    &format!(
                        "native text-only Q8 ready in {:.2}s; batched decisions; decoder boosting off; shared vocabulary {}",
                        started.elapsed().as_secs_f64(),
                        vocab_path().display()
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
        }
    }
    async fn request(&self, candidates: &[Candidate]) -> Result<Vec<Verdict>, String> {
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
        let model = self.model.clone();
        let task = tokio::task::spawn_blocking(move || {
            let mut guard = model.lock().map_err(|e| e.to_string())?;
            let result = guard
                .as_mut()
                .ok_or("decision model unavailable; restart transcrust to retry")?
                .infer(&questions, &task_options);
            if result.is_err() {
                *guard = None;
            }
            result
        });
        let answer = match tokio::time::timeout(Duration::from_secs(15), task).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => {
                let _ = options.terminate();
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
                self.disabled.store(true, Ordering::Relaxed);
                Err(e)
            }
        }
    }
    pub async fn adjudicate(&self, text: &str, observer: &Observer) -> Outcome {
        let started = Instant::now();
        let mut jobs = Vec::new();
        let mut results = Vec::new();
        let mut delivered = text.to_string();
        let mut error: Option<String> = None;
        let work = vocabulary().and_then(|terms| candidates(text, &terms));
        match work {
            Err(e) => error = Some(e),
            Ok(found) if found.is_empty() => {}
            Ok(found) => {
                jobs = found;
                match self.request(&jobs).await {
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
        let audit = json!({"input":text,"delivered":delivered,"decision_secs":elapsed,"candidates":jobs,"results":results,"error":error});
        Outcome {
            text: delivered,
            audit,
        }
    }
}

pub fn log_delivery(
    observer: &Observer,
    raw: &str,
    delivered: &str,
    asr_secs: f32,
    wait_secs: f32,
    mut audit: Value,
) {
    use std::io::Write;
    audit["raw"] = json!(raw);
    audit["delivered"] = json!(delivered);
    audit["asr_secs"] = json!(asr_secs);
    audit["wait_secs"] = json!(wait_secs);
    audit["timestamp"] = json!(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs());
    let path = observer.log_path().with_file_name("decisions.jsonl");
    if let Err(e) = (|| -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{audit}").map_err(|e| e.to_string())
    })() {
        observer.error("dec.log", &e);
    }
    observer.phase(
        "timing",
        &format!("ASR {asr_secs:.2}s; total wait to typing {wait_secs:.2}s"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn terms() -> Vec<Term> {
        parse("## Gambol\ndescription: pet dog\nheard: gamble\n## Flo\ndescription: Cars character\nheard: flow\n")
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
