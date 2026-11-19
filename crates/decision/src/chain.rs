//! Deterministic homophone chain, rules 1-5 (design-vocab-gate.md section 5).
//!
//! Port of the Python v3 reference (tools/rules-probe: rules_probe.py rules 1-5, variants.py
//! v2-local rule 5 and v2-gloss rule 3, variants_v3.py vocative list check). Labels, rule names
//! and reason strings match Python exactly; `chain-v3-golden.json` pins that.
//!
//! Additions not in Python, in [`decide`] only: [`Label::Prior`] ("no context, no model") for a
//! span that is the whole sentence (before the rules) or has one context word (after them), and
//! user-facing pass wording.
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    Name,
    Keep,
    /// No rule decided; the decision model is next.
    Pass,
    /// Sentence has no context beyond the span: the per-speaker prior decides, model skipped.
    Prior,
}

impl Label {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Keep => "keep",
            Self::Pass => "pass",
            Self::Prior => "prior",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub label: Label,
    pub rule: Option<&'static str>,
    pub reason: String,
}

// ---------------------------------------------------------------- word lists (verbatim)
const VERB_CAPABLE: &[&str] = &[
    "gamble", "flow", "run", "walk", "talk", "play", "work", "call", "help", "turn", "move",
    "live", "look", "see", "make", "take", "give", "find", "think", "know", "want", "need",
    "like", "love", "hate", "try", "ask", "use", "feel", "leave", "keep", "let", "begin",
    "seem", "show", "hear", "stand", "drink", "eat", "sleep", "write", "read", "sing", "dance",
    "jump", "swim", "fly", "drive", "ride", "cook", "bake", "draw", "paint", "build", "break",
    "fix", "cut", "hit", "push", "pull", "throw", "catch", "carry", "drop", "fall", "rise",
    "climb", "wait", "stop", "start", "finish", "open", "close", "watch", "wash", "clean",
    "laugh", "smile", "cry", "shout", "whisper", "bark", "race", "rush", "drift", "glide",
    "float", "pour", "stream", "vote", "bet", "wager", "shuffle", "deal", "win", "lose", "pay",
    "buy", "sell",
];
const COUNT_NOUNS: &[&str] = &[
    "gamble", "bandit", "dog", "cat", "horse", "bird", "car", "truck", "boat", "plane", "train",
    "bike", "book", "table", "chair", "house", "door", "window", "phone", "tool", "toy", "ball",
    "box", "bag", "hat", "shoe", "coat", "cup", "plate", "bowl", "knife", "robber", "thief",
    "pirate", "cowboy", "sheriff", "soldier", "teacher", "doctor", "farmer", "driver", "player",
    "river", "hill", "tree", "flower", "stone", "rock", "stick", "kite",
];
const DETERMINERS: &[&str] = &[
    "a", "an", "the", "this", "that", "these", "those", "some", "any", "each", "every",
    "another", "no", "one", "two", "three", "many", "few", "several", "both", "either",
    "neither", "my", "your", "his", "her", "its", "our", "their",
];
const ARTICLES: &[&str] = &[
    "a", "an", "the", "this", "that", "these", "those", "some", "any", "each", "every",
    "another",
];
const VERB_PREV: &[&str] = &[
    "to", "will", "would", "can", "could", "should", "may", "might", "must", "shall", "won't",
    "can't", "don't", "didn't", "doesn't", "wouldn't", "couldn't", "shouldn't", "wont", "cant",
    "dont", "didnt", "doesnt", "wouldnt", "couldnt", "shouldnt", "gonna",
];
const ADVERBS: &[&str] = &[
    "not", "never", "always", "just", "also", "really", "quickly", "slowly", "often", "still",
    "even", "only",
];
const SUBJECT_PRONOUNS: &[&str] = &[
    "i", "we", "you", "they",
];
const VERB_NEXT: &[&str] = &[
    "on", "away", "into",
];
const NAME_PREV: &[&str] = &[
    "walk", "call", "called", "named", "meet", "hey", "hi", "hello", "dear", "mr", "mrs", "ms",
    "miss", "dr", "uncle", "aunt",
];
const RELATIONAL: &[&str] = &[
    "dog", "cat", "puppy", "pet", "son", "daughter", "brother", "sister", "wife", "husband",
    "mom", "dad", "mother", "father", "friend", "colleague", "neighbor", "teacher", "cousin",
    "boss",
];
const ANIMATE_VERBS: &[&str] = &[
    "said", "says", "barked", "barks", "won", "wins", "ate", "eats", "likes", "liked", "loves",
    "loved", "wants", "wanted", "thinks", "thought", "asked", "asks", "told", "tells", "smiled",
    "laughed", "cried", "shouted", "whispered", "knew", "knows", "felt", "feels", "decided",
    "decides", "wondered", "promised",
];
const INANIMATE_NOUNS: &[&str] = &[
    "flow", "lightning", "gamble", "chili", "bingo", "six", "water", "storm", "rain", "weather",
    "wind", "river", "label", "meter", "sky", "thunder",
];

const FRANCHISES: &[(&str, &[&str], &[&str])] = &[
    (
        "cars",
        &["cars movie"],
        &[
            "lightning mcqueen", "mcqueen", "mater", "v8 cafe", "radiator springs", "piston cup",
            "doc hudson", "cars characters", "cars character", "cars movie", "cars film",
        ],
    ),
    ("bluey", &["bluey"], &["bluey", "heeler"]),
];

const ANIMATE_DESC: &[&str] = &[
    "dog", "pet", "cat", "person", "mom", "dad", "son", "friend", "character", "kid", "child", "girl",
    "boy", "puppy",
];

const R1: &str = "1 inflection";
const R2: &str = "2 grammar slots*";
const R3: &str = "3 bare count noun*";
const R4: &str = "4 animacy";
const R5: &str = "5 fixed phrases*";
const R6: &str = "6 speaker prior";

fn has(list: &[&str], w: &str) -> bool {
    list.contains(&w)
}

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).expect("chain regex"))
}
fn re_sent_end() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"[.!?]\s+")
}
fn re_sent_cut() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"[.!?]")
}
fn re_clause() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"[,;:]")
}
fn re_word() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"[A-Za-z0-9'\u{2019}]+")
}
fn re_poss() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"^['\u{2019}]s\b")
}
fn re_pause() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"^\s*(,|\u{2026}|\.\.\.|!)")
}
fn re_alpha() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, r"[a-z]+")
}
fn re_animate_desc() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    re(&C, &format!(r"\b({})\b", ANIMATE_DESC.join("|")))
}
fn re_list_after() -> &'static Regex {
    static C: OnceLock<Regex> = OnceLock::new();
    let item = r"[A-Za-z0-9'\u{2019}-]+(?:\s+[A-Za-z0-9'\u{2019}-]+){0,2}";
    re(&C, &format!(r"(?i)^\s*,\s*(?:(?:{item})\s*,\s*)*(?:and|or)\b"))
}

fn words(s: &str) -> Vec<String> {
    re_word().find_iter(s).map(|m| m.as_str().to_lowercase()).collect()
}

/// Parsed view of one marked occurrence (Python `Ctx`).
struct Ctx {
    heard: String,
    hl: String,
    canonical: String,
    desc: String,
    prev: Vec<String>,
    next: Vec<String>,
    after_raw: String,
    clause_start: bool,
    /// Words of the current sentence other than the span.
    sentence_words: usize,
    /// Current sentence with the span replaced by NUL, lowercased (v2-local rule 5).
    sentence_text: String,
}

impl Ctx {
    fn new(text: &str, canonical: &str, description: &str) -> Option<Ctx> {
        let a = text.find('\u{ab}')?;
        let b = text.find('\u{bb}')?;
        if b < a {
            return None;
        }
        let heard = &text[a + '\u{ab}'.len_utf8()..b];
        let before = &text[..a];
        let after = &text[b + '\u{bb}'.len_utf8()..];
        let sent_before = re_sent_end().split(before).last().unwrap_or("");
        let sent_after = re_sent_cut().split(after).next().unwrap_or("");
        let prev = words(re_clause().split(sent_before).last().unwrap_or(""));
        let next = words(re_clause().split(sent_after).next().unwrap_or(""));
        Some(Ctx {
            heard: heard.to_string(),
            hl: heard.to_lowercase(),
            canonical: canonical.to_string(),
            desc: description.to_lowercase(),
            clause_start: prev.is_empty(),
            prev,
            next,
            after_raw: after.to_string(),
            sentence_words: words(sent_before).len() + words(sent_after).len(),
            sentence_text: format!("{sent_before} \0 {sent_after}").to_lowercase(),
        })
    }
}

type Hit = Option<(Label, String)>;

fn rule1_inflection(c: &Ctx) -> Hit {
    let h = &c.hl;
    if h.contains(' ') {
        return None;
    }
    let hlen = h.chars().count();
    for suf in ["ing", "ed", "s"] {
        if h.ends_with(suf) && hlen - suf.len() >= 3 && *h != c.canonical.to_lowercase() {
            let stem = &h[..h.len() - suf.len()];
            let sc: Vec<char> = stem.chars().collect();
            let mut stems: BTreeSet<String> = BTreeSet::new();
            stems.insert(stem.to_string());
            stems.insert(format!("{stem}e"));
            if sc.len() > 3 && sc[sc.len() - 1] == sc[sc.len() - 2] {
                stems.insert(sc[..sc.len() - 1].iter().collect());
            }
            if suf == "ed" {
                stems.insert(h[..h.len() - 1].to_string());
            }
            if let Some(v) = stems.iter().find(|s| has(VERB_CAPABLE, s)) {
                return Some((
                    Label::Keep,
                    format!(
                        "kept \"{}\": the ending \"-{suf}\" on the verb \"{v}\" marks an inflected common word, which cannot be a name",
                        c.heard
                    ),
                ));
            }
        }
    }
    None
}

fn prev_verb_cue(c: &Ctx) -> Option<&str> {
    let p = &c.prev;
    if let Some(l) = p.last() {
        if has(VERB_PREV, l) {
            return Some(l);
        }
    }
    if p.len() >= 2 && has(ADVERBS, &p[p.len() - 1]) && has(VERB_PREV, &p[p.len() - 2]) {
        return Some(&p[p.len() - 2]);
    }
    None
}

fn rule2_slots(c: &Ctx, clause_start: bool) -> Hit {
    let mut keep: Vec<String> = Vec::new();
    let mut name: Vec<String> = Vec::new();
    let verb_capable = has(VERB_CAPABLE, &c.hl);
    if let (Some(cue), true) = (prev_verb_cue(c), verb_capable) {
        keep.push(format!("verb slot after \"{cue}\""));
    }
    if let (true, Some(l)) = (verb_capable, c.prev.last()) {
        if has(SUBJECT_PRONOUNS, l) {
            keep.push(format!("verb slot after subject pronoun \"{l}\""));
        }
    }
    if let (true, Some(n)) = (verb_capable, c.next.first()) {
        if has(VERB_NEXT, n) {
            keep.push(format!("verb slot before \"{n}\""));
        }
    }
    let start = c.prev.len().saturating_sub(2);
    for w in &c.prev[start..] {
        if has(ARTICLES, w) {
            keep.push(format!("common-noun slot after \"{w}\""));
            break;
        }
    }
    if let Some(l) = c.prev.last() {
        if has(NAME_PREV, l) {
            name.push(format!("name slot after \"{l}\""));
        }
    }
    if re_poss().is_match(&c.after_raw) {
        name.push("possessive 's follows".to_string());
    }
    if clause_start && re_pause().is_match(&c.after_raw) {
        name.push("clause-initial call followed by a pause mark (vocative)".to_string());
    }
    if let Some(l) = c.prev.last() {
        if has(RELATIONAL, l) && re_alpha().find_iter(&c.desc).any(|m| m.as_str() == l) {
            name.push(format!(
                "appositive after \"{l}\", which the glossary gives for {}",
                c.canonical
            ));
        }
    }
    if !keep.is_empty() && !name.is_empty() {
        return None;
    }
    if !keep.is_empty() {
        return Some((Label::Keep, format!("kept \"{}\": {}", c.heard, keep.join("; "))));
    }
    if !name.is_empty() {
        return Some((Label::Name, format!("named \"{}\": {}", c.canonical, name.join("; "))));
    }
    None
}

/// v3: a comma-list after the span means it is not a vocative.
fn rule2_v3(c: &Ctx) -> Hit {
    let sent_after = re_sent_cut().split(&c.after_raw).next().unwrap_or("");
    if c.clause_start && re_list_after().is_match(sent_after) {
        return rule2_slots(c, false);
    }
    rule2_slots(c, c.clause_start)
}

/// v2-gloss: a bare count noun names only if the glossary says the term is a person/animal/character.
fn rule3_gloss(c: &Ctx) -> Hit {
    if !has(COUNT_NOUNS, &c.hl) {
        return None;
    }
    let start = c.prev.len().saturating_sub(3);
    if c.prev[start..].iter().any(|w| has(DETERMINERS, w) || w.ends_with("'s") || w.ends_with("\u{2019}s")) {
        return None;
    }
    if !re_animate_desc().is_match(&c.desc) {
        return None;
    }
    Some((
        Label::Name,
        format!(
            "named \"{}\": \"{}\" is a singular count noun with no determiner, which a common noun cannot be here",
            c.canonical, c.heard
        ),
    ))
}

fn rule4_animacy(c: &Ctx) -> Hit {
    if !has(INANIMATE_NOUNS, &c.hl) {
        return None;
    }
    let mut n = &c.next[..];
    if let Some(f) = n.first() {
        if has(ADVERBS, f) {
            n = &n[1..];
        }
    }
    if let Some(f) = n.first() {
        if has(ANIMATE_VERBS, f) {
            return Some((
                Label::Name,
                format!(
                    "named \"{}\": \"{f}\" needs a living subject and \"{}\" is not one",
                    c.canonical, c.heard
                ),
            ));
        }
    }
    None
}

/// v2-local: markers are searched in the current sentence only.
fn rule5_local(c: &Ctx) -> Hit {
    for (fname, desc_keys, markers) in FRANCHISES {
        if !desc_keys.iter().any(|k| c.desc.contains(k)) {
            continue;
        }
        for m in *markers {
            let pat = format!(r"\b{}\b", regex::escape(m));
            if Regex::new(&pat).map_or(false, |r| r.is_match(&c.sentence_text)) {
                return Some((
                    Label::Name,
                    format!("named \"{}\": {fname} marker phrase \"{m}\" appears in the same text", c.canonical),
                ));
            }
        }
    }
    None
}

const PASSED: &str = "passed";

fn pass() -> Decision {
    Decision { label: Label::Pass, rule: None, reason: PASSED.to_string() }
}

fn run_rules(c: &Ctx) -> Option<Decision> {
    let rules: [(&'static str, &dyn Fn(&Ctx) -> Hit); 5] = [
        (R1, &rule1_inflection),
        (R2, &rule2_v3),
        (R3, &rule3_gloss),
        (R4, &rule4_animacy),
        (R5, &rule5_local),
    ];
    for (name, f) in rules {
        if let Some((label, reason)) = f(c) {
            return Some(Decision { label, rule: Some(name), reason });
        }
    }
    None
}

/// Rules 1-5 only, exactly the Python v3 chain (undecided = `Pass`, reason "passed").
/// `sentence` holds the text with the span marked as `«span»` (a preceding context
/// sentence is tolerated; every rule looks at the current sentence). `description` is the
/// glossary description of `canonical`.
pub fn decide_rules(sentence: &str, canonical: &str, description: &str) -> Decision {
    match Ctx::new(sentence, canonical, description) {
        Some(c) => run_rules(&c).unwrap_or_else(pass),
        None => pass(),
    }
}

/// Rules 1-5, then the "no context, no model" rule (TASKBOARD 2026-11-18; not in Python):
/// if no rule decided and the sentence has fewer than 2 words besides the span, the per-speaker
/// prior decides and the model is skipped.
///
/// A span that is the whole sentence ("Bingo!", "Flo!") goes to the prior *before* the rules
/// (John, 2026-11-18): a lone exclamation looks vocative to rule 2 but carries nothing to judge.
/// Until a per-speaker prior exists, callers leave a `Prior` span as heard.
pub fn decide(sentence: &str, canonical: &str, description: &str) -> Decision {
    let Some(c) = Ctx::new(sentence, canonical, description) else {
        return to_model();
    };
    if c.sentence_words == 0 {
        return prior();
    }
    if let Some(d) = run_rules(&c) {
        return d;
    }
    if c.sentence_words < 2 {
        return prior();
    }
    to_model()
}

fn prior() -> Decision {
    Decision {
        label: Label::Prior,
        rule: Some(R6),
        reason: "no context in the sentence; speaker prior decides".to_string(),
    }
}

// User-facing wording for a pass; `decide_rules` keeps Python's "passed" for parity.
fn to_model() -> Decision {
    Decision {
        label: Label::Pass,
        rule: None,
        reason: "no rule was confident; the decision model decides".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const GOLDEN: &str = include_str!("../../../tools/rules-probe/chain-v3-golden.json");

    #[test]
    fn parity_with_python_v3() {
        let cases: Vec<Value> = serde_json::from_str(GOLDEN).unwrap();
        assert_eq!(cases.len(), 150);
        let mut bad = Vec::new();
        for c in &cases {
            let g = |k: &str| c[k].as_str().unwrap();
            let d = decide_rules(g("text"), g("canonical"), g("description"));
            let want_rule = c["rule"].as_str();
            if d.label.as_str() != g("label") || d.rule != want_rule || d.reason != g("reason") {
                bad.push(format!("{}/{}/{}: got {:?}, want {} {:?} {}", g("file"), g("id"), g("arm"), d, g("label"), want_rule, g("reason")));
            }
        }
        assert!(bad.is_empty(), "{} mismatches:\n{}", bad.len(), bad.join("\n"));
    }

        #[test]
    fn prior_when_no_context() {
        for t in ["\u{ab}Bingo\u{bb}.", "\u{ab}Gamble\u{bb}", "Earlier sentence here. \u{ab}Chili\u{bb}.", "Hey \u{ab}flow\u{bb}"] {
            let d = decide(t, "Bingo", "Bluey's 4 year old sister");
            if t.starts_with("Hey") {
                assert_eq!(d.label, Label::Name, "{t}"); // a rule can still decide (name slot after hey)
                continue;
            }
            assert_eq!(d.label, Label::Prior, "{t}");
            assert_eq!(d.reason, "no context in the sentence; speaker prior decides");
            assert_eq!(decide_rules(t, "Bingo", "Bluey's 4 year old sister").label, Label::Pass, "{t}");
        }
    }

    #[test]
    fn prior_not_applied_with_context_or_when_rule_decides() {
        // two words besides the span: no prior, falls through to the model
        let d = decide("we saw \u{ab}Bingo\u{bb}", "Bingo", "Bluey's 4 year old sister");
        assert_eq!(d.label, Label::Pass);
        // one word besides the span: prior
        let d = decide("saw \u{ab}Bingo\u{bb}", "Bingo", "Bluey's 4 year old sister");
        assert_eq!(d.label, Label::Prior);
        // a rule decides even with a single context word ("to" verb slot -> keep)
        let d = decide("to \u{ab}gamble\u{bb}", "Gambol", "pet dog");
        assert_eq!((d.label, d.rule), (Label::Keep, Some("2 grammar slots*")));
        // vocative with context: rule 2 still decides
        let d = decide("\u{ab}Flow\u{bb}, come here", "Flo", "Cars movie character");
        assert_eq!(d.label, Label::Name);
        // pass wording is user-facing in decide(), Python's in decide_rules()
        let d = decide("we saw \u{ab}Bingo\u{bb}", "Bingo", "Bluey's 4 year old sister");
        assert_eq!(d.reason, "no rule was confident; the decision model decides");
    }

    #[test]
    fn lone_exclamation_goes_to_prior_before_the_vocative_rule() {
        for t in ["\u{ab}Bingo\u{bb}!", "\u{ab}Flo\u{bb}!", "We left. \u{ab}Bingo\u{bb}!"] {
            assert_eq!(decide_rules(t, "Bingo", "Bluey's 4 year old sister").label, Label::Name, "{t}");
            let d = decide(t, "Bingo", "Bluey's 4 year old sister");
            assert_eq!((d.label, d.rule), (Label::Prior, Some(R6)), "{t}");
        }
    }
}
