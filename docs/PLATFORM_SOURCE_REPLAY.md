# Platform source replay

**Status:** implemented and verified. Full workspace gates pass on 2026-10-10: policy,
fmt, all-target/all-feature check, strict Clippy, tests (925 passed, 1 ignored, 108
targets), strict rustdoc under `RUSTDOCFLAGS=-D warnings`, and all-target/all-feature
build. Implemented scope is source and package replay plus the durable native store
channel. Full W17 acceptance remains open.

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

The platform channel is new-only. A platform archive uses
`wow-project/native-project-replay/5` and the `wow-project.live-replay.v5` storage
schema, and no earlier archive acquires a platform channel it did not have. Ordinary
generation version 2 archives without the platform channel keep
`wow-project/native-project-replay/4` and `wow-project.live-replay.v4`.

Storage selection stays a closed ladder in order of specificity. Platform archives map
to v5, ordinary generation version 2 to v4, package archives to v3, load-plan archives to
v2, and legacy physical archives to v1. `wow-project.live-replay.v1` through `v4` are a
frozen published catalog and are never widened in place, so reopening one preserves its
original epoch and membership identities. The original v1 through v4 recipes are
unchanged.

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

## Update capability is not granted

A platform archive is a read-only compatibility record for the update protocol. Only the
modern physical update profile supports a durable update, and a platform channel excludes
it, so `into_update_publisher` refuses a platform archive with `DeferredCapability`.

## Not claimed

Stable logical namespace publication across differing source snapshots is now a
separate focused owner slice; see [PLATFORM_STORE_NAMESPACE.md](PLATFORM_STORE_NAMESPACE.md).
Independently owned platform graph partitions and structural fingerprints,
SkeletonInputView, and source service and CLI transport for an explicit platform
request remain open. Real mirror, performance and runtime
acceptance and full W17, E3, E0 and E2 acceptance remain open.
