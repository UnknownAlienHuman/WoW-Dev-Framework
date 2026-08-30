# `wow-project` E0-D data model

**Status:** normative semantic model for the minimal project-generation slice.

Concrete Rust layout is not prescribed. Type ownership and invariants are.

## 1. Object graph

```text
ProjectInputBundle
├── ProjectConfiguration
├── ProjectSourceOrigin
├── ProjectInputInventory
│   └── ProjectInputFile[]
├── AnalyzerBindingDeclaration
├── ProjectCapabilityPolicy
├── ProjectBudgetPolicy
└── ProjectUpdateCase[]

Validated ProjectInputBundle
    -> ProjectGenerationCandidate
    -> AnalyzerUpdateBatch (wow-emmy)
    -> AnalyzerSnapshot (wow-emmy)
    -> ProjectSnapshotCandidate
    -> published immutable ProjectSnapshot
```

All general IDs, contexts, evidence, coverage, budgets, source handles, and canonicalization use `wow-core` contracts.

## 2. Project configuration

```text
ProjectConfiguration
    project_id: ProjectId
    project_kind: fixture | repository
    configuration_schema_version: SchemaVersion
    workspace_id: ProjectWorkspaceId
    source_origin_id: ProjectSourceOriginId
    logical_root: String
    selected_profile: ProfileIdentity
    reference_generation: ReferenceGenerationId
    analyzer_pin_id: String
    analyzer_probe_contract_version: String
    analyzer_configuration_digest: ContentDigest
    analyzer_fixture_contract_id: String
    capability_policy: ProjectCapabilityPolicy
    budget_policy: ProjectBudgetPolicy
```

E0 values:

```text
project_id = fixture-project-e0-v1
project_kind = fixture
workspace_id = workspace:main:e0
source_origin_id = project-origin:fixture-project-e0-v1
logical_root = fixtures/e0/project/main
selected_profile = fixture-retail-120100-e0-v1
analyzer_fixture_contract_id = wow-emmy/e0-c/1
```

### Invariants

- profile/reference generation are exact and non-floating;
- analyzer pin/probe/config identity is accepted and non-null after implementation starts;
- logical root is normalized and contains no host-specific prefix;
- project ID/kind cannot masquerade fixture as release repository;
- capability/budget policies are explicit;
- no local machine/editor/client state is implicit configuration.

## 3. Project source origin

```text
ProjectSourceOrigin
    origin_id: ProjectSourceOriginId
    project_id: ProjectId
    workspace_id: ProjectWorkspaceId
    origin_kind: fixture_project | repository_project
    logical_root: String
    revision_identity: String
    project_generation: optional ProjectGenerationId
```

Before generation derivation, the origin is a validated declaration. A published snapshot binds it to one project generation.

## 4. Input inventory

```text
ProjectInputInventory
    project_id
    workspace_id
    declared_file_order: ProjectFileId[]
    files: ProjectInputFile[]
    inventory_digest: ContentDigest
```

```text
ProjectInputFile
    file_id: ProjectFileId
    source_origin_id
    workspace_id
    normalized_relative_path
    language_kind: lua
    role: first_party_main
    content_digest
    byte_length
    source_fixture_ref: optional String
    canonical_content_bytes: supplied outside the public manifest
```

E0 file IDs:

```text
project-file:main/clean.lua
project-file:main/generic-error.lua
project-file:main/missing-api.lua
project-file:main/secret-local.lua
```

### Invariants

- every file belongs to the declared project origin/workspace;
- path is unique, root-relative, slash-normalized, and inside root;
- content is valid UTF-8 and digest/length verified;
- only Lua first-party Main files in E0;
- no duplicate/incompatible file identity;
- no Library file in project inventory;
- declared ordering is deterministic and used only where semantically declared; generation canonicalization sorts by file ID/path unless explicit order is an input.

## 5. Analyzer binding declaration

```text
AnalyzerBindingDeclaration
    analyzer_contract_id
    accepted_pin_id
    compatibility_probe_report_id
    analyzer_configuration_digest
    required_workspace_role: Main
    required_capabilities: CapabilityId[]
    expected_library_contract_id
```

E0 required analyzer capabilities:

```text
emmy.session.ready
emmy.library.loaded
emmy.file.parsed
emmy.file.diagnostics
emmy.fact.references
emmy.fact.calls
emmy.fact.local_bindings
emmy.fact.local_flow
emmy.fact.operations
emmy.fact.guards
emmy.fact.control_flow
emmy.source_coordinates.exact
emmy.incremental.update
```

Some capabilities may be unavailable per file; project publication remains possible only according to the explicit capability policy and coverage state.

## 6. Capability policy

```text
ProjectCapabilityPolicy
    mandatory_for_publication: CapabilityId[]
    degradable_for_publication: CapabilityId[]
    explicitly_deferred: CapabilityId[]
```

E0 example:

Mandatory:

```text
project.fixture.configuration.valid
project.fixture.files.complete
project.source.registry.complete
project.generation.coherent
emmy.session.ready
emmy.source_coordinates.exact
```

Degradable/explicit per-file:

```text
emmy.file.diagnostics
emmy.fact.references
emmy.fact.local_flow
```

Deferred:

```text
project.toc.complete
project.xml.complete
project.load_graph.complete
project.graph.complete
```

Policy determines whether a coherent degraded snapshot can publish. It never converts failed capabilities into complete/clean results.

## 7. Budget policy

```text
ProjectBudgetPolicy
    max_files
    max_total_source_bytes
    max_single_file_bytes
    max_update_operations
    max_analyzer_facts
    max_generic_findings
    max_output_bytes
```

Budget values use `wow-core BudgetSpec` and are part of configuration/generation identity when they can affect analysis output.

## 8. Project file record

A published snapshot contains:

```text
ProjectFileRecord
    file_id
    source_origin_id
    workspace_id
    normalized_relative_path
    language_kind
    role
    content_digest
    byte_length
    project_generation
    analyzer_file_id
    source_handle_base
    capability_coverage_ids[]
```

`source_handle_base` identifies the file without a span. Exact span handles are produced/validated through the `wow-emmy` coordinate adapter against this record.

## 9. Generation candidate

```text
ProjectGenerationCandidate
    candidate_id
    project_configuration_digest
    selected_profile_id
    reference_generation
    analyzer_pin_id
    analyzer_configuration_digest
    final_file_manifest_digest
    project_schema_version
    derived_project_generation_id
    derivation_input_digests[]
```

Deriving the ID does not mean publication succeeded.

## 10. Project update request

```text
ProjectUpdateRequest
    expected_current_project_generation: optional ProjectGenerationId
    target_configuration: ProjectConfiguration
    file_operations: ProjectFileOperation[]
    requested_capability_policy: ProjectCapabilityPolicy
```

```text
ProjectFileOperation
    Add(ProjectInputFile + bytes)
    Update(file_id + expected_old_digest + new bytes/digest/length)
    Remove(file_id + expected_old_digest)
```

The request resolves to a final candidate inventory/configuration before generation derivation.

## 11. Analyzer update request binding

```text
ProjectAnalyzerUpdate
    target_project_generation
    expected_previous_analyzer_snapshot_id: optional
    analyzer_configuration_digest
    workspace declaration
    file operations/final manifest identities
```

This is converted to the public `wow-emmy AnalyzerUpdateBatch`. Project layer does not construct upstream analyzer types.

## 12. Analyzer snapshot binding

```text
ProjectAnalyzerBinding
    analyzer_snapshot_id
    project_generation
    profile_id
    reference_generation
    analyzer_pin_id
    analyzer_configuration_digest
    workspace_digest
    analyzer_file_manifest_digest
    analyzer_capability_coverage_ids[]
    analyzer_fact_set_ids[]
    generic_finding_ids[]
```

### Invariants

- exact project/profile/reference generation match;
- accepted pin/probe/config identity match;
- analyzer Main file manifest exactly corresponds to project file manifest;
- no extra/missing project Main file;
- Library workspace identity matches declared analyzer fixture/dependency policy but is not included as project file;
- all analyzer source handles for Main files resolve against project source registry;
- all required capabilities have explicit coverage.

## 13. Project capability records

Project-owned E0 capabilities/partitions:

```text
project.fixture.configuration.valid
    partition: project.configuration

project.fixture.files.complete
    partition: project.workspace:workspace:main:e0

project.source.registry.complete
    partition: project.source-origin:project-origin:fixture-project-e0-v1

project.generation.coherent
    partition: project.generation:<project-generation-id>

project.analyzer.snapshot.available
    partition: project.analyzer:<analyzer-snapshot-id>

project.analyzer.facts.available
    partition: project.file:<project-file-id>

project.analyzer.generic_diagnostics.available
    partition: project.file:<project-file-id>
```

Analyzer-produced coverage IDs are retained and referenced; project-owned records describe binding/publication state rather than duplicating analyzer coverage.

Deferred E2 records use `NotEvaluated`/typed unavailable state with reason `operation_not_implemented_for_milestone`.

## 14. Project snapshot candidate

```text
ProjectSnapshotCandidate
    project_generation_candidate
    project_configuration
    source_origin
    project_file_records[]
    analyzer_binding
    project_coverage_records[]
    deferred_capability_records[]
    publication_checks[]
    canonical_snapshot_digest
```

Candidate is mutable only inside the publication transaction and not exposed to consumers.

## 15. Published project snapshot

```text
ProjectSnapshot
    snapshot_schema_version
    project_generation
    selected_profile
    reference_generation
    project_configuration_digest
    source_origin
    project_file_manifest
    analyzer_binding
    project_coverage_records
    deferred_capability_records
    publication_status: Published
    canonical_snapshot_digest
```

The snapshot is immutable and content-addressable.

## 16. Project view

```text
ProjectView
    project_snapshot_identity()
    project_configuration()
    file_manifest()
    source_origin()
    file_by_id(file_id)
    file_by_path(relative_path)
    analyzer_snapshot_identity()
    analyzer_fact_sets(required capabilities/filter)
    generic_findings(filter)
    project_coverage_records(capability/partition filter)
    deferred_capabilities()
```

The view exposes no mutation and no raw upstream analyzer handle.

## 17. Publication result

```text
ProjectPublicationResult
    Published(ProjectSnapshot)
    Rejected(ProjectPublicationFailure)
    Cancelled
```

A rejected target generation is not published, even if a candidate ID was derived.

## 18. Last-known-good record

```text
LastKnownGoodProjectSnapshot
    snapshot
    retained_reason
    superseded_by_candidate_id: optional
    current_for_target_generation: false
```

It can be returned by status/degradation logic only with original identity and explicit staleness relation.

## 19. Update case

```text
ProjectUpdateCase
    case_id
    base_snapshot_id
    update_request
    expected_target_generation_inputs
    expected_publication_outcome
    expected_file_manifest
    expected_analyzer_behavior
    expected_coverage
    expected_last_known_good_behavior
```

## 20. Canonical ordering

- configuration object keys canonicalized by core JSON rules;
- files sort by `ProjectFileId`, then normalized path;
- operations canonicalized by target file ID/type only after validating no conflicting order-sensitive batch;
- coverage sorts by capability/partition/producer/ID;
- deferred capabilities sort by capability ID;
- analyzer fact/finding order remains owned by analyzer contract;
- no timestamp/temp path/discovery order in snapshot bytes.

## 21. Source/evidence separation

Project snapshot may reference:

```text
project source handles/evidence from Main files
analyzer facts/generic findings for those files
```

It does not create or contain:

```text
platform/reference evidence
restriction facets
WoW diagnostic conclusions
replacement candidates
runtime evidence
```

Those are joined later through other crates/service orchestration.
