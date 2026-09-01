# E3-C context operation contracts

**Status:** normative transport-independent service behavior.

## Common request processing

Every operation follows the applicable subset of:

```text
validate service configuration
-> validate and canonicalize service request
-> resolve configured profile aliases to exact IDs
-> resolve symbolic publication selectors once
-> acquire exact retained owner views
-> validate capabilities and compatibility
-> bind/validate ContextUniverseSet when required
-> construct exact wow-context request
-> invoke one declared context operation plan
-> validate returned semantic/render artifacts
-> derive service status and draft envelope
-> close resources in reverse order
-> finalize canonical result or failure/cancellation
```

No operation refreshes current, searches for roots, retries against another generation, or invokes undeclared lanes.

## `context_status`

Purpose: report whether configured selectors, owners, contracts, profiles, and capabilities can support E3-C operations without claiming that context generation or evaluation passed.

Request:

```text
primary project selector
optional platform selector
optional exact reference guard
requested detail = summary | capabilities | profiles | generations
budgets/cancellation
```

Behavior:

1. resolve selectors once and acquire bounded metadata/read views;
2. validate exact project/graph/reference compatibility where available;
3. report current/exact publication identities, E3-A/E3-B contract availability, profile registries, owner-port catalogs, capability/coverage/conflict state, continuation retention support, and deferred operations;
4. close all resources;
5. return `ContextStatusPayload`.

It does not build a Project Map, skeleton, semantic pack, source excerpt, or rendered artifact. `available` does not mean a later operation will be complete for every root/profile.

## `context_map`

Purpose: return a deterministic E3-B `ProjectMap` for the exact acquired primary project, optional Blizzard UI project, or explicit combined-map profile.

Request:

```text
publication selectors and reference guard
map target = Primary | Platform | Combined
exact ProjectMapProfileId or configured alias
optional exact root scope accepted by the map profile
confidence/coverage/privacy policy IDs
budget/cancellation
```

Behavior:

- bind one exact `ContextUniverseSet`;
- derive the exact project root only from the acquired view when no narrower exact map root is supplied;
- call `wow-context build_project_map`;
- validate map identity/origins/coverage/omissions/budget;
- return the unchanged `ProjectMap` in `ContextMapPayload`.

No direct Markdown rendering is invented by service. Human text projection belongs to the application and must use only service/map records. A full context renderer is requested through `context_build` or `context_render`.

## `context_inspect`

Purpose: build exact L0 and/or L1 structural detail for already-resolved roots.

Request:

```text
publication selectors/reference guard
one or more exact ContextRootSelector values
level = L0 | L1 | L0AndL1
exact L0/L1/intent/expansion profiles
requested facets/axes/relations
source-excerpt candidate policy
budget/cancellation
```

Behavior:

- reject fuzzy/name/path/natural-language selectors;
- bind exact universes;
- call `build_l0_skeleton` and/or `build_l1_skeleton` in frozen order;
- optionally invoke bounded expansion only when declared by the inspect profile;
- validate roots, source spans, relation/path distinction, evidence/coverage/conflicts/omissions;
- return unchanged skeleton artifacts and optional continuation.

Inspect does not construct a full `ContextSemanticPack` unless the request explicitly selects the build operation instead.

## `context_build`

Purpose: build one E3-B `ContextSemanticPack` and optionally one or more exact rendered artifacts.

Request:

```text
publication selectors/reference guard
exact root selectors
exact intent/map/L0/L1/expansion/selection/budget/tokenizer/privacy/source-boundary profiles
zero or more exact renderer profile IDs
continuation retention policy
budget/cancellation
```

Behavior:

1. bind exact universe set;
2. validate E3-B profiles/request;
3. invoke map/skeleton/expansion/source/selection/pack operations through the single context-engine operation contract;
4. validate the semantic pack;
5. invoke requested context renderers only after semantic validation;
6. validate each rendered artifact;
7. if continuation exists, admit exact generation-retention roots before advertising it;
8. close ordinary resources;
9. return `ContextBuildPayload`.

Renderer failure cannot mutate or relabel the semantic pack. Policy decides whether a validated semantic pack with a failed optional renderer yields `partial` or service failure; the exact decision is frozen per renderer request class.

## `context_continue`

Purpose: continue one previously returned exact E3-B frontier/page chain.

Request:

```text
bounded ServiceContextContinuation bytes/object
expected original request/pack IDs: optional exact guard
no publication current selector
no changed semantic profiles/roots/privacy/renderer/total budget
cancellation
```

Behavior:

- validate continuation schema/digest/size;
- resolve exact retained generations from the continuation and retention receipts;
- acquire exact views directly, never current;
- validate the same `ContextUniverseSet`, profiles, roots, frontier, selected/omitted manifests, and total budget chain;
- call `continue_context_semantic_pack`;
- validate new page/pack and requested original renderers;
- replace/release continuation retention receipts according to returned state;
- close resources and return exact continuation outcome.

If a generation or receipt is unavailable, fail. No automatic restart against current or reconstruction from names.

## `context_validate`

Purpose: nonrepairing validation of one bounded canonical context semantic pack or rendered artifact supplied as transport data.

Request:

```text
artifact kind/media/schema
bounded artifact bytes or typed value
validation profile
origin-closure level = StructuralOnly | ExactOwnerClosure
optional exact publication guards
budget/cancellation
```

Behavior:

- service receives bytes/value, not the input path/stdin identity;
- call the owning `wow-context` parser/validator contract;
- for `ExactOwnerClosure`, acquire the exact generations encoded in the artifact and validate origins; never current;
- return `ContextValidationPayload` with `Valid`, `Invalid`, or `NotEvaluated` plus all validation records;
- never rewrite/regenerate the artifact.

An invalid artifact is a successfully completed validation operation with an `Invalid` payload, not an internal service failure. Parser/owner/invariant failures use typed failure.

## `context_render`

Purpose: render one already validated semantic pack under one exact renderer profile without changing semantic selection.

Request:

```text
bounded canonical ContextSemanticPack bytes/value
exact RendererProfileId
exact tokenizer/framing/output policy
validation/origin-closure level
budget/cancellation
```

Behavior:

1. validate semantic pack structurally;
2. acquire exact owner views only when the validation profile requires origin closure;
3. validate the semantic pack against those exact origins;
4. call the exact `wow-context` renderer;
5. validate artifact bytes, item mappings, source boundaries, privacy/license, and token accounting;
6. close resources;
7. return `ContextRenderPayload`.

No renderer-specific semantic replan occurs unless the input is a new explicit `context_build` request with a distinct semantic profile and pack ID.

## Common outcome rules

- useful artifact + budget stop = `truncated` with continuation/omissions;
- useful artifact + nonbudget incomplete requested scope = `partial`;
- no useful artifact because required capability was legitimately unevaluable = `not_evaluated` only when the operation contract permits;
- invalid request, missing exact generation, incompatible bindings, security violation, invalid owner result, or close failure = `failed` result family;
- cancellation before publication = `cancelled` result family;
- no empty/default success;
- no source, lease, continuation internals, or private path in errors.
