# Guarded collection of inline ProjectStore generations

Updated 2026-10-09. W15 now has explicit publication release and bounded
transactional generation/partition GC. Store and native service lifecycle
regressions pass. Full W15/E2, backup/restore, object/epoch deletion and platform
fault acceptance remain open; final workspace gates are reported separately.

## Selected profile and release

`ProjectStore::create_with_gc` selects
`project-store-wal-manifested-partitions-v3`; new `LiveProjectStore` instances use
it. Static SQL adds `gc_policy` and `gc_operations` to v2. V1/v2 schemas, catalogs
and canonical epochs remain frozen: no ALTER or migration occurs. Pins work in
v2/v3; release/GC require v3. Service calls on old profiles return
`OperationNotImplementedForMilestone`.

`release_publication` requires an exact operation digest and bounded attributable
holder. It relinquishes resumability without changing current or deleting data.
Original request, complete manifest, state, validation and activation receipt
remain stored. An optional versioned receipt binds their digest, epoch and holder;
omission preserves old bytes. Exact release retries return the original release;
substitution conflicts. Released IDs remain reserved after collection. Reconcile
validates retained identities without claiming their live closure survives.

Prepare/validate/activate refuse released operations. Service update stops before
base hydration, and publication fallback cannot turn released evidence into a new
activation success. Public reconcile uses `released`. Publication and GC IDs
cannot substitute for one another.

## Authority and planning

`select_gc_policy` uses expected-current digest CAS; absence requires absence even
for identical submitted content. Policy binds its version, exact existing retained
generations and finite generation/version/payload deletion budgets. `plan_gc`
requires that exact persisted policy and never selects it implicitly.

Planning validates complete bounded inventories: generation manifests, membership,
partition seals, validations, history, all publication operations, pins, policy
and prior GC receipts. Each manifest/receipt scan has a 64 MiB aggregate bound;
payload validation has a 1 GiB bound. Malformed, missing, oversized or incomplete
inventory blocks deletion; a scanned prefix cannot establish eligibility.

Roots are current, selected-policy generations, pins, exact active reader
generations, and every unreleased operation's target and expected-current base.
Prepared manifests protect already sealed versions before membership exists.
Remaining generations protect every shared version. An unleased/unpinned old ID
is not a root; collected Exact/Publication reads fail without substitution.

The opaque compiled `ProjectGcPlan` cannot be deserialized from a report. Its
canonical report carries epoch, policy/state digests, protected/selected IDs,
measured payload bytes and writer/lease state. A weak owner identity rejects a
plan on a reopened owner. Acquire/drop increments a monotonic lease revision;
writer changes detect root/policy ABA. SQLite `data_version` detects other
connection commits. External-process readers/writers remain unsupported.

Physical partition payloads are inline in this epoch. The separate generic object
store is not adopted or collected. Logical source/evidence IDs do not authorize
deleting external objects, files or paths. No SQL, connection or dynamic table
catalog escapes the store.

## Execution and evidence

`execute_gc` reconciles the exact operation ID/request, rebuilds and compares the
entire plan, then rechecks locked `data_version` after beginning an immediate
transaction. Policy/root/publication/lease/epoch/owner changes reject stale plans.
Eligible rows delete in FK order: history, validations, membership, generations,
then unreachable partition versions. Shared versions and Prepared roots survive.
FK closure and current are checked before the exact GC receipt is inserted in the
same transaction. Cancellation rolls back; success requires autocommit and
canonical receipt readback. Unprovable outcomes stay `OutcomeUnknown`.
`reconcile_gc` retains the exact receipt across response loss/reopen; substituted
requests conflict. Batches are bounded and atomic, without filesystem deletion,
scheduling or multi-batch retry loops.

The store lifecycle uses three genuine validated publications with shared data,
pins, a reader across activation, policy/root/lease stale plans, actual reclamation,
current/shared preservation, cancellation, released-ID conflicts and durable
receipt/reopen. The native service fixture uses existing Project/Graph owners,
verifies exact surviving publication identities and rejects released activation.

Prepared partial-seal interruption, mid-transaction process death, power-loss/disk
faults, Windows sharing, quarantine, backup/restore and external object/epoch
collection remain separate operational gates. These tests do not establish full
W15/E2, Gethe/Ketho or named-client acceptance.

Final workspace policy, fmt, check, strict Clippy, tests (866 passed, 1 ignored,
106 targets), rustdoc and build passed on 2026-10-09. Logs use the isolated
WoW-Dev-Framework build target outside the workspace. The ignored consumer gate
remains unexecuted.
