# E5-C implementation plan

**Status:** normative order; implementation has not started.

0. Freeze all E0–E5-B implementation commits, owner ports, signing/authorization/observation adapters, profiles, canonical vectors, benchmarks, and checksums.
1. Implement closed request, selector, status, error, and exact publication-state primitives.
2. Implement independent `PromotionSubmission` acquisition/revalidation and blocker matrix.
3. Implement distinct `CorePackArtifact` build/validate through recognizer and graph owner ports.
4. Implement provenance/SBOM/license/notice attestation generation and validation.
5. Implement signing authorization, detached signature request, verification, revocation, and response-loss reconciliation without private key exposure.
6. Implement immutable object/catalog publication to `PublishedInactive`, exact retention, and fresh read-back validation.
7. Implement exact canary cohort planning, assignment, typed observation append/validation, and per-signal evaluation.
8. Implement finite rollout plans, stage advancement/pause, exact eligibility, and authorization.
9. Implement profile-specific current-record activation CAS and explicit LKG designation.
10. Implement rollback, revocation, deactivation, emergency profile, and effect reconciliation.
11. Implement project reindex plus recognizer/graph stale partition closure and historical immutability checks.
12. Implement canonical envelopes, audit/recovery, privacy/license/distribution boundaries.
13. Activate thin `apps/wow/e5c` only after service bytes and exit mappings freeze.
14. Run E0–E5-B regressions; synthetic and admitted real submission/publication/canary/rollout/rollback corpora; forged/revoked signing/authorization cases; response-loss/cancellation at every effect; 1/2/N worker/order/storage variation; and measured resource benchmarks.
15. Populate all implementation/profile/vector/checksum evidence and update manifest only after fresh passes.

No placeholder signing adapter, fake canary, fake observation, fake rollout, fake LKG, or fake measured success is allowed. Public release distribution remains E7.