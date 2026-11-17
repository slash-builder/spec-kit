# Changelog

All notable changes to **spec-kit** will be documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Frozen — interface lock, 2026-08-26

The **`conformance` module is locked** by data-architect ruling, effective
with the commit that lands this entry (baseline `9b215b9`; the module's
shape is unchanged — only the gate below is new), per the studio kit-lock
bar (frozen public surface +
this entry + at least one CI-enforced conformance test). It is the third
item in the studio's kit lock order and is locked jointly with
`slash-builder/storage-kit`, its first real external consumer.

**Frozen surface** — names, signatures, and field shapes of:

- `conformance::Case` — the trait, exactly `fn name(&self) -> &str` and
  `fn run(&self) -> Result<(), String>`. It must stay object-safe:
  every Kit uses it as `Box<dyn Case>` / `&dyn Case`.
- `conformance::run_suite(cases: &[&dyn Case]) -> ConformanceReport` —
  the argument type and return type both.
- `conformance::ConformanceReport` — the struct with public
  `passed: Vec<String>` and `failed: Vec<(String, String)>`, its
  `Default` impl, and the `is_success()` / `total()` accessors.

**Behavior frozen alongside the shape:** `run_suite` runs every case
instead of short-circuiting on the first failure, preserves submission
order within each bucket, carries each failure message verbatim, and
treats the empty suite as successful.

Breaking any of the above is a major-version change requiring a
data-architect ruling. Adding ergonomics *around* the shape (as
`is_success`/`total` were added) remains fine.

**Explicitly not frozen:** `diff`, `version`, `template`, `error`, and the
`spec-kit` CLI. These are real and tested but are governance tooling with
no cross-repo ABI role; they evolve under ordinary semver.

**Lock gate:** `tests/conformance_abi.rs` — an integration test, so it
links `spec_kit` as an external crate and can reach nothing but the public
surface. It exercises the ABI in the exact call shape every Kit uses.
Previously the only proof the ABI was externally usable lived in another
repo (`storage-kit`), where a signature change here would have left this
repo's CI green and broken the consumer on its next build; the gate now
fails at the source. Runs in both CI surfaces via `cargo test`.

### Added

- `tests/conformance_abi.rs` — external-consumer lock gate for the frozen
  `conformance` ABI (see the freeze entry above).
- `diff::compare(old_spec, new_spec) -> ChangeKind` — real spec-diff
  classification grounded in `##` section structure and RFC 2119
  normative keywords (MUST / MUST NOT / SHALL / SHALL NOT / REQUIRED).
- `version::{Bump, required_bump, apply_bump, next_version}` — maps a
  `ChangeKind` to the required semver bump per the studio's
  spec-versioning rules.
- `template::{REQUIRED_SECTIONS, verify}` — structural verification of a
  spec document against the required section set.
- `conformance::ConformanceReport::{is_success, total}` — ergonomics
  added around the frozen `Case`/`run_suite`/`ConformanceReport` ABI;
  the trait, function signature, and report fields are unchanged.
- `spec-kit verify`/`spec-kit diff` CLI subcommands now call the real
  library functions instead of exiting with `not yet implemented`.

### Fixed

- Documentation described an aspirational `Spec::parse`/`Spec::version` API
  that was never implemented; corrected to document the actual module
  surface (spec documents are plain `&str` markdown throughout).

- Initial scaffold: `spec-kit` Rust crate (library + `spec-kit` CLI bin).
- Public module surface: `template`, `version`, `conformance`, `diff`,
  `error`.
- `Case` trait + `run_suite()` for kit conformance harnesses.
- `template::skeleton()` returning a starter spec markdown.
- `ChangeKind` enum (Breaking / Additive / Editorial) for spec-diff
  classification.
- `SpecKitError` with `Parse`, `Version`, `Io` variants.
- `spec-kit` CLI with `new`, `verify`, `diff` subcommands (stubs).
- Project documentation: `README.md`, `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `.github/CODEOWNERS`, this `CHANGELOG.md`.

### Notes

- `diff`, `version`, and `template::verify` bodies started as
  `unimplemented!()`-adjacent stubs and are now real, tested logic (see
  Added above). `conformance::{Case, run_suite, ConformanceReport}`
  remains the frozen studio-wide conformance-test ABI.

[Unreleased]: https://github.com/slash-builder/spec-kit/commits/main
