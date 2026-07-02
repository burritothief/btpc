# CLI quick start

Create a tiny payload and build all three torrent modes:

```console
mkdir payload
printf 'hello torrent\n' > payload/hello.txt
btpc create payload -o payload-v1.torrent
btpc create payload --mode v2 --piece-length 16384 -o payload-v2.torrent
btpc create payload --mode hybrid --piece-length 16384 -o payload-hybrid.torrent
```

Inspect and verify the hybrid result:

```console
btpc inspect payload-hybrid.torrent
btpc validate payload-hybrid.torrent
btpc magnet payload-hybrid.torrent
btpc verify payload-hybrid.torrent payload
```

Use `--json` for versioned machine output. Human diagnostics and progress are sent
to stderr. Continue with the [complete CLI guide](../cli/index.md).
