# Materialized platform source input contract

**Status:** normative boundary between acquisition and `wow-project` indexing.

## Closed snapshot requirement

E3-B receives a closed snapshot. Every included entry is known before parsing and has exact root-relative path, type, length, content digest/ObjectId, provider object/member evidence, and source/security/license classification.

The snapshot is immutable. A changed byte, path, role profile, provider adapter, build expectation, exclusion, or license/security decision creates a new snapshot/candidate.

## Required manifest fields

```text
snapshot/provider/materializer IDs
provider trust and provenance evidence
immutable provider revision/tree/archive/client-extract identity
requested and observed build/flavor/profile values
declared source roots
complete ordered entry manifest
content objects/digests/lengths
excluded/unsupported entries and reasons
case/Unicode/path/link/special-file audit
provider inventory observations
license/redistribution state
security/materialization report
budgets/cancellation/completion status
canonical digest
```

## Entry types

```text
regular_file
directory_manifest
symlink_or_link              rejected by default
submodule                    rejected by default
lfs_pointer_or_filter_input  rejected unless explicit materialized-byte profile
special_file                 rejected
unknown                      rejected or explicit unsupported
```

Directories are manifest structure, not source bytes.

## Path contract

- UTF-8/root-relative canonical representation under a frozen profile;
- reject absolute, drive/UNC, `..`, NUL, alternate separators, reserved device names, trailing-dot/space ambiguity, and root escape;
- retain original provider path evidence separately;
- detect exact, case-insensitive, Unicode-normalization, separator, and platform-reserved collisions;
- do not silently case-fold or rename;
- one source entry maps to one validated content object.

## Content verification

Provider Git blob SHA or archive checksum is provenance, not the framework logical ObjectId. Materializer computes framework SHA-256 over exact logical bytes and records both.

- no newline/encoding normalization before content identity;
- no Git clean/smudge/filter/LFS execution;
- no source decompression after manifest finalization;
- archive member bytes verified before snapshot seal;
- existing ObjectId with different bytes is corruption;
- line/UTF-8 projections are derived sidecars and may be partial.

## Root and role policy

Initial mirror fixture permits only explicit roots/files such as:

```text
Interface/AddOns/**
Interface/ui-code-list.txt
Interface/ui-toc-list.txt
Interface/ui-gen-addon-list.txt
version.txt
README.md as provider metadata
```

`.github/**`, hooks, workflows, scripts, repository metadata, and unrelated files may be inventoried under ignored metadata but never become parser/analyzer input.

## Inventory reconciliation

Reconcile exact manifest entries, provider inventory lists, selected TOCs, and XML includes/scripts. Record matches, missing-on-each-side, duplicates, malformed entries, path/case differences, and unsupported types. A provider list cannot add a missing byte or erase an extra file.

## Build observation reconciliation

Record requested profile/build, branch label, immutable commit/tree, commit message, `version.txt`, TOC fields, generated-doc metadata, and any local extract/runtime observation separately. Conflict remains explicit and blocks stronger claims.

## License/redistribution

Each root/entry has evidence state:

```text
ExplicitPermitted
ExplicitRestricted
ProviderStatementOnly
Unknown
NotApplicableMetadata
```

No repository-root license evidence means `Unknown`, not unrestricted redistribution.

## Security completion

Snapshot cannot be `Complete` when a selected root has missing bytes, digest mismatch, unreviewed link/submodule/filter/special file, path collision/escape, archive budget breach, incomplete pagination/download, required policy evidence missing, cancellation, or ambiguous completion.

## Library boundary

`wow-project` receives typed manifests and content-object readers only. It does not receive credentials, network client, Git command, archive extractor, checkout path, or provider session.

## Required tests

Exact complete synthetic snapshot; branch movement; Git blob versus logical SHA-256; path and Unicode attacks; links/submodules/LFS/special files; archive duplication/bombs; workflow nonexecution; manifest/list/TOC/XML disagreement; build conflict; license states; cancellation/response loss; shuffled provider ordering; and no provider handle crossing the boundary.
