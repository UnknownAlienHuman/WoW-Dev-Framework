# Native live project commands

Physical Lua, explicitly selected standalone TOC/XML and declared multi-package
profiles expose coherent publication and acquisition through
`wow-service::live_project`. The app remains a transport over one service operation
per command. Full W13/E2 acceptance remains open.

New publications use native/storage replay v4 and generation recipe v2, binding
exact Library snapshot identities and the function-call-facts profile. Existing
V1/V2/V3 stores remain readable under their exact catalogs but reject v4 writes;
initialize a new private store for new publications. No migration occurs.

```text
wow project publish --config project.json --project <ProjectId> --store-root <new-private-directory> --operation-id <id> --expected-current absent --initialize --allow-partial
wow project read --store-root <directory> --store-generation current
wow project read --store-root <directory> --store-generation <StoreGenerationId>
wow project reconcile --store-root <directory> --operation-id <original-id>
wow project recover --store-root <directory> --format json
wow project recover --store-root <directory> --domain-current --format json
wow project update --config final-project.json --project <ProjectId> --store-root <directory> --operation-id <id> --expected-current <record-id> --library keep --allow-partial
```

`recover` observes one held physical store snapshot, including complete current
membership/seals, canonical receipts and scope coverage. It performs no repair or
implicit activation. Complete applicable coverage exits 0, incomplete coverage 2,
invalid coverage/admission 4, and cancellation 130. Unknown acknowledgment remains
explicit. The native service also exposes verified backup and restoration to a new
private path and explicit guarded physical-instance replacement with real
Project/Graph owner replay. `restore_replace` and exact `resume_replacement` are
service APIs; this CLI does not dispatch restore. Older readers and semantic IDs
survive; guarded restore from quarantine, schema migration and full acceptance remain open. See
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
and recovery evidence guard the hold; old native pairs remain coherent and new
normal operations return `StoreQuarantined`. This CLI retains that typed error
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

`update` requires an existing modern physical-Lua publication and an explicit
final input configuration. The service derives Add/Update/Remove against the
exact expected publication record, retains its actual native owner under a lease,
and rebuilds the complete graph producer chain before inactive validation and
current CAS. Standalone TOC/XML, package and legacy archives remain unavailable
for this update route. Their read/publication routes remain supported.

`--library keep|replace|clear` is required. Keep requires the supplied final
Library inventory to match the retained owner; Replace uses the supplied Library
set. Clear and empty replacement reject through the current mandatory-Library
policy. They cannot silently become Keep. There is no update initialization.

A NoChange result validates project/generation selection, keeps the original
current record and consumes no operation ID. It exits 2 for Partial coverage.
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

The standalone TOC/XML checkpoint uses native replay v2 and retains the exact
selected TOC, selection context and consumed TOC/XML bytes alongside Main/Library.
Read-back runs the same bounded loader, checks the original plan digest and
recreates the original semantic IDs without reopening source directories. Missing,
excluded and unresolved load decisions remain explicit. Surplus archived files,
changed documents/context and substituted schema versions reject.

At the standalone checkpoint, physical archives kept v1 encoding. Existing epochs reopened against their
exact original catalog without changing epoch or membership identities; they
accept physical publications only. Initialize a new private store to publish v2
loader archives. No epoch migration is performed. Fresh full workspace gates and
build passed on 2026-10-09: 849 tests passed, 1 ignored, 103 targets. The new service
regression runs the complete TOC/XML producer chain after source removal.

At the package checkpoint, replay used native/storage v3. It retains every selected package's
captured sources, including unreachable Lua, and exact bytes of all declared TOC
variants. It reconstructs dependencies, reachability, file order and namespaced
Main with the existing package owner. Unreachable files and unselected-variant
entries remain outside analyzer Main; their actual source identities stay bound.
Package sources are archived once, independently of the flattened Main namespace.
V1/V2 epochs keep their exact catalogs and reject v3 writes before current changes;
initialize a new private store for v3. Full workspace gates/build passed on
2026-10-09: 850 tests passed, 1 ignored, 103 targets. The real package service
regression deletes its source directory before publication and Exact reopen.

The generation checkpoint passed all workspace gates and build on 2026-10-09:
852 tests passed, 1 ignored, 103 targets. Frozen old archives hydrate with exact
original IDs; V1/V2/V3 stores retain their epoch/current identities and Exact
read-back after rejecting v4 publication. Full W14 reuse/removal and W13/E2
acceptance remain open.
