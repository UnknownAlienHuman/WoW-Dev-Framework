# E7-B update-client fixture shapes

- `update-client-cases.json` — exact command/service mappings, local/network separation, forbidden selectors/flags, Windows helper handoff and reconciliation.
- `CHECKSUMS.json` — service, CLI, helper, platform, path, network, output, vector, member and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`. Tests verify committed fixture bytes and never rewrite them.