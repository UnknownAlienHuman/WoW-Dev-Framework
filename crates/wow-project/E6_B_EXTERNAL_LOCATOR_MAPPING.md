# E6-B project-owned external locator mapping seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-project` owns mapping an E6-A `UnverifiedProviderLocator` into one exact retained project publication/generation. `wow-service` coordinates the request; `wow-project` performs all source identity validation.

## Operations

```text
project_external_locator_map
project_external_mapping_validate
```

## Request

```text
exact ProjectStore/ProjectPublication/ProjectGeneration/View identity
exact locator fields and raw-field origins
requested mapping classes: repository | revision | path | file digest | span | symbol | entity
mapping profile
privacy/license/consumer profile
item/byte/time limits and cancellation
```

No provider score, rank, summary, or claim influences mapping.

## Validation order

```text
validate exact retained project view and profile
-> validate repository/root identity when required
-> validate revision/source snapshot identity
-> normalize and validate path under project root policy
-> validate file/object digest when required
-> validate span coordinates against exact file bytes/fact records
-> validate symbol/entity identity through project-owned indexes
-> collect zero/one/many exact owner handles
-> report checked/unchecked/missing/conflicting fields and coverage
```

## Status

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

`NoMappingWithOwnerAuthority` requires complete relevant project source/entity coverage under the exact mapping profile and no truncation/conflict. Otherwise use `NoMappingPartial` or `NotEvaluated`.

## Hard boundaries

- No current/latest project generation resolution inside the owner operation.
- No clone/fetch/provider path following.
- No same-name, nearest-path, first, sole, search-rank, popularity, or snippet matching as exact mapping.
- No provider digest trusted without comparing owner bytes/records.
- No graph/lineage/replacement/impact conclusion.
- No source edit or remediation.
- No dependency on `wow-cbm`, `wow-service`, `wow-context`, or applications.

The request type crossing the boundary is an owner-neutral bounded locator projection defined by service/core contracts, not a direct `wow-cbm` dependency.

## Evidence

Return exact project publication/generation, stable source/entity handles, mapping implementation/profile, validated field pairs, owner coverage and negative-authority state, conflicts, omissions, budgets, cancellation, and canonical digest.

`ExactMapped` proves locator identity only. It does not verify provider summaries, traces, relationships, or intent.