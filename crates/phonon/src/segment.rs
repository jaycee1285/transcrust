//! Mopvox's pause/quietest-gap policy, transliterated for 30 ms 16 kHz frames.
//! Configurable window cap; pause policy is retained across window comparisons.
use std::collections::VecDeque;
pub struct Segmenter {
    samples: Vec<f32>,
    ends: Vec<usize>,
    db: Vec<f32>,
    speech: Vec<bool>,
    history: VecDeque<f32>,
    run: usize,
    run_min: f32,
    max_samples: usize,
}
impl Default for Segmenter {
    fn default() -> Self {
        Self {
            samples: Vec::with_capacity(96_480),
            ends: Vec::new(),
            db: Vec::new(),
            speech: Vec::new(),
            history: VecDeque::new(),
            run: 0,
            run_min: f32::MAX,
            max_samples: 96_000,
        }
    }
}
impl Segmenter {
    /// Keep the six-second pause minimum and three-second quietest-gap search.
    pub fn with_window_seconds(seconds: u32) -> Result<Self, String> {
        if !(6..=30).contains(&seconds) || seconds % 6 != 0 {
            return Err("Phonon window seconds must be 6, 12, 18, 24 or 30".into());
        }
        let max_samples = seconds as usize * 16000;
        Ok(Self {
            samples: Vec::with_capacity(max_samples + 480),
            max_samples,
            ..Self::default()
        })
    }
    fn remember(&mut self, db: f32) {
        self.history.push_back(db);
        if self.history.len() > 333 {
            self.history.pop_front();
        }
    }
    pub fn push(&mut self, frame: &[f32]) -> Option<Vec<f32>> {
        if frame.is_empty() {
            return None;
        }
        assert!(
            frame.len() <= 480,
            "feed at most 30 ms per segmentation frame"
        );
        self.samples.extend_from_slice(frame);
        let rms =
            (frame.iter().map(|s| (s * s) as f64).sum::<f64>() / frame.len() as f64).sqrt() as f32;
        let db = if rms <= 1e-9 {
            -120.
        } else {
            20. * rms.log10()
        };
        let floor = if self.history.is_empty() {
            db
        } else {
            let mut h: Vec<_> = self.history.iter().copied().collect();
            h.sort_by(f32::total_cmp);
            h[(h.len() - 1) * 10 / 100]
        };
        let speech = db > floor + 10. && db > -55.;
        if !speech {
            self.remember(db);
            self.run = 0;
            self.run_min = f32::MAX;
        } else {
            self.run_min = self.run_min.min(db);
            self.run += 1;
            if self.run >= 100 {
                self.remember(self.run_min);
                self.run = 0;
                self.run_min = f32::MAX;
            }
        }
        self.ends.push(self.samples.len());
        self.db.push(db);
        self.speech.push(speech);
        let last_speech = self
            .speech
            .iter()
            .rposition(|&s| s)
            .map(|i| self.ends[i])
            .unwrap_or(0);
        let silence = self.samples.len() - last_speech;
        let mut speech_samples = 0;
        let mut previous = 0;
        for (i, &end) in self.ends.iter().enumerate() {
            if self.speech[i] {
                speech_samples += end - previous;
            }
            previous = end;
        }
        if speech_samples >= 4800 && self.samples.len() >= 96000 && silence >= 11200 {
            return Some(self.cut(self.samples.len() - silence / 2));
        }
        if self.samples.len() >= self.max_samples {
            let from = self.samples.len() - 48000;
            let mut best = None;
            for i in 0..self.ends.len() {
                let start = if i == 0 { 0 } else { self.ends[i - 1] };
                if start >= from && best.is_none_or(|b: usize| self.db[i] < self.db[b]) {
                    best = Some(i);
                }
            }
            let at = best
                .map(|i| (self.ends[i] + if i == 0 { 0 } else { self.ends[i - 1] }) / 2)
                .unwrap_or(self.samples.len());
            return Some(self.cut(at));
        }
        None
    }
    fn cut(&mut self, at: usize) -> Vec<f32> {
        let rest = self.samples.split_off(at);
        let piece = std::mem::replace(&mut self.samples, rest);
        let drop = self
            .ends
            .iter()
            .position(|&end| end > at)
            .unwrap_or(self.ends.len());
        self.ends.drain(..drop);
        self.db.drain(..drop);
        self.speech.drain(..drop);
        for end in &mut self.ends {
            *end -= at;
        }
        piece
    }
    pub fn finish(&mut self) -> Vec<f32> {
        self.cut(self.samples.len())
    }
    pub fn pending(&self) -> usize {
        self.samples.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cuts_are_lossless_and_working_audio_is_bounded() {
        for seconds in [6, 12] {
            let mut s = Segmenter::with_window_seconds(seconds).unwrap();
            let mut collected = Vec::new();
            let mut original = Vec::new();
            for n in 0..2000 {
                let frame: Vec<_> = (0..480)
                    .map(|i| ((n * 480 + i) as f32 * 0.017).sin() * 0.1)
                    .collect();
                original.extend_from_slice(&frame);
                if let Some(piece) = s.push(&frame) {
                    collected.extend(piece);
                }
                assert!(s.pending() < seconds as usize * 16000);
            }
            collected.extend(s.finish());
            assert_eq!(original, collected);
        }
    }
}
