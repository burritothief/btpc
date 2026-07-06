# Verifying payloads

```console
btpc verify payload.torrent payload
btpc verify payload.torrent payload --fail-fast --extra-files --json
```

v1 hashes the logical concatenated file stream. v2 verifies each file through BEP
52 Merkle roots. Hybrid torrents verify both domains. Mismatches are deterministic;
unsafe paths and symlinks are never followed. Verification opens the selected
payload as a filesystem capability, walks descendants relative to opened directory
handles, and hashes retained no-follow file handles. Concurrent path replacement is
reported as unsafe or missing rather than allowing verification to leave the
selected payload root.

A completed mismatch report exits with code `6`. I/O, policy, and malformed
metainfo failures retain their category-specific exit codes.
