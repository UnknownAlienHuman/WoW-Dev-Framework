# E6-B normative fixture shapes

- `provider-session.json` — exact configuration, authorization reference, narrow session, capability/state binding, and secret exclusion.
- `query-result.json` — durable query dispatch, immutable result publication/read-back, Candidate authority, zero and continuation semantics.
- `mapping-selection.json` — project/reference owner mapping states, negative authority, explicit selection, and forbidden ranking shortcuts.
- `context-handoff.json` — exact mapped-root context build with separate external Candidate sidecar.
- `response-loss.json` — provider/store/mapping/selection/context `OutcomeUnknown`, reconciliation, cancellation, and close behavior.
- `CHECKSUMS.json` — prerequisite, adapter, profile, vector, member, and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`.