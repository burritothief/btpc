---
spec_id: CLI
title: "Command-Line Interface"
status: Accepted
owners:
  - "CLI maintainers"
source_paths:
  - "crates/btpc-cli/src"
  - "docs/cli/index.md"
test_paths:
  - "crates/btpc-cli/tests"
last_reviewed: "2026-07-01"
---

# Command-Line Interface

## Requirements

### CLI-CMD-001 — Provide core torrent workflows

- **Status:** Accepted
- **Sources:** `crates/btpc-cli/src/main.rs`, `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `ARCH-BOUND-001`

The `btpc` binary **MUST** provide `create`, `inspect`, `validate`, `verify`,
`magnet`, shell-completion, and manual-page workflows backed by `btpc-core`.

### CLI-DOC-001 — Generate reference artifacts from the command model

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/reference.rs`, `docs/cli/index.md`, `docs/reference`, `docs/completions`
- **Verification:** `crates/btpc-cli/tests/reference.rs`, `crates/btpc-cli/tests/documentation.rs`
- **Depends on:** `CLI-CMD-001`

`btpc completion generate` **MUST** generate Bash, Zsh, Fish, PowerShell, and Elvish
completion source, and `btpc manpage` **MUST** generate `btpc(1)` from the same
Clap command definition. Checked-in help and manual references **MUST** be tested
for staleness, and the documented create/inspect/validate/magnet/verify tour
**MUST** execute for v1, v2, and hybrid modes.

### CLI-IO-001 — Keep stdout machine-safe

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/context.rs`, `crates/btpc-cli/src/output.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `CLI-CMD-001`

Machine output **MUST** use stdout; progress and human diagnostics **MUST** use
stderr. JSON schemas **MUST** be versioned, and styling **MUST** be disabled for
non-terminals and `NO_COLOR`.

### CLI-EXIT-001 — Use stable documented exit categories

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/diagnostics.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `ERR-MAP-001`

The CLI **MUST** freeze and test unambiguous exit codes for usage, invalid
metainfo, I/O/path failure, payload mismatch, unsupported behavior, and interrupt.

The frozen mapping is: internal `1`, usage/Clap parsing `2`, I/O/path `3`,
invalid bencode/metainfo/resource limits `4`, unsupported `5`, verification
mismatch `6`, and cancellation/interrupt `130`.

### CLI-JSON-001 — Version create result JSON

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/output.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-cli/tests/create.rs`
- **Depends on:** `CLI-IO-001`

`btpc create --json` **MUST** emit one JSON object with schema identifier
`btpc.create.v2` on stdout and no progress output on stdout. The object contains
an exact versioned output-path object, mode, optional v1 and v2 info hashes as applicable,
file/payload/piece counts, selected piece length, optional policy identifier, and
per-phase millisecond metrics.

Filesystem path objects use schema `btpc.filesystem-path.v2`, a control-escaped
`display` string, and either `unix-bytes-hex` with a hexadecimal string or
`windows-utf16` with an array of UTF-16 code units. Exact objects are canonical in
`btpc.create.v2`, `btpc.edit.v2`, and `btpc.verify.v2`. The sibling
`output_display` and `path_display` strings are deprecated presentation aliases
retained for one schema generation and scheduled for removal in v3.

`btpc inspect --json` uses schema `btpc.inspect.v1`; raw byte strings are objects
with `encoding` equal to `utf-8` or `hex` and a corresponding `value`. `btpc
validate --json` uses schema `btpc.validate.v1`. Both commands read only the
metainfo file and never infer or access a payload path. Their additive `canonical`
boolean distinguishes valid non-canonical source bytes from canonical input;
protocol-invalid input still exits with invalid-metainfo code `4`.

`btpc verify --json` uses schema `btpc.verify.v2`, emits a boolean `valid` field
and deterministic mismatch objects, and exits with the frozen verification code
`6` when any mismatch is reported.

Plain path-only output writes native Unix path bytes. On Windows it uses the
self-describing `windows-utf16:XXXX,...` form so unpaired path code units remain
recoverable. Human prose uses only the escaped display representation.

### CLI-WRITE-001 — Protect output files

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `CREATE-OUTPUT-001`

Creation **MUST** refuse overwrite unless explicitly forced, use atomic core output,
and leave no partial destination after failure or cancellation.

### CLI-GLOBAL-001 — Apply consistent global controls

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/context.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-IO-001`, `CLI-EXIT-001`

Every command **MUST** share global `--config`, `--no-config`, `--color
auto|always|never`, repeatable `--verbose`, and `--quiet` behavior through one
execution context. `NO_COLOR`, non-terminal streams, and machine formats **MUST**
disable styling unless `--color always` is explicitly selected. Explicitly
conflicting global and command options **MUST** fail during argument resolution.

### CLI-CONFIG-001 — Load versioned TOML configuration safely

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/config/mod.rs`, `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/context.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-GLOBAL-001`, `SEC-CONFIG-001`

BTPC **MUST** support one versioned TOML configuration in the platform user config
directory, an explicit `BTPC_CONFIG` or `--config` path, and `--no-config`.
Configuration **MUST NOT** load from a project directory implicitly. Unknown keys,
unsupported schema versions, invalid values, missing references, and incompatible
options **MUST** be errors rather than ignored defaults.

### CLI-PRESET-001 — Resolve named presets deterministically

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/config/mod.rs`, `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-CONFIG-001`

Create configuration **MUST** support named presets, ordered `extends`, tracker
aliases, and tracker groups. Resolution precedence **MUST** be hardcoded defaults,
global config, selected presets in argument order, then explicit CLI values.
Repeatable list options **MUST** append in stable order and remove exact duplicates;
explicit `--clear-*` operations **MUST** reset inherited lists before later CLI
additions. Missing parents and inheritance cycles **MUST** fail with the complete
resolution chain.

### CLI-CONFIG-CMD-001 — Manage configuration through the CLI

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/config/mod.rs`, `crates/btpc-cli/src/output.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-CONFIG-001`, `CLI-PRESET-001`, `SEC-CONFIG-001`

The CLI **MUST** provide `config path`, `init`, `show`, `check`, and `explain
create`, plus tracker `list|add|remove` and preset `list|show|save|remove` commands.
Writes **MUST** be atomic, `init` **MUST** refuse overwrite without `--force`, and
normal output **MUST** redact tracker credentials. `config explain create` **MUST**
show effective values and their default/config/preset/CLI provenance without
executing creation.

### CLI-OUTPUT-001 — Keep default output minimal and offer explicit formats

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/context.rs`, `crates/btpc-cli/src/output.rs`, `crates/btpc-cli/src/render.rs`, `crates/btpc-cli/src/diagnostics.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-IO-001`, `CLI-GLOBAL-001`

Default human output **MUST** remain concise: create reports one completion line on
stderr, inspect reports a short key/value summary, validate reports `valid` or a
concise failure, verify reports a concise result, and magnet writes only the URI to
stdout. `--pretty` **MAY** enable aligned tables, trees, symbols, and expanded
summaries. Commands that expose structured data **MUST** support the applicable
subset of `human`, `plain`, `json`, `json-pretty`, and `tsv`; existing `--json`
**MUST** remain a compatibility alias until a documented deprecation completes.

### CLI-PROGRESS-001 — Render progress only when appropriate

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/context.rs`, `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-cli/src/progress.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-OUTPUT-001`

Long-running create and verify commands **SHOULD** render one restrained progress
display only when stderr is a terminal. Quiet mode, machine formats, non-terminal
stderr, and `NO_COLOR` **MUST** suppress it. Progress **MUST NOT** contaminate
stdout, leak credentials, or remain on screen after an error or interrupt.

### CLI-DIAG-001 — Produce contextual actionable diagnostics

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/diagnostics.rs`, `crates/btpc-cli/src/output.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-EXIT-001`, `ERR-MAP-001`

Errors **MUST** retain available path, field, byte offset, and category context and
**SHOULD** include a concise remediation hint. Unknown commands, presets, tracker
aliases, fields, formats, shells, and enum values **SHOULD** offer close-match
suggestions. Diagnostics **MUST** redact configured secrets and remain stable
enough for exit-code assertions without making complete prose a compatibility API.

### CLI-CREATE-002 — Provide ergonomic advanced creation options

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/config/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-cli/tests/create.rs`
- **Depends on:** `CLI-PRESET-001`, `CLI-OUTPUT-001`, `CREATE-OUTPUT-001`

Create **MUST** accept byte counts, binary units such as `4MiB`, and exponents such
as `2^22` for piece length. It **SHOULD** support target piece count, a maximum
automatic piece length, symmetric `--private`/`--public`, explicit metadata removal,
tracker aliases/groups, `creation-date` values `now|none|UNIX|RFC3339`, explicit
cross-seeding entropy, dry-run planning, and repeatable script outputs for path,
hashes, and magnet URI. Random entropy **MUST NOT** be implicit. Default mode
**MUST** remain v1 until a separate contract changes it.

### CLI-BATCH-001 — Create multiple torrents deterministically

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-cli/tests/create.rs`
- **Depends on:** `CLI-CREATE-002`, `CLI-CONFIG-001`, `PERF-POOL-001`

Create **MUST** support multiple input paths and a versioned TOML batch manifest
whose option names and semantics match CLI/config creation options. `--output`
**MUST** be single-input-only; multi-input runs use `--output-dir` or per-job batch
outputs. Bounded job concurrency **MUST** coordinate with hashing concurrency to
avoid accidental CPU oversubscription. Results **MUST** be reported in manifest
order regardless of completion order, and output collisions **MUST** be detected
before hashing begins.

### CLI-INSPECT-002 — Support field-oriented metainfo queries

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-cli/src/output.rs`
- **Verification:** `crates/btpc-cli/tests/inspect.rs`
- **Depends on:** `CLI-OUTPUT-001`, `META-FIELD-001`

Inspect **MUST** support repeatable field selectors for mode, name, size, piece
details, hashes, private state, trackers, web seeds, nodes, metadata, files,
warnings, canonicality, and unknown fields. It **MUST** offer flat file and opt-in
tree views, raw path encoding as UTF-8/escaped/hex, and offset/limit pagination.
One selected field in plain format **MUST** emit only its value. Validate **SHOULD**
support canonical-only validation and treating warnings as errors without reading
payload data. Existing JSON schemas **MUST** remain compatible or receive a new
versioned schema.

### CLI-INSPECT-DISPLAY-001 — Present a concise mkbrr-inspired torrent summary

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-cli/src/render.rs`
- **Verification:** `crates/btpc-cli/tests/inspect.rs`
- **Depends on:** `CLI-OUTPUT-001`, `CLI-INSPECT-002`, `CLI-PROGRESS-001`

Default human inspect output **MUST** use a titled, indented, aligned summary rather
than raw lowercase `key: value` lines. The stable field order is: name, mode,
applicable info hashes, total size, piece length, piece count, magnet, tracker
tiers, web seeds, DHT nodes, private state when explicitly present,
source/comment/creator/creation date when present, and file count for multi-file torrents. Sizes **MUST**
use IEC units with one decimal place while verbose or pretty output **MUST** also
show exact byte counts. Creation dates **MUST** use a readable local timestamp with
an unambiguous zone; machine output retains numeric values.

Tracker tiers, web seeds, and DHT nodes **MUST** render as grouped indented lists;
URLs are redacted and node hosts remain byte-lossless in machine output. V1 and v2 hashes **MUST** have distinct labels for hybrid
torrents. The generated magnet **MUST** use the same redaction policy as standalone
magnet output and **MUST NOT** expose configured credentials that normal CLI output
would redact. Single-file torrents **SHOULD NOT** print a redundant `Files: 1`
line; multi-file torrents **MUST** print the count.

The default summary **MUST** remain readable without color and **MUST NOT** depend
on Unicode symbols. When color is enabled, only the heading, labels, hashes/URLs,
and status values **MAY** be styled; spacing and line breaks **MUST** be identical
after ANSI removal. `--pretty` or verbose mode **MAY** add sections for source-file
path, canonicality, exact sizes, optional metadata, validation warnings, unknown
fields, and a deterministic file tree. JSON schemas and plain/TSV field-query
formats **MUST NOT** change as a side effect of human renderer improvements.

### CLI-EDIT-001 — Edit metainfo without rehashing payloads

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/handlers/mod.rs`, `crates/btpc-core/src/edit.rs`, `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-WRITE-001`, `ERR-MAP-001`

`btpc edit INPUT` **MUST** use the typed core editor and default to a distinct
`<stem>.edited.torrent` output. Atomic in-place replacement **MUST** require
`--in-place`; it conflicts with `--output`. The command **MUST** support dry-run
and a before/after summary, report whether info hashes changed, and edit supported
trackers, web seeds, nodes, comment, creator, creation date, private state, source,
and file attributes without reading or rehashing payload files. Reserved fields
**MUST NOT** be changed through an untyped generic setter.

### CLI-COMPLETE-001 — Generate and install shell integration safely

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/reference.rs`, `docs/cli/index.md`
- **Verification:** `crates/btpc-cli/tests/completion.rs`, `crates/btpc-cli/tests/reference.rs`
- **Depends on:** `CLI-DOC-001`, `RELEASE-CLI-DOC-001`

The CLI **MUST** provide `completion generate`, `completion install`, and
`completion uninstall` for Bash, Zsh, Fish, PowerShell, and Elvish. Installation
**MUST** write only to documented user completion directories and **MUST NOT** edit
shell startup files. Dry-run **MUST** show the target and generated content.
`completions SHELL` **MUST** remain a hidden compatibility alias for at least one
minor release before removal.

### CLI-COMPAT-001 — Preserve existing scripts through explicit deprecation

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/context.rs`, `crates/btpc-cli/src/output.rs`, `crates/btpc-cli/src/diagnostics.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `CLI-EXIT-001`, `CLI-JSON-001`, `CLI-COMPLETE-001`

Existing command names, flags, exit codes, stdout/stderr behavior, and versioned
JSON fields **MUST** remain compatible unless a replacement is documented and the
old interface remains as an alias for at least one minor release. Deprecation
warnings **MUST** use stderr and **MUST NOT** appear in quiet or machine output
unless they affect correctness.

## Design Rationale

The CLI is both a user surface and benchmark target. Strict stream separation and
stable JSON/exit behavior keep it usable in scripts without duplicating core logic.
Minimal default output avoids torf-style presentation surprises in automation,
while opt-in pretty rendering provides interactive ergonomics. TOML is shared by
configuration and batch jobs to keep one typed vocabulary. Unlike project-local
configuration conventions, user-scoped explicit config avoids untrusted repository
files changing torrent output.
