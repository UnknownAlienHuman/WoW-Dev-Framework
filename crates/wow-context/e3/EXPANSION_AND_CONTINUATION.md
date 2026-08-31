# E3-B expansion, selection, stopping, and continuation

**Status:** normative deterministic context-construction algorithm.

## Pipeline

```text
bind and validate ContextUniverseSet
-> normalize exact ContextRequest
-> load or build required Project Map slices
-> build required L0 skeletons
-> build required L1 root skeletons
-> enumerate profile-declared expansion frontier
-> execute bounded project/graph/reference reads stage by stage
-> create ContextCandidateItems with exact dependencies and costs
-> deduplicate by semantic identity while retaining all reasons/evidence
-> select mandatory closure
-> select optional items by profile tiers and stable ties
-> fetch approved exact source excerpts for selected candidates only
-> re-account exact semantic/render costs
-> prune or re-plan optional items only when the explicit profile permits
-> emit omissions, trace, continuation, coverage, conflicts, and stop state
-> canonicalize and validate ContextSemanticPack
```

## Request and profiles

`ContextRequest` contains only exact roots and reviewed profile IDs. Natural-language or fuzzy input is resolved before E3-B. Unknown fields or versions are rejected.

Initial reviewed intent families can include:

```text
NavigateProject
InspectContainer
InspectEntity
TraceLoad
TraceCall
TraceSignal
TraceHook
TraceState
InspectObjectTemplateMixin
ExplainApiUse
ReviewExistingFindingEvidence
PrepareChangeContext
```

Intent changes which typed facets and axes are requested. It never changes facts. `PrepareChangeContext` supplies structure; it does not generate a plan or edit.

## Expansion profile

```text
ContextExpansionProfile
    ordered stage definitions
    allowed owner operations
    stage prerequisites
    candidate kinds
    required evidence additions
    per-stage budgets
    maximum rounds/depth/frontier
    stop and no-new-evidence policy
    dependency and dedup policy
```

The schema is closed and non-Turing-complete. No source/caller code, callbacks, SQL, shell, regex programs, plugins, model prompts, or arbitrary expressions.

## Stages

A profile may activate stages such as:

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

Stage order and IDs are versioned. A stage cannot run before declared prerequisites.

## Candidate construction

Each candidate records:

- exact semantic kind and origin IDs;
- root and inclusion reasons;
- required dependencies;
- confidence, provenance, coverage, conflict, privacy, and license classes;
- selection tier and stable tie key;
- canonical semantic byte cost;
- renderer/token cost records when available;
- mandatory, optional, or forbidden state;
- expansion frontier it may unlock.

Source text is not inspected to determine semantic relevance.

## Mandatory closure

Always include, as applicable:

```text
ContextUniverseSet and compatibility state
normalized request and profile IDs
exact root identity records
source/framework boundary notices
minimum Project Map/L0/L1 context needed to interpret roots
origin/evidence/derivation records for included claims
coverage/conflict/NotEvaluated state
selection trace summary and honesty-preserving omissions
budget/token accounting and truncation/continuation state
```

If mandatory closure exceeds a hard budget, fail `context_minimum_required_content_exceeds_budget`. Do not remove evidence, boundaries, or identity.

## Optional priority tiers

A profile may define:

```text
Tier 0 mandatory closure
Tier 1 exact root facts and direct owner/source/load context
Tier 2 intent-specific direct relations and exact ReferenceView facts
Tier 3 bounded reason paths and direct neighboring L1 records
Tier 4 source documentation and exact excerpts
Tier 5 wider siblings/container context
Tier 6 explicitly requested possible/conflicted/deferred details
```

The profile defines actual tiers and stable tie keys.

## Stable ordering

Permitted keys include:

```text
item class order
normalized request root order
stage order
relation or axis kind order
source/load ordinal
canonical entity/relation/source key
profile-defined confidence/provenance order
context item ID
```

Forbidden keys include database row/insertion order, hash iteration, worker completion, provider/repository popularity, hidden addon/framework names, model/embedding score, or wall-clock timing.

## Dependencies

Selecting an item selects its required interpretation and authority closure or omits the dependent item.

Examples:

- relation requires source/target identities plus assertion/path refs;
- derived facet requires derivation template/rule and input claims;
- source excerpt requires handle, digest, range, privacy/license/boundary records;
- ReferenceFact requires exact profile/entity/coverage;
- existing finding evidence requires finding and its project/reference evidence.

Dependencies are acyclic. Cycles are contract errors.

## Deduplication

Deduplicate only identical semantic items under the same universe/profile/generation and canonical payload. Retain all root/inclusion reasons, evidence/assertion IDs, affected sections/facets, and authority distinctions.

Do not deduplicate:

- same-name entities in different universes/generations;
- direct relation and reason path;
- Proven, Derived, Possible, or conflicting alternatives;
- source quote and framework fact;
- project source declaration and ReferenceView entity;
- distinct source ranges containing equal text.

## Expansion round trace

Each round records:

```text
frontier before
owner queries issued
records received
new semantic item IDs
new evidence/source/reference IDs
new conflict/coverage/omission state
cost added
frontier after
stop decision
```

No background continuation after return.

## No-new-evidence

Stop with `NoNewEvidence` only when a completed round adds none of:

- unseen required semantic item;
- unseen exact evidence/source/reference record supporting requested facets;
- unseen conflict/coverage/omission needed for correctness;
- requested exact source excerpt range;
- unresolved mandatory dependency.

Repeated text, duplicate evidence, already-seen path, or an unchanged-budget rejection is not new evidence.

`NoNewEvidence` is not an authoritative negative claim about unrequested axes, other universes, or runtime state.

## Stop states

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

Every result states completeness impact.

## Continuation cursor

A cursor binds:

- exact universe set and owner views;
- normalized request and all profiles;
- plan, stage, and frontier IDs;
- selected and omitted manifests;
- last stable ordering keys;
- total and used budget state;
- prior page-chain digest;
- integrity digest.

A changed generation, root, profile, budget, tokenizer, renderer, privacy policy, or cursor digest causes rejection.

Continuation never hides prior truncation or omissions. Combined pages validate against one exact page chain and one total-budget policy.

## Cancellation

Check before and during every owner query, expansion loop, source fetch, tokenization, pruning pass, canonicalization, and rendering. On cancellation:

- close or release owner views/leases under owner contracts;
- do not continue in background;
- do not cache as complete;
- return typed cancellation or an explicitly allowed partial pack;
- preserve exact selected evidence and omissions when returning partial;
- never report `CompleteForRequest`.

## Validation

- selected candidates satisfy profile/authority/privacy rules;
- all dependencies are selected and acyclic;
- every enumerated excluded candidate has a deterministic omission;
- unenumerated frontiers are distinguished from budget-pruned candidates;
- deduplication retains all reasons/evidence;
- selected costs reconcile with budget reports;
- mandatory closure is present;
- selection is stable under randomized candidate/query order;
- stop and continuation behavior is reproducible.
