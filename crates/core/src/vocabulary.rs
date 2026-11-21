//! Path and persistence for the shared Markdown vocabulary.

use crate::correction::Pair;
use std::{path::PathBuf, sync::RwLock};

static PATH: RwLock<Option<PathBuf>> = RwLock::new(None);

pub fn configure(path: Option<PathBuf>) {
    *PATH.write().unwrap() = path;
}

/// Path to the user dictionary: `~/syncthing/john-vocab.md`, the one vocabulary
/// shared with mopvox and dayshade.
pub fn path() -> PathBuf {
    if let Some(path) = PATH.read().unwrap().as_ref() {
        return path.clone();
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syncthing")
        .join("john-vocab.md")
}

/// Kinds that are dayshade's alone. Habit and task names are short and
/// ordinary — `Cigs` would claim every `six` — so dictation never sees them.
const DAYSHADE_ONLY: &[&str] = &["habit", "task"];

/// Each `## Term` at column 0 of the shared vocabulary is an entry, kept unless
/// its `kind:` is dayshade's alone. A file with no `## ` headings is the old
/// `dictionary.txt`: one term per line, `#` comments and blank lines ignored.
fn parse_dictionary(contents: &str) -> Vec<String> {
    let heading = |line: &str| {
        line.strip_prefix("##")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .map(str::trim)
            .filter(|term| !term.is_empty())
            .map(str::to_string)
    };
    if !contents.lines().any(|line| heading(line).is_some()) {
        return contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect();
    }

    let mut out = Vec::new();
    let mut term: Option<String> = None;
    let mut kind = String::new();
    let mut flush = |term: &mut Option<String>, kind: &mut String| {
        if let Some(t) = term.take() {
            if !DAYSHADE_ONLY.contains(&kind.as_str()) {
                out.push(t);
            }
        }
        kind.clear();
    };
    for line in contents.lines() {
        if let Some(t) = heading(line) {
            flush(&mut term, &mut kind);
            term = Some(t);
        } else if line.starts_with('#') {
            flush(&mut term, &mut kind);
        } else if let Some(k) = line
            .trim_start_matches(['-', '*'])
            .trim_start()
            .strip_prefix("kind:")
        {
            if !line.starts_with(char::is_whitespace) {
                kind = k.trim().to_lowercase();
            }
        }
    }
    flush(&mut term, &mut kind);
    out
}

/// Entry names in the vocabulary file (dictation's view: habits/tasks excluded).
/// A missing or unreadable file has none.
pub fn canonicals(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map(|contents| parse_dictionary(&contents))
        .unwrap_or_default()
}

const ALIAS_FIELDS: &[&str] = &["heard", "homophones", "distinct"];

/// Where a learned alias is filed (John, 2026-11-20). A single word that is not
/// English can only mean the name: `distinct`, replaced directly. A real word competes
/// with the name and needs context: `homophones`, the known-issue list. Anything
/// longer stays `heard`.
pub fn alias_field(heard: &str) -> &'static str {
    let heard = heard.trim();
    if heard.contains(char::is_whitespace) || heard.contains('-') {
        "heard"
    } else if crate::lexicon::is_word(heard) {
        "homophones"
    } else {
        "distinct"
    }
}

/// Re-read and merge before writing, preserving unrelated entries and prose.
/// Same-directory atomic rename prevents a reader seeing a half-written file.
pub fn save_pairs(path: &std::path::Path, pairs: &[Pair]) -> Result<(), String> {
    if pairs.is_empty() {
        return Ok(());
    }
    if pairs.iter().any(|p| !crate::correction::learnable(p)) {
        return Err("Invalid vocabulary proposal".into());
    }
    let mut contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "# Vocabulary\n".to_string(),
        Err(e) => return Err(e.to_string()),
    };
    // Avoid mixing the legacy one-term-per-line format with Markdown sections.
    if !contents.lines().any(|l| l.starts_with("## ")) {
        let terms = parse_dictionary(&contents);
        if !terms.is_empty() {
            contents = format!(
                "# Vocabulary\n{}",
                terms
                    .iter()
                    .map(|t| format!("\n## {t}\nkind: term\n"))
                    .collect::<String>()
            );
        }
    }
    for pair in pairs {
        let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
        // A capitalization promotion creates the entry, never an alias.
        if crate::correction::case_promotion(pair) {
            let exists = lines.iter().any(|l| {
                l.strip_prefix("## ").is_some_and(|t| t.trim().eq_ignore_ascii_case(&pair.canonical))
            });
            if !exists {
                lines.extend([String::new(), format!("## {}", pair.canonical), "kind: term".into()]);
                contents = lines.join("\n") + "\n";
            }
            continue;
        }
        if let Some(start) = lines.iter().position(|l| {
            l.strip_prefix("## ")
                .is_some_and(|t| t.trim().eq_ignore_ascii_case(&pair.canonical))
        }) {
            let end = (start + 1..lines.len())
                .find(|&i| lines[i].starts_with('#'))
                .unwrap_or(lines.len());
            let excluded = lines[start + 1..end].iter().any(|l| {
                l.trim_start_matches(['-', '*'])
                    .trim_start()
                    .strip_prefix("kind:")
                    .is_some_and(|k| DAYSHADE_ONLY.contains(&k.trim().to_lowercase().as_str()))
            });
            if excluded {
                return Err(format!(
                    "{} is a habit/task entry; dictation cannot learn aliases for it",
                    pair.canonical
                ));
            }
            let known = lines[start + 1..end].iter().any(|l| {
                ALIAS_FIELDS.iter().any(|f| {
                    l.strip_prefix(f).and_then(|r| r.strip_prefix(':')).is_some_and(|list| {
                        list.split(',').any(|h| h.trim().eq_ignore_ascii_case(&pair.heard))
                    })
                })
            });
            if !known {
                let field = alias_field(&pair.heard);
                let prefix = format!("{field}:");
                if let Some(index) = (start + 1..end).find(|&i| lines[i].starts_with(&prefix)) {
                    let existing = lines[index][prefix.len()..].trim();
                    lines[index] = if existing.is_empty() {
                        format!("{prefix} {}", pair.heard)
                    } else {
                        format!("{prefix} {existing}, {}", pair.heard)
                    };
                } else {
                    // After the entry's last non-blank line, so a trailing blank stays a separator.
                    let mut at = end;
                    while at > start + 1 && lines[at - 1].trim().is_empty() {
                        at -= 1;
                    }
                    lines.insert(at, format!("{prefix} {}", pair.heard));
                }
            }
        } else {
            lines.extend([
                String::new(),
                format!("## {}", pair.canonical),
                "kind: term".into(),
                format!("{}: {}", alias_field(&pair.heard), pair.heard),
            ]);
        }
        contents = lines.join("\n") + "\n";
    }
    let parent = path
        .parent()
        .ok_or("Vocabulary path needs a parent directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = parent.join(format!(
        ".transcrust-vocab-{}-{nonce}.tmp",
        std::process::id()
    ));
    let result = (|| {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        if let Ok(metadata) = std::fs::metadata(path) {
            file.set_permissions(metadata.permissions())
                .map_err(|e| e.to_string())?;
        }
        file.write_all(contents.as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_comments_and_blanks() {
        let raw = "# names\nKubernetes\n\n  Svelte  \n# trailing\n";
        assert_eq!(parse_dictionary(raw), vec!["Kubernetes", "Svelte"]);
    }

    /// The shared vocabulary: habits and tasks are dayshade's; an indented
    /// example in the prose is not an entry.
    #[test]
    fn shared_vocabulary_keeps_habits_and_tasks_out() {
        let md = "# John's vocabulary\n\n    ## Apptrack\n    kind: task\n\n\
                  ## Wayland\nkind: term\n\n## Cigs\nkind: habit\nheard: sigs, six\n\n\
                  ## Apptrack\nkind: task\n\n## Gambol\n- kind: person\n\n## Tauri\n";
        assert_eq!(parse_dictionary(md), vec!["Wayland", "Gambol", "Tauri"]);
    }

    #[test]
    fn kept_pairs_merge_preserve_and_reload_without_restart() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("transcrust-pairs-{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vocab.md");
        std::fs::write(&path, "# Vocabulary\nKeep this prose.\n\n## Kubernetes\nkind: term\nheard: kubernetties\n\n## Cigs\nkind: habit\nheard: six\n").unwrap();
        let pair = Pair {
            heard: "kubernetis".into(),
            canonical: "Kubernetes".into(),
        };
        save_pairs(&path, &[pair.clone()]).unwrap();
        save_pairs(&path, &[pair.clone()]).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("Keep this prose."));
        // not an English word: filed as distinct, beside the existing heard line
        assert!(saved.contains("heard: kubernetties\ndistinct: kubernetis"), "{saved}");
        assert!(saved.contains("## Cigs\nkind: habit\nheard: six"));
        assert_eq!(saved.matches("kubernetis").count(), 1);
        let blocked = Pair {
            heard: "sigs".into(),
            canonical: "Cigs".into(),
        };
        assert!(save_pairs(&path, &[blocked]).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Pixel, 2026-11-21: "Australian shepherd" → "Shepherd" is case-only. It used to be
    /// rejected, so Learn never appeared. Now it creates the entry once, with no alias.
    #[test]
    fn capitalization_promotion_creates_the_entry_once() {
        let pair = |h: &str, c: &str| Pair { heard: h.into(), canonical: c.into() };
        assert!(crate::correction::case_promotion(&pair("shepherd", "Shepherd")));
        assert!(!crate::correction::case_promotion(&pair("Shepherd", "shepherd")));
        assert!(!crate::correction::valid_pair(&pair("shepherd", "Shepherd")));
        let dir = std::env::temp_dir().join(format!("transcrust-case-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vocab.md");
        std::fs::write(&path, "## Gambol\nkind: person\n").unwrap();
        save_pairs(&path, &[pair("australian shepherd", "Australian Shepherd")]).unwrap();
        save_pairs(&path, &[pair("australian shepherd", "Australian Shepherd")]).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert_eq!(saved, "## Gambol\nkind: person\n\n## Australian Shepherd\nkind: term\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// bloey corpus aliases (2026-11-20): non-words are distinct, real words are
    /// homophones (the known-issue list), phrases stay heard; an alias already filed
    /// under any field is not added again.
    #[test]
    fn learned_aliases_are_filed_by_kind() {
        assert_eq!(alias_field("Blooie"), "distinct");
        assert_eq!(alias_field("Louie"), "homophones");
        assert_eq!(alias_field("unicorns"), "homophones");
        assert_eq!(alias_field("blue e"), "heard");
        let dir = std::env::temp_dir().join(format!("transcrust-alias-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bluey-vocab.md");
        std::fs::write(&path, "## Bluey\nkind: person\nheard: blue e\n\n## Bingo\nkind: person\nhomophones: bingo\n").unwrap();
        let pair = |heard: &str, canonical: &str| Pair { heard: heard.into(), canonical: canonical.into() };
        // a batch with an invalid pair writes nothing
        assert!(save_pairs(&path, &[pair("Blooie", "Bluey"), pair("Bingo", "Bingo")]).is_err());
        save_pairs(&path, &[pair("Blooie", "Bluey"), pair("Louie", "Bluey")]).unwrap();
        save_pairs(&path, &[pair("blooie", "Bluey"), pair("Jilly", "Chilli")]).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            saved,
            "## Bluey\nkind: person\nheard: blue e\ndistinct: Blooie\nhomophones: Louie\n\n\
             ## Bingo\nkind: person\nhomophones: bingo\n\n## Chilli\nkind: term\ndistinct: Jilly\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
