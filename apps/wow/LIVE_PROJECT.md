# Native live project commands

Physical Lua, explicitly selected standalone TOC/XML and declared multi-package
profiles expose coherent publication and acquisition through
`wow-service::live_project`. The app remains a transport over one service operation
per command. Full W13/E2 acceptance remains open.

The ordinary publication profile uses native/storage replay v4 and binds exact
Library snapshot identities plus the function-call-facts profile. Older or incompatible derived stores are not opened or migrated; rebuild
them from current source and configuration. One current Project catalog admits
the payload schemas; reconstructed configuration proves exact selections.

```text
wow project publish --config project.json --project <ProjectId> --store-root <new-private-directory> --operation-id <id> --expected-current absent --initialize --allow-partial
wow project read --store-root <directory> --store-generation current
wow project read --store-root <directory> --store-generation <StoreGenerationId>
wow project reconcile --store-root <directory> --operation-id <original-id>
wow project recover --store-root <directory> --format json
wow project recover --store-root <directory> --domain-current --format json
wow project update --config final-project.json --project <ProjectId> --store-root <directory> --operation-id <id> --expected-current <record-id> --library keep --allow-partial
```

## Selected native candidate and first Skeleton profile

Selected publication/read uses `--selection <selection.json>` with schema
`wow-service/platform-project-selection/1`, exact `owner_project_id`,
`source_profile_id` and catalog intent. `composed_candidate=true` selects the
genuine `BlizzardUiIndexCandidate` with moved native source authority, final
composed graph and explicit selected fingerprint manifests. Inactive readback
and later reopen rebuild the native/core/query owners and compare the final
graph and candidate receipt under the original lease before activation/read.

Selected publication may supply `--function-fingerprints <query.json>` and
`--xml-fingerprints <query.json>`. Selected read returns candidate id/digest and
an inert `skeleton_binding`; decoded receipts cannot construct a native owner.

Selected read and Skeleton keep encoded results private until the original
read and Store close successfully in reverse order. The same checked scope is
shared with platform Context; it does not acquire or reconstruct a second view.
Mandatory close failure supersedes draft success, an earlier semantic refusal
and late cancellation. Success bytes/identities and one-time Current/Exact
selection remain unchanged.

```text
wow project skeleton --selection <selection.json> --store-root <directory> --store-generation current|<generation-id> --request <skeleton.json> [--format json|text]
```

The request schema is `wow-service/live-skeleton-read-request/1`, with
`candidate_id`, `candidate_digest` and `input`. Current/Exact resolves once;
service requires the rebuilt candidate and exact identity before the native read.
The first profile, `wow-project/skeleton-input/native-one-hop-handles/1`, exposes
one exact entity root, root support/evidence/handles and a bounded direct
neighborhood. Typed declarations/signatures/types, package/load-role projection,
neighborhood evidence closure, source excerpts, axes/paths and continuation
remain `NotEvaluated`. Results are Partial or Truncated and exit 2.

Selected update uses the retained native publisher, derives the actual final
Add/Update/Remove intent and recomposes native/core/query owners before publication.
Project NoChange also requires an identical rebuilt candidate receipt; a change
only to selected fingerprint queries publishes a new candidate. Existing expected
Current, operation identity and cancellation fences remain in force. The bounded
source-absent content/query/removal lifecycle is recorded in
[#116](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/116);
the ordinary physical update contract below is a separate route.
Full E3 acceptance remains open. Follow the current bounded claim in
[#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

`recover` observes one held physical store snapshot, including complete current
membership/seals, canonical receipts and scope coverage. It performs no repair or
implicit activation. Complete applicable coverage exits 0, incomplete coverage 2,
invalid coverage/admission 4, and cancellation 130. Unknown acknowledgment remains
explicit. The native service also exposes verified backup and restoration to a new
private path and explicit guarded physical-instance replacement with real
Project/Graph owner replay. `restore_replace` and exact `resume_replacement` are
service APIs. The CLI does not dispatch backup, replacement or quarantine
administration; `recover` only observes the current registered store. Existing leased
readers and semantic IDs survive physical-instance replacement. See
[PROJECT_RECOVERY.md](../../crates/wow-store/PROJECT_RECOVERY.md).

`--domain-current` adds native Project/Graph replay of the exact Current publication
retained by the physical report. It keeps that physical evidence beside a typed
Validated, Absent, Unverified, Failed, Incomplete or Cancelled domain observation;
other generations receive no domain verdict. Replay failure or cancellation does
not discard the completed physical report. This observation grants no repair or
activation authority. See
[PROJECT_CURRENT_RECOVERY.md](../../crates/wow-service/PROJECT_CURRENT_RECOVERY.md).

The service now exposes explicit whole-instance quarantine through
`LiveProjectQuarantineInspection` and `LiveProjectStore::quarantine`, plus the
separate read-only `QuarantinedLiveProject`. Standalone inspection can observe and
explicitly hold damaged SQL without normal writable admission. Exact raw Current
and recovery evidence guard the hold; already acquired native pairs remain coherent
while new normal operations return `StoreQuarantined`. This CLI retains that typed error
and does not dispatch quarantine or restore. See
[PROJECT_QUARANTINE.md](../../crates/wow-store/PROJECT_QUARANTINE.md).

`publish` accepts the existing explicit materialized-input, physical-file,
selected-TOC or package-universe configuration. The service constructs one original
native publisher and runs the
same graph producer chain as `wow graph build`. It captures exact Main/Library
bytes, validates native read-back while the candidate is inactive, then activates
only with the supplied expected-current CAS. Later publications omit
`--initialize` and use the exact current record ID returned by `read` or
`reconcile`. Initializing never adopts an existing retained graph store.

The store contains source text and belongs in an explicitly owned private
directory. Source files are neither executed nor changed. One owning process
uses a store at a time; another owner receives a typed busy result.

On the ordinary route, `update` requires an existing modern physical-Lua
publication and an explicit final input configuration. The service derives Add/Update/Remove against the
exact expected publication record, retains its actual native owner under a lease,
and rebuilds the complete graph producer chain before inactive validation and
current CAS. Standalone TOC/XML and package publications remain unavailable for
this update route. Their current read/publication routes remain supported.

`--library keep|replace|clear` is required. Keep requires the supplied final
Library inventory to match the retained owner; Replace uses the supplied Library
set. Clear and empty replacement reject through the current mandatory-Library
policy. They cannot silently become Keep. There is no update initialization.

On the ordinary physical route, NoChange validates project/generation selection,
keeps the original current record and consumes no operation ID. It exits 2 for Partial coverage.
Changed requests retain the existing operation fingerprint: the same ID/base/
target returns its original receipt even after another activation, while a
different target conflicts. Reconstructing that exact request may perform native
analysis and graph composition again; durable publication effects are not repeated.
Embedders can call `LiveProjectStore::update` on the existing owner while older
leased readers continue to observe their original immutable pairs.

`read` resolves Current or Exact once, holds the original transaction/generation
lease through actual native replay and owner validation, and projects exact
project/analyzer/graph/publication-set IDs and file/Library/node/edge counts.
It does not fall back to another generation. Old leased service readers remain
on their acquired pair after current advances.

`reconcile` observes the original operation and current state without repeating
publication. A committed activation receipt remains committed after late
cancellation or output loss. Reconcile the original operation ID before retrying
an uncertain effect; do not manufacture a replacement ID.

All commands support `--format json|text`. JSON is the exact bounded service DTO
plus LF. Activated/acquired pairs exit 2 because source coverage remains Partial;
observed reconciliation exits 0, absent operations 3, service/output failures 4,
invalid arguments 64 and precommit cancellation 130. These results do not close
full E2 acceptance or source/runtime compatibility.

Verified on 2026-10-09: full workspace check, strict Clippy, tests (847 passed,
1 ignored, 103 targets), rustdoc and build. The native service regression invokes
the complete producer chain, actual publication/read-back and stale-CAS rejection;
stored pair tests cover reopen, Current/Exact, old leases, mutation rejection and
public read/reconciliation projections. This is synthetic project verification,
not a real-addon or Ketho comparative acceptance run.

The service smoke also exposed invalid canonical pack constructors in the bridge,
hook and library families. Constructors now order fixture/rule IDs before the
unchanged strict validators; profiles are bridge4, hooks5 and library3. Coverage
keeps no negative authority. No acceptance fixture is regenerated.

Standalone TOC/XML and declared package inputs use the same current replay v4
envelope as other ordinary publications. The replay retains their
selected TOC/context/documents or package sources/variants and rebuilds those owners
without reopening source directories. No replay-v1/v2/v3 catalog, generation-v1
reader or store migration remains in the product; incompatible derived stores must
be recreated from source and configuration.
