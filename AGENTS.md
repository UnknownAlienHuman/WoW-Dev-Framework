# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor.

## Current execution priority

Implement missing functional code first, then build it and run focused checks.
Do not turn expanding unit-test matrices or package fixture acceptance into a
prerequisite for writing missing owners/apps. Preserve existing tests and report
unexecuted acceptance separately.

Read [docs/WORK_QUEUE.md](docs/WORK_QUEUE.md), then only the selected task PR and
its owner contracts. The operator's 2026-09-25 request explicitly authorizes the
named W01–W26 task branches and draft PR queue. For these tasks this route
supersedes historical main-only/no-task-branch and bootstrap-first instructions,
including older owner/status routers. Semantic, authority and security contracts
are unchanged. Do not load every task into one agent context.

## Current implementation frontier

- Read `docs/AUDIT_2026-09-25.md` for the verified baseline and concrete findings,
  `docs/WORK_QUEUE.md` for work selection, and `docs/PROJECT_COMPLETION_MATRIX.md`
  for the code/acceptance distinction. Older status snapshots that say no Rust
  or no workspace exists are stale, not instructions to recreate implemented code.
- At the audited baseline `Cargo.toml` activates 15 members, including real
  `wow-emmy`, `wow-project`, `wow-rules`, `wow-service`, `wow-store`, `wow-graph`
  and `wow-recognizers` slices. Re-read current Cargo membership before changing it.
- Partial executable code and ordinary CI are not complete package acceptance.
  Required E0 fixture/identity/checksum gates remain open; `apps/wow` has real
  materialized-input status/check and retained graph commands. See
  `apps/wow/LOCAL_INPUT.md` and `apps/wow/GRAPH_STORE.md`. Full R0 remains unaccepted.
- Native reference/annotation production remains source-driven and nonexecuting.
  Guarded fast-forward of existing standalone source checkouts is available;
  managed cloning and GitHub-only materialization remain incomplete.
- `apps/wow-reference-builder` contains inactive source with service/contract
  mismatches; it is not tested by the root workspace. W06/W07 repair its service
  and frontend boundaries before explicit activation.
- Continue in the existing selected task branch. Re-read remote branch/main
  heads; coordinate one writer per file/semantic seam; publish and read back
  coherent checkpoints without force-pushing. Do not create duplicate task PRs.
- W01 is code-written/unverified; other initial work PRs are specifications.
  A task document or published commit is never evidence a feature is implemented.
- Current commands, source-update policy and nonclaims:
  `docs/IMPLEMENTATION_STATUS.md`. I0–I7 remains the normative scope, while the
  current queue selects missing work from the actual code. W26 reconciles stale
  machine/human status snapshots without weakening their gate requirements.

The public repository must remain useful without any operator-only context source.

## Rust port direction

Ketho/vscode-wow-api is the primary implementation donor for the WoW annotation
service, not merely a comparison oracle. Read the actual donor modules and
`docs/KETHO_RUST_PORT.md` before changing source loading, normalization, type
lowering, annotation output, or consumer integration. Port their behavior into
Rust within the existing owner crates. Do not invent an unrelated extractor or
add Python code, embedded interpreters, wrappers or interpreter-based tests.
The repository and CI are Rust-native; `cargo xtask check` enforces this policy.
Ketho output is also the parity baseline; current Gethe source remains the
authority for current Blizzard facts.

## Mandatory route

Before any WoW task, read the target package and `.agents/skills/wow-dev/SKILL.md`. Resolve the requested flavor and moving source selector at operation start. Prefer a local Blizzard UI checkout and use GitHub only as fallback. Record the exact revision/version inspected, read all files from that same revision, and re-resolve on the next operation.

Do not hard-code a client build, Interface value, source revision, toolchain patch, dependency patch, or provider revision as permanent project truth. Exact identities belong to one evidence generation; moving selectors remain moving.

## Source updates

- `auto`: clone a missing managed checkout or fast-forward a clean, non-diverged checkout.
- `prompt`: report and ask before updating interactively.
- `never`: report without mutation.

Never reset local changes, rewrite divergence, change an unexpected origin, or switch an operator-owned branch. When network verification is unavailable, report `unverified-current`.

The implemented write is `cargo xtask update-source <checkout> <branch> --expected-head <observed-SHA>` under explicit operator authorization. Read `docs/SOURCE_CHECKOUT_UPDATES.md`; use an exclusively owned standalone checkout. Check-only remains `check-source`. Automatic clone/scheduling and durable recovery are not implemented by this command. A retained update lock requires reconciliation, not blind deletion or retry.

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

Record implementation and acceptance separately: `specified`, `coding`,
`code-written-unverified`, `focused-checks-passed`, `reviewed/merged`,
`package-accepted`. Update each state only from its actual evidence. Keep task-only
or unverified PRs draft. Merge only completed, checked, reviewed and authorized
work; never merge a specification as a completed feature. Delete task branches
only after their work is safely merged or explicitly superseded, not while queued.

Run applicable checks and report exact command, head, platform and outcome:

```text
cargo xtask check
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Use focused package checks during implementation, preserving required repository
checks and CI. Missing tooling, credentials, network or WoW runtime is `NOT-RUN`
or `NotEvaluated`, never a pass. For source work, build/verify a source manifest
with `cargo xtask`, then run the native Ketho driver against the same exact source.

## Micromodular update boundary

Keep experimental/frequently updated algorithms in small independently buildable
modules behind typed versioned bridges; read `docs/WASM_BRIDGES.md`. Use data-only
updates for compatible Gethe/Ketho resource changes and separately built Wasm for
algorithm changes. Never embed a donor inventory, execute repository scripts or
add a generic plugin capability. Full driver/service routing remains explicit.

## Publication checkpoint

Publish each coherent checkpoint to its authorized task branch; never leave its
only copy in a temporary VM. A local commit, detached GitHub object or downloadable
patch is not remote branch publication. Opening this queue does not authorize
force-pushes, automatic merges, CI disabling, provider activation or public release.

When local Git transport is unavailable, use authorized GitHub API write actions.
Do not infer read-only access from missing VM network or credentials. Re-read the
remote task head, main and changed blob identities after publication; report the
verified remote commit SHA, not merely the locally created SHA. Reconcile moved
heads without overwriting another contributor's work.

Publication and validation are separate: record unexecuted checks explicitly,
and never claim a checkpoint is tested or complete merely because its commit is
published. A failed write must be reported as unpublished.
