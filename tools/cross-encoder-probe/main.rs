//! Offline paired interpretation experiment, using published ONNX classifiers.
use ort::{session::Session, value::Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Instant};
use tokenizers::Tokenizer;
type Error = Box<dyn std::error::Error>;

#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    source: String,
    arm: String,
    split: String,
    category: String,
    state: String,
    target: String,
    heard: String,
    hypotheses: [String; 2],
    want: Option<String>,
    candidate: Value,
}

fn hash(path: &Path) -> Result<String, Error> {
    let mut f = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut bytes = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        digest.update(&bytes[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn target(context: &str) -> Result<String, Error> {
    let start = context.find('«').ok_or("missing target marker")?;
    let end = context.find('»').ok_or("missing target end")?;
    let mut from = 0;
    for (i, c) in context.char_indices() {
        if matches!(c, '.' | '!' | '?' | '\n') {
            if i < start {
                from = i + c.len_utf8();
            } else if i > end {
                return Ok(context[from..i + c.len_utf8()].trim().into());
            }
        }
    }
    Ok(context[from..].trim().into())
}

fn make_case(
    id: String,
    source: &str,
    arm: &str,
    state: String,
    question: &Value,
    want: Option<String>,
    category: &str,
    split: &str,
    candidate: Value,
) -> Result<Case, Error> {
    let (_, dictation) = state
        .split_once("\nDictation:\n")
        .ok_or("missing Dictation section")?;
    let sentence = target(dictation)?;
    let heard = sentence
        .split_once('«')
        .unwrap()
        .1
        .split_once('»')
        .unwrap()
        .0
        .to_owned();
    let unmarked = sentence.replace(['«', '»'], "");
    let hypotheses = ["keep", "name"].map(|key| {
        let meaning = question["criteria"][key]
            .as_str()
            .unwrap()
            .trim_start_matches("It is ");
        format!("The occurrence of \"{heard}\" in \"{unmarked}\" refers to {meaning}.")
    });
    Ok(Case {
        id,
        source: source.into(),
        arm: arm.into(),
        split: split.into(),
        category: category.into(),
        state,
        target: sentence,
        heard,
        hypotheses,
        want,
        candidate,
    })
}

fn prepare(path: &Path) -> Result<(), Error> {
    let mut cases = Vec::new();
    for engine in ["parakeet", "phonon"] {
        let input: Value = serde_json::from_slice(&fs::read(format!(
            "tools/pair-router-probe/phone-household-q8/evidence/{engine}-decision.json"
        ))?)?;
        for (i, c) in input["candidates"].as_array().unwrap().iter().enumerate() {
            let original = c["state"].as_str().unwrap();
            let (glossary, context) = original.split_once("\nDictation:\n").unwrap();
            let sentence = target(context)?;
            for arm in ["single", "two"] {
                let state = if arm == "single" {
                    format!("{glossary}\nDictation:\n{sentence}")
                } else {
                    original.into()
                };
                cases.push(make_case(
                    format!("{engine}-{i}"),
                    engine,
                    arm,
                    state,
                    &c["question"],
                    None,
                    "household",
                    "phone",
                    c.clone(),
                )?);
            }
        }
    }
    let synthetic: Value = serde_json::from_slice(&fs::read(
        "tools/pair-router-probe/bluey-q8-expanded-glossary/results.json",
    )?)?;
    let fixture: Value = serde_json::from_slice(&fs::read(
        "tools/pair-router-probe/bluey-expanded-suite.json",
    )?)?;
    for row in synthetic["cases"].as_array().unwrap() {
        let id = row["case"]["id"].as_str().unwrap();
        let scenario = fixture
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap();
        // Same target sentence (including paired ambiguous contexts) stays in one split.
        let key = scenario["single_context"]
            .as_str()
            .unwrap()
            .replace(['«', '»'], "")
            .to_lowercase();
        let digest = Sha256::digest(
            key.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .as_bytes(),
        );
        let category = row["case"]["category"].as_str().unwrap();
        let split = if matches!(category, "privacy-initial" | "intentional-spelling") {
            "diagnostic"
        } else if digest[0] % 5 == 0 {
            "dev"
        } else {
            "eval"
        };
        cases.push(make_case(
            id.into(),
            "synthetic",
            row["arm"].as_str().unwrap(),
            row["state"].as_str().unwrap().into(),
            &row["questions"]["decision"],
            Some(row["case"]["want"].as_str().unwrap().into()),
            category,
            split,
            Value::Null,
        )?);
    }
    fs::write(path, serde_json::to_vec_pretty(&cases)?)?;
    println!(
        "Frozen {} decisions / {} paired rows: {}",
        cases.len(),
        cases.len() * 2,
        path.display()
    );
    Ok(())
}

struct Model {
    session: Session,
    tok: Tokenizer,
    pad: i64,
    segments: bool,
    nli: bool,
}
impl Model {
    fn load(dir: &Path, nli: bool, reference: bool) -> Result<Self, Error> {
        let mut expected = if nli {
            [
                "03c2221313dc0c3eac9cec1f746d1319d33f2c2901fcce1c0f08f4daac9b6dae",
                "5124ef2ead1a10a717703bc436de7f353da76d6340e4587719b42b1693707964",
                "885d0dceae8fa5c136da9209121ec9eb11160488e840de3bc1f29353674e5712",
            ]
        } else {
            [
                "dd98f3e67837d23210a6b7550c08cced4f61845b940ac45be3565840a10f3244",
                "48564c5c7d3fa64d85d95e65414a542385f88b0f128fd8d4163fd7a57f2be05c",
                "b6575b9d5be20d6747417c8e20c5a0db1636356e0b6d422d7244c628423c4d4c",
            ]
        };
        if reference {
            expected[0] = "59fd8dd78926e15907ab419303179e4196bdedd710e888773e2f0143fe430897";
        }
        for (name, wanted) in ["model.onnx", "tokenizer.json", "config.json"]
            .iter()
            .zip(expected)
        {
            if hash(&dir.join(name))? != wanted {
                return Err(format!("checksum mismatch: {name}").into());
            }
        }
        let cfg: Value = serde_json::from_slice(&fs::read(dir.join("config.json"))?)?;
        if nli {
            for (i, label) in ["contradiction", "entailment", "neutral"]
                .iter()
                .enumerate()
            {
                if cfg["id2label"][i.to_string()] != *label {
                    return Err("unsupported NLI label order".into());
                }
            }
        }
        let mut tok =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tok.with_truncation(None).map_err(|e| e.to_string())?;
        tok.with_padding(None);
        let session = Session::builder()?
            .with_intra_threads(4)?
            .with_inter_threads(1)?
            .commit_from_file(dir.join("model.onnx"))?;
        let segments = session
            .inputs()
            .iter()
            .any(|i| i.name() == "token_type_ids");
        eprintln!(
            "inputs: {:?}",
            session
                .inputs()
                .iter()
                .map(|i| i.name())
                .collect::<Vec<_>>()
        );
        Ok(Self {
            session,
            tok,
            pad: cfg["pad_token_id"].as_i64().ok_or("missing pad id")?,
            segments,
            nli,
        })
    }

    fn infer(
        &mut self,
        pairs: &[(String, String)],
    ) -> Result<(Vec<Vec<f32>>, f64, Vec<usize>), Error> {
        let enc = pairs
            .iter()
            .map(|(a, b)| {
                self.tok
                    .encode((a.as_str(), b.as_str()), true)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let lengths: Vec<_> = enc.iter().map(|e| e.len()).collect();
        let length = *lengths.iter().max().ok_or("empty batch")?;
        if length > 512 {
            return Err(format!("refusing truncation: {length} tokens").into());
        }
        let batch = enc.len();
        let mut ids = vec![self.pad; batch * length];
        let mut mask = vec![0i64; batch * length];
        let mut types = mask.clone();
        for (row, e) in enc.iter().enumerate() {
            for j in 0..e.len() {
                ids[row * length + j] = e.get_ids()[j] as i64;
                mask[row * length + j] = 1;
                types[row * length + j] = e.get_type_ids()[j] as i64;
            }
        }
        let tensor = |data: Vec<i64>| Tensor::from_array((vec![batch, length], data));
        let started = Instant::now();
        let output = if self.segments {
            self.session.run(ort::inputs!["input_ids"=>tensor(ids)?,"attention_mask"=>tensor(mask)?,"token_type_ids"=>tensor(types)?])?
        } else {
            self.session
                .run(ort::inputs!["input_ids"=>tensor(ids)?,"attention_mask"=>tensor(mask)?])?
        };
        let ms = started.elapsed().as_secs_f64() * 1000.;
        let (shape, data) = output["logits"].try_extract_tensor::<f32>()?;
        let width = if self.nli { 3 } else { 1 };
        if shape.as_ref() != [batch as i64, width as i64] || !data.iter().all(|x| x.is_finite()) {
            return Err(format!("invalid logits {shape:?}").into());
        }
        Ok((
            data.chunks(width).map(|r| r.to_vec()).collect(),
            ms,
            lengths,
        ))
    }
}

fn probabilities(logits: &[f32]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let p: Vec<_> = logits.iter().map(|x| (*x as f64 - max).exp()).collect();
    let sum: f64 = p.iter().sum();
    p.iter().map(|x| x / sum).collect()
}
#[derive(Clone, Serialize, Deserialize)]
struct Record {
    case: Case,
    logits: [Vec<f32>; 2],
    probabilities: Option<[Vec<f64>; 2]>,
    scores: [f64; 2],
    name_margin: f64,
    forced_choice: String,
    choice: String,
    batch: usize,
    token_counts: [usize; 2],
}
#[derive(Clone, Copy, Serialize)]
struct Policy {
    margin: f64,
    floor: f64,
}
fn choice(r: &Record, p: Policy) -> &'static str {
    let best = usize::from(r.scores[1] > r.scores[0]);
    if (r.scores[1] - r.scores[0]).abs() < p.margin || r.scores[best] < p.floor {
        return "abstain";
    }
    if let Some(nli) = &r.probabilities {
        if nli[best][1] < nli[best][0].max(nli[best][2]) {
            return "abstain";
        }
    }
    if best == 1 {
        "name"
    } else {
        "keep"
    }
}
fn fit(records: &[Record], nli: bool) -> Policy {
    // All-abstain is an explicit candidate, not hidden threshold selection on eval.
    let mut best = Policy {
        margin: 1e9,
        floor: 0.,
    };
    let mut key = (0usize, 0usize);
    let margins = if nli {
        vec![0., 0.05, 0.15, 0.3, 0.6]
    } else {
        vec![0., 0.5, 1., 2., 4., 8.]
    };
    let floors = if nli {
        vec![0., 0.5, 0.7, 0.9]
    } else {
        vec![-1e9]
    };
    for margin in margins {
        for floor in &floors {
            let p = Policy {
                margin,
                floor: *floor,
            };
            let mut repairs = 0;
            let mut wrong = 0;
            for r in records.iter().filter(|r| r.case.split == "dev") {
                if choice(r, p) == "name" {
                    if r.case.want.as_deref() == Some("name") {
                        repairs += 1;
                    } else {
                        wrong += 1;
                    }
                }
            }
            if wrong < key.0 || (wrong == key.0 && repairs > key.1) {
                best = p;
                key = (wrong, repairs);
            }
        }
    }
    best
}

fn run(kind: &str, dir: &Path, fixture: &Path, out: &Path) -> Result<(), Error> {
    let nli = match kind {
        "deberta" | "deberta-reference" => true,
        "bge" => false,
        _ => return Err("expected deberta or bge".into()),
    };
    let started = Instant::now();
    let mut model = Model::load(dir, nli, kind == "deberta-reference")?;
    let load_secs = started.elapsed().as_secs_f64();
    let canaries = if nli {
        vec![
            ("A man is eating pizza.", "A man eats something."),
            ("A man is eating pizza.", "Nobody is eating."),
            ("A man is eating pizza.", "He owns a dog."),
        ]
    } else {
        vec![
            (
                "Which planet is known as the Red Planet?",
                "Mars is known as the Red Planet.",
            ),
            (
                "Which planet is known as the Red Planet?",
                "Bananas are yellow fruit.",
            ),
        ]
    };
    let pairs: Vec<_> = canaries
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    let (canary, _, _) = model.infer(&pairs)?;
    let observed: Vec<_> = canary
        .iter()
        .map(|row| {
            (0..row.len())
                .max_by(|a, b| row[*a].total_cmp(&row[*b]))
                .unwrap()
        })
        .collect();
    if kind == "deberta-reference" {
        fs::write(
            out,
            serde_json::to_vec_pretty(
                &json!({"model":kind,"graph_sha256":hash(&dir.join("model.onnx"))?,"pairs":pairs,"logits":canary,"probabilities":canary.iter().map(|r|probabilities(r)).collect::<Vec<_>>(),"expected_labels":[1,0,2],"observed_labels":observed}),
            )?,
        )?;
        println!("Reference canaries: {}", out.display());
        return Ok(());
    }
    if nli {
        for (row, want) in canary.iter().zip([1, 0]) {
            let best = (0..3).max_by(|a, b| row[*a].total_cmp(&row[*b])).unwrap();
            if best != want {
                return Err(format!("NLI canary failed: {canary:?}").into());
            }
        }
        if observed[2] != 2 {
            eprintln!("NLI neutral diagnostic missed: {canary:?}; retained in results");
        }
    } else if canary[0][0] <= canary[1][0] {
        return Err("BGE canary failed".into());
    }
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(fixture)?)?;
    let mut records = Vec::new();
    let mut batches = Vec::new();
    let mut validation = Value::Null;
    let mut groups: Vec<Vec<Case>> = Vec::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            groups.push(
                cases
                    .iter()
                    .filter(|c| c.source == source && c.arm == arm)
                    .cloned()
                    .collect(),
            );
        }
    }
    let synthetic: Vec<_> = cases
        .into_iter()
        .filter(|c| c.source == "synthetic")
        .collect();
    groups.extend(synthetic.chunks(8).map(|chunk| chunk.to_vec()));
    for (batch, group) in groups.iter().enumerate() {
        let pairs: Vec<_> = group
            .iter()
            .flat_map(|c| c.hypotheses.iter().map(|h| (c.state.clone(), h.clone())))
            .collect();
        let (logits, ms, lengths) = model.infer(&pairs)?;
        if batch == 0 {
            let (repeat, _, _) = model.infer(&pairs)?;
            let delta = logits
                .iter()
                .flatten()
                .zip(repeat.iter().flatten())
                .map(|(a, b)| (a - b).abs())
                .fold(0f32, f32::max);
            if delta > 1e-5 {
                return Err(format!("identical-batch repeat drift {delta}").into());
            }
            let (isolated, _, _) = model.infer(&pairs[..2])?;
            let isolated_delta = logits[..2]
                .iter()
                .flatten()
                .zip(isolated.iter().flatten())
                .map(|(a, b)| (a - b).abs())
                .fold(0f32, f32::max);
            validation = json!({"repeat_max_logit_delta":delta,"isolated_pair_max_logit_delta":isolated_delta,"note":"dynamic quantization may change scores with batch composition"});
        }
        eprintln!(
            "{kind}: {} occurrences / {} paired rows, {ms:.0}ms, batch {}",
            group.len(),
            pairs.len(),
            batch + 1
        );
        batches.push(json!({"batch":batch,"source":group[0].source,"arm":group[0].arm,"occurrences":group.len(),"paired_rows":pairs.len(),"graph_ms":ms}));
        for (i, c) in group.iter().enumerate() {
            let raw = [logits[2 * i].clone(), logits[2 * i + 1].clone()];
            let p = nli.then(|| [probabilities(&raw[0]), probabilities(&raw[1])]);
            let scores = if let Some(p) = &p {
                [p[0][1], p[1][1]]
            } else {
                [raw[0][0] as f64, raw[1][0] as f64]
            };
            records.push(Record {
                case: c.clone(),
                logits: raw,
                probabilities: p,
                scores,
                name_margin: scores[1] - scores[0],
                forced_choice: if scores[1] > scores[0] {
                    "name".into()
                } else {
                    "keep".into()
                },
                choice: String::new(),
                batch,
                token_counts: [lengths[2 * i], lengths[2 * i + 1]],
            });
        }
    }
    let policy = fit(&records, nli);
    for r in &mut records {
        r.choice = choice(r, policy).into();
    }
    let mut metrics = Vec::new();
    for split in ["dev", "eval", "diagnostic"] {
        for arm in ["single", "two"] {
            let subset: Vec<_> = records
                .iter()
                .filter(|r| r.case.split == split && r.case.arm == arm)
                .collect();
            for mode in ["forced", "policy"] {
                let mut repairs = 0;
                let mut wrong = 0;
                let mut repair_total = 0;
                let mut keep_total = 0;
                let mut abstain = 0;
                for r in &subset {
                    let answer = if mode == "forced" {
                        &r.forced_choice
                    } else {
                        &r.choice
                    };
                    if r.case.want.as_deref() == Some("name") {
                        repair_total += 1;
                        if answer == "name" {
                            repairs += 1;
                        }
                    } else {
                        keep_total += 1;
                        if answer == "name" {
                            wrong += 1;
                        }
                    }
                    if answer == "abstain" {
                        abstain += 1;
                    }
                }
                metrics.push(json!({"split":split,"arm":arm,"mode":mode,"n":subset.len(),"repair_total":repair_total,"repairs":repairs,"preserve_total":keep_total,"unwanted_edits":wrong,"abstentions":abstain}));
            }
        }
    }
    let mut household = Vec::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            for mode in ["forced", "policy"] {
                let original: Value = serde_json::from_slice(&fs::read(format!(
                    "tools/pair-router-probe/phone-household-q8/evidence/{source}-decision.json"
                ))?)?;
                let mut text = original["input"].as_str().unwrap().to_owned();
                let mut changes: Vec<_> = records
                    .iter()
                    .filter(|r| r.case.source == source && r.case.arm == arm)
                    .filter(|r| {
                        if mode == "forced" {
                            r.forced_choice == "name"
                        } else {
                            r.choice == "name"
                        }
                    })
                    .collect();
                changes.sort_by_key(|r| {
                    std::cmp::Reverse(r.case.candidate["start"].as_u64().unwrap())
                });
                for r in &changes {
                    let c = &r.case.candidate;
                    text.replace_range(
                        c["start"].as_u64().unwrap() as usize..c["end"].as_u64().unwrap() as usize,
                        c["canonical"].as_str().unwrap(),
                    );
                }
                household.push(json!({"source":source,"arm":arm,"mode":mode,"delivered":text,"changes":changes.len()}));
            }
        }
    }
    fs::write(
        out,
        serde_json::to_vec_pretty(
            &json!({"model":kind,"runtime":"Rust ORT CPU Q8","threads":4,"fixture_sha256":hash(fixture)?,"load_secs":load_secs,"canary_logits":canary,"canary_observed_labels":observed,"canary_expected_labels":if nli {json!([1,0,2])}else{Value::Null},"validation":validation,"policy":policy,"fit":"minimize unwanted dev edits, then maximize dev repairs; target-sentence grouped split; phone excluded","batches":batches,"metrics":metrics,"household":household,"records":records}),
        )?,
    )?;
    println!("Results: {}", out.display());
    Ok(())
}

fn main() -> Result<(), Error> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("prepare") if a.len()==2=>prepare(Path::new(&a[1])),
        Some("run") if a.len()==5=>run(&a[1],Path::new(&a[2]),Path::new(&a[3]),Path::new(&a[4])),
        _=>Err("usage: pair-probe prepare CASES.json | run deberta|bge MODEL_DIR CASES.json OUTPUT.json".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_selects_the_marked_occurrence_not_a_neighbor() {
        assert_eq!(
            target("My dog Gamble sleeps. I like to «gamble» and play poker.").unwrap(),
            "I like to «gamble» and play poker."
        );
        assert_eq!(
            target("We watched «flow». Water began to flow.").unwrap(),
            "We watched «flow»."
        );
    }
    #[test]
    fn no_evidence_support_is_not_a_forced_edit() {
        let r = Record {
            case: Case {
                id: "x".into(),
                source: "test".into(),
                arm: "single".into(),
                split: "eval".into(),
                category: "test".into(),
                state: String::new(),
                target: String::new(),
                heard: String::new(),
                hypotheses: [String::new(), String::new()],
                want: Some("keep".into()),
                candidate: Value::Null,
            },
            logits: [vec![], vec![]],
            probabilities: Some([vec![0.1, 0.2, 0.7], vec![0.1, 0.3, 0.6]]),
            scores: [0.2, 0.3],
            name_margin: 0.1,
            forced_choice: "name".into(),
            choice: String::new(),
            batch: 0,
            token_counts: [1, 1],
        };
        assert_eq!(
            choice(
                &r,
                Policy {
                    margin: 0.,
                    floor: 0.
                }
            ),
            "abstain"
        );
        assert_eq!(fit(&[r], true).margin, 1e9); // eval labels cannot train policy
    }
}
