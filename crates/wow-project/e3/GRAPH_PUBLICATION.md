# Blizzard UI source graph and persistent publication

**Status:** normative E3-B source/graph/store handoff.

## Graph universe

Every accepted source assertion belongs to:

```text
universe = blizzard_ui_source
source generation = exact BlizzardUiSourceGenerationId
```

Reference bridge targets retain `reference_api`; they are not re-keyed into the source universe.

## Registry

E3-B uses an exact reviewed graph registry bundle compatible with E2-A. The source profile may activate already defined kinds/relations and additive versioned extensions required for implementation-source structure.

Initial source entities can include:

```text
source_collection
source_root
package
toc_manifest
toc_variant
load_unit
file
virtual_source_unit
namespace
module
service
library
function
method
callback
event
api_symbol endpoint in reference universe
xml_template
frame
region
mixin
factory
registry
state_root
state_path
source_span
```

A kind is activated only when its identity and evidence contract are frozen.

## Relation families

Source-internal examples:

```text
contains / declares / defines / exports
loads / loads_before / depends_on / optional_depends_on
includes / references_template / inherits / parent_of
mixes_in / instantiates / created_by
calls / possible_calls
registers_event / handles_event
triggers_callback / subscribes_callback
hooks / sets_script
reads_state / writes_state
embeds_library / requires_library
owns / implements_role
```

Cross-universe examples:

```text
blizzard_ui_source --uses_api--> reference_api
blizzard_ui_source event registration --registered exact event relation--> reference_api event
```

`parent_of` remains object/XML semantics only. There is no generic parent relation.

## Producer partitions

```text
source-inventory:<generation>:<root-or-file-partition>
source-toc-xml-load:<generation>:<package-or-unit>
source-analyzer:<generation>:<file-or-unit>
source-recognizer:<generation>:<pack/rule/version>:<partition>
source-reference-bridge:<source-generation>:<reference-generation>:<rule>
```

Each partition is independently replaceable and has exact coverage/conflict state.

## Proposal validation

E3-B sends proposal partitions to `wow-graph`, which validates:

- exact registry and universe/profile compatibility;
- entity/relation semantic key ingredients;
- source/reference endpoint existence and generation scope;
- relation direction/cross-universe policy;
- typed attributes and size limits;
- source handles/evidence/provenance/confidence/coverage closure;
- ambiguity/conflict/cycle/multiplicity rules;
- producer partition ownership/version/replacement semantics.

The graph owns final assertion IDs, conflict records, GraphGeneration and GraphSnapshot.

## Source graph generation

```text
BlizzardUiSourceGraphManifest
    exact graph registry bundle
    source profile/generation/snapshot
    analyzer snapshot
    reference profile/generation used by bridge partitions
    ordered direct/recognizer/bridge partition manifests
    accepted/rejected proposal manifests
    entity/relation/assertion/conflict/coverage digests
    graph capability summary
    logical store schema/operation/validation bundle IDs
    GraphGenerationId / GraphSnapshotId
    canonical digest
```

## Publication bundle

`wow-project` assembles:

```text
BlizzardUiSourcePublicationBundle
    validated source candidate
    exact expected current source publication/store generation: optional
    source project logical partition versions
    exact GraphPublicationPlan
    source/reference bridge partitions
    source and graph schema/operation/validation bundles
    expected logical manifests/counts/digests
    source/license/redistribution manifests
    object/member reference manifest
    golden source/graph/bridge reads
    budgets/cancellation
```

No raw SQL, table name, connection, transaction callback, PRAGMA, row ID, host source path or provider credential.

## Dedicated store namespace

The source collection has its own:

```text
ProjectStoreId
ProjectStoreEpoch
ProjectStoreGeneration
CurrentBlizzardUiSourcePublicationRecord
```

It does not reuse a user project's current record. Physical storage can use the E2-D profile but source and user project generations remain independently selected and retained.

## Two-stage publication

### Inactive build

- validate exact source candidate, graph plan, schema set and expected base;
- materialize/reuse immutable source and graph partition versions;
- write complete target membership, source/graph/bridge/license manifests and object refs;
- execute in-transaction structural checks;
- commit target `PublishedInactive`;
- leave current unchanged.

### Fresh read-back validation

Open the exact inactive generation in a new read snapshot and validate:

- source profile/generation/build-binding consistency;
- complete root/file/package/TOC/XML/load/source-map closure;
- analyzer/fact/recognizer partition closure;
- graph assertions/endpoints/reverse indexes/conflicts/coverage;
- reference/source bridge exact endpoints and compatible profiles;
- stale source/entity/relation/bridge removal;
- no user project or other-universe leakage;
- license/redistribution/object/member closure;
- deterministic golden source/graph/bridge queries.

### CAS activation

Activate only if:

- the validation report is successful under policy;
- current source publication still matches the expected base record/digest;
- the target remains immutable and exact;
- the build-binding state is eligible for the selector's publication policy;
- mandatory license/redistribution state is resolved.

Otherwise leave the target inactive/quarantined and current unchanged.

## Current selector

A current record is keyed by an explicit selector such as:

```text
source profile ID
client game family/flavor/build/interface compatibility key
reference profile/generation compatibility policy
publication channel/policy
```

No unqualified `current`, `latest`, or nearest build fallback.

## Reader coherence

A source reader opens one current record and one exact store read snapshot, yielding a coherent set:

```text
source store generation
source profile/generation/snapshot
analyzer snapshot
source GraphGeneration/GraphSnapshot
reference profile/generation used by bridges
license/redistribution/capability state
```

An existing reader remains on its exact old SQLite snapshot after activation. New readers see the new current source publication.

## Combined user-project queries

A higher service opens independently:

```text
exact user ProjectSnapshot/GraphSnapshot
exact ReferenceView/GraphSnapshot
exact BlizzardUiSourcePublication/GraphSnapshot
```

It verifies compatibility and then executes bounded cross-universe graph/search/context operations. E3-B source publication never mutates or aliases the user project publication.

## Failure and recovery

- target build/validation/cancellation/crash does not advance current;
- committed inactive source generation can be recovered/revalidated under its original identity;
- a mismatched build binding cannot be activated by relabeling;
- prior current/last-known-good stays exact and may be returned only with explicit requested/fallback status;
- explicit rollback revalidates and CAS-activates a retained source publication;
- corruption recovery/rebuild preserves failure evidence and source/provider identities;
- GC retains current, last-known-good, active readers, evidence/debug/recovery pins and referenced partition/object closure.

## No publication shortcuts

Forbidden:

- writing accepted graph assertions without `wow-graph` validation;
- advancing current in the inactive-build transaction;
- activating based only on successful SQLite commit;
- copying old source/graph rows into a new generation without exact dependency proof;
- mixing source/reference generations to fill missing bridge facts;
- hiding rejected proposals/conflicts;
- replacing current source globally for all profiles;
- treating a nonredistributable source store as a releasable pack.
