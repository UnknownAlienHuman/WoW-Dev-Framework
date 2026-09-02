# Crate dependency graph

**Status:** normative boundary through documentation frontier E6-B.

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
| applications/transports | `wow-service` only |

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

No owner imports service or applications. Service coordinates exact public receipts only.

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

E6-A receives an already-acquired narrow transport through a host/service adapter contract but does not depend on provider SDK/MCP implementation, service, store, project, reference, graph, search, context, or applications.

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

## Active E6-B slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-reference
        ├── wow-graph
        ├── wow-context
        └── wow-cbm

host provider/credential adapters
    -> narrow ports owned by wow-service/wow-cbm contracts
```

Host adapters are not framework semantic dependencies. They construct exact authorized sessions and expose only the reviewed E6-A transport.

## E6-B flow

```text
exact provider configuration + authorization reference
-> host session factory
-> E6-A provider descriptor/state/query/result validation
-> wow-store immutable result/artifact catalog
-> wow-project OR wow-reference exact locator mapping
-> explicit selection receipt in wow-service
-> exact project/reference/graph views
-> wow-context exact mapped-root operation
-> separate Candidate sidecar + combined service envelope
-> apps/wow transport
```

`wow-store` does not interpret provider/candidate/mapping/selection/context semantics. `wow-project` and `wow-reference` consume owner-neutral locator projections and do not depend on `wow-cbm` or service. `wow-context` receives only its existing exact universe/root contract and does not depend on `wow-cbm`.

## E6 authority boundaries

```text
external candidate result   = semantic_candidate + Candidate
provider locator            = UnverifiedProviderLocator
owner mapping receipt       = exact locator-to-owner-record identity only
selection receipt           = explicit caller choice, not proof
context artifact            = exact local owner evidence
external sidecar            = provider Candidate evidence kept separate
```

No rank, score, label, stable generation, repeated result, zero result, mapping, selection, or context inclusion becomes provider semantic truth.

## Provider failure

Unavailable provider disables only the optional external lane. It cannot lower exact ReferenceView/project/graph/search/context/rules capability. E6 performs no hidden fallback.

## E5/E6 separation

External provider candidates cannot enter E5 core-pack publication evidence or production matcher semantics without a future explicit reviewed calibration/admission path; E6 creates no such path.

## Next E7-A boundary

```text
LSP/MCP/CLI-daemon transport/application
    -> wow-service only
```

Transport packages own wire/session concerns only. They cannot import lower crates, discover arbitrary tools, execute shell/source, or bypass service authorization/evidence/retention boundaries.

## Forbidden patterns

- `wow-core` depending on framework crates.
- `wow-store` interpreting domain semantics.
- `wow-graph` parsing source, running recognizers, authorizing, or calling service/apps.
- `wow-recognizers` storing durable service/publication effects or activating packs.
- `wow-project` or `wow-reference` depending on service/apps/wow-cbm for mapping.
- `wow-context` depending on wow-cbm/service or accepting provider facts as semantic inputs.
- `wow-service` reimplementing owner algorithms or exposing raw storage/signing/session/provider handles.
- `wow-cbm` depending on service/apps/store/project/reference/graph/search/context.
- `wow-cbm` owning process/session/credentials/provider index/database lifecycle.
- Generic arbitrary MCP/tool/RPC/SQL/script/model/shell API.
- Provider locator opened or mapped by service/app instead of owner.
- Cross-provider score fusion or Candidate promotion.
- Zero-result negative authority.
- Implicit candidate/mapping/root selection.
- Provider failure downgrading local exact capabilities.
- Application/transport importing any framework crate except `wow-service`.
- Sensitive adapter material, parser/session/process objects, filesystem roots, arbitrary callbacks, provider databases, or unrestricted source bodies crossing public seams.
- Production crate depending on application/transport.

Changing an edge requires exact crossing data/operation, insufficiency of current seam, cycle/identity/security/privacy/license/evidence analysis, tests/mutations, migration notes, and manifest/workstream updates.