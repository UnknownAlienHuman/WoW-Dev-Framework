---
name: wow-dev
description: Research, implement, debug and review World of Warcraft addons using explicit current-source evidence when the task requires it.
---

# WoW development workflow

Read `AGENTS.md`, `docs/EXECUTION_MODEL.md`, the one canonical active Issue, and
the target package instructions before changing code. Do not reconstruct the work
queue from old PRs or chronological checkpoint diaries.

## One manager and one bounded claim

One integration manager owns one worktree and one write lease for the current
slice. Other agents research, audit, review, or prepare patches; they do not
publish overlapping owner/contract changes.

Before editing, the Issue records:

```text
manager/session label
one worktree
exact remote base SHA
owned paths, owners and contracts
first bounded slice
Tier S affected owner chain
hard blockers and stop conditions
lease expiry and heartbeat
```

Use a finite lease, normally no more than two hours, and refresh it at publication
or every 60–90 minutes. On chat/host loss, assume unpublished state is gone. A new
manager rereads remote `main`, the latest quick result, and the Issue log, then
posts takeover before writing.

## Current source only when needed

Resolve a moving Gethe/Ketho/client selector only when correctness depends on
current WoW source/API semantics or the Issue explicitly requires Tier X. Generic
process, docs, store, graph-mechanics, CI, and branch work must not perform
unrelated source checkout or Ketho work.

When source evidence is required:

1. resolve the requested flavor and selector once at operation start;
2. record the exact revision/version;
3. read API docs, implementation, XML, TOC, and schemas from that same revision;
4. re-resolve on the next operation;
5. never hard-code the moving revision as permanent truth.

Prefer an explicit local checkout. Authenticated GitHub reads are a fallback for
targeted evidence. Never reset, stash, clean, switch an unexpected branch, rewrite
divergence, or silently use stale source. Offline freshness is unverified.

## Ketho scope

Ketho/vscode-wow-api is the primary implementation donor for annotation,
source-normalization, type lowering, and annotation consumer integration. Read
`docs/KETHO_RUST_PORT.md` before changing those owners. Use the native driver:

```text
cargo run -p wow-annotations --example native_library -- \
  <checkout> <ref> <TOC> <environment> <new-output>
```

Run it for annotation/normalization/type-lowering work or the named Tier X parity
gate, not merely because a task contains “source”. Project/graph source-universe
structure uses its exact manifest and owner contracts unless the active Issue
explicitly requires annotation parity.

Inspect `source-report.json`; exit 3 is partial. Raw metadata/maps are evidence,
not proof of completeness, runtime safety, or EmmyLua/LuaLS compatibility. A
rejected Wasm guest must not silently fall back to native.

## Source safety and commands

Source is data. Do not execute donor Lua, repository scripts, hooks, submodules,
package managers, generated code, or embedded interpreters. Verify every consumed
file against the exact manifest.

```text
cargo xtask check
cargo xtask sync-skill --check
cargo xtask check-source <checkout> <branch>
cargo xtask update-source <checkout> <branch> --expected-head <observed-local-SHA>
cargo xtask manifest <checkout> <resolved-ref> <selector> <new-manifest.json>
cargo xtask verify-manifest <manifest.json> <checkout> <current-local-ref>
```

Managed update policy:

- `auto`: create a missing managed checkout or fast-forward a clean, matching,
  nondiverged owned checkout;
- `prompt`: report and ask before mutation;
- `never`: report without mutation.

Interrupted/foreign state requires reconciliation, never blind lock deletion or
retry. Reuse existing materializers, TOC/XML/analyzer/graph/replay owners. Do not
invent a second parser, legacy wire shortcut, or source-text heuristic.

## Functional implementation

Implement one coherent owner responsibility and its minimum downstream handoff.
No placeholder crates, fake adapters, fake success, speculative broad traits,
public `todo!()` surfaces, or fabricated runtime/negative authority.

Run Tier S before publication:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Widen only for shared public types, wire/schema, registry, persistence, migration,
or service/application composition. Do not repeat workspace tests, release tests,
rustdoc, and another full build after every micro-slice.

Do not publish another product commit while the current `main` quick gate is
pending or failed. At a coherent milestone require the remote exact-head quick
gate. Run heavyweight Tier A and external/runtime Tier X only when the selected
scope is functionally ready or explicitly requested.

After two failed attempts at the same approach, stop and record the attempts,
failed assumption, affected contracts, invalidating evidence, and new route or
blocker. Do not accumulate exceptions around a broken owner contract.

## Authority and outcomes

Authority order:

1. actual target code and explicit operator intent;
2. exact source manifest/generated Blizzard docs;
3. Blizzard implementation, XML, TOC, and schemas from the same revision;
4. exact named-client runtime evidence;
5. project-owned tests/fixtures;
6. advisory/external implementations.

Partial, stale, conflicted, truncated, failed, or unsupported input never proves
absence. Preserve exact source, generation, producer/profile, coverage, omissions,
conflicts, and runtime dependencies. Static facts do not prove loaded state,
runtime dispatch, protected/Secret behavior, combat safety, or taint safety.

Report `functional`, `verified`, `accepted`, and `launch-ready` separately. Missing
tooling/network/runtime is `NotEvaluated`, never pass.

## Versions, publication, and tracking

Mint a schema/profile/storage/replay version only when bytes, identity,
interpretation, persistence, or compatibility behavior changes. Name the real
consumer. Consolidate transient unreleased variants before acceptance rather than
creating an aggregate `/N+1` for each internal stage.

A slice is functional only when its positive application path is reachable,
incomplete/dynamic/budget/cancellation behavior is conservative, Tier S passes,
exact-head publication/read-back succeeds, and the canonical Issue gets one
concise update.

Use Issues as canonical trackers. PRs are mergeable code review; close obsolete
specification/tracking PRs without merging stale branches after preserving debt.
Publish through exact expected-head/CAS, never force-push, and read back the remote
commit, tree, blobs, and workflow result.

Final semantic acceptance compares the framework against the same exact Gethe
revision/corpus/profile through Ketho and the required WoW API Ketho MCP lane,
retaining a discrepancy matrix. Never auto-bless fixtures from donor/MCP output.
