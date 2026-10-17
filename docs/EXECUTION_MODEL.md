# Contributor execution and acceptance model

**Normative claim and delivery procedure.** Read with [`AGENTS.md`](../AGENTS.md).
The selected Issue names the relevant audit section(s) in
[`AUDIT_2026-10-17.md`](AUDIT_2026-10-17.md); unrelated audit sections are out of
scope.

## Claim and ownership

Each active slice has one integration manager, one worktree/branch, and one finite
write lease. Before editing, record:

```text
manager/session and worktree; exact remote base SHA
owned paths, owners/contracts, forbidden overlaps
bounded slice and functional outcome; Tier S owner chain
hard blockers/stop conditions; lease expiry and heartbeat
```

Normally lease no longer than two hours; refresh on publication or every 60–90
minutes and release when stopping. Workers may research, review, or prepare
distinct patches, but cannot publish overlapping owner, contract, registry,
storage, or application changes. No worker quorum is required. A worker reports
base SHA, evidence, conclusion/patch, preserved invariants, checks, and uncertainty.

The selected Issue is the queue and acceptance tracker. Keep it actionable with
this short form:

```text
Outcome: <user-visible capability or defect removed>
Read: <owner paths/contracts and named audit section>
Change: <bounded code action and handoff>
Keep/Exclude: <invariants and explicit nonclaims>
Done/Check: <functional result and proportional evidence>
Real dependency: <consumer, Issue, or none>
```

An open Issue does not globally lock implementation. Refactor tasks close after
their own scoped result and obligations are recorded or transferred to named
Issues; they do not wait for full package acceptance. Do not close remaining
acceptance debt merely to resume implementation.

## Recovery and scoped blockers

At takeover, inspect remote `main`, latest Quick, current Issue/claim, and local
state. Unpublished dirty, stashed, private, or otherwise surviving work is
unverified, not disposable: preserve its diff; never reset, clean, stash, switch
an unexpected branch, or erase it. Read any named patch/artifact fully and post
takeover before writing. While Quick is pending, preparation and review may
continue in the owned worktree; do not publish another product commit.

A blocker stops only consumers that require the broken contract. Name consumer,
evidence, and unblock condition. Scoped examples: mixed/stale identities claimed
as exact; missing support or unreachable required positive path; fabricated
endpoint, runtime claim, or clean negative; invalid relation definitions; broken
admitted replay/migration; unsafe/unbounded materialization; overlapping writer
ownership. Missing candidate validation blocks publication depending on it. A
corpus exceeding a bound blocks its full-corpus claim, not independent work on a
bounded representative; a subset never proves complete roots/dependencies or
full-mirror fit.

After two failed attempts at one approach, record exact failures, failed
assumption, touched owners/contracts, invalidating evidence, and next route or
blocker in the Issue. No exception spiral or broad restart; another expensive run
needs a changed, named hypothesis.

## Validation cadence

| Level | When and scope | Required evidence |
| --- | --- | --- |
| **S — slice** | Product code, before publication; smallest complete affected owner chain. Widen for shared public types, wire/schema, graph registry, persistence/migration, or service/app composition. | `cargo xtask check`, `cargo fmt --all --check`, and `cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings`. Clippy checks those targets; run `cargo check` separately only when a different target/profile is needed. |
| **M — exact head** | Existing remote `Quick exact-head gate` on the event checkout; PR events check the merge candidate and record source/base separately. | Repository policy, formatting, workspace compile, strict workspace Clippy, retained commit/tree evidence. Build evidence only; pending/failed Quick blocks another product publication. Reconcile moved heads and rerun applicable checks. |
| **A — milestone** | Selected functional scope is ready or explicitly requested. | Required debug/release tests, rustdoc, selected Linux/Windows matrix, real CLI/app pipelines, fixture/checksum freeze, and relevant fault, cancellation, update/removal, persistence/reopen cases. Tests reach the advertised consumer; preserve failures. |
| **X — external/runtime** | Contract requires current source, parity, external, or runtime qualification. | Same flavor, exact Gethe revision, corpus, and normalization for Ketho parity and required WoW API Ketho MCP discrepancy matrix; named-client runtime evidence where needed. Missing inputs/tooling are `NotEvaluated`. |

After the selected function works, a minimal existing targeted reproduction may
check a concrete defect. Preserve existing behavioral tests. Do not make a new
fixture matrix, workspace/release campaign, strict rustdoc, or repeated full build
a prerequisite to writing/simplifying a path. Build/run the app when needed to
exercise changed code. Record checks not run; compile-only is not behavioral
proof, and missing local tools are not a pass. Docs-only edits need policy, link,
and relevant skill/instruction checks; they do not alter the existing Quick.
Do not add per-slice workflows or run unrelated source/runtime qualification.

## Measurement and compatibility

For slow/refusing paths, find the actual stage and reuse existing counters,
reports, census commands, and counting sinks. Record safely available phase time,
counts, charged bytes/method, and refusal; unreached work is `NotEvaluated`, and a
fail-fast crossing is only a lower bound. Distinguish estimates, serialized bytes,
retained allocation, and OS peak; unavailable telemetry is unmeasured. Do not
leak source/private context or alter persisted semantics for diagnostics. Measure
before naming a bottleneck; prefer the smallest demonstrated repair. New
profilers, caches, checkpoints, or sharding are not prerequisites; sharding needs
measured need and an explicit cross-package semantics decision.

Follow source-as-data, donor, materializer, and selector rules in `AGENTS.md`.
Pin diagnostic batches to recorded source/profile/input identities; they do not
count as fresh qualifications. Reuse immutable views only while all identities
match; otherwise rebuild honestly, not via a new persistent cache. Classify
limits as output-changing budget, fail-fast ceiling, or implementation invariant.
Do not remove admitted digest/replay fields without a named consumer and
compatibility proof. Diagnostics need no schema; preserve exact wire bytes when
claiming backward compatibility. Version only changed bytes, interpretation, or
admitted compatibility, naming its reader/migration. Consolidate transient
unreleased variants without dropping durable readers or minting aggregate `/N+1`
versions for internal stages.

## Publication, evidence, and state

Before publication reread remote `main`. Publish a coherent authorized change
from the manager's branch as a reviewable PR. Integration requires the authorized
action and passing Quick on the reconciled head. Use expected-head/CAS; never
force-push. Read back remote commit/tree, changed blob identities, and named
workflow result. Exact Git tree readback suffices ordinarily; fetch artifacts for
discrepancy, recovery, or milestone review. Never call unchecked artifacts
verified. Preserve milestone evidence beyond artifact expiry. Local commits,
patches, or downloads are not branch publication; published code is not verified
until its gate passes.

Keep one editable current claim/status and one concise milestone record:

```text
base -> result SHA / exact Quick; capability or defect changed
next code action/owner; measured outcome or unmeasured
checks run/not run; blocker or acceptance Issue; lease/handoff
```

Store detailed evidence once and link it. Hot work maps route to the canonical
Issue; a process-only change names its product unblock and does not reset product
work. Keep state meanings distinct: `specified` (goal described), `claimed`
(manager owns slice), `functional` (owner/app path and Tier S), `verified`
(independent exact-head M), `accepted` (selected A and required X), `closed`
(the selected task's outcome is delivered and validated; remaining obligations
are explicitly tracked), and `launch-ready`
(separate launch gate passed). PRs are review objects, not journals. Transfer
unique specifications and debt before closing stale PRs without merge. Delete
branches only with explicit authorization, reconciliation, and safe history
readback; main protection belongs to the control-plane owner, not each product
slice.
