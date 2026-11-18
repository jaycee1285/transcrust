//! vocab-score: deterministic tier scoring for john-vocab.md (design-vocab-gate.md, step 2).
//!
//! For every vocabulary entry: phonemes (override > CMUdict > espeak-ng), accent-merger
//! collapse, feature-weighted phoneme distance to the nearest frequent English words, and a
//! tier with a reason sentence. No model anywhere.
//!
//!   tier 0  homophone: identical to a frequent word after mergers, or is itself one
//!   tier 1  near: weighted distance <= NEAR (set once, before looking at results)
//!   tier 2  distinct
//!
//! Usage: vocab-score <john-vocab.md> [--data DIR] [--out PREFIX] [--commands FILE]
//! Needs `espeak-ng` on PATH only for words missing from CMUdict.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::process::Command;

/// One phoneme's worth of change. Fixed before the first run; never tuned to the canon.
const NEAR: f64 = 1.0;
const TOP_WORDS: usize = 20_000;

// ---------- phone features ----------

#[derive(Clone, Copy)]
enum F {
    /// voiced, place (0 bilabial .. 7 glottal), manner (0 stop,1 fric,2 affric,3 nasal,4 liquid,5 glide, 6 flap)
    C(bool, u8, u8),
    /// height (0 high..2 low), backness (0 front..2 back), rounded, diphthong
    V(u8, u8, bool, bool),
}

fn feat(p: &str) -> Option<F> {
    use F::*;
    Some(match p {
        "P" => C(false, 0, 0), "B" => C(true, 0, 0), "M" => C(true, 0, 3), "W" => C(true, 0, 5),
        "F" => C(false, 1, 1), "V" => C(true, 1, 1),
        "TH" => C(false, 2, 1), "DH" => C(true, 2, 1),
        "T" => C(false, 3, 0), "D" => C(true, 3, 0), "S" => C(false, 3, 1), "Z" => C(true, 3, 1),
        "N" => C(true, 3, 3), "L" => C(true, 3, 4), "R" => C(true, 4, 4), "DX" => C(true, 3, 6),
        "SH" => C(false, 4, 1), "ZH" => C(true, 4, 1), "CH" => C(false, 4, 2), "JH" => C(true, 4, 2),
        "Y" => C(true, 5, 5), "K" => C(false, 6, 0), "G" => C(true, 6, 0), "NG" => C(true, 6, 3),
        "HH" => C(false, 7, 1),
        "IY" => V(0, 0, false, false), "IH" => V(0, 0, false, false),
        "EY" => V(1, 0, false, true), "EH" => V(1, 0, false, false), "AE" => V(2, 0, false, false),
        "AH" => V(1, 1, false, false), "ER" => V(1, 1, false, false),
        "AA" => V(2, 2, false, false), "AO" => V(1, 2, true, false),
        "OW" => V(1, 2, true, true), "UH" => V(0, 2, true, false), "UW" => V(0, 2, true, false),
        "AY" => V(2, 1, false, true), "AW" => V(2, 1, true, true), "OY" => V(1, 2, true, true),
        _ => return None,
    })
}

fn is_vowel(p: &str) -> bool {
    matches!(feat(p), Some(F::V(..)))
}

/// Substitution cost in [0, 1].
fn sub(a: &str, b: &str) -> f64 {
    if a == b {
        return 0.0;
    }
    // IY/IH share features here but are different phonemes: keep them apart.
    let tense = |p: &str| matches!(p, "IY" | "UW");
    match (feat(a), feat(b)) {
        (Some(F::C(v1, p1, m1)), Some(F::C(v2, p2, m2))) => {
            let place = (p1 as f64 - p2 as f64).abs() / 7.0;
            let manner = if m1 == m2 { 0.0 } else { 0.5 };
            (if v1 != v2 { 0.25 } else { 0.0 } + 0.4 * place + manner).min(1.0)
        }
        (Some(F::V(h1, b1, r1, d1)), Some(F::V(h2, b2, r2, d2))) => {
            let mut c = 0.2 * (h1 as f64 - h2 as f64).abs() + 0.15 * (b1 as f64 - b2 as f64).abs();
            if r1 != r2 { c += 0.15 }
            if d1 != d2 { c += 0.15 }
            if tense(a) != tense(b) { c += 0.15 }
            c.clamp(0.1, 1.0)
        }
        _ => 1.0,
    }
}

/// Feature-weighted Levenshtein over phones (indel = 1).
fn wdist(a: &[String], b: &[String]) -> f64 {
    let mut prev: Vec<f64> = (0..=b.len()).map(|j| j as f64).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i as f64; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = (prev[j] + 1.0)
                .min(cur[j - 1] + 1.0)
                .min(prev[j - 1] + sub(&a[i - 1], &b[j - 1]));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Kondrak-style bigram distance: 1 - Dice over phone bigrams (with edge markers).
fn ngram_dist(a: &[String], b: &[String]) -> f64 {
    let grams = |s: &[String]| {
        let mut v: Vec<String> = Vec::new();
        let padded: Vec<&str> = std::iter::once("#").chain(s.iter().map(|x| x.as_str())).chain(std::iter::once("#")).collect();
        for w in padded.windows(2) {
            v.push(format!("{} {}", w[0], w[1]));
        }
        v
    };
    let (ga, mut gb) = (grams(a), grams(b));
    let total = ga.len() + gb.len();
    let mut shared = 0;
    for g in &ga {
        if let Some(i) = gb.iter().position(|x| x == g) {
            gb.remove(i);
            shared += 1;
        }
    }
    1.0 - (2.0 * shared as f64) / total as f64
}

// ---------- mergers ----------

enum Rule {
    Map(String, String),
    Before(Vec<String>, Vec<String>, String),
    Between(Vec<String>, String),
}

fn load_rules(text: &str) -> Vec<Rule> {
    let list = |s: &str| s.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>();
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        if let Some(r) = line.strip_prefix("map ") {
            let (a, b) = r.split_once("->").expect("map A -> B");
            out.push(Rule::Map(a.trim().into(), b.trim().into()));
        } else if let Some(r) = line.strip_prefix("before ") {
            let (ctx, rest) = r.split_once(':').expect("before X: A -> B");
            let (a, b) = rest.split_once("->").expect("before X: A -> B");
            out.push(Rule::Before(list(ctx), list(a), b.trim().into()));
        } else if let Some(r) = line.strip_prefix("between-vowels:") {
            let (a, b) = r.split_once("->").expect("between-vowels: A -> B");
            out.push(Rule::Between(list(a), b.trim().into()));
        } else {
            panic!("unknown merger rule: {line}");
        }
    }
    out
}

fn collapse(phones: &[String], rules: &[Rule]) -> Vec<String> {
    let mut p = phones.to_vec();
    for rule in rules {
        let snapshot = p.clone();
        for i in 0..p.len() {
            match rule {
                Rule::Map(a, b) if &snapshot[i] == a => p[i] = b.clone(),
                Rule::Before(ctx, a, b)
                    if a.contains(&snapshot[i]) && snapshot.get(i + 1).is_some_and(|n| ctx.contains(n)) =>
                {
                    p[i] = b.clone()
                }
                Rule::Between(a, b)
                    if a.contains(&snapshot[i])
                        && i > 0
                        && is_vowel(&snapshot[i - 1])
                        && snapshot.get(i + 1).is_some_and(|n| is_vowel(n)) =>
                {
                    p[i] = b.clone()
                }
                _ => {}
            }
        }
    }
    p
}

// ---------- pronunciations ----------

fn strip(ph: &str) -> String {
    ph.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
}

fn load_cmudict(text: &str) -> HashMap<String, Vec<Vec<String>>> {
    let mut d: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let mut it = line.split_whitespace();
        let Some(w) = it.next() else { continue };
        let word = w.split('(').next().unwrap().to_lowercase();
        d.entry(word).or_default().push(it.map(String::from).collect());
    }
    d
}

/// IPA (espeak-ng --ipa) → ARPAbet. Longest match first; unknown symbols are reported.
fn ipa_to_arpa(ipa: &str, unknown: &mut Vec<String>) -> Vec<String> {
    const T: &[(&str, &str)] = &[
        ("tʃ", "CH"), ("dʒ", "JH"), ("eɪ", "EY"), ("aɪ", "AY"), ("aʊ", "AW"), ("oʊ", "OW"),
        ("əʊ", "OW"), ("ɔɪ", "OY"), ("ɑːɹ", "AA R"), ("ɜːɹ", "ER"), ("ɜː", "ER"), ("ɚ", "ER"),
        ("ɑː", "AA"), ("ɔː", "AO"), ("iː", "IY"), ("uː", "UW"), ("i", "IY"), ("ɪ", "IH"),
        ("ɛ", "EH"), ("e", "EH"), ("æ", "AE"), ("ɐ", "AH"), ("ʌ", "AH"), ("ə", "AH"),
        ("ɑ", "AA"), ("ɒ", "AA"), ("ɔ", "AO"), ("o", "OW"), ("ʊ", "UH"), ("u", "UW"),
        ("θ", "TH"), ("ð", "DH"), ("ʃ", "SH"), ("ʒ", "ZH"), ("ŋ", "NG"), ("ɹ", "R"),
        ("ɾ", "DX"), ("ɡ", "G"), ("g", "G"), ("j", "Y"), ("w", "W"), ("h", "HH"),
        ("p", "P"), ("b", "B"), ("t", "T"), ("d", "D"), ("k", "K"), ("f", "F"), ("v", "V"),
        ("s", "S"), ("z", "Z"), ("m", "M"), ("n", "N"), ("l", "L"), ("r", "R"),
    ];
    let s: String = ipa.chars().filter(|c| !matches!(c, 'ˈ' | 'ˌ' | '_' | ' ' | '\n' | 'ʔ')).collect();
    let mut out = Vec::new();
    let mut rest = s.as_str();
    'outer: while !rest.is_empty() {
        for (k, v) in T {
            if let Some(r) = rest.strip_prefix(k) {
                out.extend(v.split(' ').map(String::from));
                rest = r;
                continue 'outer;
            }
        }
        let c = rest.chars().next().unwrap();
        if c != 'ː' {
            unknown.push(c.to_string());
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

fn espeak(word: &str, unknown: &mut Vec<String>) -> Option<Vec<String>> {
    let o = Command::new("espeak-ng").args(["-q", "--ipa", "-v", "en-us", word]).output().ok()?;
    let ipa = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (!ipa.is_empty()).then(|| ipa_to_arpa(&ipa, unknown))
}

#[derive(Clone)]
struct Pron {
    phones: Vec<String>,
    source: String,
}

/// All pronunciations of a (possibly multi-word) term, capped at 8 combinations.
fn prons(term: &str, override_: Option<&str>, dict: &HashMap<String, Vec<Vec<String>>>, unknown: &mut Vec<String>) -> Vec<Pron> {
    if let Some(o) = override_ {
        return vec![Pron { phones: o.split_whitespace().map(strip).collect(), source: "override".into() }];
    }
    let mut acc: Vec<Pron> = vec![Pron { phones: vec![], source: String::new() }];
    for w in term.split_whitespace() {
        let key = w.to_lowercase();
        let (variants, src): (Vec<Vec<String>>, &str) = match dict.get(&key) {
            Some(v) => (v.iter().map(|p| p.iter().map(|x| strip(x)).collect()).collect(), "cmudict"),
            None => match espeak(&key, unknown) {
                Some(p) => (vec![p], "espeak"),
                None => (vec![], "missing"),
            },
        };
        if variants.is_empty() {
            return vec![];
        }
        let mut next = Vec::new();
        for a in &acc {
            for (vi, v) in variants.iter().enumerate() {
                let mut phones = a.phones.clone();
                phones.extend(v.iter().cloned());
                let tag = if variants.len() > 1 { format!("{w}:{src}#{}", vi + 1) } else { format!("{w}:{src}") };
                let source = if a.source.is_empty() { tag } else { format!("{} {}", a.source, tag) };
                next.push(Pron { phones, source });
            }
        }
        next.truncate(8);
        acc = next;
    }
    acc
}

// ---------- vocabulary ----------

struct Entry {
    term: String,
    kind: String,
    phonemes: Option<String>,
}

fn parse_vocab(text: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for line in text.lines() {
        if let Some(t) = line.strip_prefix("## ") {
            out.push(Entry { term: t.trim().into(), kind: String::new(), phonemes: None });
        } else if let Some(e) = out.last_mut() {
            let l = line.trim().trim_start_matches(['-', '*']).trim();
            if let Some(k) = l.strip_prefix("kind:") {
                e.kind = k.trim().into();
            } else if let Some(p) = l.strip_prefix("phonemes:") {
                e.phonemes = Some(p.trim().into());
            }
        }
    }
    out
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let vocab_path = args.get(1).filter(|a| !a.starts_with("--")).cloned().expect("usage: vocab-score <john-vocab.md>");
    let data = get("--data").unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/data").into());
    let out = get("--out").unwrap_or_else(|| "vocab-tiers".into());

    let dict = load_cmudict(&fs::read_to_string(format!("{data}/cmudict.dict")).expect("cmudict.dict"));
    let rules = load_rules(&fs::read_to_string(format!("{data}/mergers.txt")).expect("mergers.txt"));
    let freq: Vec<String> = fs::read_to_string(format!("{data}/20k.txt"))
        .expect("20k.txt")
        .lines()
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty())
        .take(TOP_WORDS)
        .collect();
    let rank: HashMap<&str, usize> = freq.iter().enumerate().map(|(i, w)| (w.as_str(), i + 1)).collect();
    // Frequent words with CMUdict pronunciations, pre-collapsed.
    let mut common: Vec<(String, Vec<String>)> = Vec::new();
    for w in &freq {
        if let Some(vs) = dict.get(w) {
            for v in vs {
                let p: Vec<String> = v.iter().map(|x| strip(x)).collect();
                common.push((w.clone(), collapse(&p, &rules)));
            }
        }
    }
    eprintln!("cmudict {} words · frequent {} · frequent with prons {} · rules {}", dict.len(), freq.len(), common.len(), rules.len());

    let entries = parse_vocab(&fs::read_to_string(&vocab_path).expect("vocab file"));
    let mut unknown_ipa: Vec<String> = Vec::new();
    let mut md = String::from("# vocab tiers\n\nGenerated by `tools/vocab-score` (deterministic, no model). ");
    md += &format!("NEAR = {NEAR} (one phoneme of change, fixed before the first run). Mergers: `data/mergers.txt` (us-general).\n\n");
    md += "| entry | kind | tier | phonemes (source) | collapsed | nearest frequent words (weighted / bigram) | reason |\n|---|---|---|---|---|---|---|\n";
    let mut json = String::from("[\n");

    for (ei, e) in entries.iter().enumerate() {
        let ps = prons(&e.term, e.phonemes.as_deref(), &dict, &mut unknown_ipa);
        if ps.is_empty() {
            md += &format!("| {} | {} | ? | — | — | — | no pronunciation from override, CMUdict or espeak |\n", e.term, e.kind);
            continue;
        }
        // Best (closest) neighbour per frequent word, over all entry pronunciations.
        let mut best: BTreeMap<String, (f64, f64, usize)> = BTreeMap::new();
        let mut self_rank: Option<usize> = None;
        let words: Vec<String> = e.term.split_whitespace().map(|w| w.to_lowercase()).collect();
        if words.len() == 1 {
            self_rank = rank.get(words[0].as_str()).copied();
        }
        let mut chosen = ps[0].clone();
        let mut chosen_d = f64::MAX;
        for (pi, p) in ps.iter().enumerate() {
            let c = collapse(&p.phones, &rules);
            for (w, wp) in &common {
                if words.len() == 1 && w == &words[0] {
                    continue; // itself: reported separately as self_rank
                }
                if (c.len() as i64 - wp.len() as i64).abs() > 3 {
                    continue;
                }
                let d = wdist(&c, wp);
                let n = ngram_dist(&c, wp);
                let slot = best.entry(w.clone()).or_insert((f64::MAX, 1.0, pi));
                if d < slot.0 {
                    *slot = (d, n, pi);
                }
                if d < chosen_d {
                    chosen_d = d;
                    chosen = p.clone();
                }
            }
        }
        let mut near: Vec<(String, f64, f64)> = best.into_iter().map(|(w, (d, n, _))| (w, d, n)).collect();
        near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(rank[a.0.as_str()].cmp(&rank[b.0.as_str()])));
        near.truncate(3);
        let top = near.first().cloned();

        let (tier, reason) = if let Some(r) = self_rank {
            (0, format!("is itself a frequent English word (rank {r} of {TOP_WORDS})"))
        } else if let Some((w, d, _)) = &top {
            if *d == 0.0 {
                (0, format!("identical to '{w}' after mergers"))
            } else if *d <= NEAR {
                (1, format!("within {d:.2} of '{w}' (≤ {NEAR})"))
            } else {
                (2, format!("nearest frequent word '{w}' is {d:.2} away"))
            }
        } else {
            (2, "no frequent word within 3 phones of its length".into())
        };
        let collapsed = collapse(&chosen.phones, &rules).join(" ");
        let near_s = near.iter().map(|(w, d, n)| format!("{w} {d:.2}/{n:.2}")).collect::<Vec<_>>().join(", ");
        md += &format!("| {} | {} | **{}** | {} ({}) | {} | {} | {} |\n", e.term, e.kind, tier, chosen.phones.join(" "), chosen.source, collapsed, near_s, reason);
        json += &format!(
            "  {{\"entry\": {}, \"kind\": {}, \"tier\": {}, \"phonemes\": {}, \"source\": {}, \"collapsed\": {}, \"nearest\": [{}], \"reason\": {}}}{}\n",
            json_str(&e.term), json_str(&e.kind), tier, json_str(&chosen.phones.join(" ")), json_str(&chosen.source), json_str(&collapsed),
            near.iter().map(|(w, d, n)| format!("{{\"word\": {}, \"weighted\": {:.3}, \"bigram\": {:.3}}}", json_str(w), d, n)).collect::<Vec<_>>().join(", "),
            json_str(&reason),
            if ei + 1 < entries.len() { "," } else { "" }
        );
    }
    json += "]\n";
    if !unknown_ipa.is_empty() {
        unknown_ipa.sort();
        unknown_ipa.dedup();
        md += &format!("\n**Unmapped IPA symbols (dropped, check the table):** {}\n", unknown_ipa.join(" "));
        eprintln!("WARNING unmapped IPA: {}", unknown_ipa.join(" "));
    }
    fs::write(format!("{out}.md"), md).unwrap();
    fs::write(format!("{out}.json"), json).unwrap();
    eprintln!("wrote {out}.md and {out}.json ({} entries)", entries.len());

    if let Some(cmd_file) = get("--commands") {
        commands(&fs::read_to_string(cmd_file).expect("commands file"), &dict, &format!("{out}-commands.md"));
    }
}

// ---------- command-word quality (step 7) ----------

fn commands(text: &str, dict: &HashMap<String, Vec<Vec<String>>>, path: &str) {
    let plosive = |p: &str| matches!(strip(p).as_str(), "P" | "B" | "T" | "D" | "K" | "G");
    let mut md = String::from("# command-word sound check\n\n| word | phones | onset | plosives | stressed syllable closed by plosive | syllables |\n|---|---|---|---|---|---|\n");
    for w in text.split(|c: char| !c.is_alphabetic() && c != '\'').map(str::to_lowercase).filter(|w| w.len() > 1) {
        let Some(v) = dict.get(&w).and_then(|v| v.first()) else { continue };
        if md.contains(&format!("| {w} |")) {
            continue;
        }
        let onset: Vec<String> = v.iter().take_while(|p| !is_vowel(&strip(p))).map(|p| strip(p)).collect();
        let syl = v.iter().filter(|p| is_vowel(&strip(p))).count();
        let stressed = v.iter().position(|p| p.ends_with('1'));
        let closed = stressed.is_some_and(|i| {
            let coda: Vec<&String> = v[i + 1..].iter().take_while(|p| !is_vowel(&strip(p))).collect();
            // a plosive in the coda (before the next vowel; if a vowel follows, the last consonant is its onset)
            let followed = v.len() > i + 1 + coda.len();
            let coda_eff = if followed && !coda.is_empty() { &coda[..coda.len() - 1] } else { &coda[..] };
            coda_eff.iter().any(|p| plosive(p))
        });
        md += &format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            w,
            v.join(" "),
            if onset.is_empty() { "(vowel)".into() } else { onset.join("") },
            v.iter().filter(|p| plosive(p)).count(),
            if closed { "yes" } else { "no" },
            syl
        );
    }
    fs::write(path, md).unwrap();
    eprintln!("wrote {path}");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ph(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }
    fn rules() -> Vec<Rule> {
        load_rules(include_str!("../data/mergers.txt"))
    }
    #[test]
    fn mergers_collapse_what_accents_collapse() {
        let r = rules();
        assert_eq!(collapse(&ph("K AA T"), &r), collapse(&ph("K AO T"), &r)); // cot/caught
        assert_eq!(collapse(&ph("M EH R IY"), &r), collapse(&ph("M AE R IY"), &r)); // merry/marry
        assert_eq!(collapse(&ph("M EY R IY"), &r), collapse(&ph("M EH R IY"), &r)); // Mary/merry
        assert_eq!(collapse(&ph("P IH N"), &r), collapse(&ph("P EH N"), &r)); // pin/pen
        assert_eq!(collapse(&ph("L AE T ER"), &r), collapse(&ph("L AE D ER"), &r)); // latter/ladder
    }
    #[test]
    fn mergers_keep_real_contrasts() {
        let r = rules();
        assert_ne!(collapse(&ph("SH IH P"), &r), collapse(&ph("SH IY P"), &r)); // ship/sheep
        assert_ne!(collapse(&ph("K AH T"), &r), collapse(&ph("K AE T"), &r)); // cut/cat
        assert_ne!(collapse(&ph("P IH T"), &r), collapse(&ph("P EH T"), &r)); // pit/pet (no nasal)
    }
    #[test]
    fn distances() {
        let r = rules();
        let c = |s: &str| collapse(&ph(s), &r);
        assert_eq!(wdist(&c("G AE M B AH L"), &c("G AE M B AH L")), 0.0);
        let mater_matter = wdist(&c("M EY T ER"), &c("M AE T ER"));
        assert!(mater_matter > 0.0 && mater_matter <= NEAR, "{mater_matter}");
        assert!(wdist(&c("M AH K W IY N"), &c("M IY N")) > NEAR);
    }
    #[test]
    fn ipa_mapping() {
        let mut u = vec![];
        assert_eq!(ipa_to_arpa("ɡˈæmbəl", &mut u), ph("G AE M B AH L"));
        assert!(u.is_empty());
    }
}
