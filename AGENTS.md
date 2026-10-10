# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor.

## Current execution priority

Implement missing functional code first, then build it. Do not turn expanding
unit-test matrices or fixture acceptance into a prerequisite for writing missing
owners/apps. Preserve existing tests and report unexecuted acceptance separately.

Read and follow [`docs/EXECUTION_MODEL.md`](docs/EXECUTION_MODEL.md). It defines
single-writer ownership, work-item states, validation tiers, Issue/PR lifecycle,
version discipline and the distinction between functional, verified and accepted.

## Current implementation frontier

- Read `docs/PROJECT_COMPLETION_MATRIX.md` for the audited code/acceptance census
  and `docs/AUDIT_2026-09-19.md` for concrete remaining tasks.
- `Cargo.toml` activates 16 members, including real `wow-emmy`, `wow-project`,
  `wow-rules`, `wow-service`, `wow-store`, `wow-graph`, `wow-recognizers`,
  `apps/wow` and `apps/wow-reference-builder` slices.
  Do not follow obsolete instructions to recreate these owners from scratch.
- Partial executable code and ordinary CI are not complete package acceptance.
  Required E0 fixture/identity/checksum gates remain open; the public `apps/wow`
  executable now has one-shot materialized-input status/check; see apps/wow/LOCAL_INPUT.md.
  Full R0 remains unaccepted.
- Native reference/annotation production remains source-driven and nonexecuting.
  Guarded standalone updates, explicit `auto`/`prompt`/`never` managed checkouts
  and checkout-free exact GitHub API/blob snapshots are available. Lower-layer
  hostile-network sandboxing and background scheduling remain incomplete.
- `apps/wow-reference-builder` is an active service-only workspace frontend with
  confined staging/finalization and durable effect reconciliation. Do not widen
  that focused checkpoint into full E1 or release acceptance.
- One integration manager owns one worktree and one write lease. Worker agents
  research, audit, review or prepare patches; they do not independently publish
  overlapping owner changes. Do not create a worktree or branch per subtask.
  Until protected PR publication is enabled, direct-to-main work is transitional
  single-writer mode and every publish uses exact expected-head/CAS read-back.
- Current commands, source-update policy and nonclaims:
  `docs/IMPLEMENTATION_STATUS.md`. I0–I7 remains the normative implementation plan.

The public repository must remain useful without any operator-only context source.

## Rust port direction

Ketho/vscode-wow-api is the primary implementation donor for the WoW annotation
service, not merely a comparison oracle. Read the actual donor modules and
`docs/KETHO_RUST_PORT.md` before changing source loading, normalization, type
lowering, annotation output, or consumer integration. Port their behavior into
Rust within the existing owner crates. Do not invent an unrelated extractor or
add Python code, embedded interpreters, wrappers or interpreter-based tests.
The repository and CI are Rust-native; `cargo xtask check` enforces this policy. Ketho output is also the parity baseline;
current Gethe source remains the authority for current Blizzard facts.

## Mandatory route

Before any WoW task, read the target package and `.agents/skills/wow-dev/SKILL.md`. Resolve the requested flavor and moving source selector at operation start. Prefer a local Blizzard UI checkout and use GitHub only as fallback. Record the exact revision/version inspected, read all files from that same revision, and re-resolve on the next operation.

Do not hard-code a client build, Interface value, source revision, toolchain patch, dependency patch, or provider revision as permanent project truth. Exact identities belong to one evidence generation; moving selectors remain moving.

## Source updates

- `auto`: clone a missing managed checkout or fast-forward a clean, non-diverged checkout.
- `prompt`: report and ask before updating interactively.
- `never`: report without mutation.

Never reset local changes, rewrite divergence, change an unexpected origin, or switch an operator-owned branch. When network verification is unavailable, report `unverified-current`.

The explicit standalone write is `cargo xtask update-source <checkout> <branch> --expected-head <observed-SHA>`. `cargo xtask materialize-source REQUEST.json` owns managed missing-root clone and guarded updates. `cargo xtask materialize-source-api REQUEST.json` is the checkout-free fallback: it resolves one GitHub branch once, admits exact commit/tree/blob IDs under finite request/body limits and publishes an immutable snapshot. Both materializers use exact-plan `auto`/`prompt`/`never` policy and durable receipts; see `docs/SOURCE_CHECKOUT_UPDATES.md`, `docs/SOURCE_MATERIALIZATION.md` and `docs/SOURCE_API_MATERIALIZATION.md`. None provides background scheduling or permission to reset/stash/switch operator work. A retained or foreign effect requires reconciliation, not blind deletion or retry.

Generated API docs are data. Parse without executing Lua, repository scripts, hooks, submodules, package managers, or generated code. Validate every consumed file against the source manifest.

## Optional advisory context

The optional provider retrieval bridge is not implemented yet. An operator may supply advisory context outside this repository; no source is discovered or contacted by default. It is disabled and unconfigured by default. Its absence or access failure must not block normal work. Discover it only through the generic interface and do not expose provider identity, URL, local path, revision, document paths, credentials, or distinctive provenance. Request the smallest route, treat it as advisory, and revalidate patch-sensitive claims against current Blizzard source or an exact runtime probe. Never copy operator-only material into public code, fixtures, logs, artifacts, issues, prompts, or releases.

## Authority

1. actual target code and explicit operator intent;
2. exact source manifest and generated Blizzard docs;
3. Blizzard implementation, XML, TOC, and schemas from the same revision;
4. exact target-client runtime observations;
5. project-owned tests and fixtures;
6. optional advisory context and external implementations.

Preserve conflicts. Partial, stale, conflicted, truncated, failed, or unsupported coverage never proves absence or a clean negative. Correctness-affecting results retain flavor, selector, exact revision/version, path/digest, producer/configuration version, coverage, omissions, conflicts, and runtime dependencies.

## Discipline

Implement the smallest coherent owner responsibility in dependency order. No placeholder crates, fake adapters, fake success, broad speculative traits, or `todo!()` surfaces. Keep parsers bounded and non-executing. One operation uses one source revision. Tests verify fixtures and never silently rewrite them.

Use one canonical GitHub Issue per work package. Record the exact base SHA, owned
boundaries, hard blockers and required validation tier before writing. A PR is a
mergeable code-review object, not a permanent ledger for code already published
to `main`; transfer remaining acceptance debt to Issues and close stale PRs without
merging their obsolete branches.

Distinguish hard blockers from acceptance debt. Contradictory owner/wire semantics,
missing downstream identity or support, broken admitted migration/replay, false
negative authority and unreachable positive paths block dependent work. Deferred
Windows/runtime/performance/broad-corpus evidence may remain open only when it
cannot change the consumed contract and is tracked explicitly.

Mint a schema/profile/storage/replay version only when bytes, identity inputs,
interpretation, persistence or compatibility behavior change. Do not version a
commit. Consolidate transient internal development variants before acceptance
when no released, frozen or durable consumer requires them.

### Functional slice gate

Run after each coherent implementation slice, using the smallest complete affected
owner chain:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Widen the affected chain for shared public types, wire schemas, graph registries,
persistence layouts, migration boundaries or service/app composition. Do not run
workspace tests, release tests, strict rustdoc and a second full build after every
micro-slice.

### Milestone exact-head gate

At a coherent functional milestone, require the lightweight remote exact-head gate:

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

### Acceptance gates

Run the full debug/release tests, rustdoc, platform matrix, fixture/checksum freeze,
fault cases and external/runtime qualification only when the selected product scope
is functionally ready or the operator explicitly requests it. Record actual
pass/fail/skipped/NotEvaluated outcomes and the exact tested commit. Missing tools,
credentials, network or WoW runtime are skips, never passes.

For source work, build and verify a source manifest with `cargo xtask`, then run the native Ketho annotation driver against the same current local revision. Required final parity uses the same flavor, exact Gethe revision, admitted corpus and normalization profile, including the WoW API Ketho MCP discrepancy matrix where specified.

## Micromodular update boundary

Keep experimental/frequently updated algorithms in small independently buildable
modules behind typed versioned bridges; read `docs/WASM_BRIDGES.md`. Use data-only
updates for compatible Gethe/Ketho resource changes and separately built Wasm for
algorithm changes. Never embed a donor inventory, execute repository scripts or
add a generic plugin capability. Full driver/service routing remains explicit.

## Publication checkpoint

Publish each coherent authorized checkpoint without force-pushing; never leave its
only copy in a temporary VM. A local commit, detached GitHub object or downloadable
patch is not remote branch publication.

When local Git transport is unavailable, use authorized GitHub API write actions.
Do not infer read-only access from missing VM network or credentials. Re-read
remote `main` and the changed blob identities after publication; report the
verified remote commit SHA, not merely the locally created SHA. Reconcile a moved
remote head without overwriting another contributor's work.

Publication and validation are separate: record unexecuted checks as
`NotEvaluated`, and never claim a checkpoint is tested or the task complete merely
because its commit is published. A failed write must be reported as unpublished.
