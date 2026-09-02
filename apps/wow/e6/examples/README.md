# E6-B CLI fixture shapes

- `cli-cases.json` — command-to-service mapping and rejected provider, mapping, selection, context, credential, and fallback shortcuts.
- `stdout-exit-cases.json` — canonical output, Candidate authority, mapping/selection/context state, `OutcomeUnknown`, and exit mappings.
- `CHECKSUMS.json` — service, command, profile, vector, platform, and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`.
