# E4-B generation change classification

**Status:** normative.

## Preconditions

A typed change between two entities is classified only after an accepted lineage/replacement relation identifies the exact compared pair, except for scoped before-only/after-only records used to evaluate removal/introduction.

```text
accepted exact entity pair
+ exact before/after owner facts and relation manifests
+ change-classification profile
+ complete relevant field/relation coverage
-> typed ChangeRecord(s)
```

A search/fingerprint candidate pair can receive Candidate comparison facets for review, but those facets do not become accepted generation changes.

## Value-state model

Every compared field retains one of:

```text
Known(value)
ExplicitNull
Missing
Unknown
Unsupported
OmittedByProfile
Conflict
NotEvaluated
```

These states are never collapsed.

Example:

```text
Known(A) -> Known(B)          Changed when A != B
Missing -> Known(B)           Added only if before field coverage is complete
Known(A) -> Missing           Removed only if after field coverage is complete
Unknown -> Known(B)           NotEvaluated/Unknown, not Added
Known(A) -> Conflict          Conflict, not Changed-to-one-selected-value
ExplicitNull -> Missing       state transition, profile-defined semantic class
```

## Change classes

### `UnchangedIdentity`

The accepted lineage pair has no change in the requested/covered facets. This is request/profile scoped, not proof that the entity is byte- or behavior-identical.

### `Moved`

Requires accepted continuity plus exact source/container/location transition and evidence that the old location no longer owns the lineage entity where the profile requires move rather than copy.

### `Renamed`

Requires accepted continuity and exact before/after owner canonical name facts. Similar spelling is not enough.

### `Split`

Requires accepted `split_from` assertions and explicit qualifying evidence for multiple descendants. The ChangeRecord links all involved entities and does not invent one primary successor.

### `Merged`

Requires accepted `merged_from` assertions and explicit qualifying evidence for multiple ancestors.

### `CopiedOrExtractedCandidate`

Used when similar/derived content appears in a new entity while continuity, move, split or merge is not established. Remains Candidate/Possible.

### `SignatureChanged`

Compares exact callable/member signature records, preserving parameter order, optionality, nilability, variadics, return tuples/multiple returns, generic/type refs, defaults, restrictions and unknown/loss states.

### `TypeChanged`

Compares exact structured type definitions/usages, not rendered text alone. Widening/narrowing/compatible/incompatible classifications require a frozen type-compatibility profile.

### `RestrictionChanged`

Compares exact ReferenceView restriction/Secret/protected/deprecation facets or owner source-class facts as distinct authorities. Source implementation cannot manufacture Reference authority.

### `OwnershipChanged`

Compares exact owner/module/package/container relations. Multiple owners/conflicts remain explicit.

### `LoadRoleChanged`

Compares selected TOC/dependency/load-unit/phase/reachability roles under exact product/flavor profiles. Static load changes do not prove runtime execution/readiness.

### `RelationSetChanged`

Compares an explicit bounded relation kind/direction/axis set. Added/removed relations retain direct versus reason-path distinction, confidence, producer, evidence and coverage.

### `Deprecated`

Requires an exact deprecation record from an authorized producer. A source comment or name is not enough.

### `Replaced`

Requires accepted `replaced_by`; it is not inferred from lineage or migration candidacy.

### `Removed`

Requires a valid `GenerationAbsenceDecision(RemovedAfter)`.

### `Introduced`

Requires a valid `GenerationAbsenceDecision(IntroducedIn)`.

### `UnmatchedBefore` / `UnmatchedAfter`

An exact entity lacks an accepted counterpart under currently executed producers/blocks, but negative authority is insufficient for removal/introduction.

### `Conflict` / `NotEvaluated`

Used when evidence, coverage, compatibility, profile, budgets, producer failures or ambiguity prevents classification.

## Compound changes

A pair can have multiple independent changes:

```text
Renamed + Moved + SignatureChanged
OwnershipChanged + LoadRoleChanged
Deprecated + Replaced
```

Do not collapse them into one lossy generic `Changed`. The profile may generate a presentation summary, but canonical records remain typed and independently evidenced.

## Rename versus replacement

```text
same accepted lineage + name differs
    -> Renamed

distinct old/new entities + explicit supersession
    -> Replaced
```

Both may coexist only when independent evidence says the same lineage entity was renamed and also superseded in the comparison; neither is inferred from the other.

## Move versus copy

`Moved` requires continuity and old-location semantics. If both old and new comparable entities remain, classify as copy/extraction candidate or split/ambiguity unless exact evidence says otherwise.

## Removal versus out-of-scope

Before-only entity is not removed when:

- after source root/package/entity kind was excluded;
- source/materialization/TOC/XML/analyzer/recognizer/graph coverage is partial;
- relevant shard/candidate generation was truncated;
- a matching candidate component is unresolved;
- profile versions are incompatible;
- a producer failed/cancelled;
- the entity became inaccessible due to privacy/license policy;
- exact negative authority is unavailable.

## Reference availability changes

Reference `Introduced`/`Removed`/restriction/signature/type transitions require exact Reference comparison authority. A missing API from one annotation artifact or Blizzard implementation file is not enough.

## Relation diffs

For each declared relation set:

```text
before direct accepted relation identities
vs
after direct accepted relation identities
```

Compare semantic endpoints under the accepted lineage mapping. Unknown endpoint lineage or partial relation coverage yields unresolved/NotEvaluated changes; do not match endpoints by name silently.

Paths are compared as paths only when the profile explicitly requests path-change records. A changed path does not automatically mean the direct relation changed.

## Compatibility classifications

Optional structured classes may include:

```text
SourceCompatible
SourceIncompatible
PotentiallyCompatible
BehaviorUnknown
RestrictionTightened
RestrictionRelaxed
AvailabilityAdded
AvailabilityRemoved
```

Each requires a specific frozen classifier and exact inputs. Static type/signature compatibility never becomes runtime behavior or safety proof.

## Change set assembly

```text
validate accepted lineage assertions
-> enumerate profile-declared facets/relations
-> load exact before/after owner records
-> compare typed states under capability/coverage gates
-> emit independent ChangeRecords
-> evaluate before-only/after-only absence decisions
-> retain conflicts and unmatched entities
-> canonicalize GenerationChangeSet
```

## Determinism

Equivalent exact inputs/profile yield the same:

- field/relation comparison pairs;
- state transitions;
- change kinds and IDs;
- compound-change ordering;
- unmatched/removal/introduction records;
- coverage/conflicts/NotEvaluated summaries;
- canonical bytes.

No source order, row ID, reviewer chronology, search rank or display wording controls classification.
