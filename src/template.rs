//! Spec template generation and structural verification.

use crate::diff::extract_sections;
use crate::error::SpecKitError;

/// The `##` sections every Lockamy Studios protocol spec must contain, in
/// the order [`skeleton`] emits them.
pub const REQUIRED_SECTIONS: [&str; 7] = [
    "Overview",
    "Wire format",
    "Messages",
    "Errors",
    "Versioning",
    "Conformance",
    "Changelog",
];

/// Return a starter spec markdown skeleton with the standard sections every
/// Lockamy Studios protocol spec must contain.
pub fn skeleton(name: &str) -> String {
    format!(
        "# {name} — Protocol Specification

> Version: 0.1.0
> Status: draft
> Owner: <kit/repo>

## Overview

<one paragraph explaining what this protocol does and who consumes it>

## Wire format

<the bytes / JSON / proto3 / etc>

## Messages

<message-by-message reference>

## Errors

<error model>

## Versioning

This spec follows the studio's spec-versioning rules (see Spec Kit `version` module).
Changes are classified as Breaking, Additive, or Editorial.

## Conformance

<which conformance suite a producer/consumer of this protocol must pass>

## Changelog

- 0.1.0 — initial draft.
"
    )
}

/// Verify that `input` (a spec's markdown text) contains every section in
/// [`REQUIRED_SECTIONS`]. Returns the missing section titles, in order, as
/// a [`SpecKitError::Parse`] if any are absent.
pub fn verify(input: &str) -> Result<(), SpecKitError> {
    let present: Vec<String> = extract_sections(input)
        .into_iter()
        .map(|(title, _)| title)
        .collect();

    let missing: Vec<&str> = REQUIRED_SECTIONS
        .iter()
        .filter(|required| !present.iter().any(|p| p == *required))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(SpecKitError::Parse(format!(
            "missing required section(s): {}",
            missing.join(", ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_skeleton_verifies() {
        let spec = skeleton("test-protocol");
        assert!(verify(&spec).is_ok());
    }

    #[test]
    fn missing_section_fails_verification() {
        let spec = "# X — Protocol Specification\n\n## Overview\n\nText.\n";
        let err = verify(spec).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Wire format"));
        assert!(msg.contains("Changelog"));
    }

    #[test]
    fn empty_document_fails_verification() {
        assert!(verify("").is_err());
    }
}
