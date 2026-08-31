# Published view and future service handoff

**Status:** normative project-owned read seam; `wow-service` integration remains a later explicit package.

## Open published project

```text
open_published_project_view(selector, required capabilities, budgets)
```

Sequence:

1. resolve exact head once;
2. acquire store generation lease/read handle;
3. validate head/store/project/graph coherence;
4. open ProjectView;
5. open GraphView;
6. return `PublishedProjectViewLease`.

## View contents

```text
head and coherence identities
project/source/TOC/XML/load records
Lua unit/analyzer/recognizer bindings
project capability/conflict/coverage records
GraphView exact snapshot and query operations
validation/publication reports
deferred capabilities
```

No mutable builder, store connection, raw SQL, graph writer, analyzer actor, or recognizer engine.

## Service contract expectations

A future service operation may select:

```text
ExactHead(ProjectPublicationHeadId)
CurrentPublished(ProjectId)
```

The service cannot separately select current project and current graph. It receives the exact resolved head and lease.

## Status reporting

Distinguish:

```text
current published head
last-known-good head
failed target candidate
sealed inactive target
current corruption/recovery state
retention/lease pressure
deferred E3+ capabilities
```

Store readiness is not analysis cleanliness. Static publication is not runtime validation.

## Historical reads

Exact retained head/generation can be opened for regression/evidence use. It remains bound to its original profile/reference/project/analyzer/recognizer/graph/store identities.

## Capability checks

Required capabilities are checked against exact project and graph coverage. Missing/partial/conflicted/truncated capability returns typed unavailable/NotEvaluated; no empty clean collection.

## Cancellation and close

Close releases graph/project read handles and generation lease. Cancellation does not continue work in background. Lease release is attempted deterministically and failure is reported without making the underlying generation mutable.
