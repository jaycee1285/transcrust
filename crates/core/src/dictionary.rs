//! Phonetic vocabulary correction — the leg ported from murmure.
//!
//! A small local transducer mis-hears jargon, names, and code identifiers it
//! was never trained on ("Kubernetes", "Svelte", a contact's surname). This
//! pass collapses phonetic neighbourhoods onto a user-supplied dictionary: any
//! transcribed word whose Beider-Morse phonetic code collides with a dictionary
//! entry is replaced by that entry. Learned entries (those with `heard:` pairs)
//! use bounded decoder weighting instead; only their exact spelling/casing is
//! normalized here, so post-processing cannot override the decoder choice.
//!
//! Differences from murmure's `dictionary.rs`, all deliberate:
//!   * **No language set.** murmure ran the french+english rule subset. This is
//!     a single-user English app, so we use rphonetic's embedded `any` generic
//!     rules via plain `encode()` — language-agnostic and French-free.
//!   * **No shipped rule files.** The `embedded_bm` feature compiles the BM
//!     rules into the binary, so there is no `cc-rules/` resource directory to
//!     locate at runtime.
//!   * **Word-boundary-safe replacement.** murmure used `String::replace`, which
//!     can rewrite a token that appears as a substring of a larger word. We walk
//!     the text token by token and swap only whole words, preserving all
//!     surrounding punctuation and whitespace.
//!
//! If the dictionary file is absent or empty, this pass is a no-op and the text
//! is returned unchanged — the feature is opt-in.

use crate::correction::Pair;
use rphonetic::{BeiderMorse, BeiderMorseBuilder, ConfigFiles, Encoder};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

static BOOST_STRENGTH: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub fn set_boost_strength(value: f32) {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        1.0
    };
    BOOST_STRENGTH.store(value.to_bits(), Ordering::Relaxed);
}

pub fn boost_strength() -> f32 {
    f32::from_bits(BOOST_STRENGTH.load(Ordering::Relaxed))
}

pub fn boost_terms() -> Vec<String> {
    parse_dictionary(&std::fs::read_to_string(dictionary_path()).unwrap_or_default())
}

/// Words shorter than this are never phonetically corrected. Short tokens
/// ("is", "the", "a") collide with far too much under approximate matching;
/// real jargon is longer.
const MIN_WORD_LEN: usize = 3;

/// Embedded Beider-Morse rules (generic name type, `any`/`common` languages).
static CONFIG: LazyLock<ConfigFiles> = LazyLock::new(ConfigFiles::default);

// File reads are cheap; phonetic encoding is cached until the contents change.
static CACHE: Mutex<Option<(PathBuf, String, Corrector)>> = Mutex::new(None);
static PATH: RwLock<Option<PathBuf>> = RwLock::new(None);

pub fn configure(path: Option<PathBuf>) {
    *PATH.write().unwrap() = path;
}

struct Corrector {
    /// (canonical word, its `|`-split phonetic codes)
    encoded: Vec<(String, Vec<String>)>,
}

impl Corrector {
    fn from_contents(contents: &str) -> Self {
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let learned: Vec<_> = alias_pairs(contents)
            .into_iter()
            .map(|p| p.canonical)
            .collect();
        Self {
            encoded: parse_dictionary(contents)
                .into_iter()
                .map(|word| {
                    // Learned spellings are decoder preferences. Fuzzy replacement
                    // would override a deliberate choice of an ordinary word.
                    let codes = if learned.contains(&word) {
                        Vec::new()
                    } else {
                        split_codes(&bm.encode(&word))
                    };
                    (word, codes)
                })
                .collect(),
        }
    }
}

/// Path to the user dictionary: `~/syncthing/john-vocab.md`, the one vocabulary
/// shared with mopvox and dayshade.
pub fn dictionary_path() -> PathBuf {
    if let Some(path) = PATH.read().unwrap().as_ref() {
        return path.clone();
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syncthing")
        .join("john-vocab.md")
}

/// How many terms the dictation pass would load, for `doctor`.
pub fn term_count() -> Option<usize> {
    std::fs::read_to_string(dictionary_path())
        .ok()
        .map(|c| parse_dictionary(&c).len())
}

/// Kinds that are dayshade's alone. Habit and task names are short and
/// ordinary — `Cigs` would claim every `six` — so dictation never sees them.
const DAYSHADE_ONLY: &[&str] = &["habit", "task"];

/// Each `## Term` at column 0 of the shared vocabulary is an entry, kept unless
/// its `kind:` is dayshade's alone. A file with no `## ` headings is the old
/// `dictionary.txt`: one term per line, `#` comments and blank lines ignored.
fn parse_dictionary(contents: &str) -> Vec<String> {
    let heading = |line: &str| {
        line.strip_prefix("##")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .map(str::trim)
            .filter(|term| !term.is_empty())
            .map(str::to_string)
    };
    if !contents.lines().any(|line| heading(line).is_some()) {
        return contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect();
    }

    let mut out = Vec::new();
    let mut term: Option<String> = None;
    let mut kind = String::new();
    let mut flush = |term: &mut Option<String>, kind: &mut String| {
        if let Some(t) = term.take() {
            if !DAYSHADE_ONLY.contains(&kind.as_str()) {
                out.push(t);
            }
        }
        kind.clear();
    };
    for line in contents.lines() {
        if let Some(t) = heading(line) {
            flush(&mut term, &mut kind);
            term = Some(t);
        } else if line.starts_with('#') {
            flush(&mut term, &mut kind);
        } else if let Some(k) = line
            .trim_start_matches(['-', '*'])
            .trim_start()
            .strip_prefix("kind:")
        {
            if !line.starts_with(char::is_whitespace) {
                kind = k.trim().to_lowercase();
            }
        }
    }
    flush(&mut term, &mut kind);
    out
}

/// Kept heard/canonical pairs are learning records, not substitution rules.
fn alias_pairs(contents: &str) -> Vec<Pair> {
    let mut out = Vec::new();
    let mut term = String::new();
    let mut kind = String::new();
    let mut heard = Vec::new();
    let flush = |out: &mut Vec<Pair>, term: &str, kind: &str, heard: &Vec<String>| {
        if !DAYSHADE_ONLY.contains(&kind) {
            for alias in heard {
                let pair = Pair {
                    heard: alias.clone(),
                    canonical: term.to_string(),
                };
                if crate::correction::valid_pair(&pair) {
                    out.push(pair);
                }
            }
        }
    };
    for line in contents.lines().chain(std::iter::once("# end")) {
        if line.starts_with('#') {
            flush(&mut out, &term, &kind, &heard);
            term = line.strip_prefix("## ").unwrap_or("").trim().to_string();
            kind.clear();
            heard.clear();
        } else if !line.starts_with(char::is_whitespace) {
            let line = line.trim_start_matches(['-', '*']).trim_start();
            if let Some(value) = line.strip_prefix("kind:") {
                kind = value.trim().to_lowercase();
            }
            if let Some(value) = line.strip_prefix("heard:") {
                heard.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                );
            }
        }
    }
    out
}

/// Re-read and merge before writing, preserving unrelated entries and prose.
/// Same-directory atomic rename prevents a reader seeing a half-written file.
pub fn save_pairs(path: &std::path::Path, pairs: &[Pair]) -> Result<(), String> {
    if pairs.is_empty() {
        return Ok(());
    }
    if pairs.iter().any(|p| !crate::correction::valid_pair(p)) {
        return Err("Invalid vocabulary proposal".into());
    }
    let mut contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "# Vocabulary\n".to_string(),
        Err(e) => return Err(e.to_string()),
    };
    // Avoid mixing the legacy one-term-per-line format with Markdown sections.
    if !contents.lines().any(|l| l.starts_with("## ")) {
        let terms = parse_dictionary(&contents);
        if !terms.is_empty() {
            contents = format!(
                "# Vocabulary\n{}",
                terms
                    .iter()
                    .map(|t| format!("\n## {t}\nkind: term\n"))
                    .collect::<String>()
            );
        }
    }
    for pair in pairs {
        let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
        if let Some(start) = lines.iter().position(|l| {
            l.strip_prefix("## ")
                .is_some_and(|t| t.trim().eq_ignore_ascii_case(&pair.canonical))
        }) {
            let end = (start + 1..lines.len())
                .find(|&i| lines[i].starts_with('#'))
                .unwrap_or(lines.len());
            let excluded = lines[start + 1..end].iter().any(|l| {
                l.trim_start_matches(['-', '*'])
                    .trim_start()
                    .strip_prefix("kind:")
                    .is_some_and(|k| DAYSHADE_ONLY.contains(&k.trim().to_lowercase().as_str()))
            });
            if excluded {
                return Err(format!(
                    "{} is a habit/task entry; dictation cannot learn aliases for it",
                    pair.canonical
                ));
            }
            if let Some(index) = (start + 1..end).find(|&i| lines[i].starts_with("heard:")) {
                let existing = lines[index].strip_prefix("heard:").unwrap().trim();
                if !existing
                    .split(',')
                    .any(|h| h.trim().eq_ignore_ascii_case(&pair.heard))
                {
                    lines[index] = if existing.is_empty() {
                        format!("heard: {}", pair.heard)
                    } else {
                        format!("heard: {existing}, {}", pair.heard)
                    };
                }
            } else {
                lines.insert(end, format!("heard: {}", pair.heard));
            }
        } else {
            lines.extend([
                String::new(),
                format!("## {}", pair.canonical),
                "kind: term".into(),
                format!("heard: {}", pair.heard),
            ]);
        }
        contents = lines.join("\n") + "\n";
    }
    let parent = path
        .parent()
        .ok_or("Vocabulary path needs a parent directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = parent.join(format!(
        ".transcrust-vocab-{}-{nonce}.tmp",
        std::process::id()
    ));
    let result = (|| {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        if let Ok(metadata) = std::fs::metadata(path) {
            file.set_permissions(metadata.permissions())
                .map_err(|e| e.to_string())?;
        }
        file.write_all(contents.as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

fn split_codes(encoded: &str) -> Vec<String> {
    encoded.split('|').map(str::to_string).collect()
}

/// Replace any transcribed word that phonetically matches a dictionary entry.
/// All whitespace and punctuation between words is preserved exactly.
pub fn correct(text: &str) -> String {
    let path = dictionary_path();
    let contents = std::fs::read_to_string(&path).unwrap_or_default();
    let mut cache = CACHE.lock().unwrap();
    if cache
        .as_ref()
        .is_none_or(|(p, c, _)| p != &path || c != &contents)
    {
        *cache = Some((path, contents.clone(), Corrector::from_contents(&contents)));
    }
    correct_with(text, &cache.as_ref().unwrap().2)
}

/// The body of [`correct`], with the dictionary passed in so tests can supply
/// one instead of depending on the user's `dictionary.txt`.
fn correct_with(text: &str, corrector: &Corrector) -> String {
    if corrector.encoded.is_empty() {
        return text.to_string();
    }

    let bm = BeiderMorseBuilder::new(&CONFIG).build();
    let mut result = String::with_capacity(text.len());
    let mut word = String::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '\'' {
            word.push(ch);
        } else {
            if !word.is_empty() {
                result.push_str(&map_word_with(&word, &bm, corrector));
                word.clear();
            }
            result.push(ch);
        }
    }
    if !word.is_empty() {
        result.push_str(&map_word_with(&word, &bm, corrector));
    }

    result
}

/// Return the dictionary replacement for `word` if one collides phonetically,
/// otherwise `word` unchanged. `corrector` is passed in rather than read from
/// the global so tests can supply a dictionary of their own.
fn map_word_with(word: &str, bm: &BeiderMorse, corrector: &Corrector) -> String {
    if word.chars().count() < MIN_WORD_LEN {
        return word.to_string();
    }

    // Exact match is identity, not a collision, so it gets its own full pass.
    // Take the dictionary's casing: leaving engine casing in place would diverge
    // the two ASR backends, since Parakeet TDT emits "Wayland" itself and
    // Granite's CTC head emits "wayland", and only the dictionary knows which
    // spelling is canonical. Sentence-initial case is not a concern —
    // `capitalize_sentences` runs after this pass.
    //
    // This runs before the phonetic pass rather than inside it because a single
    // interleaved loop only protects an exact match from entries *after* it: an
    // earlier entry's phonetic neighbourhood could claim the word first, and
    // which entry won depended on dictionary line order.
    for (dict_word, _) in &corrector.encoded {
        if word.eq_ignore_ascii_case(dict_word) {
            return dict_word.clone();
        }
    }

    // The phonetic pass exists to rescue rare words the model never learned. A
    // word that is already ordinary English was not mis-heard, so nothing here
    // can improve it and a collision can only do damage — `Tauri` in the
    // dictionary claimed every `they're`, deterministically, until this guard.
    // Length is the wrong axis for that: `Tauri` is 5 characters and `they're`
    // is 7.
    if is_common_english(word) {
        return word.to_string();
    }

    let candidate = bm.encode(word);
    if candidate.is_empty() {
        return word.to_string();
    }
    let candidate_codes = split_codes(&candidate);

    for (dict_word, dict_codes) in &corrector.encoded {
        if dict_codes
            .iter()
            .any(|dc| candidate_codes.iter().any(|cc| cc == dc))
        {
            return dict_word.clone();
        }
    }

    word.to_string()
}

/// Is this a word ordinary enough that the model was not guessing at it?
///
/// Case- and apostrophe-sensitive only in that the comparison is lowercased;
/// `COMMON_ENGLISH` carries the contraction forms explicitly because they are
/// the ones that actually collided.
fn is_common_english(word: &str) -> bool {
    let lowered = word.to_lowercase();
    COMMON_ENGLISH.binary_search(&lowered.as_str()).is_ok()
}

/// High-frequency English, sorted for binary search.
///
/// Deliberately short. This is a guard against the corrector eating ordinary
/// speech, not a spellchecker: it needs the words a dictation engine emits
/// constantly, and every entry it does not need is a rare word it might block.
/// Blocking fails safe — the word passes through uncorrected.
const COMMON_ENGLISH: &[&str] = &[
    "about",
    "after",
    "again",
    "all",
    "also",
    "always",
    "and",
    "another",
    "any",
    "anything",
    "are",
    "aren't",
    "around",
    "back",
    "because",
    "been",
    "before",
    "being",
    "best",
    "better",
    "between",
    "both",
    "but",
    "call",
    "called",
    "can",
    "can't",
    "come",
    "could",
    "couldn't",
    "did",
    "didn't",
    "different",
    "does",
    "doesn't",
    "doing",
    "don't",
    "done",
    "down",
    "each",
    "else",
    "enough",
    "even",
    "ever",
    "every",
    "everything",
    "far",
    "few",
    "find",
    "first",
    "for",
    "from",
    "get",
    "give",
    "going",
    "good",
    "got",
    "great",
    "had",
    "hadn't",
    "has",
    "hasn't",
    "have",
    "haven't",
    "having",
    "her",
    "here",
    "here's",
    "hers",
    "him",
    "his",
    "how",
    "however",
    "i'd",
    "i'll",
    "i'm",
    "i've",
    "into",
    "isn't",
    "it's",
    "its",
    "just",
    "keep",
    "kind",
    "know",
    "last",
    "least",
    "less",
    "let",
    "let's",
    "life",
    "like",
    "little",
    "long",
    "look",
    "lot",
    "made",
    "make",
    "many",
    "maybe",
    "mean",
    "might",
    "more",
    "most",
    "much",
    "must",
    "need",
    "never",
    "new",
    "next",
    "not",
    "nothing",
    "now",
    "off",
    "often",
    "old",
    "once",
    "one",
    "only",
    "onto",
    "other",
    "our",
    "ours",
    "out",
    "over",
    "own",
    "part",
    "people",
    "per",
    "place",
    "point",
    "put",
    "quite",
    "rather",
    "real",
    "really",
    "right",
    "said",
    "same",
    "say",
    "see",
    "seem",
    "seen",
    "set",
    "she",
    "she's",
    "should",
    "shouldn't",
    "show",
    "side",
    "since",
    "some",
    "something",
    "still",
    "such",
    "sure",
    "take",
    "than",
    "that",
    "that's",
    "the",
    "their",
    "theirs",
    "them",
    "then",
    "there",
    "there's",
    "these",
    "they",
    "they'd",
    "they'll",
    "they're",
    "they've",
    "thing",
    "things",
    "think",
    "this",
    "those",
    "though",
    "thought",
    "three",
    "through",
    "time",
    "too",
    "two",
    "under",
    "until",
    "use",
    "used",
    "using",
    "very",
    "want",
    "was",
    "wasn't",
    "way",
    "we'd",
    "we'll",
    "we're",
    "we've",
    "well",
    "went",
    "were",
    "weren't",
    "what",
    "what's",
    "when",
    "where",
    "whether",
    "which",
    "while",
    "who",
    "why",
    "will",
    "with",
    "within",
    "without",
    "won't",
    "work",
    "would",
    "wouldn't",
    "yes",
    "yet",
    "you",
    "you'd",
    "you'll",
    "you're",
    "you've",
    "your",
    "yours",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_and_blanks() {
        let raw = "# names\nKubernetes\n\n  Svelte  \n# trailing\n";
        assert_eq!(parse_dictionary(raw), vec!["Kubernetes", "Svelte"]);
    }

    /// The shared vocabulary: habits and tasks are dayshade's; an indented
    /// example in the prose is not an entry.
    #[test]
    fn shared_vocabulary_keeps_habits_and_tasks_out() {
        let md = "# John's vocabulary\n\n    ## Apptrack\n    kind: task\n\n\
                  ## Wayland\nkind: term\n\n## Cigs\nkind: habit\nheard: sigs, six\n\n\
                  ## Apptrack\nkind: task\n\n## Gambol\n- kind: person\n\n## Tauri\n";
        assert_eq!(parse_dictionary(md), vec!["Wayland", "Gambol", "Tauri"]);
    }

    #[test]
    fn learned_pairs_preserve_phrases_and_exclude_dayshade_names() {
        let md = "## AppImage\nkind: term\nheard: app image\n\n## Cigs\nkind: habit\nheard: six\n";
        assert_eq!(
            alias_pairs(md),
            vec![Pair {
                heard: "app image".into(),
                canonical: "AppImage".into()
            }]
        );
    }

    #[test]
    fn kept_pairs_merge_preserve_and_reload_without_restart() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("transcrust-pairs-{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vocab.md");
        std::fs::write(&path, "# Vocabulary\nKeep this prose.\n\n## Kubernetes\nkind: term\nheard: kubernetties\n\n## Cigs\nkind: habit\nheard: six\n").unwrap();
        let pair = Pair {
            heard: "kubernetis".into(),
            canonical: "Kubernetes".into(),
        };
        let old = std::fs::read_to_string(&path).unwrap();
        assert!(!alias_pairs(&old).contains(&pair));
        save_pairs(&path, &[pair.clone()]).unwrap();
        save_pairs(&path, &[pair.clone()]).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("Keep this prose."));
        assert!(saved.contains("heard: kubernetties, kubernetis"));
        assert!(saved.contains("## Cigs\nkind: habit\nheard: six"));
        assert_eq!(saved.matches("kubernetis").count(), 1);
        assert!(alias_pairs(&saved).contains(&pair));
        let blocked = Pair {
            heard: "sigs".into(),
            canonical: "Cigs".into(),
        };
        assert!(save_pairs(&path, &[blocked]).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn learned_homophone_does_not_override_the_decoder_choice() {
        let corrector = Corrector::from_contents("## Gambol\nkind: person\nheard: gamble\n");
        let text = "I love my dog Gambol. I like to gamble and play poker with cards.";
        assert_eq!(correct_with(text, &corrector), text);
        assert_eq!(
            correct_with("I like to gamble. I like gambling.", &corrector),
            "I like to gamble. I like gambling."
        );
        assert_eq!(correct_with("my dog gambol", &corrector), "my dog Gambol");
    }

    fn empty_corrector() -> Corrector {
        Corrector {
            encoded: Vec::new(),
        }
    }

    #[test]
    fn empty_dictionary_is_identity() {
        // With no entries this pass must be a faithful pass-through, punctuation
        // and all. Built explicitly rather than relying on the test process
        // having no `~/.config/transcrust/dictionary.txt`.
        let input = "deploy to kubernetes, please.";
        assert_eq!(correct_with(input, &empty_corrector()), input);
    }

    #[test]
    fn phonetic_match_replaces_word() {
        // Build a corrector by hand so the test doesn't depend on a file.
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let dict_word = "Kubernetes".to_string();
        let dict_codes = split_codes(&bm.encode(&dict_word));

        // A plausible mis-hearing should share at least one phonetic code.
        let heard = "kubernetis";
        let heard_codes = split_codes(&bm.encode(heard));
        let collides = dict_codes
            .iter()
            .any(|dc| heard_codes.iter().any(|cc| cc == dc));
        assert!(
            collides,
            "expected '{heard}' to phonetically collide with '{dict_word}'"
        );
    }

    #[test]
    fn exact_match_takes_dictionary_casing() {
        // Engine parity: Parakeet TDT emits "Wayland", Granite's CTC head emits
        // "wayland". Both must land on the dictionary's canonical spelling.
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let dict_word = "Wayland".to_string();
        let dict_codes = split_codes(&bm.encode(&dict_word));
        let corrector = Corrector {
            encoded: vec![(dict_word, dict_codes)],
        };
        for heard in ["wayland", "WAYLAND", "Wayland"] {
            assert_eq!(map_word_with(heard, &bm, &corrector), "Wayland");
        }
    }

    #[test]
    fn preserves_newlines_and_punctuation() {
        // With no dictionary loaded this is identity; the point is that the
        // tokenizer never collapses structure.
        let input = "line one\n\nline two — done.";
        assert_eq!(correct_with(input, &empty_corrector()), input);
    }

    /// `binary_search` on an unsorted slice silently returns wrong answers, so
    /// the ordering of `COMMON_ENGLISH` is load-bearing, not cosmetic.
    #[test]
    fn common_english_list_is_sorted_and_lowercase() {
        for pair in COMMON_ENGLISH.windows(2) {
            assert!(
                pair[0] < pair[1],
                "COMMON_ENGLISH must be sorted: {:?} precedes {:?}",
                pair[0],
                pair[1]
            );
        }
        for word in COMMON_ENGLISH {
            assert_eq!(*word, word.to_lowercase(), "{word} must be lowercase");
        }
    }

    /// The regression this guard exists for. `Tauri` in the dictionary claimed
    /// every `they're` — 7 hits and 0 survivors across one transcript — because
    /// the two share a Beider-Morse code. The collision is real; the guard is
    /// what stops it mattering.
    #[test]
    fn common_words_survive_a_colliding_dictionary_entry() {
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let dict_word = "Tauri".to_string();
        let dict_codes = split_codes(&bm.encode(&dict_word));
        let corrector = Corrector {
            encoded: vec![(dict_word, dict_codes)],
        };

        for heard in ["they're", "There", "their", "were", "where"] {
            assert_eq!(
                map_word_with(heard, &bm, &corrector),
                heard,
                "'{heard}' is ordinary English and must pass through untouched"
            );
        }
        // The entry still does its actual job on a word that is not English.
        assert_eq!(map_word_with("Tauri", &bm, &corrector), "Tauri");
    }

    /// An exact match must win no matter where it sits in the file. Before the
    /// exact-match pass was hoisted out, an earlier entry's phonetic
    /// neighbourhood could claim the word and the winner depended on line order.
    #[test]
    fn exact_match_wins_regardless_of_dictionary_order() {
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let encode = |w: &str| (w.to_string(), split_codes(&bm.encode(w)));
        // "Tauri" first, so its phonetic neighbourhood gets first refusal.
        let corrector = Corrector {
            encoded: vec![encode("Tauri"), encode("Torrey")],
        };
        assert_eq!(map_word_with("Torrey", &bm, &corrector), "Torrey");
    }
}
