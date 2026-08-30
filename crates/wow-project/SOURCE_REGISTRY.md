# Project source registry

**Status:** normative E0-D first-party source identity contract.

The registry maps one project generation's logical file identities to exact content. It is not a generic filesystem capability.

## 1. Registry identity

```text
ProjectSourceRegistry
    registry_id
    project_id
    project_generation
    source_origin
    workspace_id
    file_records[]
    canonical_digest
```

One published project snapshot owns one immutable registry.

## 2. Source origin

E0 origin:

```text
origin_id: project-origin:fixture-project-e0-v1
origin_kind: fixture_project
project_id: fixture-project-e0-v1
workspace_id: workspace:main:e0
logical_root: fixtures/e0/project/main
```

The logical root is a public identity namespace, not a host path to open.

For a later repository project, origin may include a pinned repository/revision identity, still separate from local checkout location.

## 3. Project file ID

```text
ProjectFileId
    derived from:
        project/source origin ID
        workspace ID
        canonical normalized relative path
        file identity schema version
```

Content digest is not part of logical file ID. The same logical file across updates retains its ID while the generation/file record changes.

## 4. Path normalization

Accepted E0 path grammar:

```text
non-empty UTF-8
slash separators
relative to logical project root
no `.` or `..` segments after normalization
no empty segment except none
no leading slash
no trailing slash for files
no NUL/control/device namespace
case preserved
```

Reject:

```text
C:\...
/absolute/path
\\server\share
\\?\device
../escape
main/../../escape
file://...
https://token@...
percent-encoded traversal when decoded by an adapter
```

Case-collision policy must be explicit and deterministic across platforms. Recommended E0 fixture policy: exact-case canonical paths plus rejection of case-fold collisions.

## 5. File record

```text
ProjectFileRecord
    file_id
    origin_id
    workspace_id
    relative_path
    language_kind
    source_role
    content_digest
    byte_length
    project_generation
    analyzer_file_id
    source_handle_base
    coverage_ids[]
```

E0 `source_role` is exactly `first_party_main`.

## 6. Content registration

A file record can be registered only when:

- bytes supplied explicitly by fixture/harness/update request;
- valid UTF-8;
- digest and byte length verified;
- path/root/role/language valid;
- file count/byte budgets permit;
- no ID/path/case collision;
- target project generation known;
- analyzer file identity can be bound during publication.

Registry does not discover or read files by itself.

## 7. Base source handle

```text
ProjectFileSourceBase
    source origin ID
    project generation
    relative path
    content digest
    file/entity key
    span: unknown/full-file optional according to core contract
```

Exact span handles are built/validated by `wow-emmy` source-coordinate adapter against the base/file record.

A base handle is identity/evidence plumbing, not host filesystem authorization.

## 8. Analyzer integration

`wow-project` supplies:

```text
registered project source origin
ProjectFileId
normalized relative path
content digest/length
project generation
```

`wow-emmy` returns:

```text
AnalyzerFileId
exact span SourceHandle(s)
facts/findings bound to same project generation/content
```

Project publication verifies:

- every Main analyzer file maps one-to-one to a ProjectFileRecord;
- no extra/missing Main file;
- every project span handle uses the project origin/file/digest/generation;
- no Library handle claims project origin/role;
- no absolute URI/path leaks.

## 9. Lookup operations

```text
file_by_id(file_id)
file_by_path(relative_path)
resolve_source_base(file_id)
validate_project_source_handle(source_handle)
list_project_files()
contains_current_file(file_id, digest)
```

All operations are snapshot-bound and deterministic.

`file_by_path` parses/normalizes the exact canonical grammar. It does not fuzzy-match or search host filesystem.

## 10. Removed/updated files

### Update

- same logical file ID;
- new content digest/length/project generation;
- old handles remain valid only in old snapshot;
- new snapshot rejects old digest/span handles.

### Remove

- file absent in new registry;
- current lookup fails with typed absent-for-generation result;
- analyzer facts/findings absent in current snapshot;
- old snapshot remains immutable/resolvable under old generation when retained.

### Add

- new file ID/path within same origin/root;
- only visible after atomic publication.

## 11. Main versus Library

Project registry contains first-party Main files only.

Library declarations are tracked by `wow-emmy` workspace/session contract and validated through analyzer binding identity. They cannot:

- appear in project file inventory;
- become primary project finding source;
- use project origin ID;
- affect project file completeness counts as first-party files.

## 12. Origin/provenance classification

Project source evidence uses project/fixture origin provenance eligible under `wow-core` policy.

It does not claim:

```text
platform_source
reference-pack source
runtime probe
external implementation
```

A future WoW rule finding may reference both project and reference evidence independently.

## 13. Registry capabilities

```text
project.source.registry.complete
    exact registry structure/file mapping complete

project.fixture.files.complete
    all declared project inputs registered

project.source.handle.resolve
    exact file/span handle mapping available
```

An invalid/missing file or path budget can degrade/fail exact partitions. Empty registry output under failure is not complete.

## 14. Security and privacy

Public registry excludes:

- local checkout root;
- username/home directory;
- drive/UNC/device path;
- Git credentials/tokenized URL;
- temporary extraction path;
- raw source text by default;
- SavedVariables/log/client paths.

Errors include file ID/relative path/digest safely, not arbitrary source payload.

## 15. Canonicalization

Registry canonical bytes sort file records by `ProjectFileId`, then relative path.

Canonical identity includes:

```text
registry schema
project/origin/workspace IDs
project generation
file IDs/paths/roles/language/digests/lengths
```

It excludes analyzer object IDs unless required only in published snapshot binding; analyzer file IDs are supplementary binding fields and canonicalized explicitly if included.

## 16. Required tests

- valid four-file registry;
- shuffled input discovery same digest;
- traversal/absolute/UNC/device paths rejected;
- case-fold collision rejected;
- invalid UTF-8/digest/length rejected;
- duplicate ID/path rejected;
- Library file insertion rejected;
- analyzer one-to-one mapping;
- Library handle cannot use project origin;
- update keeps logical ID/new digest/generation;
- removed file absent current/available old snapshot;
- stale source handle rejected after update;
- temp root does not change public registry;
- no host/private path in errors/output.

## 17. Hard stops

- no filesystem open/list/watch API in registry;
- no symlink following;
- no fuzzy path matching;
- no absolute path public identity;
- no Main/Library role mixing;
- no content without digest/generation;
- no source handle accepted across generations by path alone;
- no project source evidence upgraded to platform authority.
