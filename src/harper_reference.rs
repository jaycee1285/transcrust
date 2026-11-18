use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{remove_overlaps, Dialect, Document};
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

