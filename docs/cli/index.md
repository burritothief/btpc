---
title: CLI guide
---

# CLI Guide

The [generated command reference](reference/index.md) documents every visible
command and option directly from the Clap command model.

`btpc` is a thin adapter over `btpc-core`. Run `btpc --help` or use the checked-in
[reference](../reference/btpc.txt). Generated completion files and the manual page
are outputs of the same Clap command definition used by the executable.

## Commands

| Command | Purpose | Machine output |
| --- | --- | --- |
| `create INPUT` | Scan and hash a file/directory, then atomically write canonical metainfo | `--json`: `btpc.create.v2` |
| `inspect TORRENT` | Display validated metadata without reading payload files | `--json`: `btpc.inspect.v1` |
| `validate TORRENT` | Validate bencode and protocol fields only | `--json`: `btpc.validate.v1` |
| `verify TORRENT PAYLOAD` | Check structure and every applicable hash domain | `--json`: `btpc.verify.v2` |
| `edit TORRENT` | Safely edit typed metainfo fields without reading payloads | `--json`: `btpc.edit.v2` |
| `magnet TORRENT` | Print a deterministic magnet URI | URI only |
| `config ...` | Locate, validate, explain, and safely update TOML configuration | selected subcommands support JSON |
| `completion ...` | Generate, install, or uninstall shell completion source | completion source |
| `manpage` | Generate `btpc(1)` roff | roff source |

Every metainfo-reading command accepts `--max-input-bytes` and
`--max-owned-bytes`; `--max-integer-digits` independently bounds decimal digit
runs. Defaults are the same conservative limits as `btpc-core`; violations use
invalid-data exit code `4`.

Path-bearing v2 JSON uses `btpc.filesystem-path.v2`: Unix paths carry exact bytes
as hexadecimal and Windows paths carry exact UTF-16 code units. `display` is safe
for presentation and escapes control characters. Deprecated `output_display` and
`path_display` aliases remain through v2 and are removed in v3; consumers should
read the exact object now. Batch creation emits one v2 create object per input in
input order.

## Configuration

BTPC loads the explicit global `--config PATH`, then `BTPC_CONFIG`, then the
platform user configuration path (`$XDG_CONFIG_HOME/btpc/config.toml` or the
equivalent user config directory). It never searches the current directory.
`--no-config` disables every implicit and environment-selected file. Values merge
in this order: built-in defaults, global `[create]`, each selected preset in CLI
order (including `extends` parents), then explicit CLI flags.

```toml
version = 1

[trackers.primary]
url = "https://tracker.example/announce"

[tracker_groups.release]
trackers = ["primary"]

[presets.base]
mode = "hybrid"
piece_length = 1048576
creation_date = 0

[presets.private]
extends = ["base"]
tracker_groups = ["release"]
private = true
```

Use `config path|init|show|check`, `config tracker ...`, `config preset ...`, and
`config explain create ...` to manage and inspect configuration. Secret-bearing
URLs are redacted by default. Configuration writes are atomic and user-private.

## Creation

```console
btpc create payload --mode hybrid --piece-length 16384 \
  --tracker https://tracker.example/announce \
  --web-seed https://seed.example/payload/ \
  --creation-date 0 --threads 1 --json \
  -o payload.torrent
```

Important controls:

- `--mode v1|v2|hybrid` selects the representation; v1 is the default.
- `--piece-length` fixes the piece size. Omit it for the deterministic automatic
  policy. It accepts bytes, `KiB`/`MiB`, or `2^N`; v2/hybrid require BEP
  52-compatible lengths of at least 16 KiB. `--target-pieces` with
  `--max-piece-length` selects a bounded core policy.
- `--threads 0` selects conservative automatic concurrency, `1` selects the
  sequential oracle, and `N` selects bounded per-operation workers.
- `--force` is required to replace an existing destination. Writes are atomic.
- `--durable` also syncs the destination directory after publication where the
  platform supports directory syncing.
- Repeated `--tracker` values create separate tiers; repeated comma-delimited
  `--tracker-tier` values preserve a tier.
- Config tracker aliases/groups are available through `--tracker-alias` and
  `--tracker-group`; `--public` explicitly writes private=false.
- Metadata has symmetric clear flags, `--creation-date` accepts
  `now|none|UNIX|RFC3339`, and `--entropy` is explicit (`none`, reproducible hex,
  or opt-in OS `random`).
- New torrents include `created by = btpc/<version>` by default. Use
  `--created-by TEXT` to override it or `--no-created-by` to omit it; the legacy
  `--clear-created-by` spelling remains an alias.
- `--dry-run` scans and validates the plan without hashing or writing. Repeat
  `--print path|info-hash-v1|info-hash-v2|magnet` for stable script lines in the
  requested order.
- Multiple inputs are accepted directly; use `--output-dir` for inferred outputs.
  `--batch jobs.toml` accepts schema version 1 with `[[jobs]]`, `input`, optional
  `output`, presets, mode, piece length, and threads. All destinations are
  preflighted before hashing and results remain in input/manifest order. The
  current conservative scheduler executes one job at a time, so the total hashing
  worker budget never multiplies across jobs.
- `--include`, `--exclude`, hidden/empty policies, root naming, symlink policy,
  and special-file policy are deterministic on supported operating systems.
  Include/exclude globs are UTF-8 text filters and fail if they encounter a
  non-UTF-8 payload path rather than matching a lossy replacement string.

Equivalent batch input uses a versioned manifest:

```toml
version = 1

[[jobs]]
input = "payload-a"
output = "torrents/payload-a.torrent"
presets = ["private"]

[[jobs]]
input = "payload-b"
mode = "v1"
threads = 1
```

## Inspection and Bytes

Human output decodes UTF-8 where safe. JSON never guesses: byte strings are
objects with `encoding` set to `utf-8` or `hex` and a `value`. `inspect` and
`validate` never infer or read a payload path.

Default summaries remain compact. `--pretty` enables aligned, symbol-prefixed
human output, while `--quiet` suppresses human summaries. Human inspect uses a
stable `Torrent info:` summary with IEC sizes, applicable hashes, a safe magnet,
and grouped tracker/web-seed URLs. `--pretty` adds exact-byte and validation
details; `-v` adds optional metadata, warnings, bounded unknown fields, and a real
nested file trie for multi-file torrents. `--tree` requests the trie directly and
honors `--offset`, `--limit`, and path encoding. Machine formats remain unchanged,
unstyled, and do not receive progress output.

```console
btpc inspect payload.torrent --json
btpc inspect payload.torrent --field hash-v1 --format plain
btpc inspect payload.torrent --files --offset 0 --limit 100 --format tsv
btpc inspect payload.torrent --tree --path-encoding escaped --format json-pretty
btpc validate payload.torrent --json
btpc validate payload.torrent --canonical --warnings-as-errors
```

`inspect --field` is repeatable and preserves selector order. Available fields
cover mode/name/size/pieces/files, both info hashes, private state, trackers,
web seeds, nodes, comment/creator/date/source, canonicality, warnings, and unknown
top-level fields. One selected scalar with `--format plain` writes only its value.
Selected or paginated JSON uses schema `btpc.inspect.selection.v1`; unselected
`--json` remains the compatible `btpc.inspect.v1` schema. File paths support
lossless `escaped` and `hex` encodings when UTF-8 is unavailable.

## Verification

```console
btpc verify payload.torrent payload
btpc verify payload.torrent payload --fail-fast --extra-files --json
```

A completed report with mismatches exits `6`; operational errors use their normal
category. `--fail-fast` stops at the first deterministic mismatch. `--extra-files`
reports regular files not represented in metainfo. Unsafe symlink paths are
reported and never followed.

## Magnets

```console
btpc magnet payload.torrent
btpc magnet payload.torrent --no-display-name --no-trackers --no-web-seeds
```

v1 magnets use `btih`, v2 magnets use multihash `btmh`, and hybrid magnets include
both exact-topic parameters in deterministic order.

## Editing

`btpc edit INPUT` writes `<stem>.edited.torrent` by default. Use `--output` for a
different copy or explicit `--in-place` for atomic replacement. `--dry-run`
validates without writing; `--diff` or `-v` includes old/new hashes. Typed set and
clear flags cover trackers (including config aliases/groups), web seeds, nodes,
comment, creator, creation date, private state, source, and file attributes.
Top-level-only edits retain info hashes, while info-dictionary edits report the
applicable changed hash domains.

## Exit Codes and Streams

| Code | Meaning |
| --- | --- |
| `0` | Success / valid payload |
| `1` | Internal error |
| `2` | Usage or argument parsing error |
| `3` | I/O or path error |
| `4` | Invalid bencode, metainfo, or resource limit |
| `5` | Unsupported policy or filesystem object |
| `6` | Payload verification mismatch |
| `130` | Cancellation or interrupt |

Machine output is written to stdout. Human diagnostics and progress are written
to stderr. Styling and the single create/verify progress display are disabled for
non-terminals, quiet mode, machine output, and when `NO_COLOR` is set. Explicit
`--color always` forces diagnostic styling. Errors retain stable categories and
exit codes while adding available path, field, byte-offset, and remediation
context; URLs are redacted before diagnostic rendering.

Versioned JSON schemas retain their existing fields. New incompatible schema
shapes require a new schema identifier rather than silently changing an existing
one. The legacy hidden `completions SHELL` command writes the same completion
bytes as `completion generate SHELL`; its warning is stderr-only and suppressed by
`--quiet`.

## Completion and Manual Installation

```console
btpc completion generate bash
btpc completion install bash
btpc completion install --dry-run fish
btpc completion uninstall zsh
btpc manpage > ~/.local/share/man/man1/btpc.1
```

When the shell argument is omitted, BTPC uses an unambiguous `SHELL` or
PowerShell environment hint; otherwise it asks for an explicit shell. Installation
uses standard per-user Bash, Zsh, Fish, PowerShell, and Elvish completion
directories, creates only the required parent directories, writes atomically, and
never edits shell startup files. Existing unrelated files require `--force`, and
uninstall removes only BTPC-marked content. The hidden `completions SHELL` alias is
retained for compatibility for one minor release. Pre-generated artifacts live in
the [shell completion guide](completion.md) and
[`reference/btpc.1`](../reference/btpc.1).
