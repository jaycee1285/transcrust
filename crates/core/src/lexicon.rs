//! English word membership from the CMU Pronouncing Dictionary headwords
//! (lowercase, BSD-2; see data/cmu-words.LICENSE). Shared by the fused-name split
//! and by vocabulary learning, which files an alias by whether it is a real word.
use std::collections::HashSet;
use std::sync::OnceLock;

const CMU: &str = include_str!("../data/cmu-words.txt");

fn words() -> &'static HashSet<&'static str> {
    static WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    WORDS.get_or_init(|| CMU.lines().collect())
}

/// Exact CMU headword; `lower` must already be lowercase.
pub fn contains(lower: &str) -> bool {
    words().contains(lower)
}

/// A CMU headword, or a regular inflection of one (CMU lists "unicorn", not
/// "unicorns"). Stems need three or more letters.
pub fn is_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    if contains(&lower) {
        return true;
    }
    for suffix in ["ing", "ed", "es", "s", "d"] {
        if let Some(stem) = lower.strip_suffix(suffix) {
            if stem.len() >= 3 && contains(stem) {
                return true;
            }
            if suffix == "ing" && stem.len() >= 2 && contains(&format!("{stem}e")) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn words_inflections_and_coinages() {
        for w in ["Louie", "really", "unicorns", "chili", "chased", "Flowing"] {
            assert!(is_word(w), "{w}");
        }
        for w in ["Blooie", "Jilly", "uniclause", "kubernetis", "Nelland"] {
            assert!(!is_word(w), "{w}");
        }
    }
}
