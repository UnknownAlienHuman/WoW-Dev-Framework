# E3-A L1 control and effect skeleton model

**Status:** normative structured projection over published analyzer/project facts; not a new parser or control-flow engine.

## Goal

Represent the control/effect structure needed for navigation and engineering review without copying full bodies or inventing behavior.

```text
published source-coordinate and normalized semantic facts
+ exact subject declaration
+ L1 skeleton/control-effect profile
-> ordered structured nodes with evidence and source spans
-> explicit unknown/collapsed/omitted regions
-> no new semantic analysis
```

## Proof ceiling

E3-A may project only facts already published by the project/analyzer/graph contracts. It cannot:

- parse Lua/XML/TOC;
- build a second AST/CST/CFG/SSA/data-flow engine;
- prove dominance, aliasing, value ranges, runtime dispatch, taint, combat, Secret, or protected legality unless an owning input fact explicitly provides the exact supported relation;
- reconstruct expressions or statements from diagnostics/prose;
- infer intent from names/comments.

## Structural node kinds

Initial closed registry:

```text
Declaration
Signature
Parameter
Member
Sequence
Branch
BranchArm
Guard
Loop
EarlyReturn
Return
DirectCall
PossibleCall
CallbackRegistration
EventRegistration
HookRegistration
ScriptRegistration
ApiUse
StateRead
StateWrite
FactoryOrInstantiation
UnknownRegion
CollapsedRegion
OmittedRegion
```

Adding a kind requires an exact input fact contract, identity, ordering, proof ceiling, source mapping, renderer behavior, and mutation tests.

## Node record

```text
ControlEffectNode
    node ID/kind
    exact subject/entity/source-unit/file IDs
    exact source handle and half-open byte span
    semantic ordinal and parent/child IDs
    input analyzer/project/graph fact IDs
    normalized proven attributes from the kind schema
    related entity/relation/evidence/coverage/conflict IDs
    confidence/provenance
    projection status
    omitted/collapsed/unknown detail refs
    canonical digest
```

No free-form property bag or source-authored label can add semantics.

## Tree and graph shape

The presentation tree follows exact published containment/source order. Cross-references such as calls, state effects, event producers, hooks, and API use remain typed relation refs, not duplicated child ownership.

A control/effect skeleton is not required to be a complete CFG. If the input cannot establish nesting/order for a region, emit `UnknownRegion` or a flat source-ordered set with explicit loss rather than guess a tree.

## Identity and ordering

Node identity includes exact input snapshot, subject, kind, input fact IDs, source span, profile, and canonical attributes. It excludes renderer line numbers, row IDs, worker order, and prose.

Order by:

```text
containing source unit
half-open source span start/end
published semantic ordinal
kind registry order
semantic target key
node ID
```

When source spans overlap or are unavailable, use the owning published semantic ordering and record the limitation.

## Branches and guards

A `Branch`/`BranchArm`/`Guard` can include only exact normalized facts such as:

```text
source span and branch ordinal
condition expression fact/source handle
known literal/operator/callee/entity refs when published
access-guard relation when explicitly produced
contained direct effects and exit headings
coverage/conflicts
```

Do not paraphrase a condition into stronger prose. A Secret/readability guard shape is not proof that all uses are safe unless the exact rule/evidence contract says so.

## Loops

Represent loop kind, source span, published iterator/condition refs, contained direct effects, and exits. Dynamic iteration targets remain unresolved/possible. Do not unroll or estimate runtime counts.

## Calls

```text
DirectCall
    exact resolved target relation/fact

PossibleCall
    exact possible/dynamic target set or unresolved callsite fact
```

Keep callsite occurrence, target entity, confidence, reason/evidence, and source span distinct. Repeated callsites are not deduplicated merely because the target is the same.

## Registrations and hooks

Keep typed systems separate:

```text
native frame event
EventRegistry native frame-event bridge
custom registry producer/subscriber
CVar callback
SetScript
HookScript
hooksecurefunc/secure post-hook structure
XML script
```

The node records structural registration only. It never claims callback delivery, runtime readiness, taint safety, combat safety, protected legality, performance, or managed-object ownership.

## State effects

`StateRead`/`StateWrite` require published state root/path facts. Exact literal paths retain exact identity. Dynamic paths retain exact known prefix/possible status. SavedVariables contents are never read or rendered.

Ordering in a source skeleton is source order, not guaranteed runtime effect order across callbacks, events, asynchronous systems, or dynamic dispatch.

## API use and restrictions

`ApiUse` links the project occurrence and exact reference entity through graph/project evidence. Restriction, availability, deprecation, Secret, or migration fields are displayed only from the exact reference/rule input and keep their own generation/provenance/coverage.

The occurrence itself does not prove runtime accessibility or safe use.

## Collapsed and unknown regions

### `CollapsedRegion`

Used when exact supported structure is intentionally compacted by profile/budget. It records exact included/omitted child manifests, source span, effect summary derived only from retained child facts, and continuation/detail route.

### `UnknownRegion`

Used when input capability cannot establish requested structure. It records source span/handle when known, missing capability, coverage/conflict, and no invented children/effects.

### `OmittedRegion`

Used for deterministic budget omission of otherwise supported nodes. It records exact counts/ID digest/cutoff/continuation.

No region is represented by an unstructured `...` marker alone.

## Effect summary

A compact effect summary is a typed multiset/index over included exact nodes:

```text
calls by direct/possible
registrations by signal/hook system
state reads/writes by exact root/path
API uses by exact reference entity
returns/early exits
unknown/collapsed/omitted counts
```

It retains member node/evidence IDs. It cannot create a new relation or hide conflicts.

## Source excerpts

Nodes may offer a detail route to an exact bounded source excerpt. L1 does not require source bytes and does not synthesize code. Excerpts follow `SOURCE_EXCERPTS_AND_SECURITY.md`.

## Budget behavior

Mandatory request-selected control/effect nodes and decisive blockers reserve budget. Optional body structure is cut only at atomic node/region boundaries. A cut emits exact loss/omission/stopping/continuation state.

## Determinism

Equivalent exact input/profile/request yields identical node IDs, hierarchy/cross-refs, ordering, effect summary, unknown/collapsed/omitted records, and canonical digest under shuffled fact order and 1/2/N workers.

## Required tests

- every node kind positive and unsupported input case;
- source-ordered sequence, nested branches, loops, and early exits;
- overlapping/missing spans;
- direct versus possible callsites;
- duplicate target at distinct callsites;
- native/custom/CVar/hook/XML system separation;
- dynamic state path prefix;
- API project occurrence versus reference evidence;
- guard shape without safety overclaim;
- unknown region when CFG/containment facts unavailable;
- collapsed region with complete child/evidence sidecar;
- budget omitted region and continuation;
- no second parser/CFG/SSA/data-flow implementation;
- no source reconstruction or prose-derived intent;
- deterministic output under fact/worker order changes.

## Hard stops

- no guessed control tree;
- no generic `effect` string without typed input/evidence;
- no runtime order/safety/taint/Secret conclusion;
- no source-authored semantics;
- no deduplication across distinct occurrences;
- no hidden unknown/collapsed/omitted region;
- no full-body reproduction by default.
