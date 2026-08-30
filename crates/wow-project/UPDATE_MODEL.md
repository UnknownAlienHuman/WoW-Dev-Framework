# `wow-project` E0-D update model

**Status:** normative explicit-update contract. E0-D has no filesystem watcher or background scan.

## 1. Update request

```text
ProjectUpdateRequest
    project_id
    expected_current_project_generation: optional
    expected_current_project_snapshot_digest: optional
    target_configuration
    file_operations[]
    request_budget
```

The request describes an intended final logical project state. It does not authorize reading undeclared host files.

## 2. Supported E0 file operations

```text
Add
    new ProjectInputFile declaration
    supplied UTF-8 bytes

Update
    existing ProjectFileId
    expected old content digest
    supplied new UTF-8 bytes/digest/length

Remove
    existing ProjectFileId
    expected old content digest
```

Only first-party Lua files under the configured Main root are supported.

## 3. Unsupported E0 updates

Typed unavailable/rejected:

```text
add TOC/XML/other language
change file to Library role
move outside configured root
add symlink/submodule/external root
scan directory automatically
load installed addon/client files
mutate editor settings
run generator/build script
apply update from floating Git branch
change profile/reference implicitly
```

Profile/reference/analyzer pin/config changes require an explicit target `ProjectConfiguration`, not an incidental file update.

## 4. Preconditions

Before deriving target state:

- project ID matches current project;
- expected current generation matches when supplied/required;
- expected current snapshot digest matches when supplied;
- every update/remove expected old file digest matches;
- add target does not already exist;
- update/remove target exists;
- no duplicate/conflicting operations for one file;
- all paths/types/roles/budgets validate;
- target configuration is explicit and internally coherent.

A stale precondition aborts before analyzer mutation.

## 5. Conflicting operation rules

Reject a batch containing ambiguous order-dependent operations such as:

```text
Add + Add same file
Update + Update same file with different expected bases
Remove + Update same file
Add existing file
Remove missing file
path collision under normalization/case policy
```

Do not silently sort and choose a winner.

A multi-step user intent must be normalized by the caller into one unambiguous final-state operation or separate generation transactions.

## 6. Final-state construction

```text
validate current snapshot and request
-> clone logical configuration/file manifest into candidate builder
-> apply operations to builder only
-> validate final configuration/source registry/file inventory
-> verify supplied bytes/digests/lengths
-> canonicalize final manifest
-> derive target ProjectGenerationCandidate
```

The current published snapshot remains immutable.

## 7. Analyzer batch construction

Map project operations to `wow-emmy AnalyzerUpdateBatch` using:

```text
target ProjectGenerationId
expected previous analyzer snapshot ID
accepted analyzer configuration digest
Main workspace declaration
exact Add/Update/Remove operations
registered project file/source identities
```

No upstream analyzer type escapes or enters the project public API.

The analyzer update batch must lead to the exact candidate final file manifest. Extra/missing files are publication errors.

## 8. Analyzer response validation

After analyzer publication, validate:

- exact target project generation;
- exact selected profile/reference generation;
- accepted pin/probe/config identity;
- exact Main workspace/file manifest and content digests;
- expected add/update/remove effects;
- no current facts/findings for removed file;
- source handles resolve against candidate source registry;
- capability/coverage records present;
- snapshot itself valid.

Failure aborts project publication.

## 9. Update result

```text
ProjectUpdateResult
    Published(ProjectSnapshot)
    Rejected(ProjectUpdateFailure)
    Cancelled(ProjectUpdateCancellation)
```

A rejected result includes:

```text
candidate generation ID when safely derivable
failed operation/phase
expected and observed generation/digest IDs
file IDs
capability/partition blockers
analyzer failure/root-cause reference
last-known-good snapshot identity when retained
```

No source contents or host paths in default error output.

## 10. Add semantics

Add validates:

- canonical new file ID/path;
- path under registered root;
- role/language allowed;
- bytes valid UTF-8;
- digest/length correct;
- file count/byte budgets;
- no normalized path collision.

A successful add creates a new project generation and analyzer file. Its facts/findings are available only after full publication.

## 11. Update semantics

Update validates exact logical file identity and expected old digest.

Rules:

- same bytes/digest may become a deterministic no-op result or rejected no-op according to explicit API policy; it must not create an arbitrary different generation;
- source spans/facts/findings from old content cannot remain current without recomputation/proof;
- new byte length/digest enter final manifest;
- analyzer invalidation follows E0-C contract;
- publication occurs only after snapshot match.

## 12. Remove semantics

Remove validates exact current file/digest.

After successful publication:

- file absent from project/analyzer current manifests;
- all current source handles/facts/findings for removed content invalid;
- source registry cannot resolve removed file for current generation;
- old immutable snapshot may still resolve it under old generation;
- project capabilities/coverage update accordingly.

## 13. Configuration/analyzer change

An explicit configuration update may change:

```text
selected profile/reference generation
accepted analyzer pin/probe identity
analyzer configuration digest
capability/budget policy
logical root/source origin policy
```

Rules:

- requires a full candidate generation derivation;
- analyzer must reconfigure/rebuild as probe/adapter requires;
- all returned state binds to the new generation;
- no old analyzer snapshot reused without exact compatibility proof;
- a pin/config update cannot be hidden inside an ordinary file update.

## 14. No-op policy

The E0 implementation must choose and document one deterministic policy:

### Recommended

If target configuration and final canonical file manifest equal current inputs:

```text
outcome = NoChange
current ProjectSnapshot returned by identity
no analyzer mutation
no new generation ID
```

NoChange is not a fake update success; it explicitly states no semantic state changed.

## 15. Cancellation

Cancellation before project publication:

- publishes no target project snapshot;
- does not relabel prior snapshot;
- if analyzer mutation cannot be safely rolled back, session/snapshot health follows E0-C failure contract and project reports exact state;
- retry requires the same expected current generation or a fresh request.

No background continuation after cancellation.

## 16. Budget enforcement

Validate before expensive work where possible:

```text
file operation count
final file count
total source bytes
single-file bytes
expected analyzer fact/finding/output budgets
```

Budget exceedance is typed rejection/partial capability only according to explicit policy. It cannot become truncated clean project state accidentally.

## 17. Determinism

Two valid update sequences ending at identical final semantic inputs must yield identical:

```text
final project configuration/file manifest
ProjectGenerationId
analyzer final snapshot identity/output (under deterministic adapter)
ProjectSnapshot canonical bytes/digest
```

Tests permute independent operation order. Conflicting operations are rejected instead of canonicalized.

## 18. Required operations

```text
validate_project_update_request
validate_project_file_operation
apply_operations_to_candidate_manifest
canonicalize_final_project_state
classify_no_change
build_analyzer_update_batch
validate_analyzer_update_effects
commit_project_publication
abort_project_publication
```

## 19. E0 update cases

- baseline initial publish;
- update `main/generic-error.lua` to clean;
- update `main/missing-api.lua` to `KnownApi`;
- remove/add one optional synthetic Main Lua file within budget;
- stale expected project generation;
- stale expected file digest;
- path traversal/role/type violation;
- conflicting same-file operations;
- analyzer update failure;
- analyzer returns wrong generation/file digest;
- cancellation;
- deterministic no-op;
- different valid operation order, same final state.

## 20. Hard stops

- no automatic file watching/scanning;
- no stale overwrite;
- no order-dependent conflict resolution;
- no analyzer mutation before cheap precondition validation;
- no current pointer change before full publication;
- no old facts/spans on new content;
- no removed file retained in current registry;
- no partial target snapshot combined with last-known-good data;
- no implicit profile/pin/config update;
- no arbitrary source execution.
