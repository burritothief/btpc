# CLI configuration

The user-scoped TOML configuration file is versioned and can define global output
settings, tracker aliases, tracker groups, and creation presets.

```console
btpc config path
btpc config show
btpc config check
btpc config explain create --preset private ./payload
```

Use `btpc config init` to create a starter file and `--no-config` to ignore implicit
and environment-selected configuration.
