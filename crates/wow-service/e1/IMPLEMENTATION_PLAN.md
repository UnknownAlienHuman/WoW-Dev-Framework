# E1-D implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — freeze prerequisites

- implement and freeze `wow-core` E0-A;
- implement and freeze `wow-store` E1-A;
- implement and freeze `wow-reference` E1-B;
- implement and freeze `wow-annotations` E1-C;
- freeze component compatibility reports and fixture checksums;
- freeze one E1 pack layout profile and builder application contract.

No E1-D Rust code before the required boundary types and IDs are real and tested.

## Phase 1 — request and component set

Implement:

- build, validate, and rebuild request types;
- exact component, profile, and layout selectors;
- compatibility and freeze validation;
- budgets and cancellation;
- typed errors.

Tests: `PACK-CONFIG-*`.

## Phase 2 — build state machine

Implement transport-independent orchestration with injectable typed component ports. No filesystem implementation yet.

Tests:

- legal and illegal transitions;
- every component failure, partial, and cancellation state;
- no fallback or substitution;
- prior state untouched.

## Phase 3 — assembly plan

Implement:

- layout profile validation;
- member inventory and ownership;
- noncyclic pack/checksum identity plan;
- license/provenance closure;
- safe artifact-relative paths;
- typed `PackMaterializationPlan`.

Tests: `PACK-ASSEMBLY-*`.

## Phase 4 — application adapters

Implement `wow-reference-builder` with only `wow-service` dependency:

- explicit config/request loading;
- materialized source adapter;
- staging/materialization adapter;
- atomic destination adapter;
- JSON/text/exit-code projection;
- no network, shell, editor, or source execution.

Tests: `PACK-APP-*`, `PACK-SEC-*`.

## Phase 5 — independent validator

Implement read-only validation from staged/final candidate bytes. Do not share unchecked in-memory assumptions with build.

Tests: `PACK-VALIDATE-*` plus mutation corpus.

## Phase 6 — rebuild comparison

Implement isolated repeated builds and structured equivalence classes. Integrate store physical determinism classification without overclaim.

Tests: `PACK-DET-*`.

## Phase 7 — recovery and cancellation

Implement safe cleanup, quarantine, reopen, finalization, and exact recovery records.

Tests: `PACK-CANCEL-*`, `PACK-RECOVER-*`.

## Phase 8 — freeze fixtures

Populate all null component, profile, request, member, report IDs and SHA-256 values in examples and `CHECKSUMS.json`. Tests verify committed bytes and never rewrite them.

## Phase 9 — integration gate

Run:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
all E1 component fixture suites
E1-D full build/validate/rebuild mutation matrix
security/path/cancellation corpus
```

Missing tools are `skipped`, never pass. No in-client WoW validation is implied by pack build tests.

## Deferred after E1-D

- source acquisition/update channels;
- full UI graph, skeleton, lineage, and search pack members;
- LSP/MCP runtime frontends;
- signing, SBOM, attestation, distribution;
- release channel activation and rollback;
- CI.
