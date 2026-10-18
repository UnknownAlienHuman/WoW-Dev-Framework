# Working and checking a change

Follow [AGENTS.md](../AGENTS.md). The selected Issue defines one deliverable;
[ARCHITECTURE.md](ARCHITECTURE.md) defines the current product scope.

## Coordination

Use one manager, one worktree/branch and one editable claim in the Issue. Record
the base commit, owned paths, intended result and a finite lease (normally up to
two hours). Refresh while working; release when stopping. Small workers may
read/review or edit distinct paths in that worktree. Do not overlap writes.

Before takeover, read remote main/PR, the current claim and local status.
Preserve dirty/private/unpublished work; never reset, clean, stash or switch
unexpected state to make a checkout convenient. Read a named recovery patch
before using it. A blocker stops only work that needs it.

After two failed attempts at one approach, record the failed assumption and
change method. Do not repeat an expensive command without a changed hypothesis.

## A useful Issue

```text
Outcome: the user action or code path made simpler
Read: the few relevant implementation/donor paths
Change: replacement and old code/options/tests to remove
Keep: concrete behavior that still matters
Check: one observable result and proportional validation
Dependency: a real prerequisite, or none
```

Retire work outside the current scope as not planned. Do not transfer an
obsolete requirement endlessly between Issues. Keep historical evidence in its
original PR/commit rather than copying a diary into active instructions.

## Validation

| Change | Normal check |
| --- | --- |
| Documentation | `cargo xtask check`, relative links, relevant instruction/skill consistency. |
| Rust | Formatting; Clippy for affected crates/callers; one existing behavior test or reproduction for the actual risk. |
| Shared type/storage/protocol | Also exercise the affected consumer and one relevant failure/reopen/version case. |
| Annotation semantics | Representative positive/negative output or consumer probe using the selected source; compare the relevant donor behavior. |
| Product milestone | Run the actual advertised workflow on representative real input and the claimed platform. |

Typical commands: `cargo fmt --all --check` and
`cargo clippy --locked -p <crate> --all-targets --all-features -- -D warnings`.
Clippy already compiles those targets; do not repeat `cargo check` for them.
Use existing tests, not a new test framework or all historic fixture matrices.
Delete tests that only enforce a removed feature. Broaden checks only for a
concrete remaining risk. Missing tools or runtime are untested, never a pass.

The existing remote Quick remains the shared compile/lint gate. Report its run
and checked commit in the PR. A green run is build evidence, not proof of every
addon/client. Source/runtime qualification is needed only for the claim made.

## Publication and handoff

Reread the remote head before publication. Publish a coherent reviewable branch
using expected-head updates; never force-push or merge a stale branch. Read back
the PR/head and CI result. Keep one short record: change, checks, remaining work,
next task. Do not download and hash CI evidence archives without a concrete
reason. Do not merge or publish a release without the relevant authorization.
