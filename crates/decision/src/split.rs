//! Capitalized-name boundary repair, run before candidate matching.
//!
//! The ASR decoders emit subword pieces. `and` exists both as a word-start piece
//! (`▁and`) and as a continuation piece. A fused name such as `Nelland` is the
//! continuation piece chosen after a registered name built from fragments
//! (John, 2026-11-19/20; ~/syncthing/transcrust-work/window-study/prefix-trial.md).
//!
//! Rule: a capitalized token that is not itself a known English word and starts with
//! a registered capitalized name is split after the name when the remainder
//! segments into known words or names. A closed-class word directly after the name
//! is tried first. Names in the remainder take the glossary's canonical spelling.
//! Deterministic; no model involvement.
use serde::Serialize;
use std::collections::HashMap;
use transcrust_core::lexicon;

const MAX_PIECE: usize = 24;

/// Closed-class words tried first, longest first.
const CLOSED: &[&str] = &[
    "with", "from", "and", "for", "but", "nor", "the", "or", "of", "to", "in", "on", "at", "by",
    "a",
];
/// Short pieces allowed in the remainder; anything else needs three or more letters.
const SHORT: &[&str] = &[
    "a", "i", "an", "and", "or", "of", "to", "in", "on", "at", "is", "as", "so", "we", "he",
    "me", "my", "by", "be", "it", "up", "us", "no", "do", "go", "the", "was", "had", "has", "but",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Split {
    /// UTF-8 byte offsets in the text given to `repair`.
    pub start: usize,
    pub end: usize,
    pub heard: String,
    pub replacement: String,
    pub reason: String,
}

pub struct Names {
    /// Single-word capitalized canonicals, longest first.
    names: Vec<String>,
    by_lower: HashMap<String, usize>,
}

impl Names {
    pub fn new<'a>(canonicals: impl IntoIterator<Item = &'a str>) -> Self {
        let mut names: Vec<String> = canonicals
            .into_iter()
            .filter(|c| {
                c.chars().next().is_some_and(char::is_uppercase)
                    && c.chars().all(char::is_alphabetic)
            })
            .map(str::to_string)
            .collect();
        names.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        names.dedup();
        let by_lower = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.to_lowercase(), i))
            .collect();
        Self { names, by_lower }
    }

    fn is_word(piece: &str) -> bool {
        (piece.len() >= 3 && lexicon::contains(piece)) || SHORT.contains(&piece)
    }

    fn inflected(piece: &str) -> bool {
        for suffix in ["ing", "ed", "es", "s", "d"] {
            if let Some(stem) = piece.strip_suffix(suffix) {
                if stem.len() >= 3 && Self::is_word(stem) {
                    return true;
                }
                if suffix == "ing" && stem.len() >= 2 && Self::is_word(&format!("{stem}e")) {
                    return true;
                }
            }
        }
        false
    }

    /// Ok(Some(i)) registered name, Ok(None) ordinary word, Err not a piece.
    fn piece(&self, piece: &str) -> Result<Option<usize>, ()> {
        if let Some(&i) = self.by_lower.get(piece) {
            return Ok(Some(i));
        }
        if Self::is_word(piece) || Self::inflected(piece) {
            return Ok(None);
        }
        Err(())
    }

    /// Fewest pieces; a name piece is preferred on ties.
    fn segment(&self, rest: &str) -> Option<Vec<(usize, usize, Option<usize>)>> {
        let n = rest.len();
        let mut best: Vec<Option<(u32, usize, Option<usize>)>> = vec![None; n + 1];
        best[n] = Some((0, n, None));
        for i in (0..n).rev() {
            if !rest.is_char_boundary(i) {
                continue;
            }
            for j in (i + 1)..=n.min(i + MAX_PIECE) {
                if !rest.is_char_boundary(j) {
                    continue;
                }
                let Some((cost, _, _)) = best[j] else { continue };
                let Ok(kind) = self.piece(&rest[i..j]) else { continue };
                let score = cost + if kind.is_some() { 2 } else { 3 };
                if best[i].map_or(true, |(c, _, _)| score < c) {
                    best[i] = Some((score, j, kind));
                }
            }
        }
        best[0]?;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let (_, j, kind) = best[i]?;
            out.push((i, j, kind));
            i = j;
        }
        Some(out)
    }

    fn closed_first(&self, lower: &str) -> Option<Vec<(usize, usize, Option<usize>)>> {
        for w in CLOSED {
            let Some(rest) = lower.strip_prefix(w) else { continue };
            let mut parts = vec![(0, w.len(), None)];
            if rest.is_empty() {
                return Some(parts);
            }
            if let Some(more) = self.segment(rest) {
                parts.extend(more.into_iter().map(|(i, j, k)| (i + w.len(), j + w.len(), k)));
                return Some(parts);
            }
        }
        None
    }

    fn split_token(&self, token: &str) -> Option<String> {
        if !token.chars().next().is_some_and(char::is_uppercase) {
            return None;
        }
        if lexicon::contains(&token.to_lowercase()) {
            return None;
        }
        for name in &self.names {
            let Some(tail) = token.strip_prefix(name.as_str()) else { continue };
            if tail.is_empty() {
                return None;
            }
            let lower = tail.to_lowercase();
            if lower.len() != tail.len() {
                continue;
            }
            let Some(parts) = self.closed_first(&lower).or_else(|| self.segment(&lower)) else {
                continue;
            };
            let mut out = name.clone();
            for (i, j, kind) in parts {
                out.push(' ');
                match kind {
                    Some(k) => out.push_str(&self.names[k]),
                    None => out.push_str(&tail[i..j]),
                }
            }
            return Some(out);
        }
        None
    }

    /// Split fused registered names. Returns the repaired text and one record per edit.
    pub fn repair(&self, text: &str) -> (String, Vec<Split>) {
        let mut output = String::with_capacity(text.len() + 16);
        let mut splits = Vec::new();
        if self.names.is_empty() {
            return (text.to_string(), splits);
        }
        let mut start = 0;
        for (end, ch) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            if ch.is_alphabetic() || ch == '\'' || ch == '’' {
                continue;
            }
            let token = &text[start..end];
            match self.split_token(token) {
                Some(replacement) => {
                    output.push_str(&replacement);
                    splits.push(Split {
                        start,
                        end,
                        heard: token.into(),
                        reason: format!(
                            "fused registered name: {:?} is not an English word; split as {:?}",
                            token, replacement
                        ),
                        replacement,
                    });
                }
                None => output.push_str(token),
            }
            if end < text.len() {
                output.push(ch);
            }
            start = end + ch.len_utf8();
        }
        (output, splits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The entries of ~/syncthing/john/pixel-vocab.md (2026-11-19); the household
    /// child's name is replaced with Rowan, as elsewhere in the tree.
    fn pixel() -> Names {
        Names::new([
            "Tauri", "Kubernetes", "Wayland", "Svelte", "Syncthing", "Gambol", "Nell", "Cigs",
            "Void", "Rowan", "Flo", "Mater", "Bluey", "Bingo", "Chili", "Bandit", "McQueen",
            "Lightning McQueen", "Flo or", "transcrust", "Cars theme", "new Zed",
        ])
    }

    fn fixed(text: &str) -> String {
        pixel().repair(text).0
    }

    #[test]
    fn real_phone_take_nelland_is_split_and_nothing_else_changes() {
        // Take a137dfff-5c5e-4df3-a87e-9551e4aff16d, Parakeet Ultra raw opening.
        let raw = "Nelland Rowan took gamble to the park this morning. Gamble chased a ball while Rowan talked about Lightning McQueen.";
        let (out, splits) = pixel().repair(raw);
        assert_eq!(
            out,
            "Nell and Rowan took gamble to the park this morning. Gamble chased a ball while Rowan talked about Lightning McQueen."
        );
        assert_eq!(splits.len(), 1);
        assert_eq!((splits[0].start, splits[0].end), (0, 7));
        assert_eq!(splits[0].replacement, "Nell and");
    }

    #[test]
    fn frozen_fusion_fixtures_repair() {
        for (heard, want) in [
            ("Nelland Rowan walked home.", "Nell and Rowan walked home."),
            ("Floand Mater had lunch.", "Flo and Mater had lunch."),
            ("Gamboland Rowan played outside.", "Gambol and Rowan played outside."),
            ("Blueyand Bingo watched television.", "Bluey and Bingo watched television."),
            ("Bingoand Bluey played a game.", "Bingo and Bluey played a game."),
            ("Materand Lightning McQueen waited outside.", "Mater and Lightning McQueen waited outside."),
            ("Rowanand Nell took Gambol home.", "Rowan and Nell took Gambol home."),
            ("Banditand Chili helped the children.", "Bandit and Chili helped the children."),
            ("Chilisaid dinner was ready.", "Chili said dinner was ready."),
            ("Nellcalled Gambol inside.", "Nell called Gambol inside."),
            ("Gambolchased a ball.", "Gambol chased a ball."),
            ("NellandRowan walked home.", "Nell and Rowan walked home."),
            ("Nellandmater waited.", "Nell and Mater waited."),
        ] {
            assert_eq!(fixed(heard), want, "{heard}");
        }
    }

    #[test]
    fn controls_and_bellwether_stay_unchanged() {
        for text in [
            "Flowing water filled the stream.",
            "Flow moved through the pipe.",
            "Flowers opened this morning.",
            "Maternal care matters.",
            "Material filled the box.",
            "Materials arrived today.",
            "Nellie walked home.",
            "Floor tiles were cold.",
            "McQueen raced past Mater.",
            "Nell-and-Rowan is a label.",
            "Bluey's hat fell off.",
            "Rowan asked whether flow would have lunch ready for meter.",
            "Chili helped the children while bandit looked for a toy.",
            "A flash of Lightning McQueen hit the window.",
            "Nell called gamble inside.",
            "We should never Gambol with the grocery money.",
            "Back at home, Louis and bingo were on television.",
        ] {
            assert_eq!(fixed(text), text);
        }
    }

    /// Known behaviour on invented tokens, pinned so it cannot drift silently.
    /// `Gambolling` is a real inflection of the verb the dog is named after.
    #[test]
    fn known_edges_on_invented_tokens() {
        for (heard, got) in [
            ("Rowana", "Rowan a"),
            ("Floof", "Flo of"),
            ("Gambolling", "Gambol ling"),
            ("Banditos", "Banditos"),
            ("Flowy", "Flowy"),
            ("Rowanid", "Rowanid"),
        ] {
            assert_eq!(fixed(heard), got, "{heard}");
        }
    }
}
