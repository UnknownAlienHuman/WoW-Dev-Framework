# Crate dependency graph

**Status:** normative implementation boundary through documentation frontier E5-B.

Dependencies point toward narrower foundations. Maximum permitted dependencies do not require activation.

## Maximum permitted direct framework dependencies

| Crate | Maximum direct dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-service` | reviewed production crates through narrow public contracts |
| applications | `wow-service` only |

## Active E5-B operation slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-graph
        └── wow-recognizers
```

`wow-reference`, `wow-annotations`, `wow-emmy`, `wow-rules`, `wow-search`, `wow-context`, and `wow-cbm` are inactive direct dependencies for E5-B. Their relevant immutable facts may already be referenced by exact project/E5-A artifacts, but E5-B does not invoke those crates directly.

## E5-A owner seam

```text
exact source/project/analyzer/graph/fact publications
    -> caller/orchestrator
    -> wow-recognizers E5-A
        -> candidate-owned shadow partitions
        -> independent wow-graph proposal validation
        -> immutable case/mutation/metric/candidate/deactivation artifacts
```

`wow-recognizers` does not import `wow-project`, `wow-store`, service, or applications. It owns E5-A validation/evaluation algorithms and emits exact immutable artifacts.

## E5-B orchestration seam

```text
exact retained E5-A artifacts
+ exact project/fact publication views
+ graph validation port
+ durable operation/store/retention/audit ports
+ review authorization port
+ holdout authorization and vault ports
    -> wow-service E5-B
    -> immutable run/review/holdout/audit/promotion-submission records
    -> apps/wow transport
```

`wow-service` does not inspect raw database tables, source bodies, analyzer sessions, matcher internals, hidden holdout membership, private credentials, or vault storage.

## Authorization boundaries

```text
strict review envelope
    -> ReviewAuthorizationPort
    -> independent candidate/graph semantic validation
    -> immutable review record

strict frozen holdout request
    -> HoldoutAuthorizationPort
    -> exact HoldoutVaultPort execution
    -> audit/disclosure/consumption records
```

Review authorization does not grant holdout access. Either authorization does not create semantic proof, alter metrics, raise confidence, or authorize E5-C publication.

GitHub account/repository role, OS user, terminal/CLI operator, file owner, commit author, and plain prose are not authorization.

## Durable effect boundary

```text
OperationId + CanonicalRequestDigest
    -> durable registration
    -> exact owner effect
    -> exact receipt/reconciliation
    -> retention
    -> reverse resource closure
    -> public result
```

Response loss never proves absence of an effect. `OutcomeUnknown` blocks blind redispatch. No public success precedes mandatory retention and closure.

## E5-C handoff

```text
exact validated PromotionSubmission
    -> future E5-C independent revalidation
    -> distinct immutable CorePackArtifact
    -> publication/signing/catalog
    -> canary/activation/rollout/rollback/last-known-good
```

E5-B has no publication, current/default mutation, canary, rollout, rollback, signing, or distribution operation.

## Existing E4 boundaries remain active

- `wow-search` returns candidates and explanations; it does not prove intent or lineage.
- `wow-graph` validates lineage/static-impact under proof ceilings; it does not call search/service.
- `wow-context` consumes explicit exact roots; search does not call context.
- `wow-service` coordinates only through narrow owner contracts.
- Applications import `wow-service` only.

## Forbidden patterns

- `wow-core` depending on another framework crate.
- `wow-store` interpreting project, graph, search, context, calibration, or authorization semantics.
- `wow-graph` parsing source, running recognizers/search, authorizing reviewers, or calling service/apps.
- `wow-recognizers` materializing repositories, storing durable service effects, authorizing review/holdout, or publishing core packs.
- `wow-project` depending on service/apps.
- `wow-context` depending on store/emmy/recognizers/rules/search/cbm/service/apps.
- `wow-service` reproducing corpus/split/matcher/mutation/metric/graph/deactivation algorithms.
- An app importing any framework crate except `wow-service`.
- Raw SQL, connection, parser, source scanner, filesystem root, private credential, vault token, or executable callback crossing a public seam.
- A production crate depending on an application.
- Test helpers becoming runtime dependencies.

## Changing the graph

A dependency change requires the exact crossing operation/data, why current seams are insufficient, cycle and identity-DAG analysis, security/privacy/license/evidence impact, tests on both sides, a mutation rejecting the shortcut, migration notes, and manifest/workstream updates.
