# Crate dependency graph

**Status:** normative boundary through documentation frontier E6-A.

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

## Active E6-A slice

```text
wow-cbm
└── wow-core
```

E6-A receives an already-acquired narrow transport through a host/service adapter contract but does not depend on provider SDK/MCP implementation, service, store, project, reference, graph, search, context, or applications.

## External candidate boundary

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

## Authority boundaries

```text
external candidate result   = semantic_candidate + Candidate
provider locator            = UnverifiedProviderLocator
owner mapping receipt       = exact locator-to-owner-record mapping only
selection receipt           = explicit user/caller choice, not proof
context handoff             = exact owner root; provider metadata remains external evidence
```

No rank/score/label/zero result becomes local truth.

## Provider failure

Unavailable provider disables only the optional external lane. It cannot lower exact ReferenceView/project/graph/search/context/rules capability. E6-A performs no hidden fallback.

## Existing E5 boundary

E5-C publication/signing/canary/rollout remains independent. External provider candidates cannot enter core pack publication evidence or production matcher semantics without a future explicit reviewed calibration/admission path; E6 does not create one.

## Forbidden patterns

- `wow-cbm` depending on service/apps/store/project/reference/graph/search/context.
- `wow-cbm` owning process/session/credentials/provider index/database lifecycle.
- Generic arbitrary MCP/tool-call API.
- Provider locator opened or converted to stable source inside E6-A.
- Cross-provider score fusion or Candidate promotion.
- Zero-result negative authority.
- Provider failure downgrading local exact capabilities.
- App importing lower crates.
- Raw SQL, provider database, credentials/private endpoints, filesystem root, executable callback, or source body crossing public seams.

Changing an edge requires exact crossing data/operation, insufficiency analysis, cycle/identity/security/privacy/license/evidence impact, tests/mutations, migration notes, and manifest/workstream updates.