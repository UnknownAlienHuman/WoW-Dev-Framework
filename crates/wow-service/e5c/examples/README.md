# E5-C normative fixture shapes

- `submission-artifact.json` — independent E5-B submission revalidation and distinct core artifact identity.
- `publication-signature.json` — attestations, detached signatures, inactive publication and fresh read-back.
- `canary-rollout.json` — exact cohort, typed observations, finite rollout, activation, and LKG.
- `rollback-revocation.json` — exact rollback, revocation, deactivation, reindex, and stale partition closure.
- `CHECKSUMS.json` — prerequisite, adapter, profile, vector, member, and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`.