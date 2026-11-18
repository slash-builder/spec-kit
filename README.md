# Spec Kit

> Methodology and conformance-harness primitives for writing protocol
> specifications you can verify.

[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Status: pre-release](https://img.shields.io/badge/status-pre--release-orange.svg)

Spec Kit is a small Rust library and CLI for authoring and verifying
protocol specifications. It enforces a consistent shape across spec
documents (frontmatter + required sections), gives you a semver-style
version-classification model for spec changes, and ships the
conformance-test primitives that every implementation of your protocol
imports to prove it conforms.

If you've ever ended up with three slightly-different versions of "the
spec" living in a wiki, a code comment, and a Google Doc — and no
mechanical way to tell whether a given implementation actually matches
any of them — this kit is the bones of a way out.

## Status

Pre-1.0. `template::{skeleton, verify}`, `diff::compare`,
`version::{required_bump, apply_bump, next_version}`, and
`conformance::{Case, run_suite, ConformanceReport}` are implemented and
tested. `conformance::{Case, run_suite, ConformanceReport}` is the frozen
studio-wide conformance-test ABI — every Kit's `tests/conformance.rs`
imports it verbatim; its shape does not change even as the rest of the
crate evolves.

Treat current versions as a design preview. Pin to an exact version if
you choose to depend on it.

## Quick start

Add it to a Rust project:

```toml
[dependencies]
slashbuilder-spec-kit = "0.1"
```

The package is `slashbuilder-spec-kit`; the library is imported as `spec_kit` and the command-line tool is `spec-kit`.

Generate a starter spec from the CLI:

```bash
cargo install slashbuilder-spec-kit
spec-kit new my-protocol > my-protocol.md
```

Write a conformance suite for an implementation:

```rust
use spec_kit::conformance::{Case, run_suite};

struct EncodeRoundtrip;

impl Case for EncodeRoundtrip {
    fn name(&self) -> &str { "encode/decode roundtrip" }
    fn run(&self) -> Result<(), String> {
        let v = my_codec::encode(&42)?;
        let back: i32 = my_codec::decode(&v)?;
        if back == 42 { Ok(()) } else { Err(format!("got {back}")) }
    }
}

let report = run_suite(&[&EncodeRoundtrip]);
assert!(report.failed.is_empty());
```

Classify the change between two spec versions:

```bash
spec-kit diff my-protocol-v1.md my-protocol-v2.md
# → Breaking | Additive | Editorial
```

## Scope

**In scope.** Spec markdown templates with required sections. Spec
versioning rules (what counts as breaking vs additive vs editorial).
Conformance-test traits and helpers a kit's `tests/` imports. A small
`spec-kit` CLI wrapping all of the above.

**Out of scope.** Spec *content* — that lives in the project that owns
the protocol. Code generation from specs (a candidate for a separate
kit later, not bundled here). Runtime protocol implementations — those
live in the kit that owns the protocol.

## Why does this exist?

Most protocol specs end up as documents that *describe* a protocol but
can't *enforce* anything about its implementations. Spec Kit's bet is
that a small, opinionated convention (spec template + change taxonomy
+ test harness) goes a long way toward keeping a spec and its
implementations honest with each other — without dragging in
heavyweight tooling like protobuf or OpenAPI when those aren't a fit.

## Building from source

```bash
git clone https://github.com/slash-builder/spec-kit.git
cd spec-kit
make all          # build + lint + test
make run -- new my-protocol   # try the CLI
```

Requires a recent stable Rust toolchain (1.74+). See the `Makefile` for
the full target list.

## Contributing

Contributions are welcome under Apache-2.0. By submitting a pull
request you agree to the Developer Certificate of Origin (DCO) — sign
your commits with `git commit -s`.

Substantial design changes should start as a brief issue describing the
intent before code lands; the project is small and the API surface is
deliberately conservative.

## License

Source code under [Apache-2.0](LICENSE). Spec documents you author with
Spec Kit are yours to license however you choose; the kit itself
suggests CC-BY-4.0 or Apache-2.0 for the spec text.

## About

Spec Kit is part of the **SlashBuilder Kit ecosystem** — a set of
reusable Rust libraries built around the BeOS-style "Kit"
pattern. Each Kit owns a self-contained domain (storage, messaging,
identity, fabric, devices, app store, design); products are composed
from Kits. Spec Kit sits at the governance root: it's a build/test
dependency of every other Kit's conformance test suite.

Maintained by [SlashBuilder](https://slashbuilder.com) / Douglas Lockamy.
