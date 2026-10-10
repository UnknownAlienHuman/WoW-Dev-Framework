# Platform raw inventory graph and native reads

The explicit raw graph profile projects every declared Included member into a
separate native inventory partition and binds exact accepted Producer references
to bounded local reads of the held source. The native lifecycle and all seven
workspace gates/build pass on 2026-10-10. Full W17/E3 acceptance remains open.

## Explicit selection and preserved identities

`PlatformGraphProfile::PackageProjectionWithRawInventoryV1` requires genuine
retained platform packages and `PackageXmlBindingProfile::SameSessionV1`.
It selects `wow-project/source-load-proposals/21` and registry version 16.
Absent selection keeps `/19`; `PackageProjectionV1` keeps `/20`; both retain
registry 15. Configuration/project-generation identity binds the selection.

The selected native replay schema is `wow-project/native-project-replay/8`, with
storage `wow-project.live-replay.v8`. Native service request
`wow-service/graph-build-request/11` must match the held raw configuration and
returns `wow-service/graph-build-result/18`. Earlier requests/results, ordinary
replay v1-v4 and platform v5-v7 retain their identities. Complete frozen V1-V7
catalogs reopen exactly and are not widened to admit v8. See
[platform replay](PLATFORM_SOURCE_REPLAY.md).

## Included members and retained assertions

`ProjectSourceGraphProposals::inventory_batch()` exposes the separate
`wow-project.platform-source-inventory` batch of `platform_raw_member` entities.
Consumers retain it before consuming the existing `into_parts()` tuple.
The batch covers canonical Included membership, including unknown/opaque,
unloaded and otherwise unconsumed files; load reachability does not remove raw
membership. It introduces no raw containment/load relations.

`ProjectGraphProvenance::raw_inventory()` exposes serialize-only
`ProjectRawInventoryManifest` metadata: the original inventory and actual
coverage, source snapshot/profile/content-manifest/admission digests, and member
paths, declared kinds, verified raw digests/lengths, source handles and evidence.
Non-Included dispositions remain in the original inventory without invented
bytes. Proven Included-byte observations remain separate from origin,
materializer, compatibility and license assertions. Declared kinds do not attest
successful decoding or analyzer health.

Successful native cursor exhaustion closes the declared Included set, not the
configured root. Original inventory coverage remains Partial; Git membership,
materializer security, client compatibility and license permission are not
promoted. See [source admission](PLATFORM_SOURCE_ADMISSION.md).

## Exact held native byte binding

`bind_platform_raw_member(project, graph, reference, stop)` validates the actual
held project and graph, reconstructs the exact inventory batch, checks complete
accepted-member accounting, resolves the native Producer address/receipt and
joins the original member's kind, digest and length. Local references, stale or
mismatched batches and wrong subjects refuse. No failed lookup falls back to
disk, Library or a newer Current.

The resulting `ProjectRawMemberReadBinding` borrows that immutable ProjectView's
source. Private fields and the absence of Deserialize prevent metadata from
constructing this capability. `read_bytes(Range<u64>, max_bytes, stop)` returns an
exact borrowed local slice; the caller supplies a positive cap of at most 64 KiB.
Range ordering, end, representability, budget and cancellation are checked.
Opaque members use byte coordinates, not invented UTF-8 or analyzer SourceSpans.
The binding exposes the original license assertion and exact Producer reference.

Byte access neither copies nor decodes the retained corpus. It provides no
serialized excerpt transport, export or release route. License metadata grants
no redistribution permission; the held local capability does not establish such
permission either.

## Bounds and durable reconstruction

The complete borrowed metadata envelope and exact empty batch envelope count
against the shared graph budget before retaining their collections. Member
bodies and array separators are charged before insertion. Original admission
bounds, cancellation and successful Included cursor exhaustion remain required;
source and raw entities also share the graph node limit.

Replay retains the original binary source/request and re-enters the real source,
package, configuration and analyzer owners. Pair acquisition reconstructs both
native source and inventory batches under the held store read. Serialize-only
metadata cannot replace those owners. Existing retained `/19` and `/20`
evidence/persistence APIs refuse this raw variant with `DeferredCapability`.
The shared raw archive limits and 32 MiB encoded-record ceiling remain unchanged.

## Scoped verification and remaining work

The [native synthetic lifecycle](../crates/wow-service/src/live_project/tests/namespaces.rs)
`raw_inventory_reopens_with_exact_byte_binding_and_frozen_v7_refuses` passes with
source disk absent. It checks Included binary/unconsumed membership and retained
exclusion, exact borrowed opaque bytes, address/read-budget refusals, composed
namespace Current reopen with original identities and a complete frozen V7
epoch refusing v8 without effects.

Final workspace gates/build completed at 2026-10-10 07:48:26 UTC:
`cargo xtask check`, `cargo fmt --all --check`, workspace all-target/all-feature
check, strict Clippy, full tests (**939 passed, 0 failed, 1 ignored, 111 targets**),
strict rustdoc under `RUSTDOCFLAGS=-D warnings`, and workspace
all-target/all-feature build all passed. These are local verification results;
remote publication and CI are not asserted.

The remaining TOC/XML/analyzer direct assertions still share their prior source
producer. This separate inventory partition does not complete the four-way E3
split or the full inventory relation model. Fingerprints, bounded
SkeletonInputView and source service/CLI transport remain future work. Full
W17/E3/package, real Gethe/Ketho corpus, performance, native fault/exhaustion and
WoW runtime acceptance remain NotEvaluated. Gethe acquisition/materialization is
deferred; W18-W26 are unchanged.
