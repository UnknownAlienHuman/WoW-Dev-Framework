# Reference Pack assembly and layout

**Status:** normative E1-D pack topology and member closure contract.

## Logical E1 layout

A default E1 layout profile may materialize:

```text
manifest.json
checksums.json
reference/
    reference.sqlite
    reference-data-manifest.json
    reference-build-report.json
annotations/
    Annotations/Core/...
    semantic-manifest.json
    file-manifest.json
    source-map.json
    projection-coverage.json
    projection-loss.json
    parity-report.json
    consumer-manifest.json
    consumer-probes/
    artifact-manifest.json
provenance/
    source-snapshot.json
    component-versions.json
    licenses.json
licenses/
    ...required notices...
```

Exact paths and required members are versioned by `ReferencePackLayoutProfile`. No code relies on this illustrative layout without a frozen profile.

## Member ownership

```text
wow-store       sealed ReferenceStore and store integrity reports
wow-reference   ReferenceData manifest/build reports and source/provenance references
wow-annotations rendered annotation files and semantic/map/loss/parity/consumer manifests
wow-service     pack-level layout/member/gate/checksum/build/validation manifests
application     safe staging/final filesystem materialization only
```

## Manifest identity

Pack ID is domain-separated and derived from stable logical inputs such as:

- profile/reference generation;
- source snapshot content identity;
- component implementation/schema/profile identities;
- component artifact manifest IDs;
- layout profile ID;
- ordered member logical identities/digests;
- license/provenance manifest IDs;
- pack contract version.

Exclude pack ID fields containing themselves, destination paths, timestamps, host, temp root, worker count, and noncanonical progress.

## Checksum manifest

Contains every materialized file in canonical normalized path order, except where the checksum file's own digest is handled by a versioned noncyclic outer rule. The profile must freeze one approach:

1. checksum all members except `checksums.json`, with pack manifest recording checksum-file digest; or
2. use a two-level manifest/object identity scheme.

Never improvise recursive self-hashing.

## Store file

The sealed `reference.sqlite` is copied or atomically materialized from the exact published immutable store generation. E1-D does not reopen it writable or normalize its bytes.

The pack records:

- logical store generation and manifest IDs;
- file length/digest as materialized;
- physical reproducibility classification;
- read-only open/integrity validation result.

## Annotation tree

All annotation paths come from the E1-C layout profile. Pack assembly may prefix them under `annotations/` through a fixed profile transform; it cannot rename declarations/files ad hoc.

Source maps refer to artifact-relative paths according to the frozen pack/layout mapping and are revalidated after prefix/materialization.

## Raw/source objects

E1 default does not embed full Blizzard source. Optional raw APIDocumentation or bounded source objects require:

- explicit member kind/profile;
- exact object identity/digest;
- license/redistribution approval;
- size/security policy;
- no executable use;
- no full-tree/default model-context exposure.

## Permissions and metadata

Canonical pack semantics do not depend on host owner/group/mtime. Materialization profile sets safe fixed file modes where supported. Executable bits are forbidden for E1 members.

## Archive/container

Directory pack is the E1 correctness baseline. Zip/tar/compression requires a later or separately frozen container profile controlling:

- path order;
- separators;
- timestamps;
- permissions;
- compression implementation/version/options;
- duplicate path and symlink policy;
- decompression limits;
- outer checksums.

No archive-byte reproducibility claim without that profile.

## Extensions

Unknown top-level members are rejected in E1 unless a versioned extension registry says:

- member kind and owner;
- schema and compatibility;
- required/optional policy;
- checksum/license/security behavior;
- impact on pack identity/eligibility.

## Tests

- exact required member set;
- optional/deferred member policy;
- path normalization/case collisions;
- checksum recursion policy;
- store and annotation prefix mapping;
- extra/missing/duplicate member;
- license/provenance per member;
- no executable bit/source tree/runtime addon packaging;
- deterministic ordered manifest.
