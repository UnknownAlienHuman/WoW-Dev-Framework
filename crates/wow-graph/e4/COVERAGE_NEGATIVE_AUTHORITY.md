# E4-B coverage and negative-authority contract

**Status:** normative.

## Independent coverage dimensions

Keep separate:

```text
before owner/source inventory coverage
after owner/source inventory coverage
before/after TOC/XML/load/analyzer/recognizer/graph coverage
before/after ReferenceView and correction coverage
before/after SearchShard/document/lane coverage
project stable-identity producer coverage
fingerprint/structural-change producer coverage
Reference transition producer coverage
search candidate producer coverage
candidate blocking/pair/component coverage
review-decision coverage
lineage proposal/assertion publication coverage
change-facet/relation comparison coverage
removal/introduction negative-authority coverage
migration/replacement coverage
static-impact traversal coverage
store/read-back/index validation coverage
pagination/continuation coverage
privacy/license/security coverage
```

No aggregate `complete` value replaces these records.

## Proof ceiling from coverage

A proposal/assertion/change result cannot exceed the weakest required coverage state. Relevant states include:

```text
Complete
Partial
Failed
Cancelled
Truncated
Conflict
Unsupported
NotEvaluated
NotApplicable
```

Storage or query completion never upgrades an incomplete owner/source/reference partition.

## Lineage assertion coverage

An accepted one-to-one continuity assertion requiring exclusivity needs:

- exact before/after entity records and identity fields;
- complete producer coverage for the qualifying stable/transition evidence;
- complete component generation within the relation profile's blocking scope;
- no unenumerated competing exact-evidence entity;
- no unresolved stable-identity/multiplicity conflict;
- validated publication/read-back.

A Candidate/Possible assertion can exist under partial coverage if its limitations are explicit and the profile permits it.

## Removal authority

`RemovedAfter` requires:

```text
exact before entity and closed subject scope
complete before existence/identity coverage
complete after source/inventory/entity-kind/package/root coverage
complete applicable lineage candidate-generation coverage
complete decisive producer/graph/query coverage
no unresolved candidate component
no conflicting continuity/replacement assertion
no relevant truncation, failure, cancellation or privacy exclusion
explicit negative-authority decision under the selected profile
```

Otherwise the entity is `UnmatchedBefore`, `NotEvaluated`, `Conflict`, or `Partial`.

A missing row, exact-name miss, FTS miss, graph miss, absent source file in an incomplete root, or no search candidates is never sufficient.

## Introduction authority

`IntroducedIn` uses the symmetric requirements for the before scope. A new after entity under incomplete before inventory remains `UnmatchedAfter`/NotEvaluated.

## Moved/renamed authority

- `Renamed` needs accepted continuity and complete exact name-field coverage for both entities.
- `Moved` needs accepted continuity, complete location/ownership/source coverage, and enough old-location/after-location evidence to distinguish move from copy according to the profile.

Incomplete copy-detection/candidate coverage can cap move to Possible/Candidate.

## Split/merge authority

Requires complete enough component and producer coverage to identify all asserted participants within the declared scope, plus explicit qualifying evidence. Unbounded “all descendants/ancestors everywhere” claims are prohibited.

A split/merge assertion states only the selected scoped participant set. Omitted/unknown candidates remain explicit.

## Replacement/deprecation authority

- Exact Reference transition evidence retains its own profile/coverage/conflict rules.
- Project-owner transition review has its own scope.
- Search candidate evidence cannot provide replacement authority.
- Missing old entity plus similar new entity is not replacement.
- Deprecation without explicit target does not imply one.

## Change classification coverage

Each facet reports its own before/after coverage:

```text
Known-to-Known change
Known-to-Missing under complete after field scope
Missing-to-Known under complete before field scope
Unknown/Unsupported/Conflict/Omitted transitions
relation-set comparison closure
```

A `GenerationChangeSet` can be complete for some facets and partial for others.

## Migration coverage

A migration candidate can be emitted with missing preconditions. A validated recipe requires complete exact source/target field/restriction/transformation evidence required by its schema. Runtime/client validation requirements may remain pending, but then the recipe must not claim runtime success or release readiness.

## Static-impact coverage

Keep separate:

- root change authority;
- target GraphSnapshot coverage;
- each relation kind/direction/axis coverage;
- path enumeration/traversal budgets;
- cross-universe bridge coverage;
- endpoint lineage mapping coverage;
- selected/omitted/unenumerated paths;
- continuation state.

A complete bounded traversal is complete only for the exact requested scope/profile/budget. It is not proof of no other or runtime impact.

## Negative query results

Query output distinguishes:

```text
FoundAccepted
FoundCandidateOrPossible
NotFoundWithAuthority
NotFoundWithPartialCoverage
ConflictBlocked
NotEvaluated
Truncated
NoNewEvidence
Cancelled
Failed
```

`NoNewEvidence` means the executed bounded frontier added no unseen relevant records. It is not entity removal, introduction, no runtime effect or global absence.

## Privacy/license exclusions

If an entity/fingerprint/source transition is hidden or unavailable under the active privacy/license profile, negative authority in the affected scope is unavailable unless an owner-provided metadata-level complete negative decision remains valid. Do not infer absence from redaction.

## Review and coverage

Manual review cannot replace missing closed inventory/candidate coverage for removal/introduction or exclusivity. It can attach new exact evidence only through a recognized evidence class; otherwise the ceiling remains partial/candidate.

## Summary rules

Coverage summaries:

- cite exact source coverage records;
- preserve the worst relevant state;
- list excluded/not-applicable partitions separately;
- expose conflicts, truncation and producer failures;
- never hide an incomplete partition because another producer is complete;
- never use counts as proof of complete scope without a manifest.

## Validation

Reject or downgrade:

- removal/introduction under partial source/profile scope;
- unique-candidate promotion after pair-budget truncation;
- move while copy/old-location coverage is unknown;
- change facet under unknown before/after state;
- replacement from search-only evidence;
- impact complete claim after relation/path budget truncation;
- negative result from empty SQL/search/graph output;
- complete snapshot with unresolved producer conflict;
- current/fallback data used to fill missing exact generation coverage.
