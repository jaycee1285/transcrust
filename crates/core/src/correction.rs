//! Platform-independent correction data and conservative vocabulary proposals.
//! No window, clipboard, keyboard or model dependencies belong here.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dictation {
    /// Stable take ID, shared with corpus WAV/JSON and correction history.
    #[serde(default)]
    pub id: String,
    pub raw: String,
    pub delivered: String,
    pub corrected: Option<String>,
    /// Current text in the destination; Copy does not change this span.
    pub buffer_text: String,
    pub mode: String,
    pub audio_secs: f32,
    pub wait_secs: f32,
    /// Only true when typing into the destination succeeded.
    pub replaceable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub heard: String,
    pub canonical: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub before: String,
    pub after: String,
}

impl Change {
    pub fn pair(&self) -> Pair {
        let clean = |s: &str| {
            s.trim()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        };
        Pair {
            heard: clean(&self.before),
            canonical: clean(&self.after),
        }
    }
}

/// Compare text read back from the destination selection with the saved span.
/// Only an exact suffix can be extended safely; no text may be replaced until
/// this returns an empty prefix. Empty/unrelated selections are not evidence
/// of a valid caret position and must never cause deletion.
pub fn unselected_prefix<'a>(expected: &'a str, selected: &str) -> Result<&'a str, &'static str> {
    if expected.is_empty() || selected.is_empty() {
        return Err("The editor did not return a nonempty selection");
    }
    expected
        .strip_suffix(selected)
        .ok_or("The editor selection does not match the saved dictation")
}

/// Complete edit review, independent of vocabulary eligibility. Whitespace is
/// retained, including punctuation, insertions, deletions and long rewrites.
/// Very large edits fall back to one complete change instead of disappearing.
pub fn changes(before: &str, after: &str) -> Vec<Change> {
    let a: Vec<&str> = before.split_inclusive(char::is_whitespace).collect();
    let b: Vec<&str> = after.split_inclusive(char::is_whitespace).collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let a = &a[prefix..a.len() - suffix];
    let b = &b[prefix..b.len() - suffix];
    if a.len().saturating_mul(b.len()) > 1_000_000 {
        return vec![Change {
            before: a.concat(),
            after: b.concat(),
        }];
    }
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i] == b[j] {
                1 + lcs[i + 1][j + 1]
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            i += 1;
            j += 1;
            continue;
        }
        let (start_i, start_j) = (i, j);
        while i < a.len() || j < b.len() {
            if i < a.len() && j < b.len() && a[i] == b[j] {
                break;
            }
            if i < a.len() && (j == b.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
                i += 1;
            } else {
                j += 1;
            }
        }
        out.push(Change {
            before: a[start_i..i].concat(),
            after: b[start_j..j].concat(),
        });
    }
    out
}

/// Legacy glossary-only callers. The review UI uses `changes` and shows every
/// edit; selecting a vocabulary phrase is an explicit, separate human action.
pub fn proposals(before: &str, after: &str) -> Vec<Pair> {
    let mut out = Vec::new();
    for change in changes(before, after) {
        let pair = change.pair();
        if valid_pair(&pair) && !out.contains(&pair) {
            out.push(pair);
        }
    }
    out
}

/// Single English function-word swaps are sentence edits, not vocabulary
/// identities. Keep their correction history without teaching a name rule.
pub fn grammar_rewrite(pair: &Pair) -> bool {
    let word = |s: &str| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "a" | "an"
                | "and"
                | "or"
                | "in"
                | "on"
                | "at"
                | "to"
                | "of"
                | "for"
                | "it"
                | "i"
                | "is"
                | "as"
                | "the"
        )
    };
    pair.canonical
        .chars()
        .next()
        .is_some_and(char::is_lowercase)
        && word(&pair.heard)
        && word(&pair.canonical)
}

/// The correction only adds capitals the heard text lacked ("shepherd" → "Shepherd").
/// Casing is produced (John, 2026-11-20), so a deliberate capital can be learned: it
/// creates the entry and never an alias, since matching is case-insensitive anyway.
pub fn case_promotion(pair: &Pair) -> bool {
    let shape = |s: &str| {
        !s.is_empty()
            && s.chars().count() <= 256
            && s.split_whitespace().count() <= 12
            && s.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '\'' | '’' | '-'))
    };
    shape(&pair.heard)
        && pair.heard != pair.canonical
        && pair.heard.to_lowercase() == pair.canonical.to_lowercase()
        && pair
            .heard
            .chars()
            .zip(pair.canonical.chars())
            .any(|(h, c)| h.is_lowercase() && c.is_uppercase())
        && !pair
            .heard
            .chars()
            .zip(pair.canonical.chars())
            .any(|(h, c)| h.is_uppercase() && c.is_lowercase())
}

/// What the Learn checkbox accepts: an alias pair, or a capitalization promotion.
pub fn learnable(pair: &Pair) -> bool {
    valid_pair(pair) || case_promotion(pair)
}

pub fn valid_pair(pair: &Pair) -> bool {
    let valid = |s: &str| {
        !s.is_empty()
            && s.chars().count() <= 256
            && s.split_whitespace().count() <= 12
            && s.chars().any(char::is_alphanumeric)
            && s.chars()
                .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '\'' | '’' | '-'))
    };
    valid(&pair.heard)
        && valid(&pair.canonical)
        && !grammar_rewrite(pair)
        && pair.heard.to_lowercase() != pair.canonical.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_verifies_the_original_span_including_punctuation() {
        let original = "We're going home.";
        assert_eq!(unselected_prefix(original, original), Ok(""));
        assert_eq!(unselected_prefix(original, "re going home."), Ok("We'"));
        assert_eq!(unselected_prefix(original, "e're going home."), Ok("W"));
        assert!(unselected_prefix(original, "We're going home").is_err());
        assert!(unselected_prefix(original, "We're going home. ").is_err());
        assert!(unselected_prefix(original, "").is_err());
        assert!(unselected_prefix(original, "different text").is_err());
        assert!(unselected_prefix("", "").is_err());
        assert_eq!(unselected_prefix("🐕 Café.", "Café."), Ok("🐕 "));
    }
    #[test]
    fn split_names_and_separate_mishearing_spans() {
        assert_eq!(
            proposals(
                "Open app image then apt track.",
                "Open AppImage then Apptrack."
            ),
            vec![
                Pair {
                    heard: "app image".into(),
                    canonical: "AppImage".into()
                },
                Pair {
                    heard: "apt track".into(),
                    canonical: "Apptrack".into()
                },
            ]
        );
    }
    #[test]
    fn prose_edits_do_not_teach_vocabulary() {
        for (a, b) in [
            ("hello world", "Hello world."),
            ("hello world", "hello lovely world"),
            ("hello lovely world", "hello world"),
        ] {
            assert!(proposals(a, b).is_empty(), "{a} -> {b}");
        }
    }
    #[test]
    fn review_keeps_long_edits_insertions_deletions_case_and_punctuation() {
        for (before, after) in [
            (
                "he was a red racer taught by a driver",
                "Todd drives a bright red racing car",
            ),
            ("hello world", "Hello, world!"),
            ("hello world", "hello lovely world"),
            ("hello lovely world", "hello world"),
            ("one two three", "123"),
            ("same  words", "same words"),
        ] {
            assert!(
                !changes(before, after).is_empty(),
                "discarded {before:?} -> {after:?}"
            );
        }
        assert!(changes("same text.", "same text.").is_empty());
        let large = changes(&"before ".repeat(1100), &"after ".repeat(1100));
        assert_eq!(large.len(), 1);
        assert_eq!(large[0].before, "before ".repeat(1100));
        assert_eq!(large[0].after, "after ".repeat(1100));
        assert!(valid_pair(&Pair {
            heard: "one two three".into(),
            canonical: "123".into()
        }));
        assert!(!valid_pair(&Pair {
            heard: "flow, maybe".into(),
            canonical: "Flo".into()
        }));
    }
    #[test]
    fn grammar_edits_remain_visible_without_becoming_name_rules() {
        assert!(!changes("and he came", "in he came").is_empty());
        for (heard, canonical) in [("and", "in"), ("in", "and"), ("I", "it")] {
            assert!(!valid_pair(&Pair {
                heard: heard.into(),
                canonical: canonical.into()
            }));
        }
        assert!(valid_pair(&Pair {
            heard: "flow".into(),
            canonical: "Flo".into()
        }));
    }
    #[test]
    fn long_unchanged_context_does_not_hide_a_short_edit() {
        let context = "word ".repeat(1000);
        assert_eq!(
            proposals(
                &format!("{context}kubernetis"),
                &format!("{context}Kubernetes")
            )
            .len(),
            1
        );
    }
}
