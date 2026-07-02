# Inspecting and validating metainfo

Inspection reports validated metadata without reading payload files:

```console
btpc inspect payload.torrent
btpc inspect payload.torrent --field hash-v1 --format plain
btpc inspect payload.torrent --tree --pretty
```

Validation checks protocol structure and can require canonical source encoding:

```console
btpc validate payload.torrent --canonical --warnings-as-errors
```

Human output decodes UTF-8 only when safe. JSON represents uncertain byte strings
with an explicit UTF-8 or hexadecimal encoding marker.
