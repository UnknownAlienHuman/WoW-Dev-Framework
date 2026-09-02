# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E5-B complete
next documentation package: E5-C immutable core-pack publication/signing/canary/rollout/rollback
implementation frontier: not started
```

No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist.

## Milestone summary

| Milestone | Documentation outcome | Documentation | Implementation |
|---|---|---:|---:|
| E0 | deterministic diagnostic vertical slice | Complete | Not started |
| E1 | ReferenceStore/View, annotations, pack build/validation | Complete | Not started |
| E2 | graph, recognizers, project indexing and ProjectStore | Complete | Not started |
| E3-A | exact Blizzard UI source universe | Complete | Not started |
| E3-B | Project Map, L0/L1 and context packs | Complete | Not started |
| E3-C | context service and CLI | Complete | Not started |
| E4-A | exact-generation search | Complete | Not started |
| E4-B | lineage, migration records and static impact | Complete | Not started |
| E4-C | search/lineage/impact service and CLI | Complete | Not started |
| E5-A | calibration corpora, shadow packs, evaluation and candidate artifacts | Complete | Not started |
| E5-B | durable runs, review authorization, sealed holdout, promotion submissions | Complete | Not started |
| E5-C | immutable core-pack publication, signing, canary, rollout and rollback | Next | Not started |
| E6 | optional Codebase Memory candidate bridge | Planned | Not started |
| E7 | LSP/MCP and release integration | Planned | Not started |

## Implementation remains E0-first

Documentation maturity does not reorder implementation. Executable work begins with:

```text
wow-core E0-A
-> fixture wow-reference + pinned wow-emmy
-> minimal wow-project
-> two bounded wow-rules
-> wow-service status/check
-> thin apps/wow
```

Later packages remain blocked until prerequisite implementation commits, profiles, fixtures, probes, benchmarks, authorization adapters, runtime evidence, and SHA-256 manifests exist.

## E0–E4 documented foundation

- **E0:** exact identity/evidence/coverage/result primitives and one deterministic diagnostic slice.
- **E1:** generic store, persistent ReferenceView, annotations, and nonrepairing Reference Pack assembly.
- **E2:** assertion graph, declarative recognizers, bounded TOC/XML/load indexing, incremental invalidation, coherent ProjectStore publication.
- **E3:** separate Blizzard UI source universe, Project Map/L0/L1/context engine, and exact service/CLI acquisition.
- **E4:** exact-generation search, explicit lineage/change/migration/static impact, and service/CLI orchestration.

All keep candidate, partial, conflict, truncation, `NotEvaluated`, source implementation, platform contract, and runtime evidence distinct.

## E5-A — calibration owner

Contract: [`../crates/wow-recognizers/e5/README.md`](../crates/wow-recognizers/e5/README.md)

E5-A owns exact candidate-source/corpus/provenance/label/split/pack validation, shadow matching, anti-overfitting mutations, graph proposal validation receipts, per-case-first metrics, candidate artifacts, and deactivation plans.

Hard stops:

```text
no commit-pin-only admission
no repository/addon/owner/path/popularity semantics
no labels/splits/reviewer/expected outputs as matcher input
no copied/forked/vendor/generated leakage across ordinary splits
no Unknown/Possible/NotEvaluated/Conflict/Partial/Truncated -> Negative/pass
no confidence above Derived/Possible
no default/core rollout or graph publication
no hard failure hidden by aggregates
no deactivation of core/foreign partitions
```

Current real evidence remains:

```text
exact user-repository revisions pinned: 8
real admitted corpus members: 0
real measured calibration runs: 0
sealed holdout generations executed: 0
implemented promotion submissions: 0
```

## E5-B — durable orchestration, review, holdout, and submission

Contracts:

- [`../crates/wow-service/e5/README.md`](../crates/wow-service/e5/README.md)
- [`../apps/wow/e5/README.md`](../apps/wow/e5/README.md)

E5-B defines 22 service operations and matching thin CLI commands for exact source/corpus/split validation/admission, durable run lifecycle, case explanation, candidate validation, reviewer authorization, holdout request/execution/audit, immutable promotion submissions, and deactivation-plan validation.

### Required state separation

```text
metric eligibility != graph validity
graph validity != reviewer authorization
reviewer authorization != holdout authorization
holdout authorization != disclosure
disclosure != promotion approval
promotion submission != publication
publication != activation
activation != runtime correctness
```

### Durable operation gate

Every effecting operation registers:

```text
OperationId + CanonicalRequestDigest
```

before dispatch. Same ID/different digest is rejected. Response loss yields `OutcomeUnknown` until exact owner/store/vault reconciliation. No blind repeat, no hidden rerun, and no public success before retention and reverse-order closure.

### Review gate

Review uses a narrow authorization port and immutable exact-candidate decision envelopes. GitHub login/repository ownership, OS user, CLI operator, file owner, commit author, metric result, or successful graph validation is not authorization.

Authorization cannot alter candidate bytes, labels, metrics, graph proof, confidence, coverage, or proof ceilings.

### Sealed-holdout gate

Before access, freeze exact holdout generation, candidate pack, candidate artifact, implementation/evaluator, run request, profiles, budgets, disclosure, retention, and contamination policy.

Every request, grant, denial, open, execution, disclosure, failure, cancellation, revocation, replay, and consumption event is auditable. A consumed or contamination-unknown generation is never called untouched. Review authorization is not holdout authorization.

### Promotion-submission gate

A `PromotionSubmission` binds exact candidate, corpus, split, run, graph, mutation, metric, review, holdout, license/privacy, deactivation, blocker, and nonclaim evidence.

Submission states such as `Prepared`, `Validated`, or `ReadyForE5CReview` do not mean published, active, default, or runtime-verified. E5-B cannot publish or activate.

### E5-B implementation gate

Before Rust:

```text
all E0–E5-A implementations and fixture bundles
exact owner catalog/read/effect/reconciliation ports
review and holdout authorization adapters/profiles
holdout vault/evaluator/disclosure/audit/consumption profiles
durable operation/idempotency/retention/recovery profiles
canonical service and CLI request/result/error/output vectors
synthetic and admitted real authorization/holdout/response-loss corpora
measured resource thresholds
all member/bundle SHA-256 values
```

Missing evidence is blocked/`NotEvaluated`, never pass.

## Next — E5-C core-pack publication lifecycle

Owner: `wow-service`; thin application: `apps/wow`; collaborators: `wow-recognizers`, `wow-store`, `wow-graph`, `wow-project`.

E5-C must define:

```text
exact PromotionSubmission selector and independent revalidation
distinct immutable CorePackArtifact
publication catalog and PublishedInactive state
detached signatures, provenance/SBOM/license/notices without committed private keys
fresh read-back validation
exact canary cohorts and typed observation profiles
guarded current/default activation
finite staged rollout and pause
explicit retained last-known-good
rollback, deactivation, revocation and stale producer-partition closure
project/graph reindex handoff
OperationId/request-digest idempotency and response-loss recovery
retention, audit, recovery and thin service-only CLI
```

E5-C cannot rewrite E5-A/B evidence, treat a signature as semantic/runtime proof, infer last-known-good as previous/newest, claim ecosystem-wide runtime correctness from a canary, or expose public distribution before E7.

## E6–E7 planned boundary

- **E6:** optional external Codebase Memory/semantic candidates; Candidate-only, degradable, no provider database writes.
- **E7:** thin LSP/MCP and release/signing/distribution integration after implementation gates.

## Roadmap discipline

- Outcomes and proof gates matter; percentages and directory counts do not.
- Stable contracts link the current external WoW KB instead of copying patch-sensitive facts.
- Missing tools, probes, benchmarks, authorizations, vaults, evaluations, or client evidence are skipped, blocked, or `NotEvaluated`, never pass.
- Architecture changes require an ADR and concrete failure of the accepted design.
- No CI/workflow without explicit owner instruction.
