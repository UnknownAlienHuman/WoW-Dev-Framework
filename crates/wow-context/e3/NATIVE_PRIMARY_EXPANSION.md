# Native Primary frontier expansion

This closed selected contract is tracked in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
The selected source-absent native lifecycle reaches this route, including sibling
paths and original support. Full E3-B/Tier A/X acceptance remains open. The broader
[expansion and stopping contract](EXPANSION_SELECTION_AND_STOPPING.md) remains
normative beyond this one selected round.

## Closed selection

`wow-context/primary-native-frontier/1` starts from the exact initial
`wow-context/primary-l1/1` skeleton and its immutable native detail map. The caller
supplies 1..32 distinct GraphEntity frontier IDs already present in that map,
excluding the initial root. Empty, excess, duplicate, root and nonmember
selections refuse. Frontier order is normalized by exact native ID and bound to
the original map/universe, profile, request and selected limits.

Each frontier is queried once against the same acquired graph owner: Both
direction, all 27 reviewed native direct relation kinds, IncludeCandidate.
One round reaches at most depth 2 from the initial root. It does not recursively
expand new endpoints, infer transitive edges or select frontiers by name.
The relation vocabulary does not assert that every producer was evaluated.
No source reparsing, source excerpts or runtime observations are added.

## Native retention and identity

[`map/expansion.rs`](../src/map/expansion.rs) exposes
`plan_context_expansion`, its exact `navigation_queries` / `admit_neighborhood`,
`prepare_context_expansion` and explanation admission, then
`expand_context_frontier` over genuine resolved receipts. The sealed
`ContextExpansionStep` exposes the original `skeletons()`, `expansion_id()`,
`canonical_bytes()` and `state()`; decoded rows are not owner authority.
`remaining_metadata_bytes()` exposes the actual post-artifact allowance for the
inspect service-envelope preflight, without changing canonical artifact bytes.
The consuming [frontier pack API](NATIVE_PRIMARY_FRONTIER_PACK.md) moves the
original skeletons, exact expansion artifact and actual private Budget once;
packing/rendering continues that ledger rather than reconstructing it from this
scalar allowance. Original inspect bytes remain unchanged.

The expansion artifact schema is `wow-context/primary-frontier-expansion/1`.
The initial L1 and its origin map remain unchanged. Aggregate origins have schema
`wow-context/project-map/primary-frontier-origins/1` and compiled descriptor
`wow-context/primary-frontier-origins/1`; they cannot masquerade as the original
one-hop map. Artifact identity binds the exact initial inputs, normalized frontier,
closed descriptors, selected limits and native retained records.

Nodes and edges deduplicate by their original native IDs under the same acquired
universe/generation. Retain every initial/frontier inclusion reason and distinguish
added records from previously selected records. Added entity/relation counts and
unselected-initial-frontier count describe only this selected map/batch.
Parallel native edge IDs and their original confidence remain distinct;
an indirect path is not a direct edge.

Every retained claim preserves genuine explanation/evidence support, exact
source handles/digests/spans and applicable package/TOC/load custody. Original
coverage, conflicts, native query receipts, truncation and scoped omissions
survive aggregation. Missing mandatory custody/support or mixed owners refuse;
serialized JSON cannot reconstruct native authority. Platform remains absent
and unavailable Reference enrichment is explicit; there is no inferred name join.

## Cumulative bounds and stopping

One context ledger accounts for original map/L1 retention, the entire planned
batch scan, resolved views, dedup indexes, inclusion reasons, evidence/support,
source/load custody and the final artifact/service envelope. Borrowed native
records are charged before retention. Per-query native limits still apply; a
fresh allowance per frontier cannot bypass aggregate limits. Mandatory overflow
refuses instead of dropping evidence or earlier selections to fit.

Planning, native query/admission loops and final encoding remain cancellable;
no background continuation occurs. Original omitted-edge/truncation records
remain visible. Partial/Truncated output retains `absence_authoritative=false`.
The one-round profile bound ends the operation, including an empty added set.
It does not classify that outcome as NoNewEvidence, CompleteForRequest or an
authoritative negative about other neighborhoods, generations or runtime.

## Inspect and pack consumers

The [inspect route](../../../apps/wow/CONTEXT_INSPECT.md) requires
`--expansion-profile-id wow-context/primary-native-frontier/1` together with
repeatable exact `--frontier` tokens and `--level l1` over an explicit GraphEntity
root. L0, Both and Project selections are outside this profile. Both query
direction is independent of the inspection level.

The service uses one Current resolution/exact-generation acquisition, existing
guards and checked resource closure. The selected request binds normalized
frontiers/profile; absent selection fields are omitted. Original initial
skeleton, map and unselected inspect transport bytes remain unchanged. No
project/graph/Reference producer, registry, persistence or replay recipe changes.

The [selected frontier build/pack/render consumer](NATIVE_PRIMARY_FRONTIER_PACK.md)
consumes this native step under its own named family; its functional verification
is pending root. Inspect still returns its unchanged artifact, not a pack.
Further rounds/continuation, broader stages, Reference enrichment, platform
composition and full E3-B/Tier A/X/Goal qualification remain open under #122.
