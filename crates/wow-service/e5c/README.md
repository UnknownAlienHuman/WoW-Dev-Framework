# `wow-service` E5-C immutable core-pack publication lifecycle

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `wow-service/e5-c/core-pack-publication-rollout-lifecycle`

## Mission

Convert one exact independently revalidated E5-B `PromotionSubmission` into a distinct immutable core recognizer-pack artifact, publish it inactive, verify signatures/provenance/license/read-back closure, gather explicitly scoped canary evidence, and perform guarded profile-specific activation, rollout, pause, rollback, revocation, and stale producer-partition closure.

```text
exact E5-B PromotionSubmission
+ exact E5-A candidate/pack/evidence artifacts
+ reviewed publication/signing/canary/rollout/rollback profiles
+ independent authorization and owner ports
-> revalidate every mandatory submission gate
-> build distinct content-addressed CorePackArtifact
-> produce provenance/SBOM/license/notice attestations
-> request detached signature through an external signing boundary
-> publish immutable PublishedInactive catalog record
-> reopen and validate exact bytes/catalog/store/graph/recognizer closure
-> define exact bounded canary cohort and observation plan
-> evaluate typed scoped canary evidence
-> advance only through finite authorized rollout stages
-> update one exact execution-profile current record by guarded CAS
-> explicitly designate retained last-known-good
-> roll back/revoke/deactivate through exact immutable records
-> reindex affected projects and validate stale producer-partition closure
```

A submission, signature, inactive publication, canary pass, rollout stage, active pointer, or last-known-good designation never becomes global runtime correctness.

## Reading order

1. `AGENTS.md`
2. `DECISIONS.md`
3. `DATA_MODEL.md`
4. `SUBMISSION_REVALIDATION_AND_ARTIFACT_BUILD.md`
5. `SIGNING_PROVENANCE_SBOM_AND_LICENSE.md`
6. `PUBLICATION_CATALOG_AND_READBACK.md`
7. `CANARY_COHORTS_AND_OBSERVATIONS.md`
8. `ACTIVATION_ROLLOUT_AND_LAST_KNOWN_GOOD.md`
9. `ROLLBACK_REVOCATION_AND_PARTITION_CLOSURE.md`
10. `DURABLE_EFFECTS_RETENTION_AND_RECOVERY.md`
11. `RESULT_ENVELOPE_AND_STATUS.md`
12. `SECURITY_PRIVACY_AND_DISTRIBUTION.md`
13. `ERROR_MODEL.md`
14. `TEST_MATRIX.md`
15. `IMPLEMENTATION_PLAN.md`
16. `CONTRACT.json` and `examples/`
17. `../../../apps/wow/e5c/README.md`

## Active dependencies

```text
wow-core
wow-store
wow-project
wow-graph
wow-recognizers
```

E5-B artifacts are acquired through exact service/store catalogs. E5-C does not directly activate `wow-reference`, `wow-emmy`, `wow-rules`, `wow-search`, `wow-context`, or `wow-cbm`.

## Public operations

```text
core_pack_status
core_pack_submission_validate
core_pack_artifact_build
core_pack_artifact_validate
core_pack_sign_request
core_pack_signature_validate
core_pack_publish
core_pack_publication_get
core_pack_publication_list
core_pack_publication_validate
core_pack_canary_plan
core_pack_canary_start
core_pack_canary_status
core_pack_canary_observation_record
core_pack_canary_evaluate
core_pack_rollout_plan
core_pack_rollout_advance
core_pack_rollout_pause
core_pack_activation_get
core_pack_activate
core_pack_last_known_good_get
core_pack_last_known_good_designate
core_pack_rollback
core_pack_revoke
core_pack_deactivate
core_pack_partition_closure_validate
```

## Explicitly deferred

- public release distribution, package registries, download channels, and updater protocols;
- LSP/MCP/daemon/session transport;
- automatic source edits or project migration;
- runtime WoW instrumentation beyond exact owner-provided canary observation adapters;
- ecosystem-wide safety/generalization claims;
- CI/release automation without explicit owner instruction.

## Completion gate

Implementation is complete only when independent submission revalidation, artifact identity, signatures/provenance/SBOM/license, inactive publication/read-back, exact canary assignment, typed observation evaluation, finite rollout, current-record CAS, last-known-good designation, rollback/revocation/deactivation, project/graph stale-partition closure, response-loss reconciliation, retention/audit/recovery, privacy, security, deterministic bytes, and thin CLI tests pass with all prerequisite pins and checksums frozen.