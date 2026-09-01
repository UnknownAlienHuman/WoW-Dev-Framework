# E4-B lineage relations and proof ceilings

**Status:** normative relation registry and acceptance policy.

## General assertion shape

Every lineage assertion binds:

```text
exact ordered GenerationPairId
relation kind/version
one or more exact from endpoints
one or more exact to endpoints
producer/partition/version
accepted proposal and review decision
supporting and opposing evidence
provenance/confidence/coverage/conflicts
applicability scope
canonical digest
```

The relation schema controls endpoint cardinality, direction, allowed universes/kinds, evidence requirements, proof ceiling, inverse/display semantics, and compatibility rules.

## Proof classes

```text
Explicit
    direct owner-authored transition/stable identity record under an exact compatible profile

Derived
    deterministic reviewed rule over complete exact owner facts with a unique supported conclusion

Possible
    structurally plausible continuity/change relation with unresolved dynamic or missing decisive evidence

Candidate
    retrieval/similarity/external/model hypothesis for review only

NotEvaluated
    required capability/evidence/coverage/profile unavailable or blocked
```

These map to graph confidence/provenance without upgrading source authority. An explicit source record can be explicit evidence about source continuity while remaining nonauthority for the public API/runtime.

## Evidence lanes and maximum ceilings

| Lane | Maximum default ceiling | Notes |
|---|---:|---|
| Exact owner transition record | `Explicit` | Must state exact endpoints/relation/applicability |
| Stable owner entity ID across generations | `Explicit` or `Derived` | Only under a frozen owner identity contract |
| Exact manifest continuity plus unique discriminators | `Derived` | Complete decisive fields required |
| Deterministic source declaration continuity | `Derived` | Same body/name alone insufficient |
| Signature/type/relation fingerprint | `Possible` | Can become `Derived` only through an explicitly approved composite rule |
| Exact path/name continuity | `Possible` | Names/paths are mutable labels |
| E4-A exact query candidate | `Candidate` | Exact string relation is not continuity proof |
| E4-A text/fuzzy/shape/graph result | `Candidate` | Rank/score never upgrades |
| Community/external/model/CBM result | `Candidate` | Deferred provider profiles |
| Runtime observation | separate future lane | Does not retroactively change static source identity |

The profile may lower, never raise, a lane's ceiling without an ADR and new fixtures.

## Relation registry

### `same_stable_identity`

Meaning: endpoint sets represent the same owner-defined logical entity across the exact pair.

Cardinality: normally 1:1. Split/merge must use their own relations.

Requires:

- exact compatible endpoint kinds/universe/owner;
- stable owner ID or unique deterministic continuity rule;
- no unresolved competing endpoint at the same ceiling;
- complete decisive coverage;
- explicit changed facets retained separately.

Does not imply unchanged contract, support, location, or runtime behavior.

### `renamed_to`

Meaning: accepted identity continuity plus canonical/display identifier change.

Requires an accepted `same_stable_identity`-compatible continuity basis and exact old/new name fields. A name similarity alone cannot establish it.

### `moved_to`

Meaning: accepted identity continuity plus source/package/owner location change.

Requires exact old/new location records and continuity basis. Moving between files/packages does not imply replacement.

### `signature_changed_to`

Meaning: accepted continuity plus exact callable signature facet change.

Includes structured parameter/return/variadic/optional/nilability facets. Unknown fields cannot be collapsed into a change.

### `type_changed_to`

Meaning: accepted continuity plus exact type/enum/structure/member facet change. Type compatibility is separately classified by the owning reference/type profile.

### `restriction_changed_to`

Meaning: exact ReferenceView restriction facet changed across compatible profiles/generations.

Requires Reference Pack contract evidence. Blizzard implementation source or project behavior cannot create this assertion alone. Runtime-only Secret/spell policy remains unresolved without runtime evidence.

### `load_or_package_changed_to`

Meaning: accepted source/project identity continuity plus exact TOC/package/load-role/location facet change. It is static metadata, not runtime readiness.

### `relation_set_changed_to`

Meaning: accepted endpoint continuity plus exact bounded relation-manifest change for a declared relation class. It preserves added/removed/unknown/conflicted relations and source coverage.

### `deprecated_in`

Meaning: exact owner contract marks an endpoint deprecated in the `to` generation/profile.

Requires explicit ReferenceView/project owner record. Source comments alone are not platform deprecation authority.

### `removed_in`

Meaning: an exact `from` endpoint has no accepted continuity target in `to` and absence is authoritative for the declared universe/kind/scope.

Requires:

- complete relevant `to` inventory/reference/graph/query coverage;
- all plausible high-ceiling continuity candidates resolved/rejected;
- no profile mismatch/conflict/truncation;
- owner-supported negative authority where required.

It does not mean runtime inaccessible outside the indexed scope.

### `introduced_in`

Meaning: an exact `to` endpoint has no accepted continuity source in `from` and earlier absence is authoritative.

Uses symmetric coverage/ambiguity gates.

### `replaced_by`

Meaning: exact owner contract or reviewed migration evidence identifies one target as a supported successor for a declared capability/scope.

It requires:

- explicit replacement/deprecation transition, or a separately approved deterministic migration rule with exact applicability;
- old/new endpoint and capability scope;
- evidence/coverage/conflict closure;
- proof ceiling independent of name/shape similarity.

`replaced_by` can connect different entity identities and can coexist with `removed_in` for the old endpoint.

### `split_into`

Meaning: one exact old logical surface transitions into multiple exact target surfaces under an explicit/group continuity contract.

Cardinality: 1:N, N >= 2.

Requires group-level scope, coverage, and evidence. It cannot be represented as multiple unrelated `same_stable_identity` assertions unless owner semantics explicitly say each target shares identity, which is normally invalid.

### `merged_into`

Meaning: multiple old surfaces transition into one target surface.

Cardinality: N:1, N >= 2. Same group/evidence rules as split.

## Relation composition

Allowed derived facets:

```text
same_stable_identity + changed name -> renamed_to
same_stable_identity + changed location -> moved_to
same_stable_identity + changed signature -> signature_changed_to
```

Forbidden implicit composition:

```text
similar name + nearby path -> same_stable_identity
removed + similar new entity -> replaced_by
replaced_by -> same_stable_identity
same_stable_identity -> compatible or safe
reason path -> direct relation
multiple Candidates -> Derived
```

## Opposing evidence

Every proposal records opposing evidence such as:

- incompatible stable IDs;
- conflicting explicit transitions;
- kind/profile/universe incompatibility;
- simultaneous distinct endpoints with same candidate signature;
- source location reuse by a different entity;
- signature/relation discontinuity beyond profile limits;
- incomplete/failed decisive partition;
- explicit owner removal without replacement.

Opposing evidence can lower the proposal ceiling, create ambiguity/conflict, or reject it. It is never silently dropped because support has a higher score.

## Review and promotion

Automatic acceptance is allowed only for explicitly listed deterministic rules with all proof gates satisfied and unique conclusion. Candidate/Possible proposals require a reviewed decision or additional exact evidence according to the publication profile.

Review cannot raise a relation above the evidence profile's maximum proof ceiling. A human preference does not turn fuzzy similarity into proof; the review must attach qualifying evidence or accept only a Candidate/Possible relation state.

## Inverse and query presentation

Directional relations can expose named inverse views, for example:

```text
renamed_to / renamed_from
moved_to / moved_from
replaced_by / replaces
split_into / split_from
merged_into / merged_from
introduced_in / introduced_from
removed_in / removed_from
```

The stored canonical direction remains `from -> to`. Inverse presentation does not create another assertion.

## Validation

- endpoints belong to exact pair sides and allowed kinds;
- cardinality and relation scope valid;
- evidence lane cannot exceed its ceiling;
- required continuity basis exists for change relations;
- replacement scope explicit;
- removal/introduction gates complete;
- no unresolved equal-ceiling competitor for unique relations;
- split/merge groups preserve all endpoints;
- opposing evidence/conflicts retained;
- assertion digest deterministic.
