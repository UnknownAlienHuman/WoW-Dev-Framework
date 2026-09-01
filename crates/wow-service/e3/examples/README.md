# E3-C service fixture shapes

All implementation-dependent IDs, exact result bytes, owner-port versions, evaluation reports, and SHA-256 values remain null only while `implementation_state = not-started`.

## Fixtures

- `context-status-request.json` — bounded status request and exact selector resolution.
- `context-map-request.json` — primary/platform/combined Project Map service request.
- `context-inspect-request.json` — exact L0/L1 root inspection.
- `context-build-request.json` — full semantic pack and renderer request.
- `context-continue-request.json` — exact retained continuation; current selectors forbidden.
- `context-artifact-requests.json` — nonrepairing validate and render transport inputs.
- `context-result-envelope.json` — canonical public success envelope shape.
- `acquisition-cases.json` — current races, guards, compatibility, capability, and exact-reference cases.
- `lifecycle-cases.json` — partial acquisition, cancellation, close, retention, and broken-transport cases.
- `CHECKSUMS.json` — prerequisite, port, profile, corpus, vector, artifact, app, and member freeze gate.

Tests validate committed bytes and never rewrite canonical fixtures automatically.
