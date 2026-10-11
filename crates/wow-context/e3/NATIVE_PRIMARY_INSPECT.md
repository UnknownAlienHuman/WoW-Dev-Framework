# Native Primary L0/L1 inspection

This page describes the implemented selected route in
[`skeleton.rs`](../src/skeleton.rs), not acceptance of the full E3-B contract.
The canonical tracker is [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
The normative [L0](L0_SKELETON.md), [L1](L1_SKELETON.md) and
[profile](CONTEXT_REQUEST_AND_PROFILES.md) contracts retain their broader scope.

## Profiles and roots

| Static descriptor | Scope | Native direct vocabulary |
|---|---|---|
| `wow-context/primary-l0/1` | Exact acquired ProjectId, or an exact `source_file` / `source_package` graph container | Contains, Defines, DependsOn, OptionalDependsOn, Owns, Loads, LoadsBefore |
| `wow-context/primary-l1/1` | Exact admitted Primary graph entity | All 27 reviewed native direct relation kinds |
| `wow-context/primary-state-root-l0/1` | Exact original ordinary selected-TOC `state_root` graph entity | Same seven structural kinds as default L0 |

The existing L0/L1 descriptors select both directions, depth 1, native node-ID then edge-ID
ordering, local metadata, excluded source bodies, unavailable tokens and
NotEvaluated continuation. Their declaration policy is
`selected_native_callable_and_lua_xml_occurrence_metadata`; their type policy is
`original_selected_native_signature_state_only`. No profile adds inferred roles.

L1 preserves Calls, state reads/writes, native events, custom signals, CVar
callbacks, script/function hooks, the seven structural kinds, inheritance,
parentage, template/mixin/instance/factory relations, API use and schema-class
relations as distinct native assertions. A path never becomes a direct edge.
The vocabulary does not assert that every producer was evaluated.

`L0Scope::Project(ProjectId)` must match the acquired project and its project
navigation map. It retains the original ProjectSourceOrigin without fabricating
a project graph node. `direct_members` are the selected source-file nodes;
`member_enumeration=retained_source_file_projection_membership` is not a census
of the filesystem. A graph-container L0 uses only selected outgoing Owns or
Contains edges for membership. The default /1 descriptor still refuses state
roots and other container kinds.

`SkeletonLevel::{L0,L1,Both}` selects one caller root. With default L0, Both
requires an explicit source-file or source-package container. Selected state-root
L0 uses an exact state_root for L0 or Both; L1 keeps its old /1 detail profile.
The same explicit root is used unchanged by both selected levels.
Standalone L0 uses structural navigation; Both shares the L1 detail-origin map.
L0 `direct_relations` selects its seven structural kinds even when Both shares
the broader detail origins; L1 retains every selected native detail relation.
L0 membership remains outgoing Owns/Contains. No implicit first-file root is used.

## Explicit state-root L0

`PrimaryL0Profile::StateRootV1` selects `wow-context/primary-state-root-l0/1`.
Its root must be the exact accepted source-partition GraphEntity for an original
ProjectGraphStateRoot from ordinary selected-TOC Main plus separate Library.
Files-only input, a project ID, a state_path or a same-name recognized root cannot
substitute for that owner. Producer, registry, storage and replay contracts stay
unchanged.

The typed StateRoot facet retains the original root and **all exact supporting
ProjectGraphStateDeclaration rows**, including repeated declarations and their
TOC ordinals, account/character scope, ambiguity and original support pairs.
It does not choose a first declaration. StatePath facets retain original literal
keys/root IDs for each selected outgoing Owns/Contains member, joined through
its exact accepted source proposal and published node. Every selected member
requires original typed path custody; missing support refuses rather than hiding
a member. The type policy is `symbolic_literal_keys_only_no_runtime_values`.

Depth/direction, deterministic ordering, source boundary and cumulative limits
remain the existing L0 policy. Both shares old L1 detail origins while L0
filters its seven structural relations. Selected member counts describe that
bounded native neighborhood, not exhaustive SavedVariables fields. Dynamic,
unresolved and alias omissions remain scoped to the original owner; zero under
Partial coverage never proves absence. Runtime values, bodies and exhaustive
field/type claims remain unavailable. The focused source-absent native lifecycle
and Windows CLI inspect/build pass; broad E3-B/Tier A/X acceptance remains open
in #122.

## Native API and custody

After `bind_context_universe_set`, `build_l0_skeleton`, `build_l1_skeleton` and
`build_primary_skeletons` consume the exact ContextUniverseSet, immutable
ProjectMap, acquired GraphPartitionRead, root and cancellation flag. L1 requires
a map with the native detail vocabulary. The owner rejects a map/universe or
graph-owner mismatch; decoded JSON cannot reconstruct this borrowed authority.
The combined API shares one aggregate reservation and builds L0 before L1.
`build_l0_skeleton_with_profile` and `build_primary_skeletons_with_profile`
accept the closed PrimaryL0Profile selection over those same owners. Existing
builders default to ContainersV1; old /1 behavior and bytes remain unchanged.

L0 emits schema `wow-context/l0-skeleton/primary/1`, exact scope, descriptor,
selected members/count, declaration rows, direct relations, detail routes and
NotEvaluated fields. Selected state L0 emits
`wow-context/l0-skeleton/primary-state-root/1` with its named descriptor and typed
state rows. L1 emits `wow-context/l1-skeleton/primary/1`, exact root,
local entities, declaration rows and original direct relations. The combined
schema is `wow-context/primary-skeleton-inspection/1`; absent levels are omitted.
L0/L1 expose their respective ID, `canonical_bytes()` and `state()` accessors.

Each layer embeds its immutable origin map and origin_map_id/universe_set_id.
That map retains exact project/configuration/graph/Reference generations and
identities, source handles and artifact digests/spans, package/TOC/load custody,
original support/evidence, coverage, conflicts and omissions. Optional platform
absence and unavailable Reference enrichment remain explicit. Native source
acquisition/validation belongs to existing owners; context does not reparse
source or execute it. IDs bind the typed core and static descriptor under the
layer's schema domain; final ID-plus-core envelope bytes are bounded separately.

Declaration rows join selected published nodes to genuine native provenance.
Callable rows use `ProjectGraphProvenance::callable_metadata`: exact function
identity, analysis ID, content digest, nearest enclosing function scope and
optional parent ID, plus original declaration/signature recipe and observation.
ChunkScopeClosure describes lexical scope, not statement position or loading.
Unselected Primary recipes retain `NotSelected` with recipe/observation omitted;
selected observations retain Observed, Partial, Unavailable or Unsupported as
applicable. Ordered declared parameter names/vararg tokens and native signature
parameter/return/type metadata keep their original order and uncertainty.
Supported signature fragments survive Partial; resolved empty returns stay
empty. Bodies and SourceFunctionFact.shape never enter this metadata projection.
Lua/XML occurrence rows retain their exact original IDs, proposals and source /
evidence handles. The owner does not invent richer XML/type/role summaries.

## Bounds and remaining qualification

`PrimaryMapLimits::new` requires validated native neighbor/explanation limits,
`max_direct_edges` in 1..=100000, and positive usize-representable
`max_input_bytes`, `max_metadata_bytes`, `max_context_work`. There are no owner
defaults or separate unbounded skeleton limits. Work reservations are finite
loop/join bounds, not CPU-time measurements. Borrowed rows are charged before
retention; Both cannot reset the shared ledger. Artifact/domain/envelope guards
include descriptors, repeated origin custody, support and accounting metadata.
Mandatory overflow refuses; original query truncation stays visible.

Every returned layer is Partial or Truncated with absence_authoritative=false.
NotEvaluated includes unselected declaration/type recipes, Reference enrichment,
control-effect projection, excerpts/exact tokens, deeper paths/continuation and
complete-corpus/runtime behavior. Root/custody/support/universe, budget,
encoding and cancellation failures retain typed owner errors; there is no empty
successful fallback. Broad containers/facets, expansion, packs/rendering,
privacy/excerpt/token/continuation and platform composition remain open, together
with the coherent Tier A campaign and required Tier X qualification. This page
records code behavior, not a claim that those gates passed.

The [service/CLI route](../../../apps/wow/CONTEXT_INSPECT.md) pins publication,
checks guards, enforces transport/output bounds and closes native resources.
