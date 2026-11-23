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
    // Phonon writes "exclamation point" this way (2026-11-22 Kale test take).
    ("exclamation pointer", "!"),
    ("exclamation point", "!"),
    ("exclamation mark", "!"),
    ("question mark", "?"),
    // Quotes and brackets: see replace_enclosers, which runs first.
    ("full stop", "."),
    ("new line", "\n"),
    ("new paragraph", "\n\n"),
    ("ellipsis", "..."),
    ("period", "."),
    ("comma", ","),
    // Phonon's spelling of "comma" (2026-11-22). Safe only because "a coma" / "in a
    // coma" fall to the determiner rule; a bare "coma" in dictation is read as the mark.
    ("coma", ","),
    ("colon", ":"),
    ("semicolon", ";"),
    ("hyphen", "-"),
    ("dash", " — "),
];

/// "new paragraph" / "new line" plus whatever punctuation the model wrapped
/// around them. Parakeet and Nemotron punctuate natively, so the command
/// arrives as "one. New paragraph. Two", which the keyword pass below never
/// matched (it needs whitespace after the keyword). A sentence terminator
/// already before the command is kept; none is invented, so a list dictated
/// with "new line" stays a list.
static LINE_BREAK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[ \t,;:]*([.!?])?[ \t,;:.]*\bnew (paragraph|line)\b[ \t,;:.!?]*").unwrap()
});

/// The punctuation a model writes on its own, next to a spoken mark it should give way to.
const MODEL_MARKS: &[u8] = b".,!?;:";

/// After one of these, a command word is the ordinary noun: "over a period of
/// time", "a dash of salt", "a new line of code", "in a coma". The same reading
/// dayshade's grammar and the decision crate's determiner rule give a name slot
/// (M5), kept to a word list here so core stays small (2026-11-22). Only the
/// word straight before counts: two words back would also swallow commands
/// ("I sent the email period"), so compound nouns ("trial period", "grace
/// period") still convert. Telling those apart needs the decision crate's grammar.
const DETERMINERS: &[&str] = &[
    "a", "an", "the", "this", "that", "these", "those", "each", "every", "another", "any", "some",
    "no", "my", "your", "his", "her", "its", "our", "their", "which", "what", "whose",
];

/// Whether the word just before byte `pos` is a determiner.
fn after_determiner(text: &str, pos: usize) -> bool {
    let before = text[..pos].trim_end();
    // The run of letters ending the text before `pos`; punctuation there means no word.
    let word = &before[before.trim_end_matches(|c: char| c.is_alphabetic() || c == '\'').len()..];
    DETERMINERS.contains(&word.to_lowercase().as_str())
}

fn replace_line_breaks(text: &str) -> String {
    LINE_BREAK
        .replace_all(text, |caps: &regex::Captures| {
            let whole = caps.get(0).expect("match");
            // "a new line of code" is a noun phrase, not a command.
            let new_at = whole.start() + whole.as_str().to_lowercase().find("new").unwrap_or(0);
            if after_determiner(text, new_at) {
                return whole.as_str().to_string();
            }
            let terminator = caps.get(1).map_or("", |m| m.as_str());
            let brk = if caps[2].eq_ignore_ascii_case("paragraph") { "\n\n" } else { "\n" };
            format!("{terminator}{brk}")
        })
        .into_owned()
}

/// "open quote" / "open paren", with the model's own mark straight after it:
/// Parakeet writes "Open quote, not today", where the comma is its guess.
static OPENER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bopen (quote|paren)\b[,;:.]?[ \t]*").unwrap()
});

/// "close quote" / "close paren", with the model's mark straight before it:
/// "not today, close quote".
static CLOSER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[ \t]*[,;:]?[ \t]*\bclose (quote|paren)\b").unwrap()
});

/// Quotes and brackets hug what they enclose (2026-11-23, John's read-aloud):
/// "she said open quote, not today, close quote." is `she said "not today."`.
/// An opener drops the model's mark after it and the space before the next
/// word; a closer drops the model's mark and space before it and keeps what
/// follows. Before this they were only ever matched with a space on both sides.
fn replace_enclosers(text: &str) -> String {
    let opened = OPENER.replace_all(text, |caps: &regex::Captures| {
        if caps[1].eq_ignore_ascii_case("quote") { "\"" } else { "(" }.to_string()
    });
    CLOSER
        .replace_all(&opened, |caps: &regex::Captures| {
            if caps[1].eq_ignore_ascii_case("quote") { "\"" } else { ")" }.to_string()
        })
        .into_owned()
}

/// Replace spoken punctuation keywords with their symbols.
/// Handles surrounding whitespace so "hello period goodbye" becomes "hello. goodbye".
fn replace_spoken_punctuation(text: &str) -> String {
    let mut result = replace_enclosers(&replace_line_breaks(text));
    // The vocabulary's names, read only if a capital needs judging (see below).
    let mut names: Option<Vec<String>> = None;

    for &(keyword, symbol) in PUNCTUATION_KEYWORDS {
        let mut search_from = 0;
        loop {
            let lower = result[search_from..].to_lowercase();
            let Some(pos) = lower.find(keyword) else {
                break;
            };
            let abs_pos = search_from + pos;
            let end = abs_pos + keyword.len();

            // A spoken mark is an explicit command, and it wins over the model's own
            // guess beside it (2026-11-22, Kale test takes): "bread period." is
            // "bread.", and Parakeet's "eggs, comma and" is "eggs, and", not "eggs,,".
            let is_mark = matches!(symbol, "." | "," | "!" | "?" | ":" | ";" | "...");
            let bytes = result.as_bytes();

            // Check word boundaries: start/end, whitespace, or (for a mark) the
            // model's own punctuation straight after the keyword.
            let at_word_start = abs_pos == 0 || bytes[abs_pos - 1].is_ascii_whitespace();
            let at_word_end = end == result.len()
                || bytes[end].is_ascii_whitespace()
                || (is_mark && MODEL_MARKS.contains(&bytes[end]));

            // "a period", "the dash": the noun, not the command.
            if !at_word_start || !at_word_end || after_determiner(&result, abs_pos) {
                search_from = end;
                continue;
            }

            // Remove space before punctuation that attaches to previous word, and,
            // for a mark, the mark the model already put there.
            let mut trim_start = if abs_pos > 0 && bytes[abs_pos - 1] == b' ' {
                abs_pos - 1
            } else {
                abs_pos
            };
            if is_mark && trim_start > 0 && MODEL_MARKS.contains(&bytes[trim_start - 1]) {
                trim_start -= 1;
            }

            // Remove the model's marks straight after a spoken mark, then a space.
            // A sentence break among them ("colon. The") also capitalised the next word.
            let mut trim_end = end;
            let mut model_broke_sentence = false;
            if is_mark {
                while trim_end < result.len() && MODEL_MARKS.contains(&bytes[trim_end]) {
                    model_broke_sentence |= matches!(bytes[trim_end], b'.' | b'!' | b'?');
                    trim_end += 1;
                }
            }
            if trim_end < result.len() && bytes[trim_end] == b' ' {
                trim_end += 1;
            }

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

            // The model heard a sentence end at a spoken comma, colon or semicolon and
            // capitalised the next word ("Note colon. The meeting", "Wait. Comma. Not
            // yet"). Mid-sentence again, an ordinary word goes back to lower case;
            // a vocabulary name, "I" or an acronym keeps its capital (2026-11-23).
            if model_broke_sentence && matches!(symbol, "," | ";" | ":") {
                let rest = &result[search_from..];
                let word: String = rest.chars().take_while(|c| c.is_alphabetic() || *c == '\'').collect();
                let mut chars = word.chars();
                let ordinary = match (chars.next(), chars.clone().next()) {
                    (Some(first), Some(second)) => first.is_uppercase() && second.is_lowercase(),
                    _ => false,
                } && crate::lexicon::contains(&word.to_lowercase());
                if ordinary {
                    let known = names.get_or_insert_with(|| crate::vocabulary::canonicals(&crate::vocabulary::path()));
                    if !known.iter().any(|n| n.split_whitespace().any(|part| part == word)) {
                        let first = word.chars().next().expect("non-empty");
                        let lowered: String = first.to_lowercase().collect();
                        result.replace_range(search_from..search_from + first.len_utf8(), &lowered);
                    }
                }
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Deterministic capitalization
// ---------------------------------------------------------------------------

static LOWERCASE_I: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bi\b").unwrap());

/// Capitalize the first letter of the transcript and the first letter after a
/// sentence terminator or newline. Existing casing is preserved everywhere
/// else; this pass never lowercases acronyms.
fn capitalize_sentences(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalize_next = true;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if capitalize_next && ch.is_alphabetic() {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }

        match ch {
            '?' | '!' | '\n' => capitalize_next = true,
            '.' if chars.peek().is_none_or(|next| next.is_whitespace()) => {
                capitalize_next = true;
            }
            // Keep looking through whitespace and opening punctuation for the
            // first actual letter: `hello. "goodbye"` -> `Hello. "Goodbye"`.
            _ if capitalize_next
                && !ch.is_whitespace()
                && !matches!(ch, '"' | '\'' | '(' | '[' | '{' | '“' | '‘') =>
            {
                capitalize_next = false;
            }
            _ => {}
        }
    }

    LOWERCASE_I.replace_all(&result, "I").into_owned()
}

// ---------------------------------------------------------------------------
// Public pipeline
// ---------------------------------------------------------------------------

/// Course correction, repetition and filler cleanup, spoken punctuation,
/// then deterministic capitalization. Vocabulary decisions run on the result.
pub fn fix_transcription(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let text = apply_course_correction(text);
    let text = clean_repetitions(&text);
    let text = remove_fillers(&text);
    let text = replace_spoken_punctuation(&text);
    capitalize_sentences(&text)
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

    // -- A spoken mark beats the model's own punctuation beside it (2026-11-22) --
    // Raw engine output on Kale-spoken test takes, not household speech.

    #[test]
    fn spoken_mark_at_the_end_absorbs_the_models_stop() {
        // Parakeet and Phonon both end the take with their own full stop.
        assert_eq!(replace_spoken_punctuation("and bread period."), "and bread.");
        assert_eq!(replace_spoken_punctuation("feed the dog exclamation point."), "feed the dog!");
    }

    #[test]
    fn spoken_comma_after_the_models_comma_is_one_comma() {
        // Parakeet acted on two spoken commas itself and left the third as a word.
        assert_eq!(
            replace_spoken_punctuation("Buy milk, eggs, comma and bread period."),
            "Buy milk, eggs, and bread."
        );
    }

    #[test]
    fn phonons_exclamation_pointer() {
        assert_eq!(replace_spoken_punctuation("feed the dog exclamation pointer."), "feed the dog!");
    }

    #[test]
    fn a_keyword_inside_a_word_is_still_left_alone() {
        assert_eq!(replace_spoken_punctuation("periodic commas"), "periodic commas");
    }

    // -- After a determiner a command word is the noun (2026-11-22) --

    #[test]
    fn determiner_makes_the_noun() {
        for text in [
            "over a period of time",
            "add a dash of salt",
            "write a new line of code",
            "she was in a coma",
            "the colon is part of the gut",
            "put a comma here",
        ] {
            assert_eq!(replace_spoken_punctuation(text), text, "{text}");
        }
    }

    #[test]
    fn commands_still_fire_without_a_determiner() {
        assert_eq!(replace_spoken_punctuation("call the vet period"), "call the vet.");
        assert_eq!(replace_spoken_punctuation("one new line two"), "one\ntwo");
        assert_eq!(replace_spoken_punctuation("milk coma eggs"), "milk, eggs");
        // A command after a full stop is not after a determiner.
        assert_eq!(replace_spoken_punctuation("Done. New paragraph. Next"), "Done.\n\nNext");
    }

    // -- John's read-aloud, 2026-11-23: Parakeet's raw lines, generic script --

    #[test]
    fn quotes_hug_what_they_enclose() {
        assert_eq!(
            replace_spoken_punctuation("Open quote, not today, close quote. She said, period."),
            "\"not today\". She said."
        );
        assert_eq!(replace_spoken_punctuation("see open paren page two close paren now"), "see (page two) now");
    }

    #[test]
    fn a_spoken_mark_undoes_the_models_sentence_capital() {
        assert_eq!(
            replace_spoken_punctuation("Note colon. The meeting moved semicolon. Bring the slides. Period."),
            "Note: the meeting moved; bring the slides."
        );
        assert_eq!(replace_spoken_punctuation("Wait. Comma. Not yet. Period."), "Wait, not yet.");
    }

    #[test]
    fn a_name_keeps_its_capital_after_a_spoken_comma() {
        // Not an ordinary English word, so not lowered.
        assert_eq!(replace_spoken_punctuation("Hi. Comma. Bluey is here."), "Hi, Bluey is here.");
        // "I" always keeps its capital.
        assert_eq!(replace_spoken_punctuation("Wait. Comma. I know."), "Wait, I know.");
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
    fn new_paragraph_absorbs_model_punctuation() {
        assert_eq!(
            fix_transcription("This is one. New paragraph. This is two."),
            "This is one.\n\nThis is two."
        );
        assert_eq!(
            fix_transcription("This is one, new paragraph, this is two."),
            "This is one\n\nThis is two."
        );
        assert_eq!(fix_transcription("New paragraph."), "\n\n");
        assert_eq!(
            fix_transcription("Milk, new line, eggs, new line, bread."),
            "Milk\nEggs\nBread."
        );
    }

    #[test]
    fn new_line_inside_a_word_is_not_a_command() {
        assert_eq!(replace_spoken_punctuation("renew lines"), "renew lines");
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
    fn capitalizes_leading_letter() {
        let result = fix_transcription("there is no way she is not guilty");
        assert_eq!(result, "There is no way she is not guilty");
    }

    #[test]
    fn capitalizes_lone_i() {
        let result = fix_transcription("i went to the store");
        assert_eq!(result, "I went to the store");
    }

    #[test]
    fn capitalizes_after_sentence_punctuation_and_newlines() {
        assert_eq!(
            capitalize_sentences("hello. goodbye? yes! absolutely\nnext line"),
            "Hello. Goodbye? Yes! Absolutely\nNext line"
        );
    }

    #[test]
    fn capitalizes_through_opening_quote() {
        assert_eq!(
            capitalize_sentences("hello. \"goodbye, friend\""),
            "Hello. \"Goodbye, friend\""
        );
    }

    #[test]
    fn comma_does_not_start_a_sentence() {
        assert_eq!(capitalize_sentences("first, second"), "First, second");
    }

    #[test]
    fn preserves_existing_acronyms_and_decimal_points() {
        assert_eq!(
            capitalize_sentences("use ONNX Runtime 1.24 today"),
            "Use ONNX Runtime 1.24 today"
        );
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
        let result = fix_transcription("i think this is really great period and i hope it works");
        assert_eq!(result, "I think this is really great. And I hope it works");
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

    // -- Engine parity --
    //
    // Both engines feed this same function (main.rs::run_transcription_pipeline)
    // but hand it differently shaped raw text: Parakeet TDT emits its own casing
    // and punctuation, Granite's CTC head emits bare lowercase. These two cases
    // lock in that the correction/vocabulary passes fire either way.

    #[test]
    fn handles_granite_shaped_bare_lowercase_output() {
        // Granite CTC: no casing, no punctuation, spoken punctuation only.
        let result = fix_transcription(
            "i uh deployed to the the cluster period no wait i rolled it back period",
        );
        assert_eq!(result, "I rolled it back.");
    }

    #[test]
    fn handles_parakeet_shaped_cased_punctuated_output() {
        // Parakeet TDT: already cased and punctuated. The same passes must fire
        // and must not fight the casing the model already produced.
        let result =
            fix_transcription("I uh deployed to the the cluster. No wait, I rolled it back.");
        assert_eq!(result, "I rolled it back.");
    }
}
