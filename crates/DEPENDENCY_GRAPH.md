# Crate dependency graph

**Status:** normative boundary through documentation frontier E6-A.

Dependencies point toward narrower foundations. Maximum edges do not require activation.

| Crate | Maximum permitted direct dependencies |
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

## Active E5-C slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-graph
        └── wow-recognizers
```

Other crates are inactive direct dependencies in E5-C. Their relevant evidence may be transitively referenced by exact E5-A/B artifacts but E5-C does not invoke them.

## E5-C owner/effect flow

```text
E5-B PromotionSubmission catalog
-> wow-service independent revalidation
-> wow-recognizers core semantic/producer validation
-> wow-graph output and partition validation
-> wow-store immutable artifact/attestation/signature/catalog publication
-> signing authorization/signing/verification ports
-> canary authorization/assignment/observation ports
-> rollout/activation/rollback/revocation authorization ports
-> wow-project exact reindex
-> wow-graph new snapshot/partition closure
-> wow-store current/LKG/retention/audit
-> apps/wow transport
```

No owner imports service or applications. `wow-store` does not interpret recognizer/project/graph semantics. `wow-recognizers` does not publish catalogs/current pointers or authorize rollout. `wow-graph` does not run recognizers or resolve current. `wow-project` does not select packs. Service coordinates exact public receipts only.

## E5-C distinct authorities

```text
E5-B review authorization
E5-C artifact validation
signing authorization
signature verification
publication authorization
canary authorization and typed evidence
rollout authorization
activation/current CAS
LKG designation
rollback/revocation/deactivation authorization
future E7 distribution authorization
runtime correctness
```

No edge collapses these states. GitHub/OS/CLI/file/commit identity is not any authorization.

## Historical immutability

Activation or rollback creates new project/recognizer/graph generations. Historical generations retain original partitions. Stale partition closure applies only to new targets and preserves foreign/core-independent/calibration partitions.

## Active E6-A slice

```text
wow-cbm
└── wow-core
```

E6-A receives an already-acquired narrow transport through a host/service adapter contract but does not depend on a provider SDK/MCP implementation, service, store, project, reference, graph, search, context, or applications.

## E6-A external candidate boundary

```text
E6-B/session owner
    -> reviewed ExternalCandidateTransportPort
    -> wow-cbm E6-A
        -> ExternalCandidateResultSet
        -> ExternalCandidateArtifact
        -> UnverifiedProviderLocator
```

`wow-cbm` never creates a reverse dependency into E6-B owners.

## Future E6-B mapping

```text
wow-service
    ├── wow-cbm
    ├── wow-project
    ├── wow-reference
    ├── wow-context
    ├── wow-store
    └── wow-core

apps/wow -> wow-service
```

Exact provider/session/credential acquisition and durable operations remain service/adapter concerns. Project/reference owners validate locator mapping. Context receives only an exact mapped owner root after an explicit selection receipt.

## E6 authority boundaries

```text
external candidate result   = semantic_candidate + Candidate
provider locator            = UnverifiedProviderLocator
owner mapping receipt       = exact locator-to-owner-record mapping only
selection receipt           = explicit user/caller choice, not proof
context handoff             = exact owner root; provider metadata remains external evidence
```

No rank, score, label, stable generation, repeated result, or zero result becomes local truth.

## Provider failure

Unavailable provider disables only the optional external lane. It cannot lower exact ReferenceView/project/graph/search/context/rules capability. E6-A performs no hidden fallback.

## E5/E6 separation

External provider candidates cannot enter E5 core-pack publication evidence or production matcher semantics without a future explicit reviewed calibration/admission path; E6 creates no such path.

## Forbidden patterns

- `wow-core` depending on framework crates.
- `wow-store` interpreting domain semantics.
- `wow-graph` parsing source, running recognizers, authorizing, or calling service/apps.
- `wow-recognizers` storing durable service/publication effects or activating packs.
- `wow-project` depending on service/apps.
- `wow-service` reimplementing owner algorithms or exposing raw storage/signing/vault/observation/provider handles.
- `wow-cbm` depending on service/apps/store/project/reference/graph/search/context.
- `wow-cbm` owning process/session/credentials/provider index/database lifecycle.
- Generic arbitrary MCP/tool-call API.
- Provider locator opened or converted to stable source inside E6-A.
- Cross-provider score fusion or Candidate promotion.
- Zero-result negative authority.
- Provider failure downgrading local exact capabilities.
- Application importing any framework crate except `wow-service`.
- Private credentials, raw SQL, parser/session objects, filesystem roots, arbitrary executable callbacks, provider databases, or unrestricted source bodies crossing public seams.
- Production crate depending on application.

Changing an edge requires exact crossing data/operation, insufficiency of current seam, cycle/identity/security/privacy/license/evidence analysis, tests/mutations, migration notes, and manifest/workstream updates.