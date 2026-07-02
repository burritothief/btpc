# BTPC Contract Specifications

This directory is the normative contract and project reference for BTPC. If source
code, tests, README guidance, todos, and specifications disagree, the specifications
win unless a newer approved change updates them in the same patch.

## Status Model

- **Draft** — unresolved design. Production code must not depend on it.
- **Accepted** — approved target contract that may not be implemented yet.
- **Implemented** — source and automated verification satisfy the requirement.
- **Deprecated** — compatibility contract retained until its named replacement.

An entire spec has an overall status in YAML front matter. Every enforceable
requirement also has its own status; the requirement status controls traceability.

## Requirement Format

Requirements use stable IDs and RFC-style language:

```markdown
### BENC-PARSE-001 — Parse exactly one value

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests/bencode_parser.rs`
- **Depends on:** None

The parser **MUST** consume exactly one value and reject trailing bytes.
```

IDs are permanent. If semantics are replaced, deprecate the old requirement and
introduce a new ID. Minor clarifications that do not alter observable behavior may
retain the ID. Tests cite requirements with `Spec: REQUIREMENT-ID` comments.

## Change Process

1. Identify affected requirement IDs before editing source.
2. Change or add the contract first and set new behavior to `Accepted`.
3. Add a failing test carrying a `Spec:` annotation.
4. Implement the behavior and make the test pass.
5. Set the requirement to `Implemented` only after verification passes.
6. Update README guidance and todos to link to the requirement IDs.

Every contract-bearing source path is listed in `ownership.toml`. Pull requests
that modify those paths update an owning spec or provide an explicit
`Spec-Sync-Waiver:` explanation in the pull-request body. Waivers explain why an
implementation-only edit leaves observable contracts unchanged; they do not waive
tests or specification validation.

## Validation

Run:

```bash
uv run python scripts/check_specs.py
```

The checker validates front matter, requirement IDs and dependencies, referenced
paths, relative links, source ownership, and implemented-requirement test
annotations. CI additionally compares changed contract-bearing source paths with
their owning specs.

## Specification Index

- [Product Direction](product.md)
- [Architecture](architecture.md)
- [Bencode](bencode.md)
- [Metainfo](metainfo.md)
- [Creation](creation.md)
- [Verification](verification.md)
- [Rust API](rust-api.md)
- [Python API](python-api.md)
- [CLI](cli.md)
- [Errors](errors.md)
- [Performance](performance.md)
- [Torrent Creation Benchmarking](benchmarking.md)
- [Testing](testing.md)
- [Security](security.md)
- [Release](release.md)
- [Production Documentation Site](documentation-site.md)

## Project Map

Start with [Product Direction](product.md) for goals, milestones, and non-goals,
then [Architecture](architecture.md) for system boundaries. Protocol contracts are
[Bencode](bencode.md), [Metainfo](metainfo.md), [Creation](creation.md), and
[Verification](verification.md). User-facing contracts are [Rust API](rust-api.md),
[Python API](python-api.md), and [CLI](cli.md). Quality and operations are covered by
[Errors](errors.md), [Security](security.md), [Testing](testing.md),
[Performance](performance.md), [Benchmarking](benchmarking.md), and
[Release](release.md). The public documentation build and deployment contract is
[Production Documentation Site](documentation-site.md).
