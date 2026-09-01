# E3-B identity-DAG tests

**Status:** normative cycle and forward-reference mutation contract.

## Canonical order

```text
owner project/graph/reference/source views
-> ContextUniverseSetId

ContextUniverseSet + reviewed profiles
-> ContextProfileSetId

ContextUniverseSet + exact roots/request fields
-> ContextRequestId

universe/profile/request + exact owner records
-> ProjectMapId / L0SkeletonId / L1SkeletonId

request/profiles/frontier rules
-> ContextExpansionPlanId / ContextFrontierId

selected semantic payloads and exact origins
-> ContextItemId / SourceExcerptItemId

candidate decisions and costs
-> SelectionTraceId / OmissionManifestId / ContextBudgetReportId

universe/request/profiles/maps/skeletons/items/trace/omissions/budget
-> ContextSemanticPackId

semantic pack + renderer/profile + exact rendered bytes
-> RenderedContextArtifactId

semantic/render artifacts
-> metrics, comparison, evaluation, validation, cache-storage, and delivery records
```

## Forbidden back-references

A semantic identity must not include:

- its own ID/digest;
- a validation report that validates it;
- renderer artifact ID/bytes when computing semantic pack identity;
- metrics/evaluation score;
- cache location/hit state;
- delivery envelope/request timestamp;
- physical store row/page/WAL identity;
- current pointer or later generation;
- continuation page produced after the artifact;
- consumer model response.

## Required mutations

| ID | Mutation | Expected |
|---|---|---|
| `CTX-DAG-001` | Semantic pack includes renderer artifact ID | reject |
| `CTX-DAG-002` | Semantic pack includes metrics/evaluation report ID | reject |
| `CTX-DAG-003` | Map ID includes pack ID | reject |
| `CTX-DAG-004` | Request ID includes expansion result ID | reject |
| `CTX-DAG-005` | Item includes selection trace that selected it | reject or split nonidentity reference |
| `CTX-DAG-006` | Budget report includes rendered bytes before renderer identity | use predicted cost only; exact rendered report is later |
| `CTX-DAG-007` | Cache key includes physical path/hit counter | reject |
| `CTX-DAG-008` | Universe set includes mutable current pointer | reject |
| `CTX-DAG-009` | Validation rewrites same artifact under unchanged ID | reject |
| `CTX-DAG-010` | Continuation cursor changes request/profile/generation | reject |
| `CTX-DAG-011` | Source excerpt ID includes Markdown output line range | reject; renderer mapping is later |
| `CTX-DAG-012` | Renderer template reads evaluation score to omit items | reject |

## Graph validation

Construct a directed dependency graph for every canonical type field. Fail if:

- any strongly connected component has more than one node;
- any node has a self-edge;
- a field points to a later layer without being explicitly nonidentity metadata;
- a supposedly noncanonical field changes the object's canonical digest;
- a semantic object cannot be built without first rendering/validating/evaluating itself.

## Serialization mutations

- reorder equivalent JSON object fields: semantic object normalizes identically;
- reorder semantic arrays where order is schema-significant: ID changes or validation rejects, according to field definition;
- inject later-layer IDs into unknown fields: unknown-field rejection;
- use retired `ContextBundleCore` and current `ContextSemanticPack` as separate objects: reject;
- include timestamp/host/process/cache state in canonical payload: reject.

## Continuation chain

Continuation page IDs form a forward chain over one exact request/universe/profile and prior page manifest. A prior semantic pack does not include future page IDs. Combined-page validation occurs in a later chain report and does not alter page IDs.

## Acceptance

The machine schema and Rust type graph must pass automated cycle/field-layer checks. Tests verify frozen fixture identities and never regenerate them in place.
