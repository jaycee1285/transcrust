//! Vocabulary preference at the greedy decoder seam, not text substitution.
//! Fresh implementation of prefix scoring with failure/backoff transitions.
//! Near-miss gates and alpha calibration are documented in murmure-reference.md.
use std::collections::{BTreeMap, VecDeque};

#[derive(Default)]
struct Node {
    edges: BTreeMap<u32, usize>,
    fail: usize,
    parent: usize,
    depth: usize,
    terminal: bool,
    pending: f32,
    credit: f32,
}

pub struct Boost {
    nodes: Vec<Node>,
    state: usize,
    alpha: f32,
    pub phrases: usize,
    pub changes: usize,
}

impl Boost {
    pub fn disabled() -> Self {
        Self::new(&[], 0.0)
    }
    pub fn vocabulary(pieces: &[String], blank: u32) -> Self {
        let terms = crate::dictionary::boost_terms();
        let mut sequences = Vec::new();
        let pieces: Vec<_> = pieces
            .iter()
            .enumerate()
            .filter(|(id, p)| *id != blank as usize && !p.is_empty() && !p.starts_with('<'))
            .map(|(id, p)| (id as u32, p.replace('▁', " ")))
            .collect();
        for term in &terms {
            // Both are spelling variants of the canonical term, never heard aliases.
            for spelling in [term.clone(), term.to_lowercase()] {
                if let Some(ids) = tokenize(&format!(" {spelling}"), &pieces) {
                    if !sequences.contains(&ids) {
                        sequences.push(ids);
                    }
                }
            }
        }
        let alpha = if terms.is_empty() {
            0.0
        } else {
            (3.5 - (terms.len() as f32 / 5.0).log10()).clamp(1.0, 3.5)
                * crate::dictionary::boost_strength()
        };
        Self::new(&sequences, alpha)
    }

    fn new(sequences: &[Vec<u32>], alpha: f32) -> Self {
        let mut nodes = vec![Node::default()];
        for sequence in sequences.iter().filter(|s| !s.is_empty()) {
            let mut state = 0;
            for &token in sequence {
                state = if let Some(&next) = nodes[state].edges.get(&token) {
                    next
                } else {
                    let next = nodes.len();
                    let depth = nodes[state].depth + 1;
                    nodes.push(Node {
                        parent: state,
                        depth,
                        ..Default::default()
                    });
                    nodes[state].edges.insert(token, next);
                    next
                };
            }
            nodes[state].terminal = true;
        }
        for i in 1..nodes.len() {
            let edge = if nodes[i].depth == 1 {
                1.0
            } else {
                2.0 + (nodes[i].depth as f32).ln()
            };
            nodes[i].credit = nodes[nodes[i].parent].pending + edge;
            nodes[i].pending = if nodes[i].terminal {
                0.0
            } else {
                nodes[i].credit
            };
        }
        let mut queue: VecDeque<_> = nodes[0].edges.values().copied().collect();
        while let Some(parent) = queue.pop_front() {
            let edges: Vec<_> = nodes[parent].edges.iter().map(|(&t, &n)| (t, n)).collect();
            for (token, child) in edges {
                let mut fail = nodes[parent].fail;
                while fail != 0 && !nodes[fail].edges.contains_key(&token) {
                    fail = nodes[fail].fail;
                }
                nodes[child].fail = nodes[fail].edges.get(&token).copied().unwrap_or(0);
                queue.push_back(child);
            }
        }
        Self {
            nodes,
            state: 0,
            alpha,
            phrases: sequences.len(),
            changes: 0,
        }
    }

    fn next(&self, mut state: usize, token: u32) -> usize {
        loop {
            if let Some(&next) = self.nodes[state].edges.get(&token) {
                return next;
            }
            if state == 0 {
                return 0;
            }
            state = self.nodes[state].fail;
        }
    }

    fn score(&self, token: u32) -> f32 {
        let next = self.next(self.state, token);
        // Abandoned partials refund their accumulated score; completed phrases
        // keep their credit. This also supports overlapping prefix/suffix terms.
        self.alpha * (self.nodes[next].credit - self.nodes[self.state].pending)
    }

    pub fn select(&mut self, logits: &[f32], blank: u32) -> u32 {
        let raw = logits
            .iter()
            .enumerate()
            .max_by(|(ai, a), (bi, b)| a.total_cmp(b).then_with(|| bi.cmp(ai)))
            .map_or(blank, |(i, _)| i as u32);
        if self.alpha <= 0.0 || self.nodes.len() == 1 {
            return raw;
        }
        let mut top: Vec<u32> = Vec::with_capacity(20);
        for (id, &value) in logits.iter().enumerate() {
            if id == blank as usize || !value.is_finite() {
                continue;
            }
            if top.len() == 20 && value <= logits[*top.last().unwrap() as usize] {
                continue;
            }
            let at = top.partition_point(|&t| logits[t as usize] >= value);
            top.insert(at, id as u32);
            if top.len() > 20 {
                top.pop();
            }
        }
        let base = logits
            .get(raw as usize)
            .copied()
            .unwrap_or(f32::NEG_INFINITY);
        let mut best = raw;
        let mut score = base + if raw == blank { 0.0 } else { self.score(raw) };
        for (rank, &token) in top.iter().enumerate() {
            let next = self.next(self.state, token);
            let continuing = next != 0 && self.nodes[next].depth > 1;
            let gate = if continuing && self.nodes[self.state].depth >= 3 {
                20
            } else {
                5
            };
            if rank >= gate || base - logits[token as usize] > self.alpha {
                continue;
            }
            // Rank alone can call a very distant runner-up a near miss. Cap
            // the raw logit gap too, including after a long matched prefix.
            let candidate = logits[token as usize] + self.score(token);
            if candidate > score {
                best = token;
                score = candidate;
            }
        }
        if best != blank {
            self.state = self.next(self.state, best);
            if best != raw {
                self.changes += 1;
            }
        }
        best
    }
}

// Shortest exact segmentation using the export's own vocabulary. No unknown
// pieces, no guessed IDs. Unrepresentable terms are simply not armed.
fn tokenize(text: &str, pieces: &[(u32, String)]) -> Option<Vec<u32>> {
    if text.len() > 256 {
        return None;
    }
    let mut paths: Vec<Option<Vec<u32>>> = vec![None; text.len() + 1];
    paths[0] = Some(Vec::new());
    for (at, _) in text.char_indices() {
        let Some(path) = paths[at].clone() else {
            continue;
        };
        for (id, piece) in pieces {
            if text[at..].starts_with(piece) {
                let end = at + piece.len();
                if paths[end].as_ref().is_none_or(|p| p.len() > path.len() + 1) {
                    let mut next = path.clone();
                    next.push(*id);
                    paths[end] = Some(next);
                }
            }
        }
    }
    paths.pop().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preference_can_tip_a_near_miss_but_not_a_strong_ordinary_word() {
        let mut boost = Boost::new(&[vec![1]], 1.0);
        assert_eq!(boost.select(&[2.0, 1.5, -10.0], 2), 1);
        assert_eq!(boost.select(&[5.0, 1.5, -10.0], 2), 0);
    }
    #[test]
    fn distant_candidate_is_not_rescued_even_with_a_large_weight() {
        let mut boost = Boost::new(&[vec![6]], 100.0);
        assert_eq!(
            boost.select(&[10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, -10.0], 7),
            0
        );
    }
    #[test]
    fn blanks_preserve_prefix_and_abandonment_refunds_it() {
        let mut boost = Boost::new(&[vec![1, 2]], 1.0);
        assert_eq!(boost.select(&[-10.0, 10.0, -10.0, -10.0], 3), 1);
        assert_eq!(boost.select(&[-10.0, -10.0, -10.0, 10.0], 3), 3);
        assert!(boost.score(0) < 0.0);
        assert!(boost.score(2) > 0.0);
        assert_eq!(boost.select(&[10.0, -10.0, -10.0, -10.0], 3), 0);
        assert_eq!(boost.state, 0);
    }
    #[test]
    fn completed_phrase_keeps_its_credit_and_can_start_again() {
        let mut boost = Boost::new(&[vec![1, 2]], 1.0);
        boost.select(&[-10.0, 10.0, -10.0, -10.0], 3);
        boost.select(&[-10.0, -10.0, 10.0, -10.0], 3);
        assert_eq!(boost.score(0), 0.0);
        assert_eq!(boost.score(1), 1.0);
    }
    #[test]
    fn exact_tokenization_handles_unicode_and_rejects_missing_pieces() {
        let pieces = vec![
            (0, " ".into()),
            (1, "Gam".into()),
            (2, "bol".into()),
            (3, "é".into()),
        ];
        assert_eq!(tokenize(" Gambol", &pieces), Some(vec![0, 1, 2]));
        assert_eq!(tokenize(" é", &pieces), Some(vec![0, 3]));
        assert_eq!(tokenize(" Nell", &pieces), None);
    }
    #[test]
    fn strong_model_choice_survives_even_after_an_engaged_prefix() {
        let mut boost = Boost::new(&[vec![1, 2, 3, 4]], 1.0);
        for token in [1, 2, 3] {
            let mut logits = vec![-10.0; 6];
            logits[token] = 10.0;
            boost.select(&logits, 5);
        }
        assert_eq!(boost.select(&[5.0, -10.0, -10.0, -10.0, 1.5, -10.0], 5), 0);
    }
    #[test]
    fn suffix_phrase_can_complete_after_a_longer_prefix_is_abandoned() {
        let mut boost = Boost::new(&[vec![1, 2, 3], vec![2, 4]], 1.0);
        for token in [1, 2, 4] {
            let mut logits = vec![-10.0; 6];
            logits[token] = 10.0;
            assert_eq!(boost.select(&logits, 5), token as u32);
        }
        assert!(boost.nodes[boost.state].terminal);
        assert_eq!(boost.score(0), 0.0);
    }
    #[test]
    fn zero_weight_is_identity() {
        let mut boost = Boost::new(&[vec![1]], 0.0);
        assert_eq!(boost.select(&[2.0, 1.9, -10.0], 2), 0);
    }
}
