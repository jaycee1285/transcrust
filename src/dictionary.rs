//! Phonetic vocabulary correction — the leg ported from murmure.
//!
//! A small local transducer mis-hears jargon, names, and code identifiers it
//! was never trained on ("Kubernetes", "Svelte", a contact's surname). This
//! pass collapses phonetic neighbourhoods onto a user-supplied dictionary: any
//! transcribed word whose Beider-Morse phonetic code collides with a dictionary
//! entry is replaced by that entry.
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

use rphonetic::{BeiderMorse, BeiderMorseBuilder, ConfigFiles, Encoder};
use std::path::PathBuf;
use std::sync::LazyLock;

/// Words shorter than this are never phonetically corrected. Short tokens
/// ("is", "the", "a") collide with far too much under approximate matching;
/// real jargon is longer.
const MIN_WORD_LEN: usize = 3;

/// Embedded Beider-Morse rules (generic name type, `any`/`common` languages).
static CONFIG: LazyLock<ConfigFiles> = LazyLock::new(ConfigFiles::default);

/// The user dictionary, pre-encoded to phonetic codes once per process.
/// Editing `dictionary.txt` takes effect on the next run.
static CORRECTOR: LazyLock<Corrector> = LazyLock::new(Corrector::load);

struct Corrector {
    /// (canonical word, its `|`-split phonetic codes)
    encoded: Vec<(String, Vec<String>)>,
}

impl Corrector {
    fn load() -> Self {
        let words = load_dictionary();
        if words.is_empty() {
            return Self {
                encoded: Vec::new(),
            };
        }
        let bm = BeiderMorseBuilder::new(&CONFIG).build();
        let encoded = words
            .into_iter()
            .map(|word| {
                let codes = split_codes(&bm.encode(&word));
                (word, codes)
            })
            .collect();
        Self { encoded }
    }
}

/// Path to the user dictionary: `~/syncthing/john-vocab.md`, the one vocabulary
/// shared with mopvox and dayshade.
pub fn dictionary_path() -> PathBuf {
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

fn load_dictionary() -> Vec<String> {
    let Ok(contents) = std::fs::read_to_string(dictionary_path()) else {
        return Vec::new();
    };
    parse_dictionary(&contents)
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

fn split_codes(encoded: &str) -> Vec<String> {
    encoded.split('|').map(str::to_string).collect()
}

/// Replace any transcribed word that phonetically matches a dictionary entry.
/// All whitespace and punctuation between words is preserved exactly.
pub fn correct(text: &str) -> String {
    correct_with(text, &CORRECTOR)
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
    "about", "after", "again", "all", "also", "always", "and", "another", "any", "anything",
    "are", "aren't", "around", "back", "because", "been", "before", "being", "best", "better",
    "between", "both", "but", "call", "called", "can", "can't", "come", "could", "couldn't",
    "did", "didn't", "different", "does", "doesn't", "doing", "don't", "done", "down", "each",
    "else", "enough", "even", "ever", "every", "everything", "far", "few", "find", "first",
    "for", "from", "get", "give", "going", "good", "got", "great", "had", "hadn't", "has",
    "hasn't", "have", "haven't", "having", "her", "here", "here's", "hers", "him", "his", "how",
    "however", "i'd", "i'll", "i'm", "i've", "into", "isn't", "it's", "its", "just", "keep",
    "kind", "know", "last", "least", "less", "let", "let's", "life", "like", "little", "long",
    "look", "lot", "made", "make", "many", "maybe", "mean", "might", "more", "most", "much",
    "must", "need", "never", "new", "next", "not", "nothing", "now", "off", "often", "old",
    "once", "one", "only", "onto", "other", "our", "ours", "out", "over", "own", "part",
    "people", "per", "place", "point", "put", "quite", "rather", "real", "really", "right",
    "said", "same", "say", "see", "seem", "seen", "set", "she", "she's", "should", "shouldn't",
    "show", "side", "since", "some", "something", "still", "such", "sure", "take", "than",
    "that", "that's", "the", "their", "theirs", "them", "then", "there", "there's", "these",
    "they", "they'd", "they'll", "they're", "they've", "thing", "things", "think", "this",
    "those", "though", "thought", "three", "through", "time", "too", "two", "under", "until",
    "use", "used", "using", "very", "want", "was", "wasn't", "way", "we'd", "we'll", "we're",
    "we've", "well", "went", "were", "weren't", "what", "what's", "when", "where", "whether",
    "which", "while", "who", "why", "will", "with", "within", "without", "won't", "work",
    "would", "wouldn't", "yes", "yet", "you", "you'd", "you'll", "you're", "you've", "your",
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
