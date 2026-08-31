# Blizzard UI source acquisition, pinning, and provenance

**Status:** normative boundary between external materialization and E3-A indexing.

## Library input

E3-A accepts only a closed `BlizzardUiSourceSnapshot`. It does not accept a repository URL, branch, tag, latest-build request, installed client root, CDN path, or network callback.

A materializer must provide:

```text
provider/repository/source-origin metadata
exact commit/tree/archive revision where applicable
exact configured roots
complete ordered file inventory
canonical path and case records
content object ID/digest/length for every included file
explicit excluded/ignored/unsupported records
symlink/reparse/submodule/LFS/external-reference records
license/provenance records
materializer version/config/security report
complete/partial/failed coverage
canonical snapshot digest
```

## Candidate provider

`Gethe/wow-ui-source` may be used as a mirror/provider fixture. Its default branch, newest commit, repository name, or release label must never be used as a floating production input.

Before implementation freeze:

- exact repository full name/provider class;
- exact commit and tree SHA or equivalent archive identity;
- complete selected-root content manifest;
- provider-to-client build/profile assertion and evidence;
- license/provenance/redistribution classification;
- materializer implementation/config/version;
- source snapshot and fixture digests.

## Source authority class

Mirror source is classified as implementation-source evidence for the exact bytes observed. It is not automatically:

- official API documentation;
- proof of runtime code loaded by a particular client session;
- proof of event payload accessibility;
- proof of Secret/taint/protected/forbidden behavior;
- proof that an implementation path is public or supported;
- permission to redistribute the source.

The profile records the exact evidence class and confidence.

## Root confinement

Configured roots are explicit normalized relative roots inside the materialized snapshot. Resolution rejects:

- absolute paths;
- `..` escapes;
- drive/UNC/device paths;
- NUL/control characters;
- case-fold collisions under the target profile;
- paths that resolve outside the snapshot;
- implicit symlink/reparse/submodule traversal;
- unmaterialized LFS pointers treated as source bytes.

## Inventory completeness

Every entry under configured roots is one of:

```text
IncludedKnown
IncludedUnknownKind
ExcludedByReviewedProfile
UnsupportedSpecialEntry
ExternalReferenceNotMaterialized
Conflict
Failed
```

Silently omitted files are forbidden. Unknown extensions remain inventory records even when no parser consumes them.

## Byte policy

- Hash exact materialized bytes before normalization.
- Keep raw-byte identity separate from decoded-text identity.
- Record encoding/BOM/line-ending normalization and loss.
- Reject unsupported/ambiguous decoding or mark affected text/parser capability `NotEvaluated`.
- Do not use Git blob SHA as the only content identity; freeze canonical cryptographic content digests.
- Content-addressed objects are local persistence details and do not grant redistribution rights.

## Build/profile assertion

The materializer or caller supplies a reviewed compatibility assertion linking source snapshot to a target client product/flavor/build/Interface/ReferenceProfile. E3-A validates consistency but does not guess it from dates, branch names, latest commits, or package metadata.

Conflicting build labels block publication or produce an explicitly allowed partial candidate; they never trigger cross-version merging.

## Determinism

Equivalent byte inventories and profiles yield the same source snapshot identity regardless of:

- clone directory;
- fetch time;
- host OS;
- Git checkout order;
- filesystem enumeration;
- provider network state;
- archive extraction temp root;
- worker count.

## Materializer tests

Required cases include:

- exact clean snapshot;
- reordered file enumeration;
- case collision;
- symlink/junction escape;
- submodule/LFS pointer;
- malformed/unknown bytes;
- excluded file accounting;
- incomplete archive;
- provider rename with identical bytes;
- branch advances after materialization;
- build/profile mismatch;
- license unknown/conflict;
- cancellation and budget exhaustion.
