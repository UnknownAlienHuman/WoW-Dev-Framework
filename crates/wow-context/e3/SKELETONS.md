# E3-A L0 and L1 skeleton contract

**Status:** normative semantic skeleton projection.

## Principle

A skeleton exposes exact structure needed for navigation and reasoning while preserving evidence and proof limits. It is not generated code, reconstructed source, or a behavioral summary invented from names.

## Two-stage pipeline

```text
exact project/graph/reference records
-> consumer-neutral SkeletonRecord graph
-> renderer-specific JSON/Markdown/compact text
```

Semantic skeleton IDs are independent of renderer paths, lines, whitespace, wrapping, and output format.

## Supported subject families

Initial E3-A profiles may support:

```text
project/package/TOC variant/load unit
physical file/XML document/virtual Lua unit
namespace/module/service/library/registry/state root
function/method/callback/event/API symbol
XML template/frame/region/mixin/factory
state path/source span
```

An unsupported kind returns `Unsupported`/loss, not a guessed generic skeleton.

## L0 fields

L0 answers “what is this and where does it sit?”

Required where available:

```text
exact entity key/kind/universe/generation
stable logical/display name from source records
project/package/file/source-unit ownership
selected load unit/phase/direct load position
accepted universal roles with producer/confidence
public structural surface headings and counts
important direct relations by allowed lanes
source/evidence handles
coverage/conflict/ambiguity/truncation state
next-detail routes
```

L0 excludes:

```text
function bodies
full signatures/member lists when over L0 profile
transitive call/load/ownership expansion
full source excerpts
behavior/purpose prose inferred from names
runtime/taint/Secret/performance claims
```

## L1 fields

L1 answers “what exact direct structure is needed for this task?”

Depending on subject/profile:

```text
exact callable signature, parameters, returns, receiver, source position
selected member/field/enum/event payload records
selected direct calls and API uses
native/custom/CVar signal registration and producer links
hook/script/factory/mixin/inheritance relations
state root/path reads and writes
load/ownership/lifecycle direct chains and bounded reason paths
source-backed declaration, branch/guard, call, registration, and state-access skeleton nodes
coverage/conflict/ambiguity/evidence for every included detail
bounded source excerpt routes or excerpts when explicitly requested
```

L1 excludes unrelated body details and full project neighborhoods.

## Skeleton headings and labels

Heading/role vocabulary is frozen in `SkeletonProfile`. Labels are derived from exact kinds/relations, for example:

```text
Package
Selected TOC variant
Load unit
Module role
Native frame event registration
Custom registry producer/subscriber
CVar callback
Secure post-hook structure
SavedVariables state root
Direct platform API use
```

Do not let source documentation create headings or rule labels.

## Signatures

Callable skeletons preserve exact available:

```text
owner/receiver
parameter/member order and names
types/optionality/nilability/variadic/default state
multiple return positions
deprecation/availability/restriction refs
source and analyzer fact IDs
coverage/conflicts
```

Unknown/partial type/member state remains explicit. Do not widen to `any` or omit silently.

## Relations

Skeleton relation summaries name:

```text
lane and relation kind
direction
exact source/target entity keys
confidence/provenance/coverage
supporting assertion/evidence IDs
reason path when inclusion is path-derived
conflicts/ambiguity
```

A path-derived relation is displayed as a path/reachability explanation, not a direct edge.

## Source-backed structural nodes

Allowed nodes are exact normalized structures from published project/analyzer facts:

```text
declaration/signature/member
resolved/unresolved direct reference
call and result-use heading
registration/hook/script heading
state access/assignment heading
guard/branch/control-flow relation when proven
return/parameter heading
```

Rules:

- source handle/span/content digest required;
- no AST/CST/internal analyzer object leakage;
- no CFG construction in context crate;
- no operation legality or WoW rule conclusion;
- unknown/dynamic facts remain Possible/NotEvaluated;
- children ordered by exact source/semantic ordinal.

## Skeleton coverage

For each declared field family report:

```text
Exact
ExactWithEvidenceSidecar
CompactButCompleteForDeclaredFields
LossyDeclared
Unsupported
NotEvaluated
Truncated
```

A skeleton can be compact-complete only for the exact profile field set and input capabilities. It cannot claim full source/entity completeness.

## Omission and counts

When member/relation/node lists exceed budget:

- retain exact total/processed/included/omitted counts when available;
- retain a partition/ID digest or bounded omitted manifest;
- keep mandatory conflict/blocker records;
- provide deterministic continuation/detail routes;
- never display `...` without structured truncation state.

## Deduplication

Deduplicate exact equivalent presentation records only when:

- semantic subject and field values equal;
- all producer/assertion/evidence/coverage refs remain attached;
- conflict/ambiguity distinctions are preserved;
- source positions that matter are retained;
- profile defines deterministic grouping.

Similar names/messages/source snippets are not equality.

## Renderer boundary

Renderers consume validated skeleton model and can produce:

```text
canonical JSON
human Markdown
compact line-oriented text
future transport-specific projection
```

Renderers cannot add/remove semantic records, hide blockers, execute source, or change confidence. Renderer-specific loss is separate.

## Determinism

Equivalent input/request/profile yields identical:

```text
SkeletonIds
member/relation/source-node sets and order
coverage/loss/omission/detail routes
canonical semantic bytes/digest
```

The same semantic skeleton rendered differently remains the same skeleton with distinct renderer artifact IDs.

## Required tests

- every supported subject family at L0 and applicable L1;
- exact/partial/conflict/ambiguous/unsupported/truncated fields;
- signature optional/nil/multiple-return preservation;
- direct relation versus path distinction;
- native/custom/CVar/hook/state role separation;
- source-node exact span/content/generation closure;
- no analyzer internal type leakage;
- no inferred code/body/purpose;
- all material fields evidence-linked;
- deterministic grouping/dedup/ordering;
- tight-budget mandatory blocker survival;
- renderer semantic equivalence.

## Hard stops

- no source reconstruction or code generation;
- no documentation-derived semantics;
- no generic `related` or `parent` collapse;
- no confidence/coverage upgrade;
- no direct edge invented from path;
- no skeleton record without exact input/derivation closure;
- no full body/source by default;
- no runtime or fix claim.
