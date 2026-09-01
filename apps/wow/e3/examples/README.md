# E3-C CLI fixture shapes

- `cli-cases.json` — exact selector/root/profile/config/artifact command-to-service mappings.
- `stdout-exit-cases.json` — envelope JSON, text, artifact, stderr, exit, cancellation, and broken-pipe vectors.
- `CHECKSUMS.json` — service/app/profile/platform/vector/member freeze gate.

All IDs, byte vectors, platform-adapter results, and SHA-256 values remain null only while implementation state is `not-started`. Tests verify committed fixtures and never rewrite them automatically.
