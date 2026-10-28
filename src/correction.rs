//! Platform-independent correction data and conservative vocabulary proposals.
//! No window, clipboard, keyboard or model dependencies belong here.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dictation {
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

/// Word-level LCS. Only substitutions of 1–3 words are candidates. Punctuation,
/// case, insertions, deletions and long rewrites never teach the dictionary.
/// Shared prefix/suffix removal keeps long dictations with small edits cheap.
pub fn proposals(before: &str, after: &str) -> Vec<Pair> {
    let a: Vec<&str> = before.split_whitespace().collect();
    let b: Vec<&str> = after.split_whitespace().collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let a = &a[prefix..a.len() - suffix];
    let b = &b[prefix..b.len() - suffix];
    if a.len() > 512 || b.len() > 512 {
        return Vec::new();
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
        if (1..=3).contains(&(i - start_i)) && (1..=3).contains(&(j - start_j)) {
            let clean = |words: &[&str]| {
                words
                    .join(" ")
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            };
            let pair = Pair {
                heard: clean(&a[start_i..i]),
                canonical: clean(&b[start_j..j]),
            };
            if valid_pair(&pair) && !out.contains(&pair) {
                out.push(pair);
            }
        }
    }
    out
}

pub fn valid_pair(pair: &Pair) -> bool {
    let valid = |s: &str| {
        !s.is_empty()
            && s.chars().count() <= 64
            && s.split_whitespace().count() <= 3
            && s.chars().any(char::is_alphabetic)
            && s.chars()
                .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '\'' | '’' | '-'))
    };
    valid(&pair.heard)
        && valid(&pair.canonical)
        && pair.heard.to_lowercase() != pair.canonical.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
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
            (
                "I think we should go home",
                "Perhaps staying here would be better",
            ),
        ] {
            assert!(proposals(a, b).is_empty(), "{a} -> {b}");
        }
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
