# `wow-project` implementation contract

**Status:** E0-D implementation-ready contract; no Rust code yet. Full TOC/XML/load/project-graph indexing remains an E2 responsibility.

## Mission

`wow-project` owns the coherent first-party project state consumed by analysis and rules. It validates one explicit workspace/input manifest, derives the canonical project generation, coordinates analyzer updates against that target generation, and publishes one immutable `ProjectSnapshot` only when the project file state and analyzer snapshot agree.

E0-D is intentionally narrow: one closed Lua workspace, one selected fixture profile/reference generation, one analyzer adapter snapshot, and deterministic update/publication semantics. It does not parse TOC or XML, build a graph, infer addon ownership, scan installed addons, or persist project state.

## E0-D outcome

A future implementation agent must prove this seam:

```text
closed ProjectInputBundle
    + exact ProfileIdentity / ReferenceGenerationId
    + exact accepted wow-emmy pin/configuration identity
    -> validate project configuration and first-party file inventory
    -> derive target ProjectGenerationId
    -> register project source origin and files
    -> submit one generation-bound AnalyzerUpdateBatch to wow-emmy
    -> receive one validated AnalyzerSnapshot for the same generation
    -> assemble and publish immutable ProjectSnapshot
    -> update one file into a new coherent generation
    -> reject partial/mixed/stale publication
```

The published snapshot is the only E0 project read surface for `wow-rules` and `wow-service`.

## Owned responsibilities

- project/workspace identity and configuration;
- first-party source-root registration;
- explicit project input inventory;
- normalized project file identity, relative path, content digest, and role;
- canonical `ProjectGenerationId` derivation;
- project update-set validation;
- analyzer-update request assembly for the target project generation;
- analyzer-snapshot compatibility validation;
- immutable `ProjectSnapshot` publication;
- project-side source-file registry used by exact span handles;
- project capability/coverage records;
- generation-coherent read views for analyzer facts and generic findings;
- publication failure isolation and previous-snapshot retention;
- deterministic project manifest/snapshot serialization;
- E2 expansion into TOC/XML/load/dependency/state/event/hook facts when that milestone activates.

## Explicit non-responsibilities

`wow-project` does not:

- decide whether a WoW API exists;
- store reference/restriction facts;
- implement generic or WoW diagnostic algorithms;
- normalize upstream Emmy diagnostics or semantic facts;
- parse TOC/XML in E0-D;
- build graph nodes/edges or recognizer roles in E0-D;
- persist SQLite/project databases;
- rank search results;
- call Codebase Memory or external repositories;
- select a floating current profile;
- discover arbitrary repositories or installed addons;
- mutate editor configuration;
- execute Lua, repository hooks, build scripts, or tests;
- claim in-client/runtime behavior;
- expose mutable analyzer or project internals;
- publish a snapshot whose analyzer state belongs to another generation.

## Required reading

Before implementation, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`../wow-emmy/CONTRACT.json`](../wow-emmy/CONTRACT.json)
6. [`../wow-emmy/SESSION_MODEL.md`](../wow-emmy/SESSION_MODEL.md)
7. [`AGENTS.md`](AGENTS.md)
8. [`DECISIONS.md`](DECISIONS.md)
9. [`DATA_MODEL.md`](DATA_MODEL.md)
10. [`GENERATION_AND_PUBLICATION.md`](GENERATION_AND_PUBLICATION.md)
11. [`UPDATE_MODEL.md`](UPDATE_MODEL.md)
12. [`SOURCE_REGISTRY.md`](SOURCE_REGISTRY.md)
13. [`ERROR_MODEL.md`](ERROR_MODEL.md)
14. [`TEST_MATRIX.md`](TEST_MATRIX.md)
15. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
16. [`CONTRACT.json`](CONTRACT.json)
17. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

Normative repository sources:

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## Direct dependencies in E0-D

The E0-D Rust slice may depend directly only on:

```text
wow-core
wow-emmy
```

Although the long-term dependency graph permits `wow-store`, `wow-graph`, and `wow-recognizers`, those edges remain inactive in E0-D.

Do not activate a dependency merely because it appears in the maximum-permitted graph.

## E0 project fixture

E0-D uses one project fixture identity:

```text
project_id: fixture-project-e0-v1
workspace_id: workspace:main:e0
project_kind: fixture
selected_profile_id: fixture-retail-120100-e0-v1
source_origin_id: project-origin:fixture-project-e0-v1
```

The project file set binds to the Main files declared by the merged E0-C analyzer fixture:

```text
main/clean.lua
main/generic-error.lua
main/missing-api.lua
main/secret-local.lua
```

The source text has one logical owner in the fixture bundle selected before implementation. E0-D references exact file content IDs/digests; it does not maintain an independent divergent copy.

## Project configuration

```text
ProjectConfiguration
    project_id
    project_kind
    workspace declarations
    source roots
    selected ProfileIdentity
    selected ReferenceGenerationId
    accepted analyzer pin/probe identity
    analyzer configuration digest
    project schema version
    capability policy
    input and output budgets
```

No field is inferred from the local machine, installed WoW client, editor, or floating branch.

## Project generation

The E0 `ProjectGenerationId` is derived from canonical:

```text
project configuration identity
selected profile/reference generation
accepted analyzer pin/probe identity
analyzer configuration digest
ordered normalized project file manifest and content digests
project schema version
```

It excludes:

```text
wall-clock time
temporary checkout path
worker/thread ID
hash-map iteration
session memory address
Git credentials
rendered diagnostic text
```

A change to any generation input produces a different target project generation. This includes analyzer pin/configuration changes because they can change project semantic results.

## Source registry

`wow-project` owns one registered first-party source origin and file manifest. `wow-emmy` may construct exact span handles only against this registry and the supplied target generation.

Every project file record contains:

```text
ProjectFileId
workspace ID
source origin ID
normalized relative path
content digest
byte length
language kind
first-party role
selected project generation
```

No absolute host path, symlink escape, tokenized URL, or Library file can masquerade as first-party project source.

## Analyzer integration

E0-D calls only the normalized `wow-emmy` adapter seam.

Publication protocol:

```text
validate project update and derive target generation
-> build analyzer update batch with target ProjectGenerationId
-> apply/index through wow-emmy
-> receive AnalyzerSnapshot
-> verify snapshot project/profile/reference/pin/config/file identities
-> assemble ProjectSnapshot
-> publish atomically
```

If analyzer update/index/snapshot validation fails:

- the target generation is not published;
- no mixed/partial project snapshot is exposed;
- the prior immutable snapshot may remain last-known-good under its original generation;
- the failure and affected capabilities are explicit.

## Project snapshot

```text
ProjectSnapshot
    project generation identity
    selected profile/reference generation
    project configuration digest
    project file manifest
    source registry
    accepted analyzer pin/probe/config identity
    analyzer snapshot ID and validated read view
    project capability/coverage records
    publication status and canonical digest
```

The snapshot does not copy or reinterpret analyzer facts/findings. It validates and exposes generation-bound views/handles.

## E0 project capabilities

```text
project.fixture.configuration.valid
project.fixture.files.complete
project.generation.coherent
project.source.registry.complete
project.analyzer.snapshot.available
project.analyzer.facts.available
project.analyzer.generic_diagnostics.available
```

E2 capabilities such as the following are absent or explicitly `NotEvaluated`, never fake-complete:

```text
project.toc.complete
project.xml.complete
project.load_graph.complete
project.state_index.complete
project.event_hook_index.complete
project.graph.complete
```

## Update model

E0 supports a closed set of file updates:

```text
add first-party Lua file within registered root
update exact file with expected old digest
remove exact first-party file
replace analyzer configuration/pin only through explicit project configuration change
```

One update transaction targets one new project generation. Stale expected generation/file digest rejects the update.

No hidden filesystem watch, background scan, editor callback, or automatic repository discovery exists in E0-D.

## Required operations

Concrete Rust names may change only with a matching contract update. Required semantics are defined in the component documents:

```text
validate_project_configuration
inventory_project_inputs
register_project_source_origin
normalize_project_file
build_project_file_manifest
derive_project_generation_id
validate_project_update
build_analyzer_update_batch
validate_analyzer_snapshot_for_project
assemble_project_snapshot
validate_project_snapshot
publish_project_snapshot
open_project_view
project_file_by_id
project_file_by_path
analyzer_facts_for_file
analyzer_generic_findings
project_capability_records
canonicalize_project_manifest
canonicalize_project_snapshot
retain_last_known_good_snapshot
```

## E0-D hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No TOC/XML parser or load graph.
- No `wow-store`, `wow-graph`, or `wow-recognizers` dependency.
- No repository-wide filesystem scan/watch.
- No installed-addon or SavedVariables universe.
- No automatic Git/branch/profile discovery.
- No arbitrary source or repository code execution.
- No editor-setting mutation.
- No diagnostic/rule logic.
- No API/restriction authority.
- No snapshot publication after analyzer failure/mismatch.
- No prior snapshot relabeled as the target generation.
- No empty-success placeholders for deferred E2 capabilities.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- project configuration and explicit file inventory;
- initial project-generation derivation inputs;
- successful baseline snapshot binding;
- file update/remove/add cases;
- analyzer failure and generation-mismatch publication cases;
- last-known-good behavior;
- pending byte/generation checksum freeze.

Actual source digests, analyzer pin/config identity, project-generation IDs, and bundle SHA-256 values are frozen after E0-A/E0-C implementation exists and before the first `wow-project` Rust commit.

## Definition of done

E0-D implementation is complete only when:

```text
one explicit project configuration validates
one first-party source origin and exact four-file manifest publish
one canonical ProjectGenerationId is derived deterministically
wow-emmy receives and returns the same target project generation
one AnalyzerSnapshot validates against project/profile/reference/pin/config/files
one immutable ProjectSnapshot publishes atomically
one file update produces a new coherent generation
stale digest/generation and analyzer mismatch reject publication
failed analyzer update leaves prior snapshot under its old generation only
project and library source roles never mix
all deferred TOC/XML/graph capabilities remain explicit unavailable/NotEvaluated
randomized input/update order leading to the same final state yields byte-identical canonical snapshot output
no project source/repository code executes
all TEST_MATRIX cases pass
```

Until then, this directory remains an implementation-ready project-generation contract, not a functioning project index.
