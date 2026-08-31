# E3-B semantic pack, rendering, and cache identity

**Status:** normative.

## Artifact layers

```text
ContextSemanticPack
    canonical selected context and trust metadata

RenderedContextArtifact
    deterministic presentation of one exact semantic pack

ContextCacheKey
    exact reusable-artifact identity; no physical cache storage
```

The semantic pack is the canonical E3-B result. Rendered text cannot introduce facts, change authority, hide required state, or become a different truth source.

## Semantic pack sections

A canonical pack contains ordered typed sections:

```text
PackIdentity
UniverseAndCompatibility
RequestAndProfiles
BoundaryNotices
ProjectMapReferences
L0SkeletonReferences
L1SkeletonReferences
SelectedFacts
SelectedRelationsAndReasonPaths
ReferenceFacts
SourceHandlesAndExcerpts
ExistingFindingEvidence
ConflictsAndCoverage
Omissions
SelectionTrace
BudgetAndTokenAccounting
ContinuationAndStopState
Validation
```

Optional empty presentation sections may be omitted by a renderer profile; mandatory semantic records remain in the pack.

## Context items

Every item has:

- stable item ID, kind, and schema;
- exact universe, generation, and scope;
- typed payload;
- origin fact, assertion, reference, and source IDs;
- evidence, provenance, confidence, coverage, and conflict references;
- derivation template/rule ID when derived;
- root and inclusion reasons;
- privacy, license, and source-boundary class;
- canonical payload digest.

E3-B has no canonical generic free-form summary item.

## Canonical JSON

The canonical JSON representation is lossless for semantic fields:

- UTF-8;
- frozen field ordering and canonicalization;
- arrays already in stable semantic order;
- canonical number, string, boolean, and null representation;
- source strings structurally escaped;
- no timestamps, host paths, process IDs, cache state, database physical state, or incidental metrics in semantic identity;
- exact byte digest and length.

A parser round trip reconstructs the same semantic pack and ID.

## Deterministic Markdown

Markdown is a mechanically checkable projection:

- section order and labels come from a frozen renderer/template catalog;
- facts are emitted from typed templates with exact item IDs and origins;
- no free-form generated prose;
- confidence, provenance, coverage, conflicts, omissions, and budget limits remain explicit;
- reason paths render as paths rather than direct-edge statements;
- source data uses the structural line-record boundary from `SOURCE_BOUNDARIES_PRIVACY_AND_SECURITY.md`;
- encoding, line endings, whitespace, escaping, and templates are frozen;
- a renderer trace maps output ranges or lines to semantic item and template IDs.

## Typed framework templates

Permitted templates include shapes such as:

```text
ENTITY <label> [<kind>] id=<id> source=<source-handle>
RELATION <source> --<kind>/<confidence>--> <target> assertion=<id>
PATH <root> => <target> via <ordered-relation-ids>
COVERAGE <capability>/<partition>: <state>
OMITTED <scope>: <reason> affects=<facet>
```

Inputs are typed values. Source text cannot modify templates or labels.

## Rendering loss

Canonical JSON is lossless. A compact Markdown profile may omit low-level presentation fields only when:

- the semantic pack remains the authority;
- the renderer profile explicitly declares the omission;
- trust-critical identity, origins, evidence, coverage, conflicts, omissions, budgets, and source boundaries remain represented or exact-linked;
- a `RenderingLossRecord` identifies fields and items;
- validation proves no statement becomes stronger, ambiguous, or misleading.

Default Markdown preserves all mandatory trust metadata.

## Renderer budgets

Before rendering:

- calculate deterministic template, escaping, framing, and boundary overhead;
- validate exact or safe-upper-bound budget;
- do not alter semantic selection without an explicit distinct semantic request/profile.

After rendering:

- measure exact bytes;
- run the exact tokenizer or declared estimator/bound profile;
- compare hard limits;
- validate item-to-output mapping and source boundaries;
- fail or execute explicit deterministic replanning; never clip raw output.

## Multiple renderings

One semantic pack may produce canonical JSON and deterministic Markdown. Future formats require reviewed profiles. Rendered artifact IDs differ by renderer, schema, encoding, token/framing profile, and exact bytes. The semantic pack ID remains unchanged only when semantic content is unchanged.

## Existing findings

When an existing finding is an exact root, the pack may include its original evidence, coverage, and remediation metadata. Rendering does not rerun the rule, change severity, or claim validity for another generation.

## Cacheable artifacts

```text
ProjectMap
L0Skeleton
L1Skeleton
ContextExpansionPlan
ContextSemanticPack
RenderedContextArtifact
```

Each is immutable under its exact identity.

## Cache key closure

A cache key binds every input that can change output:

```text
context schema and implementation contract profile
exact ContextUniverseSet and owner generations
exact roots and normalized request
map, L0, L1, intent, expansion, and selection profiles
confidence, provenance, and coverage policy
budget profile and request limits
tokenizer/estimator implementation, version, config, vocabulary, and framing digest
source excerpt, privacy, license, consumer trust, and boundary profiles
renderer, template, canonicalization, encoding, and line-ending profiles
continuation chain/page identity where applicable
```

Clock, host, process, thread, local path, cache hit count, SQLite row/page order, WAL/checkpoint state, and network state are excluded.

## Cache validation

A cache hit is accepted only when:

- exact key matches;
- artifact schema/profile is supported;
- digest and length verify;
- all generation/view/profile identities match;
- origins resolve when the validation profile requires it;
- privacy/license/consumer and tokenizer/renderer profiles match;
- partial, cancelled, failed, or truncated state is not presented as complete;
- continuation chain is exact.

If validation cannot be completed, return a miss or `NotEvaluated`, not trust by filename.

## No floating cache entries

Forbidden keys include:

```text
current
latest
project path alone
repository or branch name alone
symbol/display name alone
model name alone
budget number without profile
renderer format without version
```

## Cross-generation reuse

An older map, skeleton, or pack is never returned as a new generation. An immutable content-addressed subrecord may be reused only when semantic payload and all origin/profile dependencies are identical; it retains its original identity and receives a new exact generation-bound wrapper or reference where the active contract permits.

No same-name or same-text shortcut.

## Partial and cancelled cache entries

A partial pack may be cacheable only under an explicit profile and retains exact partial, omission, stop, and continuation state. It cannot satisfy a complete request or a different optional-universe/privacy policy.

Failed or cancelled intermediate artifacts are not reusable as success.

## Physical cache boundary

A higher layer may provide a narrow storage adapter:

```text
get(ContextCacheKey) -> immutable bytes or object handle
put_if_absent(ContextCacheKey, validated artifact)
retain or release under external policy
```

`wow-context` validates returned artifacts but does not accept arbitrary callbacks, SQL, file paths, or mutable cache objects in semantic operations.

## Corruption and mismatch

- digest, schema, or profile mismatch: reject/miss;
- incomplete bytes/object: reject;
- wrong generation, privacy, tokenizer, or renderer: reject;
- unavailable required origin evidence: miss or validation failure by profile;
- never repair by changing content under the same ID;
- recompute from exact inputs when allowed.

## Rebuild comparison

Run with 1, 2, and N workers; shuffled owner batches; cold/warm higher-layer cache; different temporary roots and database physical layout; equivalent reordered request JSON; and duplicate evidence/text.

Compare exact normalized requests, map/skeleton IDs and canonical bytes, selected/omitted sets, traces, budgets, semantic pack ID/bytes, rendered artifact ID/bytes/token records, continuation chain, and canonical validation reports.

## Nonclaims

A pack or rendering does not assert that:

- selected context is sufficient for every future task;
- omitted context is globally irrelevant;
- source comments are true;
- possible relations are proven;
- static structure is runtime behavior;
- downstream consumers cannot mishandle untrusted source;
- token estimates equal provider billing without exact framing/tokenizer;
- the pack is a fix, plan, or proof of completion.
