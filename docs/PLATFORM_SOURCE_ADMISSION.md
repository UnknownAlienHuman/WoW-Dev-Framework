# Platform source admission

**Status:** implemented. Two native cases and workspace policy, fmt, check, strict
Clippy, tests (921 passed, 1 ignored, 108 targets), strict rustdoc and build passed
on 2026-10-09. The owner is written in
[platform_source/mod.rs](../crates/wow-project/src/platform_source/mod.rs), with
[profile.rs](../crates/wow-project/src/platform_source/profile.rs) and
[model.rs](../crates/wow-project/src/platform_source/model.rs). Native borrowed traversal
is in [raw_inventory.rs](../crates/wow-project/src/platform_source/raw_inventory.rs). The root records
actual gate results; implemented scope is not full W17 acceptance.

E3-A is normative for this boundary:
[SOURCE_ACQUISITION_AND_PINNING](../crates/wow-project/e3/SOURCE_ACQUISITION_AND_PINNING.md)
for closed input, inventory and byte policy,
[COVERAGE_AND_AUTHORITY](../crates/wow-project/e3/COVERAGE_AND_AUTHORITY.md) for
authority separation and negative claims, and
[UNIVERSE_AND_PACKAGE_MODEL](../crates/wow-project/e3/UNIVERSE_AND_PACKAGE_MODEL.md) for
universe separation.

## What the owner does

`ProjectInputDirectory::admit_platform_source(profile, inventory, stop)` verifies one
explicit, already-local platform-source input against its declared inventory. It
canonicalizes the inventory against the profile, opens each configured root confined
through the existing disk owner, reads every declared included member through the
existing raw read with mandatory digest and length, and retains the exact bytes. It
returns `AdmittedPlatformSource` or an error.

It does not acquire, clone or fetch source, does not resolve a moving selector, does not
enumerate the filesystem on its own, does not parse TOC, XML or Lua, does not run an
analyzer, does not build a graph, and does not interpret runtime behavior.

## Profile shape and finite limits

`BlizzardUiSourceProfile::new` and `from_json` produce an immutable profile on schema
`wow-project/platform-source-profile/1` whose digest is recomputed and rechecked on
every `validate`. The request carries a profile ID, a source class, a target, roots,
exclusions and limits.

Finite limits are enforced in both directions: every limit must be positive with
`max_file_bytes` at most `max_total_bytes`, and none may exceed the owner cap.

| Field | Owner cap |
| --- | --- |
| profile envelope | 256 KiB |
| roots | 64, disjoint and non-nesting |
| selected TOCs per profile | 1024, unique ignoring case |
| exclusions | 1024, unique ignoring case |
| inventory entries | 200000 |
| total included bytes | 256 MiB |
| single member bytes | 32 MiB, at most total |
| inventory envelope | 64 MiB |

Raw-owner caps do not widen downstream Lua or parser limits.

Paths and case are enforced exactly. Root and entry paths must be valid configured
relative paths, ASCII under the current case policy, with no absolute path, parent
escape, drive or UNC prefix or control character. Roots may not collide or nest after
case folding, and a root may not be a selected TOC. Every entry path must sit under
exactly one root, and no duplicate may exist ignoring case. An Included or Excluded leaf
may not be an ancestor of another entry, so a file and a directory cannot share one
path.

Known kinds follow the extension, and the entry's declared kind must agree: `.lua` is
Lua, `.toc` is Toc, `.xml` is Xml, `.xsd` is Schema, and anything else is Unknown. An
Unknown entry keeps its inventory identity even though no parser consumes it, and a
mismatch between declared kind and extension is rejected rather than corrected.

## Source class and origin must agree

`PlatformSourceClass` has exactly two values, and both the reference profile kind and
the origin revision must match the chosen one. `SyntheticFixture` requires a
`ProfileKind::Fixture` reference profile and a `PlatformSourceRevision::Fixture`
digest. `VendorUiSourceMirror` requires a `ProfileKind::Release` reference profile and
either a Git revision, with a nonzero lowercase-hex commit and tree at the format's
exact length of 40 for `sha1` or 64 for `sha256`, or an `Archive` content digest.

The profile ID namespace must equal the reference profile namespace. Provider and
repository are bounded exact text that rejects URLs, backslashes, control characters and
floating branch tokens, so a branch or tag name cannot masquerade as an identity. A
fixture class can never become mirror evidence, and the pairing is checked before any
member IO.

## Inventory is asserted input

`PlatformSourceInventory::from_json` bounds the envelope before decoding, then
`canonicalize` validates and sorts without performing source IO. The returned value is
still an input DTO, never an admission capability.

Binding is exact: the schema must be `wow-project/platform-source-inventory/1`, the
profile digest and the target must equal the profile's, inventory roots must be exactly
the profile's roots, and declared per-root counts must equal the observed accounting of
all supplied entries. Exclusions are exact files inside a root that are not selected
TOCs, and an Included member matching an exclusion rejects.

Each entry takes exactly one disposition. `Included` carries a content digest, byte
length and an optional Git object ID whose shape is checked against the format but never
authenticated as membership. `Excluded` must name a reviewed exclusion rule. `Unsupported`
and `External` carry evidence and never cause reads. `Conflict` and `Failed` carry a
bounded reason. Included and excluded byte totals are checked against the profile
limits before any file is opened.

## The receipt is a byte receipt

`PlatformSourceAdmissionReceipt` is serialize-only and carries the profile digest, the
content manifest digest, the source snapshot ID, the admission digest, a coverage
record and the full caller inventory. `AdmittedPlatformSource` also exposes the profile
and per-path `source_bytes` for later consumers.

Four identities are deliberately different, and none is a floating branch or a provider
display label:

- the **content manifest digest** hashes the configured root names plus the sorted entry
  material, so it reflects admitted content;
- the **source snapshot ID** binds profile, target, exact revision and content manifest,
  so the same bytes and profile give the same ID regardless of host directory, time,
  checkout order or enumeration order;
- the **admission digest** independently binds the full canonical input evidence,
  including provider, materializer, root evidence and license;
- the **profile digest** covers the canonical profile request.

## Asserted provenance versus verified bytes

The two halves of the input are not the same kind of fact, and the receipt does not
blend them.

**Asserted by the caller, checked only for shape and consistency:** the provider and
repository identity, the exact revision, the materializer producer, version,
configuration and report digests, the root inventory evidence digests, the license
record, and the compatibility evidence. The owner verifies these are present, canonical,
bounded and mutually consistent. It does not and cannot attest that the materializer
ran securely, that a client really loads this build, or that the license permits
anything.

**Verified by the owner, exactly:** that every declared included member exists inside its
configured root, that its raw bytes match the declared digest and length, that no
unmaterialized LFS pointer is accepted as content, that paths are confined with no
duplicate or case-colliding entries, and that declared entry counts equal the observed
accounting.

`coverage` states this in the standard states. Inventory coverage is Partial, because an
explicit list can never prove that an unlisted entry is absent. Declared included bytes
are Complete when at least one member was verified, and NotApplicable when the
inventory includes nothing. The verified file and byte counts are the observed totals.

`unevaluated` names the capabilities byte admission does not touch: root completeness,
Git membership, materializer security, client compatibility, license permission,
decoding, package load, analyzer, graph, API contract and runtime. Nothing downstream
may read an admission result as an affirmative claim about any of them.

## Retained bytes and the missing members

All verified included bytes are retained, including Unknown extensions and data no
parser consumes, so a later consumer reaches the same bytes without a re-read, a
normalization or a semantic inference. `source_bytes` returns a retained path or
`FileNotPresent`, so an undeclared or excluded path is an error rather than a silent
empty result.

`AdmittedPlatformSource::raw_inventory(stop)` returns a borrowed
`PlatformRawInventory`. Its fallible `next(stop)` yields each Included member in
canonical path order, including binary and otherwise unconsumed files.
`PlatformRawMember` exposes the original path, declared kind, verified digest,
length and exact borrowed byte slice. `receipt()` retains every disposition,
omission and original coverage assertion. These native types have private fields
and cannot be constructed from serialized receipts.

Traversal reuses sealed membership and bytes without disk reads, decoding,
hashing again or copying the corpus. Checked entry/member/byte accounting uses
the admitted profile limits. Cancellation or another error terminates that
cursor and subsequent calls retain its original error; clearing cancellation
cannot skip an advanced member. A fresh cursor requires the actual source owner.
Only successful exhaustion closes traversal of the Included set; a prefix does
not. Traversal creates no source handles, evidence IDs, graph entities or semantic
facts. Original identities and Partial inventory coverage remain unchanged.

The native disk-absent binary/omission/cancellation case passes on 2026-10-10.
Workspace policy, fmt, all-target/all-feature check, strict Clippy, tests
(933 passed, 1 ignored, 110 targets), strict rustdoc and build pass. This qualifies
the borrowed traversal slice, not complete raw-member graph projection or E3.

## Refusals

Admission returns an error and no owner-held result on a profile that fails validation,
a profile and inventory binding mismatch, a changed, missing or truncated declared byte,
a path outside its configured root, a duplicate or case-colliding path, a declared count
that disagrees with observed accounting, an unmaterialized LFS pointer, a budget
exhaustion, or cancellation. A refusal never repairs, truncates or re-enumerates.

## Not claimed

An admitted snapshot is not a complete `BlizzardUiSourceSnapshot`, not a platform
project, not a package selection, not a graph, not a SkeletonInputView, and not a
redistributable artifact. It carries no official API documentation authority and no
proof that runtime code loads in any client session, that an event payload is
accessible, or that a Secret, taint, protected or forbidden behavior exists. It does not
prove that an implementation path is public or supported, and it grants no
redistribution rights.

Later [package specialization](PLATFORM_SOURCE_PACKAGES.md),
[configuration/Main/source graph](PLATFORM_SOURCE_CONFIGURATION.md) and
[genuine replay/durable publication](PLATFORM_SOURCE_REPLAY.md) now consume this
bounded owner. [Stable logical namespace publication](PLATFORM_STORE_NAMESPACE.md)
is also executable. Source acquisition, full mirror materialization, complete
raw-member graph projection, platform partitions/fingerprints, SkeletonInputView, source
service/CLI transport, runtime and full W17/E2 acceptance remain open.
