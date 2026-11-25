//! Nearby context words as evidence for a name (John, 2026-11-23/24). Open code,
//! not wired into decisions: today's rules and UltraBERT already got all 27
//! senses of John's 24-line read right, so the tie-breaker had nothing to fix.
//! It stays callable for when the evidence is a knowledge graph rather than
//! a part-of-speech pair: whatever supplies the context words (a hand-approved
//! `context:` list, corrections, a graph's neighbours), this finds them.
//!
//! The rule: a context word next to the span, or within `window` words, and in
//! the same sentence when `same_sentence` is set. The sentence boundary is what
//! keeps "Do not gamble with her health" a verb after a sentence about the dog.
//! Study: `~/syncthing/transcrust-work/context-tiebreaker-11-23/`.

use std::collections::{BTreeMap, BTreeSet};

/// The choices, kept as data so a caller can sweep them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Words on each side of the span that count (John: next to, or within 2-4).
    pub window: usize,
    /// Stop at `.`, `?`, `!` and line breaks.
    pub same_sentence: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { window: 4, same_sentence: true }
    }
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn sentence_end(c: char) -> bool {
    matches!(c, '.' | '?' | '!' | '\n')
}

/// The words around the byte range `span` of `text`, within the options' reach.
fn around(text: &str, span: std::ops::Range<usize>, opts: Options) -> (Vec<String>, Vec<String>) {
    let mut before = &text[..span.start];
    let mut after = &text[span.end..];
    if opts.same_sentence {
        before = before.rfind(sentence_end).map_or(before, |i| &before[i + 1..]);
        after = after.find(sentence_end).map_or(after, |i| &after[..i]);
    }
    let b = words(before);
    let a = words(after);
    let b = b[b.len().saturating_sub(opts.window)..].to_vec();
    let a = a[..a.len().min(opts.window)].to_vec();
    (b, a)
}

/// The context words found near `span` in `text`, sorted. Matching ignores case.
pub fn near(text: &str, span: std::ops::Range<usize>, context: &[&str], opts: Options) -> Vec<String> {
    let wanted: BTreeSet<String> = context.iter().map(|w| w.to_lowercase()).collect();
    let (b, a) = around(text, span, opts);
    b.into_iter()
        .chain(a)
        .filter(|w| wanted.contains(w))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Candidate context words for `name`: every word within reach of each exact,
/// case-sensitive occurrence of `name` in `texts`, with how many texts it came
/// from. A proposal for a person to approve, never a list to use as is: the
/// 11-23 harvest gave Gambol "dog" and "puppy", but also "love" and "took".
pub fn harvest(texts: &[&str], name: &str, stop: &[&str], opts: Options) -> BTreeMap<String, usize> {
    let skip: BTreeSet<String> = stop
        .iter()
        .map(|w| w.to_lowercase())
        .chain(words(name))
        .collect();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for text in texts {
        let mut in_text = BTreeSet::new();
        for (at, _) in text.match_indices(name) {
            let (b, a) = around(text, at..at + name.len(), opts);
            in_text.extend(b.into_iter().chain(a).filter(|w| !skip.contains(w)));
        }
        for w in in_text {
            *seen.entry(w).or_default() += 1;
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(text: &str, word: &str) -> std::ops::Range<usize> {
        let at = text.find(word).expect("word in text");
        at..at + word.len()
    }

    const DOG: &[&str] = &["dog", "puppy", "shepherd", "vet"];

    #[test]
    fn a_context_word_within_the_window_counts() {
        let t = "My puppy Gambol hates the vacuum cleaner.";
        assert_eq!(near(t, span(t, "Gambol"), DOG, Options::default()), ["puppy"]);
        let t = "His favourite dog is our Australian shepherd Gamble.";
        assert_eq!(near(t, span(t, "Gamble"), DOG, Options { window: 2, ..Options::default() }), ["shepherd"]);
    }

    #[test]
    fn the_sentence_boundary_keeps_the_verb() {
        let t = "The vet said Gambol needs a smaller dinner. Do not gamble with her health.";
        let s = span(t, "gamble");
        assert!(near(t, s.clone(), DOG, Options::default()).is_empty());
        // Without the boundary, "vet" from the sentence before would pull it to the name.
        let open = Options { same_sentence: false, window: 10 };
        assert_eq!(near(t, s, DOG, open), ["vet"]);
    }

    #[test]
    fn harvest_counts_texts_and_skips_stop_words_and_the_name() {
        let texts = ["My dog Gambol slept.", "I love my dog Gambol.", "Gambol ran to the park."];
        let got = harvest(&texts, "Gambol", &["my", "i", "to", "the"], Options::default());
        assert_eq!(got.get("dog"), Some(&2));
        assert_eq!(got.get("love"), Some(&1));
        assert!(!got.contains_key("gambol") && !got.contains_key("my"));
    }
}
