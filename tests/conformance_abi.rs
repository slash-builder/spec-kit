//! Interface-lock gate for the frozen `conformance` ABI (2026-08-26).
//!
//! `conformance::{Case, run_suite, ConformanceReport}` is the studio-wide
//! conformance-test ABI: every Kit's `tests/conformance.rs` imports these
//! exact signatures, and each Kit's own lock depends on them not moving.
//! `slash-builder/storage-kit` is the first real external consumer,
//! resolving this crate from Nexus.
//!
//! That cross-repo consumer is real but it is not a *guard*: a signature
//! change here would leave this repo's CI green and break storage-kit's
//! only on its next build. This suite closes that gap. It lives in
//! `tests/` on purpose — an integration test links `spec_kit` as an
//! external crate, so it can reach nothing but the public surface, which
//! is exactly the property the ABI must have. It exercises the ABI the way
//! a Kit does, so breaking the ABI breaks *this* repo's CI, at the source.
//!
//! Per the studio lock bar, changing anything asserted here is a
//! major-version change requiring a data-architect ruling — adding
//! ergonomics around the shape is fine, altering the shape is not.

use spec_kit::conformance::{run_suite, Case, ConformanceReport};

/// A Kit-shaped case: an external type implementing the frozen trait.
/// That this compiles at all is half the assertion — `Case` must be
/// implementable, and object-safe, from outside the crate.
struct Fixture {
    name: &'static str,
    outcome: Result<(), String>,
}

impl Case for Fixture {
    fn name(&self) -> &str {
        self.name
    }
    fn run(&self) -> Result<(), String> {
        self.outcome.clone()
    }
}

fn pass(name: &'static str) -> Fixture {
    Fixture {
        name,
        outcome: Ok(()),
    }
}

fn fail(name: &'static str, why: &str) -> Fixture {
    Fixture {
        name,
        outcome: Err(why.to_string()),
    }
}

/// The call shape every Kit uses verbatim: box the cases, collect
/// `&dyn Case` references, hand the slice to `run_suite`, assert on the
/// report. If this stops compiling, every Kit's conformance suite has
/// stopped compiling.
#[test]
fn kit_call_shape_compiles_and_reports_success() {
    let cases: Vec<Box<dyn Case>> = vec![Box::new(pass("alpha")), Box::new(pass("beta"))];
    let case_refs: Vec<&dyn Case> = cases.iter().map(|c| c.as_ref()).collect();
    let report: ConformanceReport = run_suite(&case_refs);

    assert!(report.is_success());
    assert_eq!(report.total(), 2);
    assert!(report.failed.is_empty());
}

/// A failing case must be reported as `(name, message)` with the message
/// carried verbatim — Kits print this straight to CI output, so the tuple
/// shape and the message fidelity are both contractual.
#[test]
fn failure_is_reported_as_name_and_verbatim_message() {
    let cases: Vec<Box<dyn Case>> = vec![
        Box::new(pass("alpha")),
        Box::new(fail("beta", "expected 3 blocks, found 2")),
    ];
    let case_refs: Vec<&dyn Case> = cases.iter().map(|c| c.as_ref()).collect();
    let report = run_suite(&case_refs);

    assert!(!report.is_success());
    assert_eq!(report.total(), 2);
    assert_eq!(report.passed, vec!["alpha".to_string()]);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, "beta");
    assert_eq!(report.failed[0].1, "expected 3 blocks, found 2");
}

/// `run_suite` runs every case rather than short-circuiting on the first
/// failure, and preserves submission order within each bucket. Kits rely
/// on this to report all failures in one CI run instead of one per push.
#[test]
fn every_case_runs_and_order_is_preserved() {
    let cases: Vec<Box<dyn Case>> = vec![
        Box::new(fail("one", "first")),
        Box::new(pass("two")),
        Box::new(fail("three", "second")),
        Box::new(pass("four")),
    ];
    let case_refs: Vec<&dyn Case> = cases.iter().map(|c| c.as_ref()).collect();
    let report = run_suite(&case_refs);

    assert_eq!(report.total(), 4, "run_suite short-circuited on a failure");
    assert_eq!(report.passed, vec!["two".to_string(), "four".to_string()]);
    let failed_names: Vec<&str> = report.failed.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(failed_names, vec!["one", "three"]);
}

/// The empty suite is vacuously successful — a Kit wiring its first case
/// must not have to special-case the zero-case path.
#[test]
fn empty_suite_is_vacuously_successful() {
    let report = run_suite(&[]);
    assert!(report.is_success());
    assert_eq!(report.total(), 0);
}

/// `ConformanceReport` is publicly constructible via `Default` with both
/// fields public, so a Kit can aggregate several sub-suites into one
/// report without going through `run_suite`.
#[test]
fn report_is_externally_constructible_with_public_fields() {
    let mut report = ConformanceReport::default();
    assert!(report.is_success());

    report.passed.push("aggregated-pass".to_string());
    report
        .failed
        .push(("aggregated-fail".to_string(), "why".to_string()));

    assert!(!report.is_success());
    assert_eq!(report.total(), 2);
}
