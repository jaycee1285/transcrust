//! Modes: a model plus a post-transcription profile.
//!
//! Until now an "engine" was a model directory, and the tray switched between
//! directories. `Granite — Long` breaks that: it is the *same* directory with a
//! different profile. So the switchable unit is a [`Mode`] — `(model, profile)`
//! — and the model layer goes back to being only about what is on disk.
//!
//! Switching between two modes that share a model does not rebuild the
//! service, so toggling Granite ↔ Granite—Long is instant and never reloads
//! 527 MB.

use crate::model::{discover_models, InstalledModel, ModelKind};

/// What happens to the engine's raw text before the shared
/// `postprocess::fix_transcription` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// Inject what the engine said. Correct for Parakeet, which already emits
    /// contractions, casing and punctuation.
    Raw,
    /// Long-form repair for engines that emit bare, system-shaped text.
    ///
    /// This is the experiment surface. Today it is a deterministic contraction
    /// pass; the intended replacement is a learned normaliser (s1-mini or
    /// similar) that can also resolve false starts and self-corrections, which
    /// no deterministic rule can. Keeping a measurable deterministic baseline
    /// here is the point — it is what the learned version has to beat.
    Long,
}

impl Profile {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Raw => "",
            Self::Long => " — Long",
        }
    }
}

/// One switchable entry in the tray and on the toggle hotkey.
#[derive(Clone, Debug)]
pub struct Mode {
    pub label: String,
    pub model: InstalledModel,
    pub profile: Profile,
}

/// Every mode available on this machine, best-first.
///
/// Base modes come straight from `discover_models`, preserving its ordering, so
/// an unpinned install still resolves to Parakeet. A `— Long` variant is
/// offered only for engines whose raw output actually needs repair: Parakeet
/// emits grown-up English natively and would gain nothing but risk.
pub fn discover_modes(config_path: Option<&str>) -> Vec<Mode> {
    let mut modes = Vec::new();
    for model in discover_models(config_path) {
        modes.push(Mode {
            label: model.label.clone(),
            model: model.clone(),
            profile: Profile::Raw,
        });
        if wants_long_profile(model.kind) {
            modes.push(Mode {
                label: format!("{}{}", model.label, Profile::Long.suffix()),
                model,
                profile: Profile::Long,
            });
        }
    }
    modes
}

/// Granite's CTC head emits bare lowercase with expanded contractions — it is
/// built to feed a pipeline, not a reader. Parakeet emits finished prose, so a
/// repair pass on it is all downside.
fn wants_long_profile(kind: ModelKind) -> bool {
    matches!(kind, ModelKind::Granite)
}

/// Apply a mode's profile to raw engine text, before the shared pipeline.
pub fn apply_profile(profile: Profile, text: &str) -> String {
    match profile {
        Profile::Raw => text.to_string(),
        Profile::Long => restore_contractions(text),
    }
}

/// Expansions that are safe to reverse regardless of what follows.
///
/// The exclusions are the interesting part, and they are why a deterministic
/// pass can only go so far:
///
/// * `I have` → `I've` is wrong before a noun (`I have a car`) and right before
///   a participle (`I have heard` → `I've heard`). Telling those apart needs
///   part-of-speech, so `have`/`has`/`had` as main verbs are left alone.
/// * `let us` → `let's` breaks `let us know`.
/// * Spoken numerals (`3 songs` → `three songs`) are left alone too: the right
///   answer depends on whether it is a count, a date, a version or a quantity,
///   and getting it wrong is worse than leaving it.
///
/// A learned normaliser handles all three. That gap is the measurement this
/// mode exists to make.
const CONTRACTIONS: &[(&str, &str)] = &[
    // Negations — safe in every context.
    ("can not", "can't"),
    ("cannot", "can't"),
    ("could not", "couldn't"),
    ("did not", "didn't"),
    ("does not", "doesn't"),
    ("do not", "don't"),
    ("had not", "hadn't"),
    ("has not", "hasn't"),
    ("have not", "haven't"),
    ("is not", "isn't"),
    ("are not", "aren't"),
    ("was not", "wasn't"),
    ("were not", "weren't"),
    ("will not", "won't"),
    ("would not", "wouldn't"),
    ("should not", "shouldn't"),
    // Pronoun + be / will / would — also unambiguous.
    ("I am", "I'm"),
    ("I will", "I'll"),
    ("I would", "I'd"),
    ("you are", "you're"),
    ("we are", "we're"),
    ("they are", "they're"),
    ("it is", "it's"),
    ("that is", "that's"),
    ("there is", "there's"),
];

/// Case-preserving whole-phrase replacement. Matches on word boundaries so
/// `it is` inside `bit isolated` is untouched.
fn restore_contractions(text: &str) -> String {
    let mut out = text.to_string();
    for (expanded, contracted) in CONTRACTIONS {
        out = replace_phrase(&out, expanded, contracted);
    }
    out
}

fn replace_phrase(text: &str, phrase: &str, replacement: &str) -> String {
    let lower = text.to_lowercase();
    let phrase_lower = phrase.to_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;

    while let Some(found) = lower[cursor..].find(&phrase_lower) {
        let start = cursor + found;
        let end = start + phrase.len();
        let before_ok = start == 0 || !is_word_char(text[..start].chars().next_back());
        let after_ok = end >= text.len() || !is_word_char(text[end..].chars().next());
        if before_ok && after_ok {
            out.push_str(&text[cursor..start]);
            // Keep the original's leading case: "It is" -> "It's", not "it's".
            let original_leads_upper = text[start..end]
                .chars()
                .next()
                .is_some_and(char::is_uppercase);
            if original_leads_upper {
                let mut chars = replacement.chars();
                if let Some(first) = chars.next() {
                    out.extend(first.to_uppercase());
                    out.push_str(chars.as_str());
                }
            } else {
                out.push_str(replacement);
            }
            cursor = end;
        } else {
            out.push_str(&text[cursor..end]);
            cursor = end;
        }
    }
    out.push_str(&text[cursor..]);
    out
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.is_some_and(|c| c.is_alphanumeric() || c == '\'')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_profile_is_a_pass_through() {
        let text = "I can not wait. It is awesome.";
        assert_eq!(apply_profile(Profile::Raw, text), text);
    }

    /// The exact shape Granite produced on the bench clip.
    #[test]
    fn long_profile_repairs_granite_shaped_output() {
        let granite = "it is awesome I can not wait I know I am freaking out you are a very good actor";
        assert_eq!(
            apply_profile(Profile::Long, granite),
            "it's awesome I can't wait I know I'm freaking out you're a very good actor"
        );
    }

    #[test]
    fn leading_case_is_preserved() {
        assert_eq!(restore_contractions("It is fine"), "It's fine");
        assert_eq!(restore_contractions("I am here"), "I'm here");
        assert_eq!(restore_contractions("Can not do it"), "Can't do it");
    }

    /// Word boundaries: the phrase must not match inside another word.
    #[test]
    fn substrings_inside_words_are_untouched() {
        assert_eq!(restore_contractions("a bit isolated"), "a bit isolated");
        assert_eq!(restore_contractions("scan nothing"), "scan nothing");
    }

    /// The ambiguous cases are deliberately skipped; a learned normaliser is
    /// the only thing that can decide them.
    #[test]
    fn ambiguous_expansions_are_left_alone() {
        assert_eq!(restore_contractions("I have a car"), "I have a car");
        assert_eq!(restore_contractions("I have heard it"), "I have heard it");
        assert_eq!(restore_contractions("let us know"), "let us know");
        assert_eq!(restore_contractions("I have 3 songs"), "I have 3 songs");
    }

    #[test]
    fn repeated_phrases_are_all_replaced() {
        assert_eq!(
            restore_contractions("it is what it is"),
            "it's what it's"
        );
    }

    /// Parakeet already emits contractions; running the pass on its output must
    /// be a no-op rather than a double-contraction.
    #[test]
    fn already_contracted_text_is_unchanged() {
        let parakeet = "It's awesome. I can't wait. I'm freaking out.";
        assert_eq!(restore_contractions(parakeet), parakeet);
    }

    #[test]
    fn only_granite_offers_a_long_variant() {
        assert!(wants_long_profile(ModelKind::Granite));
        assert!(!wants_long_profile(ModelKind::Parakeet));
    }

    #[test]
    fn long_modes_are_labelled_distinctly() {
        assert_eq!(Profile::Raw.suffix(), "");
        assert_eq!(Profile::Long.suffix(), " — Long");
    }
}
