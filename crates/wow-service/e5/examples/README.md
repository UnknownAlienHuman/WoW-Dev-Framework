# E5-B normative fixture shapes

- `artifact-selectors.json` — exact selectors and unique/none/conflict states.
- `run-lifecycle-cases.json` — idempotency, response loss, cancellation, retention, and closure.
- `review-holdout-cases.json` — independent review and holdout authorization, audit, disclosure, and consumption.
- `promotion-submission.json` — immutable submission, blockers, nonclaims, and E5-C handoff.
- `CHECKSUMS.json` — prerequisite, adapter, profile, vector, member, and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`.