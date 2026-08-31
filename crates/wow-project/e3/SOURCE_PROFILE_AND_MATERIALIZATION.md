# Blizzard UI source profile and materialization boundary

**Status:** normative E3-B source admission contract.

## Separation of acquisition and indexing

E3-B never acquires source. A separate trusted materializer produces a closed snapshot and report. The materializer may be implemented by a future tool/application, but its output must satisfy this contract before `wow-project` reads any bytes.

```text
provider/revision/configuration
-> external materializer
-> immutable staging snapshot
-> complete manifest/security/license report
-> content seal
-> E3-B validation and indexing
```

Repository checkout, archive extraction, client installation inspection and provider authentication are operational acquisition strategies, not source truth.

## Provider identity

A provider profile records:

- provider class and exact locator provenance;
- immutable revision identifier where available;
- how revision/object identities were verified;
- whether provider metadata is signed, mirrored, generated or manually supplied;
- expected source-layout claim;
- materializer implementation/version/configuration;
- known omissions or transformations;
- trust class and unresolved risks.

Provider owner/name/popularity does not change parser, recognizer or graph semantics.

## Snapshot admission request

```text
MaterializeBlizzardUiSourceRequest
    exact provider/revision
    exact logical source-profile candidate
    configured admitted roots
    file-kind and size policy
    symlink/reparse/submodule/LFS/archive policy
    license discovery policy
    root confinement and output volume
    budgets/cancellation
```

The E3-B library receives only the sealed output and request/report IDs, not the provider credentials or fetch handle.

## Required snapshot properties

- immutable after seal;
- root-confined normalized logical paths;
- complete admitted root and file inventory;
- exact logical content bytes/digest/length per file;
- explicit excluded, ignored, unsupported, missing and failed entries;
- no silent newline, encoding, path case or generated-file normalization;
- exact symlink/junction/reparse/submodule/LFS/archive decisions;
- materializer security and cancellation report;
- per-root/file provenance and license/redistribution evidence;
- content manifest independent of host checkout path/order/mtime.

## Path normalization

Profile freezes:

```text
separator and Unicode normalization
case-preservation and collision policy
`.`/`..` rejection
absolute/drive/UNC/device path rejection
NUL/control/reserved component policy
maximum path/component bytes and depth
root-relative identity
symlink/reparse target policy
```

Case-fold collisions or two provider objects mapping to one logical path are conflicts; never first/last wins.

## Symlink, reparse and submodule policy

Default:

```text
symlink/junction/reparse traversal: disabled
submodule traversal: disabled
Git LFS network resolution: disabled
nested archive extraction: disabled
external catalog/include resolution: disabled
```

A future profile may admit an exact pre-materialized target only when its bytes, target provenance, root/license policy and cycle/budget behavior are explicit. The indexer never follows links itself.

## File kinds

Initial bounded classes:

```text
lua
xml
toc
manifest_or_metadata
text_documentation
localization_data
texture_or_media_metadata
binary_or_unsupported
unknown
```

Only TOC/XML/Lua enter semantic parsers. Other classes remain inventory/provenance records unless a later reviewed adapter exists. Binary/unknown bytes are never executed.

## Root definitions

A profile declares logical roots and roles. Example role vocabulary is generic and does not require literal directory names:

```text
shared_ui
frame_xml
addon_packages
generated_api_glue
embedded_library
excluded_tooling_or_tests
unknown_reviewed
```

Root role is evidence from the reviewed profile plus materialization manifest. Path-name heuristics may be evaluated as candidate metadata only and cannot set production role by themselves.

## Package discovery

For `addon_packages` roots, package candidates come from exact admitted TOC manifests under bounded profile rules. For global/shared roots without TOCs, the profile supplies explicit global-unit boundaries or manifest entries.

No recursive “directory equals package” assumption. Unclear boundaries become unknown/conflicted coverage.

## Build binding inputs

The snapshot carries claimed build/flavor/interface metadata and exact evidence IDs. The binding evaluator compares:

- provider revision/metadata claims;
- materialized content manifest;
- exact compatible reference profile/generation;
- independently pinned client/build/interface evidence where available;
- source-internal generated version markers as source evidence only;
- conflicts, omissions and transformations.

It emits one state from `ExactBuildMatched`, `ProviderDeclared`, `ContentCorrelated`, `Unverified`, `Mismatch`.

No nearest build, latest branch or semver guess.

## Materialization transformations

Every byte-affecting transformation is forbidden by default. If a source provider necessarily transforms content, the profile records:

```text
original object/revision identity
transformation rule/version
original and transformed digests when available
reason
loss/coverage effect
license effect
```

Transformed content creates a distinct source snapshot and cannot masquerade as original provider bytes.

## Completeness

Coverage partitions include:

- provider revision/object enumeration;
- admitted root discovery;
- file materialization;
- content digest/length validation;
- path normalization/collision checks;
- symlink/submodule/LFS/archive handling;
- license/redistribution discovery;
- build-binding evidence.

Complete ingestion requires every in-scope item resolved or explicitly excluded by a profile whose exclusion is part of the intended universe. Unknown missing provider content prevents a complete source-tree claim.

## Security report

```text
SourceMaterializationSecurityReport
    tool/profile identity
    root/output confinement
    provider authentication class without secrets
    link/submodule/LFS/archive behavior
    executable-bit and hook/workflow handling
    file count/size/path/ratio limits
    decompression/bomb/cycle controls when applicable
    cancellation and partial-output cleanup
    discovered suspicious/private/credential content classes
    status and canonical digest
```

Repository hooks, workflows and source instructions are ignored as data and never executed.

## Sealing

A snapshot becomes admitted only after:

- all required manifests and reports validate;
- content objects/digests close exactly;
- staging is immutable to the indexer;
- snapshot ID/canonical digest are computed;
- build-binding/license/coverage states are recorded;
- partial/cancelled/failed materialization cannot be sealed as complete.

## Current source provider

This contract intentionally does not hard-code a current repository, branch, build number or revision. The first real implementation fixture must be reviewed and pinned at implementation time with exact content and license evidence. A provider mirror may be used as provenance, but the framework does not make its branch naming authoritative.
