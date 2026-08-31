# Blizzard UI source indexing pipeline

**Status:** normative E3-B source-to-candidate pipeline.

## Reuse of E2-C

E3-B reuses the E2-C project index architecture:

```text
materialized snapshot
roots/universes/packages/files
TOC/XML/load
physical and virtual Lua units
wow-emmy AnalyzerSnapshot
fact adapters
recognizers
graph proposal validation
incremental invalidation
candidate
```

E3-B adds a multi-root source-collection profile, source/reference bridge inputs, source-specific publication identity and license/build-binding gates. It does not fork the parser/analyzer contracts.

## Phase 1 — request/profile validation

Validate exact:

- source profile and materialized snapshot;
- build-binding and reference compatibility state;
- admitted logical roots and packages/global units;
- parser/analyzer/annotation/recognizer/graph/store profiles;
- license/redistribution/security/budget policies;
- expected base source publication for update;
- cancellation.

A floating source locator or unqualified current/latest profile is invalid.

## Phase 2 — source inventory

For every admitted root:

- validate normalized root-relative file IDs;
- validate canonical content digests/lengths and source handles;
- preserve file kind, encoding/newline/binary state;
- preserve provider object provenance separately;
- classify admitted/excluded/unsupported/missing/conflicted entries;
- validate license and redistribution records;
- build exact source inventory partitions.

Filesystem enumeration is never performed by `wow-project`.

## Phase 3 — package and global-unit model

### TOC packages

Under declared package roots:

- discover package candidates only from the exact admitted inventory and profile rules;
- parse TOC using the E2-C bounded nonexecuting parser;
- select exactly one variant per package/profile;
- preserve required/optional dependencies, source file order, LoadOnDemand/bootstrap metadata and SavedVariables declarations;
- resolve files only inside the exact snapshot and declared source roots.

SavedVariables names are source declarations only; contents are never read.

### Global/shared units

For roots without TOCs, the profile supplies explicit global-unit manifests or deterministic reviewed boundary rules. Each unit has ordered members and direct include/load relationships. Directory names alone do not create package semantics.

## Phase 4 — XML

Use the E2-C bounded streaming XML contract:

- DTD, external entities, XInclude, network/catalog resolution and execution disabled;
- exact includes, templates, objects, frames/regions, inheritance, parents, scripts and unknown records;
- include/script resolution confined to the snapshot/root profile;
- bounded cycles/depth/fanout/bytes;
- inline Lua materialized as exact virtual source units with physical XML mappings;
- malformed portions downgrade only affected capabilities.

## Phase 5 — static load model

Build direct package/unit/file/XML/script/load edges and ordered load roles:

```text
shared/global prelude under exact profile
selected TOC package order
XML include/script expansion
inline virtual Lua units
bootstrap/normal/conditional/unknown phases
required/optional dependency edges
```

Reachability classes remain `Reachable`, `ConditionallyReachable`, `Unreachable`, `Unknown/NotEvaluated`.

No static model proves client execution success, frame readiness, event delivery or runtime state.

## Phase 6 — Lua workspace

Build an exact analyzer workspace:

```text
Main source universe
    admitted Blizzard UI physical Lua units
    XML external Script files
    XML inline virtual Lua units

Library universe
    exact generated annotation artifact and declared analyzer libraries
```

First-party user project sources are absent. The source universe and analyzer library universe remain distinct.

The update plan names exact file/unit IDs, bytes/digests, load/order metadata, source mappings and profile/config IDs.

## Phase 7 — analyzer snapshot

`wow-emmy` returns one immutable `AnalyzerSnapshot` bound to the source generation and unit manifest.

Validate:

- exact implementation/pin/config/annotation identities;
- every requested physical/virtual unit;
- source coordinates/maps;
- generic diagnostics/facts and coverage;
- no project/reference/source-universe collapse;
- cancellation/partial/failure state.

No raw analyzer session becomes a persisted source fact.

## Phase 8 — fact adapters

Adapters emit typed partitions for:

```text
source inventory/root/package/global unit
TOC directives/files/dependencies/load roles/SavedVariables declarations
XML includes/templates/objects/parents/inheritance/scripts
Lua declarations/symbols/members/calls/assignments/indexing/registrations/hooks/state operations
load reachability and direct edges
source/reference resolution ingredients
```

Every fact retains exact source generation, source handles, evidence, provenance, confidence, coverage and adapter-loss records.

No adapter parses raw Lua again or reconstructs unsupported analyzer facts from text.

## Phase 9 — recognizers

Run only the approved universal core pack/profile over normalized facts. E3-B can identify structural conventions already covered by the pack:

- packages/load/dependencies;
- XML templates/objects/inheritance/scripts;
- frames/mixins/factories where exact facts exist;
- native events, EventRegistry bridges/custom signals and CVar callbacks;
- hooks;
- libraries;
- state roots and literal paths.

Named Blizzard-source path/name heuristics are not activated in E3-B. Any future calibration pack remains separately versioned, evaluated and removable without changing core semantics.

## Phase 10 — direct source graph proposals

Project-owned adapters propose exact source entities/relations that do not require recognizer inference, such as files, packages, declarations, direct containment/load/include/definition and analyzer-proven references.

Recognizers propose their own partitions. Proposal producer classes remain separate.

## Phase 11 — reference/source bridges

Using exact compatible reference graph/entity views and bridge profile, build bridge proposals according to [`REFERENCE_AND_PROJECT_BRIDGES.md`](REFERENCE_AND_PROJECT_BRIDGES.md).

Ambiguous/missing/incompatible endpoints produce `NotEvaluated`, `Possible` or conflicts; never nearest-name resolution.

## Phase 12 — graph validation

`wow-graph` independently validates:

- registry bundle and universe;
- semantic key ingredients;
- source/reference endpoint identities;
- relation schemas/direction/cross-universe policy;
- attributes/confidence/provenance/evidence/coverage;
- conflicts/cycles/multiplicity;
- producer partitions and replacement plan.

Rejected proposals remain in the candidate report.

## Phase 13 — candidate assembly

Assemble `BlizzardUiSourceIndexCandidate` only after source, package/load, analyzer, recognizer, graph and bridge manifests reconcile.

Candidate state does not exceed the weakest blocking axis:

- incomplete source inventory/build binding/license may block publication eligibility;
- partial analyzer/recognizer coverage remains partial;
- graph conflicts remain visible;
- a technically complete candidate can still be local-analysis-only or non-current-eligible.

## Phase 14 — E2-D publication

Build a dedicated source publication set, then use the E2-D protocol:

```text
inactive store/source/graph generation
-> commit
-> fresh exact read snapshot
-> source/project/graph/bridge/license validation and golden queries
-> stale-base CAS current source record
```

No user project current record is touched.

## Worker model

Parsing/analyzer/fact/recognizer/proposal work may be parallel under deterministic partitioning. Merge/output order is canonical by root/package/file/entity/partition keys. Worker count/scheduling cannot enter IDs.

## Failure isolation

- one malformed file downgrades only its dependent partitions where safe;
- source inventory/build/profile conflict may block the entire candidate;
- analyzer failure does not permit raw-source fallback;
- recognizer failure does not remove direct source facts but downgrades recognizer coverage;
- bridge failure does not erase source graph facts but blocks bridge capabilities;
- publication failure leaves current source publication unchanged.
