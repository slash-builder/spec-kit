//! # Spec Kit
//!
//! Protocol-specification methodology + conformance harness primitives for
//! the SlashBuilder Kit ecosystem.
//!
//! Spec Kit is governance, not runtime. It produces the *shapes* of specs
//! and the *primitives* every Kit's conformance test suite imports.
//!
//! See the README for how Spec Kit fits the Kit model.

pub mod conformance;
pub mod diff;
pub mod error;
pub mod template;
pub mod version;

pub use error::SpecKitError;

/// Crate version, sourced from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
