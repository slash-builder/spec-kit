//! Spec-version semantics.
//!
//! Specs are versioned with semver, but the studio maps that to a concrete
//! rule: the class of change ([`crate::diff::ChangeKind`]) determines which
//! semver component must move. This module is that mapping, plus small
//! helpers for applying it to a [`Version`].

use crate::diff::ChangeKind;

pub use semver::Version;

/// The semver component that must be bumped for a given class of spec
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    /// A breaking change requires a major bump.
    Major,
    /// An additive change requires a minor bump.
    Minor,
    /// An editorial change requires (at least) a patch bump.
    Patch,
}

/// Map a [`ChangeKind`] to the semver bump the studio's spec-versioning
/// rules require: Breaking → major, Additive → minor, Editorial → patch.
pub fn required_bump(kind: ChangeKind) -> Bump {
    match kind {
        ChangeKind::Breaking => Bump::Major,
        ChangeKind::Additive => Bump::Minor,
        ChangeKind::Editorial => Bump::Patch,
    }
}

/// Apply a [`Bump`] to `version`, following standard semver bump semantics
/// (bumping a component resets every component to its right to zero).
pub fn apply_bump(version: &Version, bump: Bump) -> Version {
    match bump {
        Bump::Major => Version::new(version.major + 1, 0, 0),
        Bump::Minor => Version::new(version.major, version.minor + 1, 0),
        Bump::Patch => Version::new(version.major, version.minor, version.patch + 1),
    }
}

/// Classify the change from `old_spec` to `new_spec` and compute the next
/// version directly from `current`. Convenience wrapper over
/// [`crate::diff::compare`] + [`required_bump`] + [`apply_bump`].
pub fn next_version(current: &Version, old_spec: &str, new_spec: &str) -> Version {
    let kind = crate::diff::compare(old_spec, new_spec);
    apply_bump(current, required_bump(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaking_requires_major() {
        assert_eq!(required_bump(ChangeKind::Breaking), Bump::Major);
    }

    #[test]
    fn additive_requires_minor() {
        assert_eq!(required_bump(ChangeKind::Additive), Bump::Minor);
    }

    #[test]
    fn editorial_requires_patch() {
        assert_eq!(required_bump(ChangeKind::Editorial), Bump::Patch);
    }

    #[test]
    fn major_bump_resets_minor_and_patch() {
        let v = Version::new(1, 4, 7);
        assert_eq!(apply_bump(&v, Bump::Major), Version::new(2, 0, 0));
    }

    #[test]
    fn minor_bump_resets_patch() {
        let v = Version::new(1, 4, 7);
        assert_eq!(apply_bump(&v, Bump::Minor), Version::new(1, 5, 0));
    }

    #[test]
    fn patch_bump_increments_patch_only() {
        let v = Version::new(1, 4, 7);
        assert_eq!(apply_bump(&v, Bump::Patch), Version::new(1, 4, 8));
    }

    #[test]
    fn next_version_follows_breaking_change() {
        let old = "## Wire format\n\nThe id MUST be a UUID.\n";
        let new = "## Wire format\n\nThe id is a string.\n";
        let current = Version::new(1, 2, 3);
        assert_eq!(next_version(&current, old, new), Version::new(2, 0, 0));
    }

    #[test]
    fn next_version_follows_additive_change() {
        let old = "## Overview\n\nText.\n";
        let new = "## Overview\n\nText.\n\n## Extensions\n\nOptional.\n";
        let current = Version::new(1, 2, 3);
        assert_eq!(next_version(&current, old, new), Version::new(1, 3, 0));
    }
}
