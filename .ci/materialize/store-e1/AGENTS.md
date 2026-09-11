# `wow-store` implementation rules

- Preserve content-addressed immutability: never overwrite an existing object with different bytes.
- Rehash all bytes crossing the persistent boundary; filenames and manifests are not trusted alone.
- Mutable publication is compare-and-swap through the checksummed ref journal only.
- Incomplete evidence is an error or absent result, never a fabricated snapshot/ref state.
- Keep paths normalized and bounded. Do not follow or create symlink-based store structures.
- No source discovery, Lua execution, network access, live-client inference, raw SQL surface, or hidden global store.
- New persistent formats require a new schema string and backward-read tests before write adoption.
