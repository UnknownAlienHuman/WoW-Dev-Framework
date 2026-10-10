# Platform source replay

**Status:** implemented and verified. Original v5 checkpoint evidence on 2026-10-10: policy,
fmt, all-target/all-feature check, strict Clippy, tests (925 passed, 1 ignored, 108
targets), strict rustdoc under `RUSTDOCFLAGS=-D warnings`, and all-target/all-feature
build. Implemented scope is source and package replay plus the durable native store
channel. Full W17 acceptance remains open.

The subsequent selected /6 checkpoint passed all seven workspace gates/build
with 936 tests passed, 1 ignored and 111 targets; see
[package XML bindings](PLATFORM_PACKAGE_XML_BINDINGS.md).

The subsequent package graph `/7` checkpoint passed 938 tests, 1 ignored across
111 targets; see [selected package graph](PLATFORM_PACKAGE_GRAPH.md). The new
explicit raw inventory `/8` checkpoint passes all seven workspace gates/build,
with 939 passed, 0 failed, 1 ignored and 111 targets, completed
2026-10-10 07:48:26 UTC. It adds a separate Included-member graph/native local-read
binding; see [raw inventory graph](PLATFORM_RAW_INVENTORY_GRAPH.md). These scoped
results do not establish complete W17 or corpus/runtime acceptance.

Byte admission is documented in
[PLATFORM_SOURCE_ADMISSION](PLATFORM_SOURCE_ADMISSION.md), package specialization in
[PLATFORM_SOURCE_PACKAGES](PLATFORM_SOURCE_PACKAGES.md), and the owner-bound
configuration in [PLATFORM_SOURCE_CONFIGURATION](PLATFORM_SOURCE_CONFIGURATION.md).
Publication behavior is in
[REPLAY_PUBLICATION](../crates/wow-project/REPLAY_PUBLICATION.md).

## What now replays

The platform channel holds the original profile request, the exact declared inventory,
every retained raw member, and the package request with its expected binding digest.
Hydration re-enters the real source and package owners, validating the profile,
inventory, observed byte lengths/digests and source/load/Main binding. It performs
no filesystem or network access. Serialized receipts cannot construct these owners.

Raw members are carried as byte sequences, never as decoded text, so unknown extensions
and undecodable data retain their exact bytes. Main fixture references are retained as
exact path and reference pairs, and hydration restores them by sorting both sides rather
than by positional matching.

The durable native store lifecycle uses generation schema version 2 with the platform
channel present, and the archive re-derives its own project and analyzer snapshot
identities through a fresh publish and compares them against the captured ones before
the pair is returned.

## Additive schema, no legacy rewrite

The platform channel is new-only. An unselected platform archive uses
`wow-project/native-project-replay/5` and the `wow-project.live-replay.v5` storage
schema, and no earlier archive acquires a platform channel it did not have. Ordinary
generation version 2 archives without the platform channel keep
`wow-project/native-project-replay/4` and `wow-project.live-replay.v4`.

Explicit same-session package XML binding selection adds
`wow-project/native-project-replay/6` and `wow-project.live-replay.v6`; see
[package XML bindings](PLATFORM_PACKAGE_XML_BINDINGS.md). It requires the typed
selection and genuine platform channel. /6 without that selection and /5 with
it refuse, with no fallback.

Explicit `PlatformGraphProfile::PackageProjectionV1` selects
`wow-project/native-project-replay/7` and `wow-project.live-replay.v7`, with source
graph `/20` and registry 15. The new
`PlatformGraphProfile::PackageProjectionWithRawInventoryV1` selects
`wow-project/native-project-replay/8` and `wow-project.live-replay.v8`, source `/21`
and registry 16. Both require genuine platform packages and same-session bindings;
schema/profile substitution refuses without fallback.

Storage selection stays a closed ladder in order of specificity: selected raw
graph archives map to v8, selected package graph to v7, selected package XML
bindings to v6, unselected platform archives to v5, ordinary generation version 2
to v4, package archives to v3, load-plan archives to v2, and legacy physical archives
to v1. New epochs admit v1-v8; complete published V1-V7 catalogs remain frozen and
reopen against their exact membership. They are never widened in place, preserving
original epoch and membership identities. Ordinary v1-v4 and prior platform v5-v7
recipes are unchanged.

Replay capture refuses an archive whose schema, platform presence and generation schema
version disagree, so a v5 claim on a non-platform archive or a platform archive claiming
v4 is rejected.

## Sealed checks carried into replay

The binding is revalidated on capture against the held source and its load and Main plans,
and the Main files are validated against the Main plan. On rebuild, the package request
re-specializes the re-admitted source and the resulting binding digest must equal the
captured one, so a package set or plan that would change the binding is refused.

Raw members share the ordinary archive budgets with Library: 8,192 files,
64 Libraries, 16 MiB per file and 32 MiB total raw source. The complete encoded
record also stays within the store's 32 MiB ceiling; JSON byte sequences can
exceed that ceiling before the raw limit is reached. Capture counts the borrowed
encoded corpus before copying it. Fixture-reference metadata has separate finite
count/path/reference bounds and contributes to the encoded record budget.
Reordered, duplicate, missing and surplus retained members reject.

`ProjectReplay::from_json` bounds the raw envelope before strict typed decoding
and archive validation. Direct `Deserialize` constructs an inert DTO; owner
admission still requires `hydrate`. Stored reads enforce the same record ceiling.

## Selected raw inventory graph replay

The `/8` selector participates in configuration/project-generation identity.
Hydration re-admits the original raw source and package request, then rebuilds
the configured native owners. Pair acquisition recomputes both the source batch
and the separate `wow-project.platform-source-inventory` batch, requiring exact
registry, coverage, graph and publication membership. Deserialized inventory
metadata or read-capability labels do not replace this native validation.

Native service request `wow-service/graph-build-request/11` must match the held
raw graph configuration and returns `wow-service/graph-build-result/18`.
Default `/19` and package `/20` routes keep their request/result identities;
their retained evidence/persistence APIs refuse the raw variant with
`DeferredCapability` instead of discarding it or falling back.

The native synthetic lifecycle verifies disk-absent `/8` publication and namespace
reopen with the exact original project/analyzer, source/load/Main and graph
identities, Included bytes and bounded native read binding. A complete frozen V7
epoch refuses v8 without changing Current, epoch or membership and retains no
operation for the refused request. This is scoped compatibility evidence, not
catalog migration or full replay fault acceptance.

## Update capability is not granted

A platform archive is a read-only compatibility record for the update protocol. Only the
modern physical update profile supports a durable update, and a platform channel excludes
it, so `into_update_publisher` refuses a platform archive with `DeferredCapability`.

## Not claimed

Stable logical namespace publication across differing source snapshots is now a
separate focused owner slice; see [PLATFORM_STORE_NAMESPACE.md](PLATFORM_STORE_NAMESPACE.md).
The separate raw inventory partition is executable; remaining TOC/XML/analyzer
direct assertions still share the prior source producer. The full four-way split,
structural fingerprints, bounded SkeletonInputView, and source service and CLI
transport for an explicit platform request remain open. Gethe acquisition and
materialization remain deferred. Real mirror, performance and runtime
acceptance and full W17, E3, E0 and E2 acceptance remain open.
