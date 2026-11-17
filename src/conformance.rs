//! Conformance-harness primitives — the studio-wide test ABI.
//!
//! `Case`, `ConformanceReport`, and `run_suite` are **frozen**: every Kit's
//! `tests/conformance.rs` imports these exact signatures to prove an
//! implementation conforms to its spec. Do not change them; add ergonomics
//! (like [`ConformanceReport::is_success`]) instead of altering the shape.

/// Conformance-harness primitives. Kits import these in `tests/conformance.rs`.
pub trait Case {
    fn name(&self) -> &str;
    fn run(&self) -> Result<(), String>;
}

/// Report from running a suite of conformance cases.
#[derive(Debug, Default)]
pub struct ConformanceReport {
    pub passed: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl ConformanceReport {
    /// `true` if every case in the suite passed.
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    /// Total number of cases run (passed + failed).
    pub fn total(&self) -> usize {
        self.passed.len() + self.failed.len()
    }
}

/// Run a set of conformance cases and collect results.
pub fn run_suite(cases: &[&dyn Case]) -> ConformanceReport {
    let mut report = ConformanceReport::default();
    for case in cases {
        match case.run() {
            Ok(()) => report.passed.push(case.name().to_string()),
            Err(e) => report.failed.push((case.name().to_string(), e)),
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Pass;
    impl Case for Pass {
        fn name(&self) -> &str {
            "pass"
        }
        fn run(&self) -> Result<(), String> {
            Ok(())
        }
    }

    struct Fail;
    impl Case for Fail {
        fn name(&self) -> &str {
            "fail"
        }
        fn run(&self) -> Result<(), String> {
            Err("boom".to_string())
        }
    }

    #[test]
    fn all_passing_reports_success() {
        let report = run_suite(&[&Pass, &Pass]);
        assert!(report.is_success());
        assert_eq!(report.total(), 2);
    }

    #[test]
    fn any_failure_reports_not_success() {
        let report = run_suite(&[&Pass, &Fail]);
        assert!(!report.is_success());
        assert_eq!(report.total(), 2);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].0, "fail");
    }
}
