# `wow-release` fixture shapes

- `release-tool-cases.json` — exact command/service mappings, forbidden execution/secret/provider inputs, explicit pipeline stages and reconciliation.
- `CHECKSUMS.json` — service, command, transport, output, CI, security, vector, member and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`. Tests verify committed bytes and never rewrite fixtures.