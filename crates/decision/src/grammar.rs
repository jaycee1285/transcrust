//! Sounds-like candidates and the grammar that must license them (2026-11-20).
//!
//! ASR misses a name with a *real word* most of the time (bloey corpus: 59–71% of
//! missed names), so a closed alias list can't keep up. This module proposes a
//! registered single-word name wherever a token, or two adjacent tokens, share its
//! consonant skeleton, then asks local grammar whether the name is licensed there.
//! Mechanisms after the clean-room spec (`~/syncthing/linguistic-mechanisms.md`):
//!
//! - **M5:** after a preposition, a singular count noun with no determiner can't be a
//!   common noun ("ready for meter" → Mater).
//! - **Coordination:** joined by "and"/"or" to another registered name
//!   ("Bingo and Blue Wee" → Bluey).
//! - **No English reading:** the token is in neither Harper's dictionary nor CMU
//!   ("Totoroki" → Todoroki).
//! - **M10:** anything else keeps the transcript. A sounds-like candidate never
//!   reaches the model: no stated reason, no change.
use harper_core::spell::{Dictionary, FstDictionary};
use std::sync::{Arc, OnceLock};

fn dictionary() -> &'static Arc<FstDictionary> {
    static DICT: OnceLock<Arc<FstDictionary>> = OnceLock::new();
    DICT.get_or_init(FstDictionary::curated)
}

/// Consonant skeleton: vowels and glides dropped, voicing folded, doubles collapsed.
/// Totoroki and Todoroki → "ttrk"; meter, Mater and matter → "mtr"; "blue wee" → "pl".
pub fn skeleton(word: &str) -> String {
    let w: Vec<char> = word.to_lowercase().chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let mut out = String::new();
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        let next = w.get(i + 1).copied();
        let mapped = match (c, next) {
            ('s', Some('h')) | ('c', Some('h')) | ('t', Some('h')) | ('p', Some('h')) => {
                i += 1;
                match c { 'p' => Some('f'), 't' => Some('t'), _ => Some('s') }
            }
            ('c', Some('e' | 'i' | 'y')) => Some('s'),
            ('c' | 'k' | 'q' | 'g', _) => Some('k'),
            ('d' | 't', _) => Some('t'),
            ('b' | 'p', _) => Some('p'),
            ('v' | 'f', _) => Some('f'),
            ('z' | 's', _) => Some('s'),
            ('x', _) => { out.push('k'); Some('s') }
            ('a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'w' | 'h', _) => None,
            (other, _) => Some(other),
        };
        if let Some(m) = mapped {
            if !out.ends_with(m) {
                out.push(m);
            }
        }
        i += 1;
    }
    out
}

/// Research, opt-in (2026-11-21): polyg2p with English data built locally from CMUdict,
/// directory named by `TRANSCRUST_IPA_DATA`. Off when unset.
fn g2p() -> Option<&'static polyg2p::G2p> {
    static G2P: OnceLock<Option<polyg2p::G2p>> = OnceLock::new();
    G2P.get_or_init(|| {
        std::env::var_os("TRANSCRUST_IPA_DATA").map(|d| polyg2p::G2p::new(Some(std::path::Path::new(&d))))
    })
    .as_ref()
}

/// Phones of one word, stress and length marks removed. Lowercased first: polyg2p
/// spells out an unknown capitalized word letter by letter.
fn phones(word: &str) -> Option<Vec<char>> {
    let ipa = g2p()?.phonemize(&word.to_lowercase(), polyg2p::Lang::En);
    Some(ipa.chars().filter(|c| !matches!(c, 'ˈ' | 'ˌ' | 'ː') && !c.is_whitespace()).collect())
}

fn phone_distance(a: &[char], b: &[char]) -> usize {
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + (ca != cb) as usize).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

/// IPA-similars guard 2: a word Harper knows as a proper noun (Billy, Milly) is someone
/// else's name, never a mishearing of this one.
fn is_known_proper_noun(word: &str) -> bool {
    readings(word).or_else(|| readings(&word.to_lowercase())).is_some_and(|m| m.is_proper_noun())
}

fn readings(word: &str) -> Option<harper_core::DictWordMetadata> {
    dictionary().get_word_metadata_str(word).map(|m| m.into_owned())
}

fn has_english_reading(word: &str) -> bool {
    transcrust_core::lexicon::is_word(word) || readings(word).is_some()
}

/// A word that can open a noun phrase: a determiner, possessive, number or adjective.
fn opens_noun_phrase(word: &str) -> bool {
    word.chars().all(|c| c.is_ascii_digit())
        || readings(word).is_some_and(|m| m.is_determiner() || m.is_adjective() || m.is_possessive_pronoun())
}

fn is_preposition(word: &str) -> bool {
    word != "to" && readings(word).is_some_and(|m| m.preposition)
}

/// A singular count noun reading with no plural or mass-only reading, and not a proper noun.
fn singular_count_noun(word: &str) -> bool {
    readings(word).is_some_and(|m| {
        m.is_noun() && m.is_singular_noun() && m.is_countable_noun() && !m.is_mass_noun_only()
            && !m.is_plural_noun() && !m.is_proper_noun()
    })
}

/// Why a sounds-like candidate becomes the name, or `None` (M10: keep the transcript).
/// `prev`/`next` are the lowercase words of the same sentence around the span.
pub fn licenses_name(heard: &str, prev: &[String], next: &[String], names: &[String], canonical: &str) -> Option<String> {
    let lower = heard.to_lowercase();
    // A span that opens with a determiner is already a complete common noun phrase
    // ("a ball"): grammar licenses the ordinary reading, so the transcript stays (M10).
    if lower.split_whitespace().next().is_some_and(|w| readings(w).is_some_and(|m| m.is_determiner())) {
        return None;
    }
    let is_name = |w: &String| names.iter().any(|n| n.eq_ignore_ascii_case(w) && !n.eq_ignore_ascii_case(canonical));
    // Coordination: a conjunct of another registered name.
    let before = prev.len() >= 2 && matches!(prev[prev.len() - 1].as_str(), "and" | "or") && is_name(&prev[prev.len() - 2]);
    let after = next.len() >= 2 && matches!(next[0].as_str(), "and" | "or") && is_name(&next[1]);
    if before || after {
        let other = if before { &prev[prev.len() - 2] } else { &next[1] };
        return Some(format!("{heard:?} is joined to the name {other:?} by \"{}\"; names are said together", if before { &prev[prev.len() - 1] } else { &next[0] }));
    }
    // No English reading at all (single tokens only; a pair has spaces).
    if !lower.contains(' ') && !has_english_reading(&lower) {
        return Some(format!("{heard:?} is not an English word and has the same consonants as {canonical:?}"));
    }
    // M5: preposition + bare singular count noun.
    if !lower.contains(' ') {
        if let Some(p) = prev.last() {
            if is_preposition(p) && !opens_noun_phrase(p) && singular_count_noun(&lower) {
                return Some(format!(
                    "after {p:?}, {heard:?} would need a determiner as a common noun; with none, it reads as the name {canonical:?}"
                ));
            }
        }
    }
    None
}

/// Spans whose skeleton matches a single-word registered name: (start, end, canonical).
/// Single tokens need a 3+ consonant skeleton; pairs are joined and need 2+.
pub fn sounds_like_spans(text: &str, names: &[String]) -> Vec<(usize, usize, String)> {
    let tokens: Vec<(usize, usize)> = regex::Regex::new(r"[\p{L}']+")
        .unwrap()
        .find_iter(text)
        .map(|m| (m.start(), m.end()))
        .collect();
    let keyed: Vec<(String, String)> = names
        .iter()
        .filter(|n| !n.contains(' ') && n.chars().next().is_some_and(char::is_uppercase))
        .map(|n| (n.clone(), skeleton(n)))
        .filter(|(_, s)| s.len() >= 2)
        .collect();
    // IPA proposer (opt-in): names with 4+ phones, by phone distance ≤1.
    let spoken: Vec<(String, Vec<char>)> = if g2p().is_some() {
        keyed.iter().filter_map(|(n, _)| phones(n).filter(|p| p.len() >= 4).map(|p| (n.clone(), p))).collect()
    } else {
        Vec::new()
    };
    let mut out = Vec::new();
    for (i, &(a, b)) in tokens.iter().enumerate() {
        let word = &text[a..b];
        // Measurement switch: TRANSCRUST_IPA_NO_GUARD lets known proper nouns through.
        let guarded = std::env::var_os("TRANSCRUST_IPA_NO_GUARD").is_none() && is_known_proper_noun(word);
        let word_phones = if spoken.is_empty() || guarded { None } else { phones(word) };
        for (name, key) in &keyed {
            if word.eq_ignore_ascii_case(name) {
                continue;
            }
            let ipa_hit = word_phones.as_ref().is_some_and(|wp| {
                spoken.iter().any(|(n, np)| n == name && phone_distance(wp, np) <= 1)
            });
            if (key.len() >= 3 && skeleton(word) == *key) || ipa_hit {
                out.push((a, b, name.clone()));
            }
            if let Some(&(c, d)) = tokens.get(i + 1) {
                let pair = format!("{}{}", word, &text[c..d]);
                if text[b..c].trim().is_empty() && skeleton(&pair) == *key && !pair.eq_ignore_ascii_case(name) {
                    out.push((a, d, name.clone()));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletons_fold_vowels_voicing_and_glides() {
        assert_eq!(skeleton("Todoroki"), skeleton("Totoroki"));
        assert_eq!(skeleton("Mater"), skeleton("meter"));
        assert_eq!(skeleton("Bluey"), skeleton("BlueWee"));
        assert_ne!(skeleton("Gambol"), skeleton("gambit"));
    }

    #[test]
    #[ignore = "diagnostic: prints Harper's readings"]
    fn print_readings() {
        for w in ["meter", "water", "for", "to", "the", "gamble", "matter", "Louie", "Louis", "Billy", "Milly", "silly", "Jilly", "really", "blue"] {
            let m = readings(w);
            eprintln!(
                "{w}: present={} noun={} singular={} countable={} mass_only={} plural={} proper={} verb={} prep={}",
                m.is_some(),
                m.as_ref().is_some_and(|m| m.is_noun()),
                m.as_ref().is_some_and(|m| m.is_singular_noun()),
                m.as_ref().is_some_and(|m| m.is_countable_noun()),
                m.as_ref().is_some_and(|m| m.is_mass_noun_only()),
                m.as_ref().is_some_and(|m| m.is_plural_noun()),
                m.as_ref().is_some_and(|m| m.is_proper_noun()),
                m.as_ref().is_some_and(|m| m.is_verb()),
                m.as_ref().is_some_and(|m| m.preposition),
            );
        }
    }

    #[test]
    fn lexicon_readings() {
        // Harper marks nouns countable by default ("water" too), so M5 never leans on mass.
        assert!(singular_count_noun("meter"));
        // "gamble" also reads as a surname in Harper, so it can't trip M5.
        assert!(!singular_count_noun("gamble"));
        assert!(is_preposition("for"));
        assert!(!is_preposition("to"));
        assert!(opens_noun_phrase("the"));
        assert!(!has_english_reading("totoroki"));
    }
}
