# Configuration and presets

BTPC resolves creation settings in a deterministic order: built-in defaults, global
configuration, selected presets, then explicit command arguments. Use
`--no-config` for a config-free run.

```console
btpc config path
btpc config init
btpc config preset list
btpc config explain create --preset private ./payload
btpc create ./payload --preset private -o payload.torrent
```

Configuration supports tracker aliases, tracker groups, and inherited ordered
presets. Inspect the resolved plan with `config explain` before a large creation.
