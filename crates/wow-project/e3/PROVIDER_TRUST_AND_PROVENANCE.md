# Platform source provider trust and provenance

**Status:** normative evidence classification. Provider trust controls labels and eligibility; it never changes source bytes.

## Trust classes

```text
PublisherOfficialSnapshot
LocalClientExtract
PinnedCommunityMirror
UserSuppliedSnapshot
HistoricalArchive
UnknownProvider
```

Exact bytes may be useful implementation evidence while build mapping, completeness, license, or origin remains uncertain.

## Initial fixture classification

`Gethe/wow-ui-source@027d26c3406d3de2cbd2b1f67d468fe033a1bcd4` is:

```text
trust class: PinnedCommunityMirror
branch label: live, provenance only
commit message: 12.1.0 (69497)
version.txt observation: 12.1.0.69497
git tree: b95256b3ebce23fbbef3603d0b5550f7d90cd013
source root: Interface/
repository automation: .github/workflows/export.yml, ignored/not executed
root LICENSE path in pinned tree: not found by current tree audit
redistribution state: Unknown / policy-restricted pending explicit evidence
```

Do not relabel the mirror as Blizzard-official. A KB source-selection policy does not alter provider provenance.

## Provenance chain

```text
ProviderRecord
-> AcquisitionRequest
-> immutable revision/tree/archive/client-build evidence
-> materializer implementation/profile
-> provider object/member receipts
-> logical content objects and complete path manifest
-> MaterializedPlatformSourceSnapshot
-> corpus/project/analyzer/graph/store publications
-> context source pointers
```

Every step retains IDs/digests and completion/conflict state.

## Claim classifications

```text
mirror-source-confirmed
local-extract-confirmed
publisher-source-confirmed
reference-generated-doc-confirmed
project-index-derived
graph-derived
runtime-confirmed
historical
unverified
```

## Provider conflicts

Conflicts include branch/profile mismatch, commit/version disagreement, mirror/local-extract differences, inventory omissions, generated-doc build disagreement, or incomplete acquisition. Link all observations and affected capabilities; do not silently choose a winner.

## Cross-provider comparison

Align exact paths and logical content digests under a frozen profile. Equality does not prove equal provenance/license. Difference does not prove a patch change when build mapping is unresolved.

## Runtime boundary

Only exact runtime/client evidence establishes installed build and observed runtime state. Source metadata cannot establish hotfix/data behavior, combat/restriction context, payload accessibility, taint, or performance.

## Source-pointer format

```text
provider/repository@immutable-revision:path:half-open-byte-span-or-line-projection
exact source symbol when available
content digest/source handle
build/flavor/corpus generation
claim classification
```

## Required tests

Every trust class; mirror never official; same bytes/different provenance; build observation conflicts; incomplete acquisition; branch movement; provider rename/fork; malicious metadata; license states; exact source pointers; and runtime claim attempted from source metadata.
