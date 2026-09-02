# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E5-B.

Documentation-ready remains `implementation_state = not-started` until executable code, probes, fixtures, checksums, benchmarks, authorization adapters, vault/reconciliation infrastructure, and evaluations exist.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture
-> E0-E wow-rules
-> E0-F wow-service + apps/wow diagnostics

-> E1-A wow-store foundation
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validate

-> E2-A wow-graph
-> E2-B wow-recognizers core matcher
-> E2-C full wow-project candidate
-> E2-D ProjectStore coherent publication

-> E3-A Blizzard UI source universe
-> E3-B Project Map/L0/L1/context
-> E3-C context service/CLI

-> E4-A exact-generation search
-> E4-B lineage/migration/static impact
-> E4-C search/lineage/impact service/CLI

-> E5-A calibration corpora, shadow packs, evaluation and candidates
-> E5-B durable calibration orchestration, review, holdout audit and submissions
-> E5-C immutable core-pack publication/signing/canary/rollout/rollback

-> E6 optional Codebase Memory candidate bridge
-> E7 LSP/MCP/release integration
```

## Global rules

- One agent owns one primary package/crate.
- Shared seams are proposed before dependent implementation.
- Missing implementation, tool, probe, benchmark, authorization, vault, runtime test, or source evidence is blocked/`NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact pinned source/runtime evidence.
- Repository/addon/path/provider/popularity/model names never become hidden production semantics.
- Search rank and calibration metrics never become intent, lineage, review, publication, activation, safety, or runtime authority.
- Applications import `wow-service` only.
- No CI/workflow without explicit owner instruction.

## E5-A owner work

Primary contract: [`wow-recognizers/e5/`](wow-recognizers/e5/README.md)

```text
exact candidate sources/publications
+ provenance/license/privacy/labels/splits
+ E2-B declarative calibration pack
+ mutation/evaluation profiles
-> candidate-owned shadow partitions
-> independent graph validation
-> per-case/mutation/metric reports
-> candidate and deactivation artifacts
```

Hard stops: no commit-pin-only admission, donor-name semantics, split leakage, hidden Negative coercion, confidence above `Derived`/`Possible`, default graph publication, core activation, or deletion of foreign/core partitions.

## E5-B service/application work

Primary contracts:

- [`wow-service/e5/`](wow-service/e5/README.md)
- [`../apps/wow/e5/`](../apps/wow/e5/README.md)

### Ownership

`wow-service` coordinates exact retained E5-A artifacts, project/fact publications, graph validation, durable operation/storage/retention/audit ports, review authorization, and holdout authorization/vault ports. `apps/wow` is transport-only.

### Required order

```text
register durable OperationId + CanonicalRequestDigest
-> acquire exact retained artifacts in fixed order
-> validate compatibility/visibility/privacy/license
-> invoke E5-A owner operations
-> persist exact effect receipts
-> independently validate graph state
-> independently authorize review when requested
-> independently authorize and audit holdout access when requested
-> classify disclosure and candidate-lineage consumption
-> build/validate immutable PromotionSubmission when requested
-> admit retention/audit records
-> close resources in reverse order
-> return canonical envelope
```

### Hard stops

```text
no latest/best/highest-metric/first/last/sole selection
no E5-A algorithm reimplementation
no blind retry after uncertain effect
no GitHub/OS/CLI/file/commit identity as authorization
no review permission reused as holdout permission
no holdout access before candidate/run/evaluator/profile freeze
no hidden or unaudited holdout disclosure
no consumed/unknown holdout called untouched
no blocker hidden by aggregate metrics or review
no source/label/split/candidate mutation
no core publication/activation/canary/rollout/rollback
no public success before retention/closure
no credentials or hidden labels/source in normal output
```

### Implementation gate

Before E5-B Rust:

```text
implemented/frozen E0-E5-A prerequisites
exact owner catalog/read/effect/reconciliation ports
review and holdout authorization adapters/profiles
holdout vault/evaluator/disclosure/audit/consumption profiles
durable operation/idempotency/retention/recovery profiles
canonical service and CLI request/result/error/output vectors
synthetic and admitted real authorization/holdout/response-loss corpora
measured resource thresholds
all member/bundle SHA-256 values
```

## Next — E5-C

E5-C owns one exact `PromotionSubmission` handoff and must independently revalidate it before creating a separate immutable `CorePackArtifact`.

Required scope:

```text
immutable publication candidate and catalog
independent submission/candidate/pack revalidation
signing/provenance/SBOM/license/notice attestations
PublishedInactive + fresh read-back validation
exact canary cohorts and observation profiles
guarded current/default activation and finite rollout stages
explicit last-known-good
rollback/revocation/deactivation/stale producer-partition closure
idempotency/response-loss/retention/audit/recovery
thin service-only CLI
```

E5-C cannot rewrite E5-A/B evidence, treat signatures as semantic proof, claim global runtime correctness from canary success, infer last-known-good as previous/newest, or expose public distribution before E7.

## Seam request format

```text
requesting package/crate
owning crate
required operation/data
rejected workaround
why existing seam is insufficient
smallest proposed seam
cycle/identity/security/privacy/license/evidence impact
fixture/mutation proving it
freeze/migration impact
```

Do not implement a missing seam in the wrong crate.