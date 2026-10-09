# Native live project commands

Physical Lua and explicitly selected standalone TOC/XML profiles expose coherent
publication and acquisition through `wow-service::live_project`. The app remains
a transport over one service operation per command. Multi-package replay is
still unavailable.

```text
wow project publish --config project.json --project <ProjectId> --store-root <new-private-directory> --operation-id <id> --expected-current absent --initialize --allow-partial
wow project read --store-root <directory> --store-generation current
wow project read --store-root <directory> --store-generation <StoreGenerationId>
wow project reconcile --store-root <directory> --operation-id <original-id>
```

`publish` accepts the existing explicit materialized-input, physical-file or selected-TOC
configuration. The service constructs one original native publisher and runs the
same graph producer chain as `wow graph build`. It captures exact Main/Library
bytes, validates native read-back while the candidate is inactive, then activates
only with the supplied expected-current CAS. Later publications omit
`--initialize` and use the exact current record ID returned by `read` or
`reconcile`. Initializing never adopts an existing retained graph store.

The store contains source text and belongs in an explicitly owned private
directory. Source files are neither executed nor changed. One owning process
uses a store at a time; another owner receives a typed busy result.

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

Physical archives keep v1 encoding. Existing physical epochs reopen against their
exact original catalog without changing epoch or membership identities; they
accept physical publications only. Initialize a new private store to publish v2
loader archives. No epoch migration is performed. Fresh full workspace gates and
build passed on 2026-10-09: 849 tests passed, 1 ignored, 103 targets. The new service
regression runs the complete TOC/XML producer chain after source removal.
