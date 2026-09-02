# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E5-C.

Documentation-ready remains `implementation_state = not-started` until executable code, exact fixtures/checksums, probes, benchmarks, authorization/signing/observation adapters, runtime evidence, and evaluations exist.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture
-> E0-E wow-rules
-> E0-F wow-service + apps/wow diagnostics
-> E1 Reference Pack stack
-> E2 graph/recognizers/project/ProjectStore
-> E3 Blizzard UI source/context/service
-> E4 search/lineage/migration/static-impact/service
-> E5-A calibration owner
-> E5-B durable calibration review/holdout/submission
-> E5-C core-pack publication/signing/canary/rollout/rollback
-> E6-A wow-cbm external semantic candidates
-> E6-B service/CLI mapping/context orchestration
-> E7 LSP/MCP/public release integration
```

## Global rules

- One agent owns one primary package/crate.
- Missing implementation/tool/probe/benchmark/authorization/signing/vault/observation/runtime evidence is blocked or `NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact source/runtime evidence.
- Repository/addon/owner/path/provider/popularity/model identity never becomes hidden semantics.
- Applications import `wow-service` only.
- No CI/workflow without explicit owner instruction.

## E5-C publication lifecycle

Primary contracts:

- [`wow-service/e5c/`](wow-service/e5c/README.md)
- [`../apps/wow/e5c/`](../apps/wow/e5c/README.md)

```text
exact E5-B PromotionSubmission
-> independent E5-C revalidation
-> distinct CorePackArtifact
-> provenance/SBOM/license/notices
-> detached signing + independent verification
-> PublishedInactive
-> fresh read-back validation
-> exact canary cohort/assignment/observations/evaluation
-> finite authorized rollout stages
-> profile-specific current-record CAS
-> explicit retained last-known-good
-> exact rollback/revocation/deactivation
-> new project/graph generations with stale partition closure
```

### Active owners

```text
wow-service: orchestration, authorization use, durable effects, envelopes
wow-recognizers: core pack semantics and producer namespace
wow-graph: graph output/partition/closure validation
wow-project: exact reindex and new project generations
wow-store: immutable objects/catalog/current/retention/GC
apps/wow: transport only
```

### Hard stops

```text
no trust in submission label without exact revalidation
no relabelled candidate as core artifact
no signature as semantic/runtime proof
no private key/KMS/HSM/vault/deployment credential in repository/CLI/results
no publication side-effect activation
no canary percentage without exact population/membership
no untyped anecdote/issue/model signal
no missing/partial/conflict/NotEvaluated required signal as pass
no time-only or open-ended rollout
no latest/best/previous/default activation or rollback target
no stale current CAS rebase
no inferred previous/newest last-known-good
no rollback history rewrite
no historical project/graph mutation
no stale partition or hidden coverage change
no blind effect retry after response loss
no public distribution in E5-C
```

### Implementation gate

Before E5-C Rust:

```text
implemented/frozen E0–E5-B prerequisites
exact submission/artifact/recognizer/graph/project/store ports
signing and authorization adapters/profiles without committed secrets
provenance/SBOM/license/reproducibility profiles
canary population/privacy/observation/signal profiles
finite rollout/activation/LKG/rollback/revocation/closure profiles
response-loss/retention/audit/recovery profiles
canonical service/CLI/artifact/signature/state vectors
synthetic and admitted real canary/rollout/rollback corpora
measured thresholds and all SHA-256 manifests
```

## Next — E6-A

Owner: `wow-cbm`; direct dependency remains `wow-core` only.

E6-A must define exact reviewed provider descriptors, external-state classes, bounded allow-listed query transport, Candidate-only normalization, provider-local scoring, unverified source locators, zero-result negative-authority prohibition, continuation/cache, optional degradation, privacy/license/security, and a later E6-B mapping/service handoff.

It must not read/write provider databases, expose arbitrary MCP/tool calls, invoke models, map locators into project truth, create graph/lineage/replacement proof, or make exact local workflows depend on provider availability.

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