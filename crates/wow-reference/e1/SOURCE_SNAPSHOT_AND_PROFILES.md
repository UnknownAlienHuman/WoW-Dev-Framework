# Source snapshots and reference profiles

**Status:** normative E1-B input identity, provenance, ordering, eligibility, and isolation contract.

## 1. Acquisition boundary

`wow-reference` does not clone/download/update source. A higher tool/provider materializes a source tree and supplies:

```text
trusted configured snapshot root (private runtime path)
SourceSnapshotManifest
file content set
provider/acquisition/licensing records
```

The crate verifies the manifest against actual bytes under the root before ingestion.

## 2. Provider versus content identity

```text
ProviderProvenanceRecord
    provider kind/name
    repository/export/source URL or identity
    revision/branch/tag/build labels
    acquisition tool/version
    acquired-at supplemental timestamp
    provider-reported metadata
    license/notice refs
```

```text
MaterializedContentIdentity
    canonical file manifest
    normalized snapshot-relative paths
    per-file digest/length/encoding
    logical content-set digest
```

Provider records are provenance. `MaterializedContentIdentity` + exact profile data backs reference facts.

Equivalent logical file sets from two providers may produce the same source/content/reference identity while retaining distinct provenance records.

## 3. Snapshot root validation

- root is configured/trusted by caller, not source text;
- all manifest paths root-relative/slash-normalized;
- reject absolute/traversal/device/unsafe link/reparse escape;
- reject duplicate normalized paths/case-collision according to target platform-independent policy;
- reject missing/unlisted required files;
- extra files classified by explicit ignore/optional/unexpected policy;
- verify byte lengths/digests before parsing;
- enforce total/file count/size budgets;
- public manifests/errors do not expose absolute root.

## 4. Canonical file manifest

```text
SnapshotFileManifestEntry
    source_file_id
    normalized path
    content sha256
    byte length
    encoding/BOM/line-ending observations
    file kind/dialect
    declared semantic partition/order
    required/optional/ignored status
    license/provenance refs
```

Canonical content-set digest uses normalized entries and semantic manifest version, not filesystem mtime/inode/order.

## 5. Build/profile identity

```text
ReferenceProfile
    stable ProfileId
    flavor/edition/project ID
    Interface number
    client version/build
    source content manifest/digest
    snapshot ID
    parser/evaluator/normalizer versions
    schema versions
    correction-set ID
    input partition policy
    eligibility class
```

Rules:

- no bare `current`/`latest` profile ID;
- build/Interface/flavor/source digest contradictions reject;
- same Interface with different source/build can be distinct profiles/generations;
- PTR/beta/live/classic remain separate;
- one ReferenceData generation selects one profile only;
- profile identity changes when source/correction/schema/parser semantics change as defined.

## 6. Eligibility classes

### `fixture`

Synthetic/minimal corpus, incomplete by design, never release authority.

### `candidate`

Real materialized source but one or more release gates pending/partial.

### `release`

All mandatory identity/input/license/parser/evaluator/normalizer/schema/correction/store/integrity gates satisfied for declared capabilities.

Eligibility is explicit and cannot be inferred from build label/provider or successful parsing alone.

## 7. Input partition policy

A versioned policy maps source files/registrations to capability partitions and semantic order.

```text
InputPartitionPolicy
    policy_id/version
    profile/flavor applicability
    partitions[]
    file selection/mapping rules
    semantic ordering rules
    required/optional dependencies
    completeness criteria
    expected systems/registration families
```

No broad filesystem glob without manifest validation. Rules may be exact path/prefix/TOC-derived/source-registration classifications, but must be deterministic and tested against the pinned snapshot.

## 8. File and registration order

Some generated APIDocumentation semantics depend on declared load/registration order.

Ordering sources, in authority order:

1. exact source/TOC/declaration order proven in selected snapshot;
2. exact manifest policy order for independently generated files;
3. source span order within a file;
4. canonical ID/path order only when semantics explicitly unordered.

Never rely on directory enumeration, Git API result, thread completion, or hash-map order.

The build report records which ordering source applied.

## 9. Required versus optional inputs

### Required

Missing/failing input blocks declared capability completeness and may reject release eligibility.

### Optional

Absence is explicit `NotApplicable`/not selected; failure may make only optional capability unavailable.

### Unexpected

File/registration not covered by policy is recorded and may downgrade completeness/security until reviewed.

### Ignored

Ignored only through explicit versioned rule and reason, with digest/count evidence. No silent “not relevant” filtering.

## 10. Snapshot changes

Any byte/path/file-set/order/profile identity change produces a new snapshot/content identity. A new ReferenceGeneration build may reuse unchanged content-addressed objects/facts only through an exact incremental/reuse contract; E1-B first implementation may rebuild deterministically.

No mutation of an existing snapshot/profile/generation identity.

## 11. Licensing and redistribution

Snapshot manifest records licenses/notices/provenance. Raw Blizzard UI/source redistribution policy and final Reference Pack artifact contents require explicit review. E1-B may store bounded derived/raw metadata objects as allowed by the accepted project policy; it does not assume all source bodies can be redistributed.

Source handles can identify provider/revision/path without embedding full source.

## 12. Current KB routing

The external KB may identify the current target build/source. Before releasing a profile, revalidate against current route and exact source. The resulting profile remains pinned and durable; later live changes create another profile/generation rather than mutating it.

## 13. Profile selection

Build request requires exact `ReferenceProfile`. ReferenceView requires exact `ProfileId`/`ReferenceGenerationId`. No implicit selection from:

```text
local WoW installation
TOC in an unrelated addon
editor settings
current Git branch/provider HEAD
system clock
last-used profile
```

Higher service/application can present configured choices but must pass exact identity.

## 14. Profile isolation tests

- same symbol with changed signature in two profiles stays separate;
- restriction facets do not leak;
- missing entity in profile A cannot query profile B;
- PTR facts do not satisfy live query;
- correction set/profile mismatch rejects;
- ReferenceStore/profile/generation mismatch rejects;
- same logical source from two providers can converge while preserving provider provenance;
- same provider label with changed bytes creates new content identity;
- fixture profile never release-eligible.

## 15. Required operations

```text
validate_snapshot_root
validate_source_snapshot_manifest
verify_snapshot_file_manifest
compute_materialized_content_identity
validate_provider_provenance
build_reference_profile
validate_reference_profile
classify_profile_eligibility
build_input_partition_policy
validate_input_partition_policy
resolve_ordered_input_partitions
verify_partition_completeness_inputs
```

## 16. Required tests

- valid fixture/candidate/release profile shapes;
- build/Interface/flavor/source contradictions;
- path traversal/link/case collision/duplicate/missing/extra files;
- file digest/length/encoding mismatch;
- provider change same bytes versus same provider changed bytes;
- required/optional/unexpected/ignored partition behavior;
- deterministic semantic order versus randomized filesystem/thread order;
- PTR/live/flavor isolation;
- no absolute root/time/mtime/provider prose in canonical digest;
- release eligibility blocked by missing license/input/parser/correction/store gate.

## 17. Hard stops

- no network/source acquisition;
- no floating profile;
- no provider-as-authority;
- no unverified bytes;
- no filesystem-order semantics;
- no silent extra/missing/ignored input;
- no mixed profiles;
- no fixture release masquerade;
- no public absolute root/path leakage;
- no mutation of pinned snapshot/profile identity.
