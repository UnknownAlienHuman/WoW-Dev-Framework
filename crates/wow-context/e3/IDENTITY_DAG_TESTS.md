# E3-A artifact identity DAG mutation tests

**Status:** normative supplement to [`TEST_MATRIX.md`](TEST_MATRIX.md).

The semantic context pipeline is a directed acyclic identity graph:

```text
ContextInputSnapshotId / ContextRequestId
-> ContextPlanId / ContextFrontierId
-> ProjectMapId / SkeletonId / ControlEffectNodeId / SourceExcerptId / evidence-loss IDs
-> ContextBundleCoreId
-> ContextRendererArtifactId
-> ContextMetricsId
-> ContextEvaluationReportId
-> ContextBundleEnvelopeId
```

An earlier artifact may not include, hash, or require a later artifact ID. A later artifact may reference earlier immutable IDs.

| ID | Mutation or case | Expected |
|---|---|---|
| `CTX-DAG-001` | `ContextBundleCore` contains renderer, tokenizer, metrics, evaluation, or envelope IDs | reject `context_artifact_identity_cycle_forbidden` |
| `CTX-DAG-002` | `ContextBundleCoreId` hashes final Markdown/JSON bytes or exact token count | reject; semantic core must remain renderer-independent |
| `CTX-DAG-003` | `ContextRendererArtifactId` feeds back into source excerpt, skeleton, Project Map, plan, or input identity | reject |
| `CTX-DAG-004` | Metrics or evaluation changes alter `ContextBundleCoreId` | reject |
| `CTX-DAG-005` | Timing, memory, model score, worker count, path, row ID, or lease enters a semantic ID | reject `context_volatile_field_in_canonical_identity` |
| `CTX-DAG-006` | Same semantic core rendered by JSON, Markdown, and compact-line profiles | same `ContextBundleCoreId`, distinct renderer IDs |
| `CTX-DAG-007` | Same renderer bytes counted by two exact tokenizer profiles | same renderer ID, distinct tokenizer result IDs |
| `CTX-DAG-008` | Evaluation report references the outer envelope that already references the report | reject cycle |
| `CTX-DAG-009` | Outer envelope references core, renderer, metrics, and evaluation in canonical order | pass |
| `CTX-DAG-010` | Bundle validation repairs a missing later artifact by mutating the semantic core | reject; validation is nonrepairing |
| `CTX-DAG-011` | Renderer omits a mandatory core record and attempts to compensate in metrics/evaluation | reject renderer semantic mismatch/hard-gate failure |
| `CTX-DAG-012` | Rebuild under shuffled input, query completion, and 1/2/N workers | identical semantic DAG IDs and canonical core bytes |

## Required graph validation

The implementation test suite must construct the artifact-reference graph from frozen schemas and assert:

- all reference directions match the declared layer order;
- no self-edge or cycle exists;
- every referenced earlier artifact exists and its digest validates;
- later artifact absence can make an envelope/profile incomplete but cannot invalidate or rewrite an otherwise valid semantic core;
- semantic-core equality is evaluated before renderer/tokenizer/metric/evaluation equality;
- renderer/tokenizer/evaluation comparison never upgrades context or domain authority.

## Freeze gate

The closed `context-bundle.json` fixture must freeze valid and invalid DAG vectors, expected error codes, canonical bytes, and IDs before the first E3-A Rust commit.
