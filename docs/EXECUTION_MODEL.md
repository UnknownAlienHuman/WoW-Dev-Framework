# Contributor execution and acceptance model

**Status:** normative contributor process. The operational clarification of
2026-10-11 removes global audit pauses and duplicate work, not product safety or
acceptance requirements. Read this with `AGENTS.md`; package behavior, source
safety and admitted compatibility contracts remain in force.

## 1. Small context, one current task

Start with `AGENTS.md`, this file, the current body of the canonical Issue, its
latest claim, and the target owner's instructions/contracts. Read historical
comments only for the exact evidence needed. Do not reconstruct the queue from
old PR bodies, every audit or a chronological checkpoint diary.

The Issue identifies the goal, current bounded slice, inputs/outputs, owner paths,
actual dependencies, nonclaims, validation and next code action. A later queue
correction explicitly supersedes stale instructions; an old stop comment is not
an additional gate. Do not recreate implemented owners because an old contract
or audit says they are absent: reconcile the discrepancy first at that owner.

Progress means a usable application capability or a demonstrated defect removed.
Commit count, test count, agent count and new profile numbers are not progress
metrics. Research ends in a decision or patch, not another general audit project.

## 2. One manager, one worktree, bounded ownership

One integration manager owns one worktree and one write lease for the active
slice. Before editing, record in one claim:

```text
manager/session label and worktree
exact remote base SHA
owned paths/contracts and forbidden overlaps
next integrated slice and functional outcome
Tier S affected owner chain
hard blockers / stop conditions
lease expiry and heartbeat
```

Normally a lease is at most two hours; refresh it on publication or every 60–90
minutes, and release it when stopping. Workers can read, review or prepare
patches but cannot publish overlapping owner/registry/replay/service changes.
Separate managers require genuinely disjoint ownership, not different task names
for the same files. Never overwrite another manager's unpublished state.

On crash/takeover, read remote HEAD, Quick and the current claim. Unpublished work
is unverified, not disposable: inspect any surviving dirty worktree and retain
its diff before reconciliation. Do not reset, clean, stash, or erase it. Use an
identified patch/artifact only after reading it. Post takeover before writing;
do not presume that missing chat context proves local files were lost.

## 3. Parallelize useful work, not ceremonies

The manager integrates one coherent patch at a time. Give each needed worker a
distinct deliverable: exact source evidence, a counter/copy-cost table, an owned
patch, or an independent review of the selected diff. Reuse completed findings.
Do not spawn agents to fill slots or require every worker to repeat every audit.
No fixed all-worker approval quorum or model switch is a prerequisite to coding.

A worker returns: base SHA, evidence/file ranges, conclusion or patch, preserved
invariants, checks and uncertainty. The manager resolves actual disagreements,
integrates useful work and redirects redundant workers. Missing results block
only the decision that genuinely depends on them.

While remote Quick is pending, the manager may read, review and prepare the next
patch in the same owned worktree. The prohibition is on publishing the next
product commit over pending/failed Quick, not on all productive work. Keep the
next patch separate from the exact already-tested revision.

## 4. Scoped blockers and functional completion

A hard blocker stops the consumer that requires the broken contract:

- mixed/stale identities presented as exact;
- missing evidence/support or unreachable required positive path;
- fabricated endpoint, runtime claim or clean negative;
- invalid relation definitions or broken admitted replay/migration;
- unsafe/unbounded materialization;
- overlapping writer ownership.

Name the consumer, evidence and unblock condition. An open Issue is not by itself
a global lock. A corpus exceeding a selected bound blocks that full-corpus claim;
it need not block independent code on an explicitly bounded representative input.
A subset never becomes evidence of complete roots/dependencies or full-mirror fit.
Missing candidate validation still blocks publication that requires the candidate.

Separate bounded functional unblock from final composed-corpus acceptance. Do not
require unfinished producers to pass final capacity qualification before coding
them. Equally, do not close acceptance debt just to resume implementation. Keep
actual incompatible interfaces blocked; work on independent paths meanwhile.

The current W17 queue is Issue #106. Its immediate capacity repair is #121;
final corpus qualification remains #113. Read their current bodies, not copied
historical SHAs or obsolete ordering in an audit index.

## 5. Validation: code first, smallest sufficient gate

### Tier S — coherent functional slice

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Use the smallest complete affected owner chain. Widen for actual shared public
wire/registry/store/migration/application changes. Preserve existing tests; use an
existing focused reproduction when checking a concrete defect. Do not expand
whole fixture matrices or repeat workspace/release tests, rustdoc and another
full build after every internal counter or helper change. Build an executable
when needed to run changed code, not because unrelated documentation changed.
Record tests not run; compile-only is never behavioral proof.

Tier S passes before product publication. A pending/failed current-head Quick
blocks another product publication until resolved. Local tool absence is an
explicit limitation, never a pass; use available exact-head verification rather
than inventing a successful local run.

### Tier M — exact-head verification

The existing Quick workflow checks repository policy, formatting, workspace
compilation and strict Clippy and retains commit/tree/toolchain evidence. Retain
its exact result. This is independent build evidence, not package acceptance.
Do not add another checkpoint workflow for each slice.

### Tier A — functional milestone acceptance

After the chosen functional scope is ready, run the required debug/release tests,
rustdoc, Linux/Windows matrix, real application pipelines, fixture/checksum freeze,
fault/cancellation/update/removal and persistence/reopen cases. Tests must reach
the advertised consumer, not just an analyzer sidecar. Preserve failures; do not
weaken expectations or auto-bless identities to produce a green result.

### Tier X — current/external/runtime acceptance

Use the same flavor, exact Gethe revision, admitted corpus and normalization on
both sides of the required Ketho and **WoW API Ketho MCP** comparison. Retain a
discrepancy matrix and named-client runtime evidence where static facts cannot
answer. Missing tooling, credentials, source or runtime remains `NotEvaluated`.
No final semantic or launch claim bypasses its named external/runtime gates.

## 6. Measurement without a new subsystem

For a slow/refusing path, first locate its actual stage and existing counters.
Reuse typed owner reports, existing census commands and counting sinks. A bounded
diagnostic records phase time, counts, charged bytes/method and the refusal;
unreached phases remain `NotEvaluated`. Report all safely available counts, not
one guessed next limit. Diagnostic output must not leak source text, host paths
or private context or silently alter persisted semantic output.

A fail-fast crossing is a lower bound at interruption, not final required size.
Distinguish estimates, serialized bytes, retained allocation and OS sampled peak.
Do not call every charged copy a bug or remove accounting without a valid bound.
Measure before claiming CPU, memory, compilation or a specific algorithm is the
bottleneck. Record phase wall times; mark unavailable telemetry as unmeasured.

After two failures of the same hypothesis, write a short correction in the same
Issue: evidence, failed assumption and next code change. Do not restart an entire
research programme. A further expensive run needs a changed, named hypothesis.

Prefer the smallest supported repair: reuse an existing bounded owner, eliminate
a demonstrated redundant copy, stream through an existing bounded sink, or select
an existing finite capacity with justified headroom. A new general profiler,
cache, checkpoint system or sharding architecture is not a default prerequisite.
Sharding requires measured need and an explicit cross-package semantics decision.

## 7. Source batches, reuse and compatibility

Resolve current Gethe/Ketho selectors only when correctness needs current source
or Tier X. Generic process, CI, graph mechanics and store work do not require
unrelated source acquisition. Follow all `AGENTS.md` source-as-data restrictions
and existing materializers; never execute donor Lua/scripts or reset foreign state.

A diagnostic batch may pin one recorded revision/profile/input set across
hypothesis changes. Record each changed request; such attempts are not fresh
current-source qualifications. Re-resolve for the next qualification or when the
source itself is the changed input. Do not rerun source/analyzer work merely due
to an unrelated documentation SHA.

Reuse an acquired immutable view only while all required source/configuration/
report identities remain valid. If reuse is unavailable, record it and rebuild
honestly; do not create a persistent cache or relabel reports just for measurement.

Classify touched limits by actual behavior: output-changing budget, fail-fast
execution ceiling or implementation invariant. This classification is a design
aid, not permission to remove currently admitted digest/replay fields. Such a
change needs its own consumer/compatibility proof. Diagnostic-only observations
do not require a new persisted schema. Preserve exact existing wire bytes when
claiming backward compatibility.

Mint a profile/schema/replay/storage version only for changed bytes, interpretation
or admitted compatibility, naming the real consumer. Consolidate transient
unreleased variants deliberately, not by dropping durable readers. Do not create
an aggregate `/N+1` for each internal stage of an already selected capability set.

## 8. Publication, compact evidence and work states

Before publication reread remote `main`. Publish only an authorized coherent
change through expected-head/CAS, without force. Reconcile a moved head; rerun the
applicable checks for the reconciled tree. Direct-to-main remains transitional
single-writer mode until protected PR publication is enabled.

Verify the remote commit/tree and changed blob identities plus the named Quick
result. An exact Git readback can verify the tree; do not require both individual
blob refetches and a full artifact download on every ordinary slice. Download and
inspect artifacts when needed for a discrepancy, recovery or milestone review.
Never label an unchecked artifact verified. Preserve milestone evidence beyond
short artifact expiry, with immutable identifiers and actual outcomes.

Keep one editable current-status/claim comment and one concise milestone result:

```text
base -> result SHA / exact Quick result
capability or defect changed
next code action and ownership
measured stage/counter outcome, or unmeasured
checks run / not run
remaining blocker or acceptance Issue
lease and worker handoff
```

Do not recopy detailed hashes, every check and every nonclaim into the Issue,
work map, status file and a new audit. Hot work maps route to the canonical Issue;
update capability/completion documents at a meaningful milestone. A process-only
change must name its product unblock and cannot automatically reset product work.

States remain distinct:

- `specified` / `claimed`: scope exists / an identified manager owns the work;
- `functional`: required owner/application path exists and Tier S passed;
- `verified`: independent exact-head Tier M passed;
- `accepted`: selected Tier A and required Tier X passed;
- `closed`: scope accepted or explicitly transferred obligations to named Issues;
- `launch-ready`: the separate named launch gate passed.

PRs are mergeable code review, not permanent journals for code already in `main`.
Preserve unique specifications and move remaining debt before closing stale PRs
without merge. Delete only explicitly authorized, reconciled branches after safe
history retention/readback. Keep protection of `main` against force/deletion and
the transition to protected PRs in the control-plane owner, not a recurring
unrelated detour for every product slice.
