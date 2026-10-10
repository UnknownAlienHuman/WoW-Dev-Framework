# Stable platform store namespace

The platform route now publishes changing source snapshots into one explicitly
selected logical store. Two native core cases and two service cases pass on
2026-10-10. Workspace policy, fmt, all-target/all-feature check, strict Clippy,
tests (929 passed, 1 ignored, 109 targets), strict rustdoc under
`RUSTDOCFLAGS=-D warnings` and all-target/all-feature build pass. Full W17/E3
acceptance remains open.

## Identity and admission

`wow-store::project::ProjectStoreNamespace` derives `ProjectStoreId` from the
versioned store identity, Project store kind, complete logical namespace label and
owner ProjectId. For the platform service, that label is the complete admitted
source `ProfileId`. A filesystem root is only a locator. Source profile digest,
source snapshot, graph universe, Reference generation and host paths are excluded
from this stable identity and remain exact publication inputs.

A new `wow-store/project-namespace-epoch/1` manifest retains the validated
descriptor and derives its epoch from the exact catalog, v3 physical profile,
compiled SQLite runtime/schema, native canonicalization version and actual
security/limit policies. Reopening reconstructs the selected manifest and compares
both its typed value and canonical bytes. Caller-supplied policy strings cannot
replace compiled admission.

Only fresh namespace stores are created. Existing graph-owned epochs retain their
original schema, identity tuple and omitted namespace fields. The physical v1/v2
to v3 migration is not a conversion of an old owner into a logical namespace.

## Native publication and held reads

`ProjectPublicationBundle::build_in_namespace` and `from_replay_in_namespace`
validate the descriptor against genuine platform configuration, source/package
owners and graph. The native owner ProjectId and complete source profile must
match; the actual graph universe must equal the sealed package binding.

The original semantic records, member versions, pair header and publication-set
recipe are derived first. Namespace metadata enters only the new generation
binding scope, `native-platform-project-pair-namespace-v1`, alongside exact source
profile digest, source snapshot and graph universe. Selecting a different store
root does not change semantic identities.

`AcquiredProjectPair::read` dispatches from the actual admitted epoch retained by
its `ReadSnapshot`. It hydrates the same native source/project/analyzer/graph
owners, reconstructs the entire expected binding map and compares exact members
and owner. Missing, extra or substituted bindings refuse. A namespace failure
never retries through the legacy graph-owner mode. An older held reader keeps its
original epoch, transaction, publication and generation lease after Current moves.

## Explicit service selection

Create `PlatformStoreSelection` with the native `ProjectId` and complete source
`ProfileId`, then use `LiveProjectStore::create_in_namespace` or
`open_in_namespace`. Selected publish, read, reconcile and update methods verify
the selection against the admitted epoch before returning success.

The one-shot APIs are `publish_input_in_namespace`, `read_live_project_in_namespace`,
`reconcile_live_project_in_namespace` and `update_input_in_namespace` in
`wow_service::live_project`. Publication accepts a real
`LocalProjectInput::new_with_package_plans` and composes the existing complete
graph producer chain from the retained publisher. Native validation precedes
Current CAS and activation. Results expose the admitted store ID, namespace and
owner with an explicit platform scope; legacy result bytes omit these new fields.

Platform update remains unsupported by the package/physical-update capability
guards. The selected namespace does not grant incremental-update capability.
There is no file-based platform request or public platform CLI transport yet.

## Verification scope

Two core cases verify stable store/epoch identity across distinct host roots,
distinct owner ProjectIds, exact reopen, and preservation of a legacy epoch.
Tampering with the authoritative namespace registry's descriptor, catalog,
canonicalization/security policy or schema refuses before physical admission.

The native service cases use two genuine admitted source snapshots with the same
logical profile/ProjectId and differing source/project/graph identities. They
publish through exact Current CAS, preserve an old leased reader, reopen both
generations, compare semantic members under legacy and namespace selection, and
refuse foreign owners/profiles. A frozen V4 catalog reopens with its original
epoch and member identities. Another case composes the selected one-shot service
route through actual graph producers and reads/reconciles its exact publication.
The fixture source directories are removed before publication/readback.

Independent platform inventory/TOC/XML/analyzer partitions, structural
fingerprints, SkeletonInputView, source service/CLI transport and complete W17/E3
acceptance remain open. Real Gethe/Ketho corpus, performance and WoW runtime
acceptance have not been run by this checkpoint.

See [platform replay](PLATFORM_SOURCE_REPLAY.md),
[native publication](../crates/wow-project/REPLAY_PUBLICATION.md) and the
[current work map](PROJECT_WORK_MAP.md).
