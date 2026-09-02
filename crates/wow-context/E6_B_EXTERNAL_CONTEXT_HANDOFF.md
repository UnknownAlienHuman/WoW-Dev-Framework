# E6-B exact mapped-root context handoff seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-context` continues to accept exact owner roots only. E6-B does not add external-provider semantics to the context crate.

## Input

The service supplies the normal E3-B inputs:

```text
exact ContextUniverseSet
exact project/reference/graph generations
one exact root already validated by a project/reference mapping owner
one exact context request and profile
budgets, privacy/license policy, cancellation
```

The service may retain an external result/mapping/selection sidecar, but that sidecar is not passed as semantic facts, ranking hints, expansion seeds beyond the exact mapped root, or rendering instructions.

## Operations

Existing E3-B operations are reused when their exact root/profile contracts permit:

```text
context_map
context_inspect
context_build
context_continue
context_validate
context_render
```

No E6-specific parser, mapper, ranker, model, or external-provider dependency is added.

## Root discipline

- The mapped root must belong to the acquired exact universe.
- `wow-context` does not verify the external locator or selection receipt; service validates those preconditions.
- No parent/nearest/same-name/current/search fallback.
- Unsupported root kinds return `NotEvaluated` or a typed invalid request.
- Continuation binds the same exact root and universe.

## Evidence separation

`ContextSemanticPack` contains only normal exact project/reference/graph evidence. It does not contain provider score/rank, summary, generic trace, unverified locator fields, or claims that mapping/selection verified external semantics.

A rendered artifact may display an externally supplied sidecar only outside `wow-context`, in the E6-B service/application composition layer.

## Dependency boundary

`wow-context` retains its existing direct dependencies:

```text
wow-core
wow-project
wow-reference
wow-graph
```

It does not depend on `wow-cbm`, `wow-service`, `wow-store`, or applications.

## Tests

Add exact-root ownership mismatch, unsupported mapped root, provider-metadata injection, continuation root substitution, privacy/source limits, cancellation, and deterministic 1/2/N worker cases when E6-B implementation begins.