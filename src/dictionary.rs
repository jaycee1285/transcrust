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

/// Path to the user dictionary: one term per line, `#` comments and blank lines
/// ignored. Lives next to `config.toml`.
pub fn dictionary_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("transcrust")
        .join("dictionary.txt")
}

fn load_dictionary() -> Vec<String> {
    let Ok(contents) = std::fs::read_to_string(dictionary_path()) else {
        return Vec::new();
    };
    parse_dictionary(&contents)
}

fn parse_dictionary(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
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

    let candidate = bm.encode(word);
    if candidate.is_empty() {
        return word.to_string();
    }
    let candidate_codes = split_codes(&candidate);

    for (dict_word, dict_codes) in &corrector.encoded {
        // Same word already — take the dictionary's casing and stop before any
        // phonetic neighbour can claim it. Returning `word` here instead would
        // leave engine casing in place, which diverges the two ASR backends:
        // Parakeet TDT emits "Wayland" itself, Granite's CTC head emits
        // "wayland", and only the dictionary knows which spelling is canonical.
        // Sentence-initial case is not a concern — `capitalize_sentences` runs
        // after this pass.
        if word.eq_ignore_ascii_case(dict_word) {
            return dict_word.clone();
        }
        if dict_codes
            .iter()
            .any(|dc| candidate_codes.iter().any(|cc| cc == dc))
        {
            return dict_word.clone();
        }
    }

    word.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_and_blanks() {
        let raw = "# names\nKubernetes\n\n  Svelte  \n# trailing\n";
        assert_eq!(parse_dictionary(raw), vec!["Kubernetes", "Svelte"]);
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
}
