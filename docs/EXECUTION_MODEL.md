# Contributor execution and acceptance model

**Status:** normative contributor process for humans and automated agents.

This document separates fast functional implementation from milestone verification
and final acceptance. Package contracts and source-authority rules remain normative
for product behavior.

## 1. Agent context packet

An implementation agent starts from the smallest sufficient context:

```text
AGENTS.md
this execution model
one canonical active GitHub Issue
target owner package instructions/contracts
exact external evidence required by that slice
```

Do not make the agent read every historical PR, checkpoint diary, or full work map
before coding. Historical records are evidence, not the active prompt.

A work item is ready only when the Issue states:

```text
work package and bounded current slice
manager/session label and one worktree
exact remote base commit
owned crates/modules/public contracts
inputs, outputs and explicit nonclaims
hard blockers and stop conditions
Tier S affected owner chain
functional completion criteria
acceptance debt kept open
lease expiry and heartbeat cadence
```

## 2. Canonical Issue and work states

Every active work package has exactly one canonical Issue. A pull request is a
mergeable code-review object, not a permanent ledger. When equivalent/newer code
is already in `main`, transfer remaining debt to Issues and close the obsolete PR
without merging its branch.

Use these states:

| State | Meaning |
|---|---|
| `specified` | Scope and dependency order exist; no implementation claim. |
| `claimed` | One manager owns an exact base and bounded write lease. |
| `functional` | Reachable owner/application path exists and Tier S passes. |
| `verified` | Independent exact-head Tier M passes and evidence is retained. |
| `accepted` | Required Tier A and Tier X gates for the selected scope pass. |
| `closed` | Work is accepted or every remaining obligation moved to named Issues. |

Never collapse `functional`, `verified`, and `accepted` into “done”. Publication,
compilation, package acceptance, platform acceptance, runtime evidence and launch
readiness are different claims.

## 3. Writer ownership and lease

One integration manager owns one worktree and one write lease for the active
slice. Worker agents may research, audit, review, or prepare patches, but do not
independently publish overlapping changes to the same owner or contract.

Before editing, post the claim described above. A normal transitional lease is at
most two hours and is refreshed at publication or every 60–90 minutes. Release it
explicitly when stopping.

Before publication, reread remote `main`:

- unchanged: publish through exact expected-head/CAS;
- moved with disjoint compatible work: reconcile and rerun the gate;
- overlapping or contract-changing work: stop and resolve ownership;
- never force-push over another contributor.

Until protected PR publication is enabled, direct-to-main is transitional
single-writer mode, not permission for several agents to race.

### Crash and takeover

If the chat, host, worktree, or tool session disappears, a replacement manager:

1. reads remote `main` and its latest quick result;
2. reads the canonical Issue claim/log;
3. assumes unpublished local state is lost;
4. uses a patch/artifact only when its exact identity is recorded and read back;
5. posts takeover/release before writing.

A stale claim is not silently inherited. Do not guess what the previous agent
“probably” changed.

## 4. Validation tiers

### Tier S — functional slice

Run after each coherent implementation slice:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Use the smallest complete owner chain. Widen when a shared public type, wire
schema, graph registry, persistence layout, migration boundary, or service/app
composition changes. Do not run workspace tests, release tests, strict rustdoc,
and a second full build after every micro-slice.

Tier S passes before publication. Do not stack another product commit while the
current `main` quick gate is pending or failed. A failed quick gate blocks further
product publication until repaired on the observed head.

### Tier M — milestone exact-head gate

At a coherent functional milestone, run the lightweight remote workflow on the
exact published head:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

Retain commit/tree/toolchain and actual outcomes. Tier M is independent build
evidence, not full behavioral acceptance.

### Tier A — package acceptance

Run once when the selected product scope is functionally ready, or when explicitly
requested:

```text
workspace debug tests
workspace release tests where required
strict rustdoc
selected Linux/Windows matrix
real CLI/application pipeline fixtures
fixture/checksum/identity freeze
fault, cancellation and persistence compatibility cases
```

The heavyweight manual CI lane owns this tier. Failure returns the package to
`functional`; do not weaken fixtures or relabel gaps.

### Tier X — current/external/runtime qualification

Run only where the contract requires it:

```text
one freshly resolved exact Gethe revision
native Ketho donor/parity comparison
WoW API Ketho MCP discrepancy matrix
named-client WoW runtime probes
external consumer or distribution evidence
```

Use the same flavor, exact source revision, admitted corpus, and normalization
profile on both sides. Missing tooling/runtime is `NotEvaluated`, never pass.

## 5. Current-source routing

Resolve moving Gethe/Ketho/client selectors only when correctness depends on
current WoW source/API semantics or the Issue explicitly requires Tier X.
Generic process, documentation, store, graph-mechanics, CI, and branch work must
not contact unrelated source providers.

When current source is required, resolve the selector once at operation start,
record the exact revision, and read all relevant docs/implementation/XML/TOC from
that same revision. Re-resolve later; do not hard-code moving truth.

The native Ketho annotation driver is required for annotation,
source-normalization/type-lowering changes or named parity gates. A project/graph
source-universe structural task uses its exact manifest and owner contract unless
the Issue specifically requires the annotation driver.

## 6. Hard blockers and acceptance debt

A hard blocker stops dependent implementation:

- contradictory owner/wire semantics;
- missing identity, evidence, support, or a reachable positive path;
- broken admitted migration/replay compatibility;
- stale/ambiguous generation presented as exact;
- fabricated endpoint or false negative authority;
- a selected real corpus that cannot fit the claimed bounded profile;
- unresolved concurrent ownership;
- a downstream named query/axis that cannot bind the actual registry.

Acceptance debt may remain only when it cannot change the consumed contract.
Examples: later Windows/runtime qualification, performance measurement, broad
corpus coverage, and additional near-negative fixtures. Track it in a named Issue
and keep the relevant package/launch gate blocked.

## 7. Failure-loop rule

After two failed attempts at the same approach, stop and record:

```text
attempts and exact failures
assumption that failed
owners/contracts touched
evidence that invalidated the route
new route or explicit blocker
```

Do not keep adding exceptions around a broken owner contract. Update/create a
blocking Issue before continuing dependent work.

## 8. Slice and version discipline

A slice implements one coherent owner responsibility and its minimum downstream
handoff. Do not create a commit merely to update prose or increment a profile.

Mint a new schema/profile/storage/replay version only when bytes, identity inputs,
interpretation, persistence, or compatibility behavior change. A commit number is
not a schema version. Name the actual consumer and migration/reopen obligation.

Classify every retained version as:

```text
released/external
persisted and admitted
frozen fixture
transient unreleased development variant
```

Consolidate transient variants before acceptance. Do not create a monotonically
increasing aggregate profile for every internal stage when explicit capability
composition or one selected final profile is the real product contract.

A slice is functionally complete only when:

- the intended positive path is reachable through the owning application route;
- dynamic, ambiguous, incomplete, budget, and cancellation cases are conservative;
- Tier S passes;
- exact-head/CAS publication and read-back succeed;
- the Issue receives one concise state/evidence update.

## 9. Documentation and evidence

Keep hot routing concise:

- `PROJECT_WORK_MAP.md`: exact current verified head, one active Issue, blockers,
  dependencies, next 3–5 slices, links to backlog/evidence;
- `IMPLEMENTATION_STATUS.md`: executable capability census and nonclaims;
- `PROJECT_COMPLETION_MATRIX.md`: package/launch states and named blockers;
- canonical Issue: current work log and acceptance debt;
- historical evidence: bounded checkpoint file, Issue comment, or workflow artifact.

Do not repeat the same test count, profile ladder, toolchain text, and nonclaims in
several documents after each commit. Preserve unique future task specifications
before closing/deleting old PR branches.

Milestone evidence must remain recoverable after short workflow-artifact expiry.
Record immutable commit/tree/toolchain/artifact digest in the Issue or checkpoint,
or retain the artifact long enough for the selected review lifecycle.

## 10. Publication, PRs, and branches

Publish only a coherent authorized slice. Read back:

```text
remote commit SHA
remote tree SHA
changed blob identities
named workflow/check outcome
```

A local commit, detached object, patch, or download is not branch publication. A
published commit is not verified until its gate passes.

Issues are canonical trackers. PRs contain code intended to merge. Never merge an
obsolete specification branch after equivalent/newer code is in `main`. Close it
after transferring debt, then archive/read back history and delete only reconciled
branches. Unreconciled history remains explicit.

Protect `main` from force-push/deletion and require linear history. When PR-based
publication is enabled, require the lightweight gate, not the heavyweight
acceptance matrix, before merge.

## 11. Completion rule

Report only:

```text
functional   — application path exists and Tier S passed
verified     — exact-head Tier M passed
accepted     — selected Tier A and required Tier X passed
launch-ready — named launch gate independently closed
```

Anything else names the missing gate, blocker, or follow-up Issue.