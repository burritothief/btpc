# Editing metainfo

Create a validated copy by default:

```console
btpc edit payload.torrent --output edited.torrent --comment "reviewed"
```

Use `--in-place` only when atomic replacement is intended. Top-level fields such as
comment and creator preserve info hashes. Changes inside the `info` dictionary,
including private state, source, and file attributes, change applicable hashes.

Python uses one three-state parameter per optional field: `UNCHANGED` preserves,
`None` removes, and a typed value replaces.
