# E5-B implementation plan

**Status:** normative order; implementation has not started.

0. Freeze all E0–E5-A implementation commits, fixtures, owner ports, authorization/vault adapters, profiles, canonical vectors, benchmarks, and checksums.
1. Implement closed request/selector/status/error primitives.
2. Implement durable operation registration, exact acquisition order, retention admission, reverse closure, and `OutcomeUnknown` reconciliation.
3. Add service wrappers for E5-A source/corpus/admission/split operations without copying algorithms.
4. Add visible-split run submit/get/list/cancel/retry and case explanation.
5. Add candidate build/validate and deactivation-plan validation.
6. Add reviewer authorization and immutable review records.
7. Add sealed-holdout request/grant/vault execution/audit/disclosure/consumption.
8. Add immutable promotion prepare/validate/get.
9. Freeze conservative result envelopes and canonical serialization.
10. Activate thin `apps/wow/e5` only after service bytes and exit mappings freeze.
11. Run E0–E5-A regressions, synthetic and admitted real corpora, forged/revoked/replayed credentials, response-loss/cancellation at every effect boundary, 1/2/N workers, shuffled scheduling, and resource benchmarks.
12. Populate implementation/profile/vector/checksum evidence and update the manifest only after fresh passes.

No placeholder reviewer, vault, run, measurement, submission, or authorization adapter is permitted. Core publication/signing/canary/activation/rollout/rollback remains E5-C.