# Editing metainfo

Create a validated copy by default:

```console
btpc edit payload.torrent --output edited.torrent --comment "reviewed"
```

Use `--in-place` only when atomic replacement is intended. Top-level fields such as
comment and creator preserve info hashes. Changes inside the `info` dictionary,
including private state, source, and file attributes, change applicable hashes.
Top-level-only edits preserve the exact original `info` byte slice, even when its
source encoding is noncanonical. Canonical serialization remains an explicit
output choice. Hybrid real-file attribute edits update both v1 and v2 file
representations atomically; padding attributes remain v1-only.

Python uses one three-state parameter per optional field: `UNCHANGED` preserves,
`None` removes, and a typed value replaces.
