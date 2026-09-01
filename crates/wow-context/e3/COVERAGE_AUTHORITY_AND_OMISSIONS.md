# E3-B coverage, authority, conflicts, and omissions

**Status:** normative.

## Authority preservation

E3-B consumes and preserves existing authority classes:

```text
user project source
Blizzard UI implementation source
Reference Pack platform contract
analyzer fact/diagnostic
recognizer-derived structural relation
graph assertion/path/conflict
existing rule finding/evidence
external/runtime/history: deferred unless a later exact universe profile activates them
```

Context selection, grouping, templating, rendering, counting, and repetition do not upgrade any authority class.

## Claim origin closure

Every canonical fact/facet/relation/path/reference item identifies:

- exact bound universe and generation;
- owning input record ID and schema/profile;
- source handle/span where applicable;
- graph assertion/path and producer partition where applicable;
- ReferenceView entity/fact/correction/coverage where applicable;
- evidence and derivation inputs;
- provenance/confidence;
- coverage/conflicts/NotEvaluated blockers.

A typed presentation template is not evidence. It points to the exact items it formats.

## Coverage dimensions

Keep independent:

```text
universe compatibility coverage
primary project/source input coverage
Blizzard UI source input coverage
ReferenceView coverage
GraphView query/producer coverage
Project Map projection coverage
L0 projection coverage
L1 facet/neighborhood coverage
expansion-stage/query coverage
selection/enumeration coverage
budget/pruning coverage
source excerpt/privacy/license coverage
semantic-pack validation coverage
renderer field/item coverage
token accounting coverage
continuation/page-chain coverage
```

No single `complete=true` collapses these dimensions.

## Request-relative completeness

A pack can be `CompleteForRequest` only when:

- all request-required universes/capabilities are bound and compatible;
- mandatory map/L0/L1/facet operations completed;
- every required input partition is complete enough for the requested claim class;
- no unresolved conflict blocks a required facet;
- no required candidate was unenumerated, budget-pruned, privacy/license denied, unsupported, failed, truncated, or cancelled;
- semantic pack validation passes;
- each requested renderer/token gate passes independently;
- omissions do not affect required facets.

This is not whole-project, whole-graph, runtime, or future-task completeness.

## Partial pack

A profile may allow an explicit partial pack when optional or declared-degradable inputs are unavailable. The pack states:

- exact missing/partial capability/universe/partition;
- why it is missing;
- which requested facets/items are affected;
- whether source/reference/graph authority is unavailable;
- omissions and stop state;
- what exact additional input/continuation could improve coverage.

It never substitutes older or cross-profile data unless the request explicitly binds it as a separate universe.

## Conflicts

Context preserves conflicts rather than selecting a winner. Examples:

```text
same semantic field has incompatible source assertions
multiple graph producer claims violate exclusivity/multiplicity
source and ReferenceProfile compatibility mismatch
multiple exact template/mixin targets
coverage reports disagree
privacy/license policies conflict
renderer/template cannot represent a required semantic distinction
```

Conflict items include exact competing records, affected facets/capabilities, resolution state, and selection impact.

A renderer may compact conflict presentation but cannot hide one side or state an unreviewed resolution.

## Confidence

Preserve canonical confidence:

```text
Proven
Derived
Possible
Candidate (future universes only)
```

A `Possible` call/event/template/state relation remains possible in L0/L1/map/pack/rendering. Multiple possible sources do not create proof. Source-class `Proven` remains source evidence, not platform contract/runtime proof.

## Negative claims

E3-B does not manufacture negative authority. It can include an exact negative decision already supplied by owner contracts when:

- exact scope/capability/profile is known;
- relevant source/reference/graph coverage is complete;
- no conflict/truncation/budget omission affects it;
- the owner record explicitly supports negative authority.

Otherwise render one of:

```text
not found under partial coverage
not evaluated
conflict
budget/query truncated
no new evidence in the requested frontier
```

An omitted or unselected item is never evidence of absence.

## Omission records

Every known candidate not selected receives an exact `ContextOmissionRecord` when its absence could matter to interpretation or completeness.

Reasons include:

- profile/intent excluded;
- confidence/provenance excluded;
- optional universe absent;
- unsupported capability;
- upstream partial/failed/truncated/conflicted;
- privacy/license denied;
- duplicate semantic item covered elsewhere;
- budget pruned;
- renderer limit/loss;
- cancellation before selection;
- deferred future universe/feature.

For `DuplicateCovered`, record the selected equivalent item and all retained inclusion/evidence reasons.

## Unenumerated versus pruned

Distinguish:

```text
EnumeratedAndSelected
EnumeratedAndOmitted
NotEnumeratedBecauseQueryBound
NotAvailableFromInput
UnknownBecauseCapabilityNotEvaluated
```

A count of budget-pruned enumerated candidates is not the same as an upstream graph frontier never queried.

## Counts and summaries

Any count/group summary states:

- selected count;
- total known in the queried/closed scope where available;
- input/query/projection coverage;
- omitted/unresolved count/state;
- exact member manifest or continuation when required.

Do not render a partial count as “all functions/events/etc.”

## Existing findings

A finding can be included only for its exact project/profile/generation. Context preserves:

- rule/provider/version;
- severity/remediation tier already owned by the finding;
- primary project source and platform/reference evidence;
- coverage/NotEvaluated state;
- superseded/stale identity if explicitly recorded.

Context does not revalidate or update it.

## Summary facets

Canonical compact summaries are structured facets generated from exact fields and reviewed templates, for example:

```text
role = service (Derived, recognizer rule X)
registers native events = [E1,E2] (exact relation IDs)
reads state root = R (Possible path P)
uses API namespace = N (exact resolved ReferenceEntity IDs)
```

Free-form “this module is responsible for…” text is not canonical unless quoted as source documentation and labeled untrusted.

## Validation

- every claim has complete origin closure;
- no authority/confidence upgrade;
- all relevant conflicts preserved;
- every omission reason and completeness impact valid;
- selected/omitted/unenumerated counts reconcile;
- negative decisions come from owner records;
- partial packs cannot render as complete;
- no-new-evidence distinguished from absence;
- renderings retain mandatory coverage/conflict/omission labels.
