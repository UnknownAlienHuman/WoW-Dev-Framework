# Contributor execution and acceptance model

**Status:** normative contributor process for humans and automated agents.

This document separates fast functional implementation from milestone verification
and final acceptance. It exists to prevent parallel writers, repeated full-suite
churn, stale tracking PRs, version proliferation and unsupported completion claims.
Package contracts and source-authority rules remain normative for product behavior.

## 1. Canonical work item

Every active work package has exactly one canonical GitHub Issue. The Issue records:

```text
work package and current slice
integration manager / writer
exact base commit
owned crates, modules and public contracts
hard blockers
validation tier required for this slice
functional completion criteria
acceptance debt kept open after functional completion
```

A pull request is a mergeable code-review object. Do not keep a stale branch open
merely to use its PR body as a project ledger. When implementation is already in
`main`, preserve remaining work in the canonical Issue and close the obsolete PR
without merging it.

## 2. Writer ownership

One integration manager owns one worktree and one write lease for the active slice.
Worker agents may research, audit, review, or prepare patches for that manager, but
they do not independently publish overlapping owner changes.

Before editing, the manager records the exact remote base SHA and owned boundaries.
Before publication, the manager rereads remote `main`:

- unchanged head: publish with an exact expected-head/CAS update;
- moved head with disjoint compatible work: reconcile and rerun the applicable gate;
- overlapping or contract-changing work: stop and resolve ownership before writing;
- never force-push over another contributor.

Until protected PR publication is enabled, direct-to-main work is a transitional
single-writer mode, not permission for several agents to race on `main`.

## 3. Work-item states

Use these states consistently:

| State | Meaning |
|---|---|
| `specified` | Scope and dependency order are defined; no implementation claim. |
| `claimed` | One writer owns an exact base and bounded slice. |
| `functional` | The intended owner/application path exists and the slice gate passes. |
| `verified` | An independent exact-head milestone gate passes and artifacts are retained. |
| `accepted` | Required fixtures, platforms, external parity and runtime gates for the selected scope pass. |
| `closed` | Functional and acceptance requirements are either complete or explicitly transferred to named follow-up Issues. |

Never collapse `functional`, `verified`, and `accepted` into one word such as
"done". A published commit without checks is only published. A green workspace
build is not package, platform, runtime, or launch acceptance.

## 4. Validation tiers

### Tier S — functional slice

Run after each coherent implementation slice:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Use the smallest complete affected owner chain. Widen it when a shared public type,
wire schema, graph registry, persistence layout, migration boundary, or service/app
composition changes. Do not run the full workspace test, release-test, rustdoc and
second full build campaign after every micro-slice.

### Tier M — milestone exact-head gate

At a coherent functional milestone, run the lightweight remote workflow on the
exact published head:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

Retain the tested commit/tree and actual command outcomes. This gate is independent
build evidence, not full behavioral acceptance.

### Tier A — package acceptance

Run once when the selected product/work-package scope is functionally ready, or
when explicitly requested:

```text
workspace debug tests
workspace release tests where required
strict rustdoc
selected Linux/Windows matrix
real CLI/application pipeline fixtures
fixture/checksum/identity freeze
fault, cancellation and persistence compatibility cases
```

The manual heavyweight CI lane owns this tier. A failure returns the work package
to `functional`; do not patch around it by weakening fixtures or relabeling gaps.

### Tier X — external/current/runtime qualification

Run only where the package contract requires it:

```text
one freshly resolved exact Gethe revision
native Ketho donor/parity comparison
WoW API Ketho MCP discrepancy matrix
named-client WoW runtime probes
external consumer or distribution evidence
```

Use the same flavor, exact source revision, admitted corpus and normalization
profile on both comparison sides. Missing tooling or runtime remains
`NotEvaluated`, never pass.

## 5. Hard blockers versus acceptance debt

A hard blocker stops dependent implementation:

- contradictory owner or wire semantics;
- missing identity/evidence/support needed by downstream code;
- broken migration or replay compatibility for an already admitted persistent format;
- stale/ambiguous source generation presented as exact;
- an unreachable positive path, fabricated endpoint, or false negative authority;
- unresolved concurrent ownership of the same contract.

Acceptance debt may remain open while dependent functional code proceeds only when
it cannot change the consumed contract. Examples include deferred Windows/runtime
qualification, performance measurements, broad corpus coverage and additional
near-negative fixtures. Record the debt in a named Issue and keep the relevant
launch/package gate blocked.

## 6. Slice and version discipline

A slice implements one coherent owner responsibility and its minimum downstream
handoff. Do not create a commit merely to update prose or increment a profile.

Mint a new schema/profile/storage/replay version only when serialized bytes,
identity inputs, interpretation, persistence, or compatibility behavior changes.
A commit number is not a schema version. For internal profiles that have never
been released, frozen in accepted fixtures, or admitted into durable external
state, consolidate transient development variants before acceptance instead of
supporting every intermediate experiment forever.

When compatibility must be retained, state the actual consumer and migration or
replay obligation. "Preserve everything" without an admitted consumer is not a
sufficient reason for permanent version surface.

## 7. Documentation and evidence

Keep current routing concise:

- `docs/PROJECT_WORK_MAP.md`: current frontier, dependencies, hard blockers, next
  bounded slices and latest verified checkpoint;
- `docs/IMPLEMENTATION_STATUS.md`: executable capability census and nonclaims;
- `docs/PROJECT_COMPLETION_MATRIX.md`: package/launch state, updated at milestones;
- canonical Issue: active work log and acceptance debt;
- historical checkpoint detail: append-only issue comment or a bounded checkpoint
  record, not repeated prose copied into every routing document.

Do not append the same test count, toolchain text and nonclaims to several files
after every commit. Record exact evidence once and link to it.

## 8. Publication

Publish only a coherent slice. Use non-force expected-head/CAS publication and
read back:

```text
remote commit SHA
remote tree SHA
changed blob identities
workflow/check result when applicable
```

A local commit, detached Git object, patch, or downloadable artifact is not branch
publication. A published commit is not validated until its named gate completes.

## 9. PR and branch lifecycle

- Issues are canonical trackers.
- PRs contain code intended to merge, or are closed when superseded by direct-main
  publication.
- Never merge an obsolete specification branch after equivalent/newer code is in
  `main`.
- Close superseded PRs with a final status and links to remaining Issues.
- After closure, archive/read back branch history where required, then remove only
  reconciled branches. Unreconciled history stays explicit.
- Keep `main` protected from force-push and deletion. If required status checks are
  enabled, require the lightweight gate, not the full acceptance matrix.

## 10. Completion rule

A work package may be reported as:

```text
functional      — owner/application path exists and Tier S passed
verified        — exact-head Tier M passed
accepted        — selected Tier A and required Tier X gates passed
launch-ready    — only when the named launch gate independently closes
```

Anything else must name the missing gate, blocker, or follow-up Issue.