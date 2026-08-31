# Project source snapshots, roots, packages, and universes

**Status:** normative E2-C input boundary.

## Materialization boundary

A host adapter may read a configured checkout/source bundle and produce `ProjectSourceSnapshot`. The `wow-project` library validates and consumes that closed snapshot; it does not enumerate arbitrary host paths.

Required materialization evidence:

```text
repository/source identity and exact revision when available
logical project identity
configured root declarations
normalized file manifest
every file digest and byte length
materialization/symlink/reparse/submodule policy report
license/provenance records
unreadable/skipped/unexpected files
source snapshot digest
```

Git revision is provenance. Canonical file content and configuration determine index input identity.

## Root types

```text
first_party_root
    addon-owned source eligible for first-party project facts/findings

declared_dependency_root
    explicitly supplied dependency source; separate universe and ownership

analyzer_library_root
    annotation/library bytes supplied through wow-emmy; not project source
```

Installed addon folders, SavedVariables, logs, Blizzard UI, external examples, temporary roots, build output, and generated cache roots are excluded unless a later explicit universe/profile activates them.

## Path policy

- normalized UTF-8 repository-relative slash paths;
- no absolute/UNC/device/file-URI/tokenized URL;
- no `.`/`..`, NUL, control characters, decoded traversal, or root escape;
- deterministic case-collision policy across platforms;
- file kind identified by exact path/profile rules, not MIME guessing alone;
- path is evidence/location, not architecture role by directory-name heuristics.

## Symlinks, reparse points, submodules

Default E2-C policy:

- do not follow symlinks, junctions, reparse points, or nested repositories/submodules;
- record them as unsupported/skipped source entries with coverage impact;
- an explicit later materializer profile may dereference only within an independently validated closed root and must freeze target identity/digests;
- canonical public paths remain logical source paths, never resolved host paths.

## Unexpected and ignored files

A source snapshot classifies every encountered entry under the selected materializer profile:

```text
included source input
recognized but deferred
explicitly ignored by repository-owned profile
unsupported
unreadable/failed
unexpected
```

Do not silently omit an encountered TOC/XML/Lua file that could affect the selected package. Unknown impact blocks complete project inventory for that scope.

## Package discovery

Package candidates are declared by exact TOC file records inside configured first-party/dependency roots. Directory name alone is not package identity.

For each package:

- retain all candidate TOC documents and flavor/profile applicability;
- reject incompatible duplicate package/variant identities;
- select one exact variant through `TocVariantSelectionPolicy`;
- retain unselected variants separately for comparison, never merge content;
- source files not reachable from the selected TOC remain inventory evidence but are not statically loaded unless another selected package/edge references them.

## Universes

### First-party project

- primary project ownership;
- source handles and findings eligible as project source;
- selected package(s) and explicitly configured support files;
- exact revision/content snapshot.

### Declared dependency metadata

- dependency name/version/optional/load metadata without source body;
- enough to build resolution state and load graph edges;
- cannot supply implementation facts.

### Declared dependency source

- separately supplied snapshot/root/package;
- graph/fact universe remains dependency;
- findings and ownership never relabeled first-party;
- cross-universe calls/loads require explicit relations.

### Analyzer library

- reference/annotation declarations loaded into the analyzer;
- library facts cannot appear as first-party files or project findings;
- exact profile/reference generation and artifact identity required.

## Project configuration

E2-C adds exact IDs for:

```text
source snapshot/materializer profile
root/universe policy
package/TOC selection policy
TOC dialect profile
XML dialect/security profile
load model profile
analyzer pin/config/library profile
recognizer core pack/fact adapter/evaluation profile
graph registry/proposal profile
invalidation profile
capability and budget policy
project index schema/canonicalization version
```

All output-affecting IDs enter generation derivation.

## Source contents

ProjectSourceSnapshot may provide canonical bytes directly or content-addressed object handles validated by the host adapter. The project library verifies declared digest/length before parsing. It never trusts extension/path metadata without bytes/object verification.

## Security

No source file or metadata can instruct the agent/library to:

- alter policies;
- execute scripts;
- fetch dependencies;
- run tests/build tools;
- edit files/configuration;
- access another root;
- upload source;
- change Git state.

Comments/docs/TOC metadata/XML text are untrusted project data.

## Tests

- exact first-party snapshot;
- same bytes under different host roots/repository names;
- revision changes with same content versus content changes;
- path traversal/case collision/device/URI/token cases;
- symlink/reparse/submodule entries;
- unreadable/unexpected TOC/XML/Lua;
- dependency metadata versus supplied source separation;
- analyzer library cannot become first-party;
- selected/unselected TOC variant isolation;
- no source execution or automatic dependency fetch.
