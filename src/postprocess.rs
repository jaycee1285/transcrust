use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{remove_overlaps, Dialect, Document};
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Course correction
// ---------------------------------------------------------------------------

/// Regex patterns for mid-sentence corrections. Everything before (and
/// including) the trigger phrase is discarded, keeping only what follows.
///
/// Ported from VoiceAI's CourseCorrector.java.
static CORRECTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // "no wait" / "no actually" / "no sorry"
        r"(?i)^.*\bno,?\s+wait,?\s+",
        r"(?i)^.*\bno,?\s+actually,?\s+",
        r"(?i)^.*\bno,?\s+sorry,?\s+",
        // "actually never mind" / "never mind"
        r"(?i)^.*\bactually,?\s+never\s*mind,?\s+",
        r"(?i)^.*\bnever\s*mind,?\s+",
        // "I mean" / "what I meant was"
        r"(?i)^.*\bi\s+mean,?\s+",
        r"(?i)^.*\bwhat\s+i\s+meant\s+(was|is),?\s+",
        // "scratch/delete/forget/ignore that"
        r"(?i)^.*\bscratch\s+that,?\s+",
        r"(?i)^.*\bdelete\s+that,?\s+",
        r"(?i)^.*\bforget\s+that,?\s+",
        r"(?i)^.*\bignore\s+that,?\s+",
        // "or rather" / "or actually" / "or better yet"
        r"(?i)^.*\bor\s+rather,?\s+",
        r"(?i)^.*\bor\s+actually,?\s+",
        r"(?i)^.*\bor\s+better\s+yet,?\s+",
        // "let me rephrase" / "let me start over"
        r"(?i)^.*\blet\s+me\s+rephrase,?\s+",
        r"(?i)^.*\blet\s+me\s+start\s+over,?\s+",
        // "wait no" / "hold on"
        r"(?i)^.*\bwait,?\s+no,?\s+",
        r"(?i)^.*\bhold\s+on,?\s+",
        // "not X but Y" → keep Y
        r"(?i)^.*\bnot\s+\w+,?\s+but\s+",
    ]
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
});

/// Inline "word no sorry word" → keeps the second word.
static INLINE_CORRECTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\w+)\s+no\s+sorry\s+(\w+)\b").unwrap());

fn apply_course_correction(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = text.to_string();
    for pat in CORRECTION_PATTERNS.iter() {
        result = pat.replace_all(&result, "").to_string();
    }
    result = INLINE_CORRECTION.replace_all(&result, "$2").to_string();
    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// Repetition / stutter cleaning
// ---------------------------------------------------------------------------

// Backreference patterns require fancy-regex.
static PHRASE_REPEAT_3: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?i)\b(\w+\s+\w+\s+\w+)(\s+\1)+\b").unwrap());
static PHRASE_REPEAT_2: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?i)\b(\w+\s+\w+)(\s+\1)+\b").unwrap());
static TRIPLE_REPEAT: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?i)\b(\w+)(\s+\1){2,}\b").unwrap());
static DOUBLE_REPEAT: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?i)\b(\w+)\s+\1\b").unwrap());
static MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());

fn fancy_replace_all(re: &FancyRegex, text: &str, rep: &str) -> String {
    let mut result = text.to_string();
    // Apply repeatedly until no more matches (handles overlapping patterns)
    loop {
        match re.replace_all(&result, rep) {
            std::borrow::Cow::Borrowed(_) => break,
            std::borrow::Cow::Owned(new) if new == result => break,
            std::borrow::Cow::Owned(new) => result = new,
        }
    }
    result
}

fn clean_repetitions(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = text.to_string();
    // Longest patterns first
    result = fancy_replace_all(&PHRASE_REPEAT_3, &result, "$1");
    result = fancy_replace_all(&PHRASE_REPEAT_2, &result, "$1");
    result = fancy_replace_all(&TRIPLE_REPEAT, &result, "$1");
    result = fancy_replace_all(&DOUBLE_REPEAT, &result, "$1");
    result = MULTI_SPACE.replace_all(&result, " ").to_string();
    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// Filler word removal
// ---------------------------------------------------------------------------

/// Pure hesitation sounds — always removed.
static PURE_FILLERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"\buh\b", r"\bum\b", r"\bumm\b", r"\berm\b", r"\ber\b", r"\bhmm\b", r"\bhm\b",
        r"\bahh?\b", r"\behh?\b",
    ]
    .iter()
    .map(|p| Regex::new(&format!(r"(?i){p}\s*,?\s*")).unwrap())
    .collect()
});

/// Discourse fillers — removed when surrounded by other content.
static DISCOURSE_FILLERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"\byou know\b",
        r"\bkinda\b",
        r"\bsort of\b",
        r"\bkind of\b",
        r"\bbasically\b",
    ]
    .iter()
    .map(|p| Regex::new(&format!(r"(?i){p}\s*,?\s*")).unwrap())
    .collect()
});

/// "like" used as filler in specific contexts.
static FILLER_LIKE_COMMA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i),\s*like\s*,").unwrap());
static FILLER_LIKE_BEFORE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\blike\s*,\s*(uh|um|so|you know)").unwrap());
static FILLER_LIKE_INITIAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^like\s+").unwrap());

fn remove_fillers(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = text.to_string();

    for filler in PURE_FILLERS.iter() {
        result = filler.replace_all(&result, " ").to_string();
    }
    for filler in DISCOURSE_FILLERS.iter() {
        result = filler.replace_all(&result, " ").to_string();
    }

    result = FILLER_LIKE_COMMA.replace_all(&result, ",").to_string();
    result = FILLER_LIKE_BEFORE.replace_all(&result, "").to_string();
    result = FILLER_LIKE_INITIAL.replace_all(&result, "").to_string();

    // Normalize spaces
    result = MULTI_SPACE.replace_all(&result, " ").to_string();
    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// Spoken punctuation replacement (existing)
// ---------------------------------------------------------------------------

/// Spoken punctuation keywords mapped to their symbols.
/// Order matters: longer phrases must come before shorter ones to avoid partial matches.
const PUNCTUATION_KEYWORDS: &[(&str, &str)] = &[
    ("exclamation point", "!"),
    ("exclamation mark", "!"),
    ("question mark", "?"),
    ("open quote", "\""),
    ("close quote", "\""),
    ("open paren", "("),
    ("close paren", ")"),
    ("full stop", "."),
    ("new line", "\n"),
    ("new paragraph", "\n\n"),
    ("ellipsis", "..."),
    ("period", "."),
    ("comma", ","),
    ("colon", ":"),
    ("semicolon", ";"),
    ("hyphen", "-"),
    ("dash", " — "),
];

/// Replace spoken punctuation keywords with their symbols.
/// Handles surrounding whitespace so "hello period goodbye" becomes "hello. goodbye".
fn replace_spoken_punctuation(text: &str) -> String {
    let mut result = text.to_string();

    for &(keyword, symbol) in PUNCTUATION_KEYWORDS {
        let mut search_from = 0;
        loop {
            let lower = result[search_from..].to_lowercase();
            let Some(pos) = lower.find(keyword) else {
                break;
            };
            let abs_pos = search_from + pos;
            let end = abs_pos + keyword.len();

            // Check word boundaries: must be at start/end or adjacent to whitespace
            let at_word_start =
                abs_pos == 0 || result.as_bytes()[abs_pos - 1].is_ascii_whitespace();
            let at_word_end =
                end == result.len() || result.as_bytes()[end].is_ascii_whitespace();

            if !at_word_start || !at_word_end {
                search_from = end;
                continue;
            }

            // Remove space before punctuation that attaches to previous word
            let trim_start = if abs_pos > 0 && result.as_bytes()[abs_pos - 1] == b' ' {
                abs_pos - 1
            } else {
                abs_pos
            };

            // Remove space after keyword
            let trim_end = if end < result.len() && result.as_bytes()[end] == b' ' {
                end + 1
            } else {
                end
            };

            // Build replacement: symbol + space after (unless end-of-string or newline)
            let needs_space_after = trim_end <= result.len()
                && !symbol.ends_with('\n')
                && !symbol.ends_with(' ')
                && trim_end < result.len();
            let replacement = if needs_space_after {
                format!("{symbol} ")
            } else {
                symbol.to_string()
            };

            result.replace_range(trim_start..trim_end, &replacement);
            search_from = trim_start + replacement.len();
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Public pipeline
// ---------------------------------------------------------------------------

/// Full post-processing pipeline for transcription output.
///
/// Order: course correction → repetition cleaning → filler removal →
/// spoken punctuation → phonetic dictionary.
///
/// **Harper is intentionally out of the loop.** It is a probabilistic grammar
/// black box that re-ranks tokens toward general English *before* the dictionary
/// can claim them — directly working against this app's whole purpose (exact
/// non-standard vocabulary). murmure, the pipeline this is ported from, never
/// ran a grammar pass at this stage either. If a deterministic capitalization /
/// punctuation pass turns out to be wanted, it should be a small purpose-built
/// thing, not a 20 MB linter. [`apply_harper`] is kept for reference only.
pub fn fix_transcription(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let text = apply_course_correction(text);
    let text = clean_repetitions(&text);
    let text = remove_fillers(&text);
    let text = replace_spoken_punctuation(&text);
    crate::dictionary::correct(&text)
}

/// Apply harper's curated grammar/style fixes, minus the rules that fight voice
/// transcription. Auto-applies the first suggestion of each remaining lint.
///
/// Not wired into [`fix_transcription`] — see that function's note. Kept so the
/// known-good harper config isn't lost if we decide we want a grammar loop back.
#[allow(dead_code)]
fn apply_harper(text: &str) -> String {
    let dict = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dict, Dialect::American);

    // Disable rules that don't make sense for voice transcription
    linter.config.set_rule_enabled("LongSentences", false);
    linter.config.set_rule_enabled("AvoidCurses", false);
    linter.config.set_rule_enabled("FillerWords", false);
    linter.config.set_rule_enabled("Hedging", false);
    linter.config.set_rule_enabled("BoringWords", false);
    linter.config.set_rule_enabled("DiscourseMarkers", false);
    linter.config.set_rule_enabled("UnclosedQuotes", false);
    // SpellCheck would "correct" injected jargon/proper nouns toward its curated
    // dictionary, clobbering the phonetic dictionary pass that runs after this.
    linter.config.set_rule_enabled("SpellCheck", false);

    let doc = Document::new_plain_english_curated(text);
    let mut lints = linter.lint(&doc);
    remove_overlaps(&mut lints);

    // Only keep lints that have at least one suggestion (auto-fixable)
    lints.retain(|l| !l.suggestions.is_empty());

    if lints.is_empty() {
        return text.to_string();
    }

    // Sort by span start descending so we can apply fixes back-to-front
    // without invalidating earlier spans
    lints.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    let mut chars: Vec<char> = text.chars().collect();

    for lint in &lints {
        lint.suggestions[0].apply(lint.span, &mut chars);
    }

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Course correction --

    #[test]
    fn course_correction_no_wait() {
        assert_eq!(
            apply_course_correction("let's meet tomorrow no wait let's do friday"),
            "let's do friday"
        );
    }

    #[test]
    fn course_correction_scratch_that() {
        assert_eq!(
            apply_course_correction("send to john scratch that send to mike"),
            "send to mike"
        );
    }

    #[test]
    fn course_correction_i_mean() {
        assert_eq!(
            apply_course_correction("it costs ten i mean twenty dollars"),
            "twenty dollars"
        );
    }

    #[test]
    fn course_correction_or_rather() {
        assert_eq!(
            apply_course_correction("we should go left or rather right"),
            "right"
        );
    }

    #[test]
    fn course_correction_not_but() {
        assert_eq!(
            apply_course_correction("not tomorrow but friday"),
            "friday"
        );
    }

    #[test]
    fn course_correction_no_sorry() {
        // "no sorry" is a major correction trigger — discards everything before it
        assert_eq!(
            apply_course_correction("send to john no sorry send to mike"),
            "send to mike"
        );
    }

    #[test]
    fn course_correction_no_trigger() {
        assert_eq!(
            apply_course_correction("this is perfectly fine"),
            "this is perfectly fine"
        );
    }

    // -- Repetition cleaning --

    #[test]
    fn cleans_triple_repeat() {
        assert_eq!(clean_repetitions("I I I think"), "I think");
    }

    #[test]
    fn cleans_double_repeat() {
        assert_eq!(clean_repetitions("the the problem"), "the problem");
    }

    #[test]
    fn cleans_phrase_repeat() {
        assert_eq!(clean_repetitions("you know you know"), "you know");
    }

    #[test]
    fn no_repeat_noop() {
        assert_eq!(clean_repetitions("hello world"), "hello world");
    }

    // -- Filler removal --

    #[test]
    fn removes_uh_um() {
        assert_eq!(remove_fillers("I uh think um yes"), "I think yes");
    }

    #[test]
    fn removes_you_know() {
        assert_eq!(
            remove_fillers("it was you know really good"),
            "it was really good"
        );
    }

    #[test]
    fn removes_filler_like() {
        assert_eq!(remove_fillers("like we should go"), "we should go");
    }

    #[test]
    fn filler_noop() {
        assert_eq!(remove_fillers("this is fine"), "this is fine");
    }

    // -- Spoken punctuation (existing tests) --

    #[test]
    fn spoken_period() {
        assert_eq!(
            replace_spoken_punctuation("hello period goodbye"),
            "hello. goodbye"
        );
    }

    #[test]
    fn spoken_comma() {
        assert_eq!(
            replace_spoken_punctuation("first comma second"),
            "first, second"
        );
    }

    #[test]
    fn spoken_question_mark() {
        assert_eq!(
            replace_spoken_punctuation("are you sure question mark"),
            "are you sure?"
        );
    }

    #[test]
    fn spoken_new_line() {
        assert_eq!(
            replace_spoken_punctuation("line one new line line two"),
            "line one\nline two"
        );
    }

    #[test]
    fn spoken_exclamation() {
        assert_eq!(
            replace_spoken_punctuation("wow exclamation point"),
            "wow!"
        );
    }

    #[test]
    fn no_partial_match() {
        assert_eq!(
            replace_spoken_punctuation("periodically we check"),
            "periodically we check"
        );
    }

    #[test]
    fn end_of_string_punctuation() {
        assert_eq!(
            replace_spoken_punctuation("that is all period"),
            "that is all."
        );
    }

    // -- Full pipeline --

    #[test]
    fn casing_left_as_spoken() {
        // Harper is out of the loop, so we no longer capitalize sentences.
        // Casing is whatever the model emitted — that's the deliberate tradeoff
        // for not letting a grammar black box touch the text.
        let result = fix_transcription("there is no way she is not guilty");
        assert_eq!(result, "there is no way she is not guilty");
    }

    #[test]
    fn lone_i_not_capitalized() {
        // Without harper, "i" stays "i". A purpose-built capitalization pass
        // could fix this later if wanted; we don't fake it here.
        let result = fix_transcription("i went to the store");
        assert_eq!(result, "i went to the store");
    }

    #[test]
    fn empty_string() {
        assert_eq!(fix_transcription(""), "");
    }

    #[test]
    fn already_correct() {
        let input = "Hello world.";
        let result = fix_transcription(input);
        assert_eq!(result, input);
    }

    #[test]
    fn full_pipeline() {
        // Spoken punctuation still resolves; casing is left untouched.
        let result = fix_transcription("i think this is really great period and i hope it works");
        assert!(result.contains('.'), "Expected period, got: {result}");
    }

    #[test]
    fn full_pipeline_with_correction() {
        let result = fix_transcription("send to john no wait send to mike period");
        assert!(
            result.contains("mike") || result.contains("Mike"),
            "Expected mike after correction, got: {result}"
        );
        assert!(
            !result.contains("john") && !result.contains("John"),
            "Expected john to be removed, got: {result}"
        );
    }

    #[test]
    fn full_pipeline_with_fillers_and_stutter() {
        let result =
            fix_transcription("i i think uh we should you know go to the the store");
        assert!(
            !result.contains(" uh "),
            "Expected filler removed, got: {result}"
        );
        // Should clean up the double "the" and stutter "i i"
        assert!(
            !result.contains("the the"),
            "Expected stutter cleaned, got: {result}"
        );
    }
}
