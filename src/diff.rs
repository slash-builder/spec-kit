//! Spec-diff classification.
//!
//! `compare` classifies the change between two versions of a spec document
//! (the markdown shape produced by [`crate::template::skeleton`]) as
//! [`ChangeKind::Breaking`], [`ChangeKind::Additive`], or
//! [`ChangeKind::Editorial`].
//!
//! The rule, grounded in the spec's `##` section structure and RFC 2119
//! normative keywords:
//!
//! - **Breaking** — a `##` section present in the old spec is missing from
//!   the new one, *or* a normative statement (a sentence containing
//!   `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, or `REQUIRED`) present in the
//!   old spec no longer appears verbatim in the new spec (covers both
//!   removal and in-place rewording of a requirement).
//! - **Additive** — no breaking condition, but the new spec introduces a
//!   `##` section or normative statement that didn't exist before (e.g. a
//!   new optional section, or a new `MUST` describing a new capability).
//! - **Editorial** — neither of the above: only prose, formatting, or
//!   non-normative wording changed.

use std::collections::HashSet;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Classification of a change between two spec versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Renames, type changes, removals, semantic shifts.
    Breaking,
    /// New fields, new messages, new variants — safe to add.
    Additive,
    /// Wording, formatting, comments — no wire effect.
    Editorial,
}

/// RFC 2119-style normative keywords a spec's requirements are phrased with.
const NORMATIVE_KEYWORDS: [&str; 5] = ["MUST NOT", "MUST", "SHALL NOT", "SHALL", "REQUIRED"];

/// Classify the change from `old_spec` to `new_spec`. Both are the full
/// markdown text of a spec document.
pub fn compare(old_spec: &str, new_spec: &str) -> ChangeKind {
    let old_sections = extract_sections(old_spec);
    let new_sections = extract_sections(new_spec);

    let old_titles: HashSet<&str> = old_sections.iter().map(|(t, _)| t.as_str()).collect();
    let new_titles: HashSet<&str> = new_sections.iter().map(|(t, _)| t.as_str()).collect();

    let section_removed = old_titles.difference(&new_titles).next().is_some();

    let old_normative = normative_statements(&old_sections);
    let new_normative = normative_statements(&new_sections);

    let normative_removed = old_normative.difference(&new_normative).next().is_some();

    if section_removed || normative_removed {
        return ChangeKind::Breaking;
    }

    let section_added = new_titles.difference(&old_titles).next().is_some();
    let normative_added = new_normative.difference(&old_normative).next().is_some();

    if section_added || normative_added {
        return ChangeKind::Additive;
    }

    ChangeKind::Editorial
}

/// Extract the ordered list of `(section title, section body text)` for
/// every level-2 (`##`) heading in `md`. Body text runs until the next
/// heading of level 2 or shallower.
pub(crate) fn extract_sections(md: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut in_h2 = false;
    let mut heading_buf = String::new();
    let mut collecting_heading = false;

    for event in Parser::new(md) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                collecting_heading = true;
                heading_buf.clear();
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                collecting_heading = false;
                sections.push((heading_buf.trim().to_string(), String::new()));
                in_h2 = true;
            }
            Event::Text(text) | Event::Code(text) if collecting_heading => {
                heading_buf.push_str(&text);
            }
            Event::Text(text) | Event::Code(text) if in_h2 => {
                if let Some((_, body)) = sections.last_mut() {
                    body.push_str(&text);
                    body.push(' ');
                }
            }
            Event::Start(Tag::Heading { level, .. }) if level != HeadingLevel::H2 => {
                // A heading at a different level closes the current H2 section.
                in_h2 = false;
            }
            _ => {}
        }
    }

    sections
}

/// Collect every normative sentence (containing an RFC 2119 keyword) across
/// all sections, normalized (whitespace-collapsed) so cosmetic reformatting
/// of the same requirement doesn't register as a change.
fn normative_statements(sections: &[(String, String)]) -> HashSet<String> {
    let mut statements = HashSet::new();
    for (_, body) in sections {
        for sentence in split_sentences(body) {
            let normalized = normalize(&sentence);
            if NORMATIVE_KEYWORDS
                .iter()
                .any(|kw| contains_word(&normalized, kw))
            {
                statements.insert(normalized);
            }
        }
    }
    statements
}

fn split_sentences(text: &str) -> Vec<String> {
    text.split(['.', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whole-word (case-sensitive) containment check — avoids `MUST` matching
/// inside e.g. `AMUSTACHE`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    haystack
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(needle.split_whitespace().count().max(1))
        .any(|window| window.join(" ") == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::skeleton;

    #[test]
    fn identical_specs_are_editorial() {
        let spec = skeleton("test-protocol");
        assert_eq!(compare(&spec, &spec), ChangeKind::Editorial);
    }

    #[test]
    fn prose_only_change_is_editorial() {
        let old = "## Overview\n\nA short description of the protocol.\n";
        let new = "## Overview\n\nA slightly longer description of the protocol.\n";
        assert_eq!(compare(old, new), ChangeKind::Editorial);
    }

    #[test]
    fn removed_section_is_breaking() {
        let old = "## Overview\n\nText.\n\n## Wire format\n\nBytes.\n";
        let new = "## Overview\n\nText.\n";
        assert_eq!(compare(old, new), ChangeKind::Breaking);
    }

    #[test]
    fn removed_normative_statement_is_breaking() {
        let old = "## Wire format\n\nThe length prefix MUST be 4 bytes.\n";
        let new = "## Wire format\n\nThe length prefix is 4 bytes.\n";
        assert_eq!(compare(old, new), ChangeKind::Breaking);
    }

    #[test]
    fn reworded_normative_statement_is_breaking() {
        let old = "## Wire format\n\nThe length prefix MUST be 4 bytes.\n";
        let new = "## Wire format\n\nThe length prefix MUST be 8 bytes.\n";
        assert_eq!(compare(old, new), ChangeKind::Breaking);
    }

    #[test]
    fn new_optional_section_is_additive() {
        let old = "## Overview\n\nText.\n";
        let new = "## Overview\n\nText.\n\n## Extensions\n\nOptional extension points.\n";
        assert_eq!(compare(old, new), ChangeKind::Additive);
    }

    #[test]
    fn new_normative_statement_without_removal_is_additive() {
        let old = "## Messages\n\nThe Hello message exists.\n";
        let new =
            "## Messages\n\nThe Hello message exists. A new Ping message MUST be supported.\n";
        assert_eq!(compare(old, new), ChangeKind::Additive);
    }

    #[test]
    fn added_and_removed_together_is_breaking() {
        // Additive content doesn't downgrade a breaking removal.
        let old = "## Wire format\n\nThe id MUST be a UUID.\n";
        let new = "## Wire format\n\nThe id is a string.\n\n## Extensions\n\nNew stuff.\n";
        assert_eq!(compare(old, new), ChangeKind::Breaking);
    }
}
