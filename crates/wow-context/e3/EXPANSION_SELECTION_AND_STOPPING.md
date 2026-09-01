# E3-B expansion, selection, deduplication, and stopping

**Status:** normative deterministic context-construction algorithm.

## Pipeline

```text
bind and validate ContextUniverseSet
-> normalize exact ContextRequest
-> load/build required Project Map slice(s)
-> build required L0 skeleton(s)
-> build required L1 root skeleton(s)
-> enumerate profile-declared expansion frontier
-> execute bounded project/graph/reference reads stage by stage
-> create ContextCandidateItems with exact dependencies and costs
-> deduplicate by semantic identity while retaining all inclusion reasons/evidence
-> select mandatory closure
-> select optional items by profile tiers and stable ties
-> fetch approved exact source excerpts for selected candidates only
-> re-account exact semantic/render costs
-> prune/replan optional items if the explicit profile permits
-> emit omissions, trace, continuation, coverage, conflicts, and stop state
-> canonicalize and validate ContextSemanticPack
```

## Expansion stages

A profile may activate ordered stages such as:

```text
0 BoundaryAndIdentity
1 MapAndContainerContext
2 RootEntityFacts
3 DirectOwnershipAndLoad
4 IntentSpecificDirectRelations
5 ExactReferenceEnrichment
6 BoundedReasonPaths
7 SelectedNeighborL1
8 SourceExcerptCandidates
9 ApprovedSourceExcerpts
10 EvidenceConflictCoverageClosure
```

Stage IDs/order are versioned. A stage cannot execute before its declared prerequisites.

## Candidate item construction

Each candidate records:

- exact semantic kind and origin IDs;
- root/inclusion reason(s);
- required dependencies;
- confidence/provenance/coverage/conflict/privacy/license classes;
- selection tier and stable tie key;
- canonical semantic byte cost;
- renderer/token cost records where available;
- whether it is mandatory, optional, or forbidden;
- expansion frontier it may unlock.

Source text is not inspected to calculate semantic relevance.

## Mandatory closure

Always include, as applicable:

```text
ContextUniverseSet and compatibility state
normalized request and profile IDs
exact root identity records
source/framework boundary notices
minimum Project Map/L0/L1 identity needed to interpret roots
origin/evidence/derivation records for included claims
coverage/conflict/NotEvaluated state
selection trace summary and all omission records needed for honesty
budget/token accounting and truncation/continuation state
```

Profile-required root signatures/source spans/direct relations are also mandatory.

If mandatory closure exceeds any hard semantic/output limit, fail `context_minimum_required_content_exceeds_budget`. Do not remove evidence or boundary metadata.

## Optional priority tiers

A typical profile may use:

```text
Tier 0 mandatory closure
Tier 1 exact root facts and direct owner/source/load context
Tier 2 intent-specific direct relations and exact ReferenceView facts
Tier 3 bounded reason paths and direct neighboring L1 records
Tier 4 source documentation and exact source excerpts
Tier 5 wider siblings/container context
Tier 6 additional possible/conflicted/deferred details when explicitly requested
```

The profile defines actual tiers. Same-tier ordering uses immutable stable keys only.

## Stable tie-breaking

Tie keys may include:

```text
item class order
root order from normalized request
stage order
relation/axis kind order
source/load ordinal
canonical entity/relation/source key
confidence/provenance order defined by profile
context item ID
```

Forbidden tie-breakers:

- SQL row or insertion order;
- hash iteration;
- worker/query completion time;
- source popularity/star count;
- repository/addon/provider names as importance;
- file length unless explicitly a cost, not relevance;
- model/embedding/LLM score;
- wall-clock recency unless it is an exact explicit request field in a future profile.

## Dependencies

Selecting an item includes its required interpretation/authority closure or omits the dependent item. Examples:

- relation item requires source/target identities and underlying assertion/path refs;
- derived facet requires derivation rule/template and input claim refs;
- source excerpt requires source handle/digest/range/privacy/license/boundary records;
- ReferenceFact requires exact reference entity/profile/coverage;
- existing finding evidence requires finding identity and its project/reference evidence.

Dependencies are acyclic and validated. Cyclic candidate dependencies are a contract error.

## Deduplication

Deduplicate only identical semantic items under the same universe/profile/generation and canonical payload. Preserve:

- every root/inclusion reason;
- every supporting evidence/assertion ID;
- every affected facet/section;
- original confidence/provenance/coverage/conflict distinctions.

Do not deduplicate:

- same-name entities in different universes/generations;
- direct relation and reason path;
- `Proven`, `Derived`, `Possible`, conflicted alternatives;
- source quote and framework fact;
- project source declaration and ReferenceView contract entity;
- two distinct source ranges with equal text.

## Progressive rounds

Each expansion round has a frozen maximum frontier, operations, depth, item count, bytes/tokens, and wall/CPU budget. The planner records:

```text
frontier before
queries issued
records received
new semantic item IDs
new evidence/source/reference IDs
new conflicts/coverage records
cost added
frontier after
stop decision
```

No asynchronous/background continuation after return.

## No-new-evidence stop

Stop with `NoNewEvidence` only when the completed round adds none of:

- unseen required semantic item;
- unseen exact evidence/source/reference record supporting the requested facets;
- unseen conflict/coverage/omission state necessary for correctness;
- requested source excerpt bytes/ranges;
- unresolved mandatory dependency.

Repeated text, duplicate evidence, already-seen path, or an item pruned by unchanged budget does not count as new evidence.

`NoNewEvidence` is not an authoritative negative claim about runtime, other universes, or unrequested axes.

## Other stop states

```text
CompleteForRequest
NoNewEvidence
HardBudgetReached
StageDepthOrFanoutReached
InputCapabilityUnavailable
ConflictBlocked
PrivacyOrLicenseBlocked
RendererBudgetBlocked
Cancelled
Failed
ContinuationAvailable
```

The pack reports the exact state and completeness impact.

## Continuation

Continuation cursor binds:

- exact universe set and input views;
- normalized request and all profiles;
- plan/stage/frontier IDs;
- selected and omitted semantic item manifests;
- last stable ordering keys;
- used/remaining budgets where supported;
- integrity digest.

A cursor against another generation, request, profile, budget, tokenizer, renderer, or privacy policy is rejected.

Continuation never hides earlier truncation/omissions. Combined pages validate to one exact continuation chain.

## Cancellation

Check before and during every owner query, candidate expansion loop, source excerpt fetch, tokenization, pruning pass, canonicalization, and rendering. On cancellation:

- close/return owner views and leases;
- do not continue in background;
- do not write a cache entry as complete;
- return typed cancellation or an explicitly allowed partial semantic pack with cancellation state;
- preserve exact already-selected evidence/omissions if returning partial;
- never claim `CompleteForRequest`.

## Selection validation

- all selected candidates satisfy profile/authority/privacy rules;
- all dependencies selected and acyclic;
- all excluded candidates have deterministic omission decisions;
- no duplicate semantic item loses inclusion/evidence reasons;
- selected costs reconcile with budget reports;
- mandatory closure present;
- stable selection under randomized candidate/query order;
- stop/no-new-evidence condition reproducible.
