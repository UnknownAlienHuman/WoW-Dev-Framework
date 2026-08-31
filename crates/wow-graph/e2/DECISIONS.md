# E2-A graph decisions

**Status:** normative.

## GRAPH-001 — Assertions are primary

Entities and relations are supported by immutable producer assertions. Snapshot views never replace or erase the underlying assertion set.

## GRAPH-002 — Semantic identity is producer-independent

`EntityKey` and `RelationKey` identify graph subjects. Producer/partition/evidence belong to assertion identity.

## GRAPH-003 — Universe/profile/generation scope is explicit

Project, reference, dependency, external candidate, and runtime universes cannot collide by display name or path.

## GRAPH-004 — One graph, multiple explicit views

Ownership, load, object, inheritance, registration, lifecycle, state, call, and later lineage are axes/views over one assertion store.

## GRAPH-005 — No generic parent semantics

`parent_of` is valid only as the explicitly defined object/XML relation. It is forbidden as a universal parent edge.

## GRAPH-006 — Registries are immutable per graph generation

Kind/relation/attribute/axis definitions are versioned inputs. Untrusted projects cannot register executable or arbitrary semantics at runtime.

## GRAPH-007 — Relation schemas define direction

Every relation kind specifies endpoint kinds, direction, inverse semantics, uniqueness, multiplicity, cycle policy, evidence requirements, confidence bounds, and axis membership.

## GRAPH-008 — Attributes are schema-bound

No unrestricted JSON property bags. Attribute definitions use bounded typed values and explicit identity/query roles.

## GRAPH-009 — Producer partition replacement is atomic

A new producer version/generation replaces one exact partition; stale assertions disappear together or not at all.

## GRAPH-010 — Producers do not write graph storage directly

They submit validated batches/plans. Graph validates semantics; store executes registered physical operations.

## GRAPH-011 — Conflicts are retained

Incompatible assertions produce conflict records and dependent capability impact. No popularity or last-write-wins resolution.

## GRAPH-012 — Confidence is not upgraded

Aggregation may conservatively downgrade a view; it cannot promote `Possible` or `Candidate` to `Derived`/`Proven`.

## GRAPH-013 — Coverage remains partitioned

Complete graph storage does not imply source/recognizer completeness. Query negative claims require exact relevant coverage.

## GRAPH-014 — Snapshot publication binds exact inputs

One `GraphGenerationId` binds registry, reference/project generations, producer partition manifests, conflicts, coverage, and logical store generation.

## GRAPH-015 — Readers are immutable snapshots

A query never switches to a newer graph generation mid-request.

## GRAPH-016 — Queries are bounded and deterministic

No public unbounded traversal, whole-graph dump, or iteration-order-dependent output.

## GRAPH-017 — Paths do not become edges

Transitive reachability is a query result with reason paths, not a silently persisted direct relation.

## GRAPH-018 — Candidate assertions are opt-in

Default exact/impact views exclude candidate/external semantic assertions.

## GRAPH-019 — Cycles are relation/axis-specific

Calls/state/general relations may cycle. Load-order or hierarchy cycles can be invalid/conflicting according to the registry definition.

## GRAPH-020 — Continuation is snapshot-bound

Continuation cursors bind exact query normalization, snapshot, budgets/profile, and last stable ordering key.

## GRAPH-021 — Logical schema is graph-owned; physical lifecycle is store-owned

Graph defines records, indexes, registered operations, and validation. Store owns SQLite/WAL/transactions/migrations/recovery.

## GRAPH-022 — No second parser or raw-source fallback

Graph consumes normalized facts/assertions only. It never reopens Lua/XML/TOC to infer missing edges.

## GRAPH-023 — No model inference in correctness path

LLM/semantic suggestions may later enter an external candidate universe only; they cannot construct authoritative project/reference assertions.

## GRAPH-024 — Progressive context over bulk export

Normal consumers resolve exact roots, inspect direct relations, expand selected axes/paths, and stop at no-new-evidence. Bulk export is administrative/deferred.

## GRAPH-025 — Physical IDs are private

SQLite row IDs and insertion order never appear as public stable graph identity.
