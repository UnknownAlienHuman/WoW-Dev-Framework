# E4-C CLI fixtures

These closed JSON shapes define the application transport boundary. All implementation-dependent IDs, bytes, digests and exit vectors remain `null` only while `implementation_state = not-started`.

- `cli-cases.json` — command, selector, one-call, continuation, migration and impact vectors.
- `stdout-exit-cases.json` — exact stdout framing, text/artifact, validation and exit-code vectors.
- `review-artifact-cases.json` — strict review/artifact input, authorization-boundary and privacy vectors.
- `CHECKSUMS.json` — prerequisite, parser/platform profile, canonical byte and member freeze gate.

No fixture contains a private signing key, access token, credential, confidential source body or real review secret.
