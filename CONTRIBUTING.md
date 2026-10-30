# Contributing

Start with [AGENTS.md](AGENTS.md). Choose one [queue step](docs/REFACTOR_QUEUE.md)
and its [producer/consumer route](docs/REPOSITORY_MAP.md); this page explains how
to check the change and keep the repository usable for the next agent.

## Before editing

From the repository root, inspect `git status --short`, `git branch --show-current`
and `git rev-parse HEAD`. Read the open PR/head for the same task before choosing a
branch; do not overwrite, reset or stash another contributor's work. Reuse the
manager's single worktree. Historical Issues and fixtures are evidence, not scope.

Define input -> expected output, owner paths, caller and exclusions in the existing
task/PR. Read that handoff and the available donor/library API, then choose the
smallest change under [the implementation decisions](docs/IMPLEMENTATION.md#small-implementation-decisions).
Do not create a planning file or another audit before every edit. A helper without
a consuming caller is not completion. Delegate a bounded read or connected change;
the manager resolves shared ownership instead of joining independently invented
architectures.

## Choose the checks

These are format, source and lint checks, not product tests. Tests remain deferred
under [AGENTS.md](AGENTS.md#verification-and-handoff).

| Change | Required check |
| --- | --- |
| Prose, navigation or instruction changes | Read the changed claims against source; run `cargo +stable run --locked -p xtask -- check` and inspect `git diff --check`. This is the locked form of `cargo xtask check`, not a workspace build. |
| Canonical skill | Edit `.agents/skills/wow-dev/SKILL.md`, run `cargo +stable xtask sync-skill --write`, inspect all four copies, then the prose checks. Do not edit discovery copies independently. |
| Private Rust implementation | `cargo +stable fmt --all --check`, then the affected package command below. Inspect the real producer and consumer even if the type signature did not change. |
| Public Rust type/function/error or serialized field | The Rust checks plus affected reverse consumers. Use the dependency/caller lookup below; compiling only the defining crate is insufficient. |
| Cargo dependency, feature, shared-core or cross-cutting change | Inspect manifest/lock changes and affected targets. Use full workspace Clippy when the affected consumer set is broad or uncertain, not for prose. |
| Workflow or shell | Review triggers, checkout revision, permissions, expressions and quoting; `bash -n` checks changed shell scripts only. Repository policy and Clippy do not validate Actions YAML or shell behavior. See [automation checks](.github/README.md#reading-a-check-result). |

For a private change choose the relevant command, not every row:

| Owner | Strict Clippy command |
| --- | --- |
| Project | `cargo +stable clippy --locked -p wow-project --lib --all-features -- -D warnings` |
| Emmy adapter | `cargo +stable clippy --locked -p wow-emmy --lib --all-features -- -D warnings` |
| Service | `cargo +stable clippy --locked -p wow-service --lib --all-features -- -D warnings` |
| CLI/LSP/MCP app | `cargo +stable clippy --locked -p wow-cli --bin wow --all-features -- -D warnings` |
| Native generator | `cargo +stable clippy --locked -p wow-reference-builder --lib --bin wow-reference-builder --all-features -- -D warnings` |
| Repository tooling | `cargo +stable clippy --locked -p xtask --bin xtask --all-features -- -D warnings` |

`apps/wow` is package `wow-cli`, executable `wow`. Other package names come from
their `Cargo.toml`, not a guessed directory alias. For a public boundary, identify
reverse dependencies and actual calls before choosing additional `-p` selectors:

```sh
cargo +stable tree --locked --workspace --edges normal,build --target all --depth workspace --invert wow-project
git grep -n -F 'ProjectLoadPlan' -- crates apps
```

Substitute the changed package/symbol. The [Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html)
is a dependency map, not proof of which symbol a consumer uses. Check callers,
re-exports and wire conversions. Multiple affected libraries can share one Clippy
invocation; include affected binaries as well. The broad fallback matching CI is:

```sh
cargo +stable clippy --locked --workspace --lib --bins --all-features -- -D warnings
```

Do not add a separate `cargo check`, `--all-targets` or `clippy --fix` pass. Clippy
compiles selected targets; a Linux pass does not check Windows-only code. Record
unverified platform paths when the required toolchain is unavailable. Check the
actual Rust version with `rustc +stable -Vv`; `stable` is not an immutable pin.

## Review the handoff

Read the diff before adding machinery. For each new public API, abstraction,
state store or dependency, identify its current caller or the invariant it
protects; remove additions with neither. Follow the changed input through success,
partial and refusal paths. Look for duplicate file reads, analyzer construction,
source copies and format conversions across the boundary. Compare the displaced
caller and remove its now-exclusive code/options. This is source review, not a
new metrics gate or report template; few lines do not excuse hidden errors or
unsafe shortcuts.

| Area | Read through before considering the change ready |
| --- | --- |
| Source selection | Caller input -> Project admission -> selected bytes/plan -> Service limitations. Missing input, empty input, malformed input and unsafe paths must follow the [scope table](docs/IMPLEMENTATION.md#requested-scope-limitations), not a single catch-all. |
| Analysis/configuration | Validated settings and snapshots -> configured owner -> diagnostics/errors -> reuse decision. Preserve the [engine contract](docs/EMMYLUA_UPSTREAM.md); no second parser or silent default fallback. |
| Generated metadata/maps | Source field -> normalized fact -> emitted file/report -> actual reader. A report field with no consuming reader is not delivered support; inspect [the handoff](docs/KETHO_RUST_PORT.md#output-and-source-locations). |
| Frontend | Arguments/wire input -> shared Service -> rendered result/error/exit. Review exact paths, coordinates and stale-result handling under [FRONTENDS.md](docs/FRONTENDS.md). |
| Removal | Search callers, re-exports, Cargo entries, fixtures, docs and help text. Remove obsolete exclusive assets with the last caller; do not delete still-used code merely because it is labeled legacy. |

Use the queue's completion cases as review questions now; execute and record them
only when the product path is ready and validation is authorized. Compilation,
source tracing and an observed command/protocol result remain distinct evidence.

## Update the right place

| Fact changed | Update |
| --- | --- |
| Delivered/open behavior | Relevant row of [CURRENT_STATUS.md](docs/CURRENT_STATUS.md); do not mark a planned interface delivered because a lower helper compiles. |
| Work order or remaining prerequisite | Existing step of [REFACTOR_QUEUE.md](docs/REFACTOR_QUEUE.md); remove stale instructions rather than append a competing plan. |
| Source evidence or actual operation run | Existing row of [IMPLEMENTATION_TRACE_REGISTER.md](docs/IMPLEMENTATION_TRACE_REGISTER.md), with revision and source or command/result link. Unexecuted stays unexecuted. |
| Behavior/contract | Its [owning guide](docs/README.md#working-guides), plus actual caller/help text where relevant. Architecture changes only when product scope or ownership changes. |
| File/symbol move or new entry point | Affected row of [REPOSITORY_MAP.md](docs/REPOSITORY_MAP.md), inbound links and the documentation index if the reading route changed. |
| Donor/API/protocol revision | [Donor update procedure](docs/DONORS.md#updating-a-reviewed-revision), affected guide, pins and notices. Fetching newer data is not compatibility verification. |
| Contributor/check workflow | [AGENTS.md](AGENTS.md), this page and the affected [automation](.github/README.md) or [xtask](tools/xtask/README.md) instructions. Update open PR descriptions that still prescribe removed procedure. |

Change only affected rows; a prose cleanup does not advance the source-review
baseline or implementation state. Keep historical evidence pinned. No duplicate
status diary, correction counter, generated file inventory or new mandatory gate.

## Failure and handoff

Read the first failing check and its actual command/revision. Broken links or
skill-copy drift use [xtask recovery](tools/xtask/README.md#check-failures). A network,
missing-toolchain or unavailable-target failure is not a code failure or a pass.
Do not remove `--locked`, relax lints, delete tests or refresh all dependencies to
make an unrelated change green. After two failures inspect the assumption and
change approach; record the remaining blocker without a retry diary.

Re-read the remote head before publishing. If only GitHub API access is available,
read files at that exact SHA, create a tree/commit on it and update the same branch
with an expected-head guard. If it moved, reconcile instead of force-overwriting.
Code search may describe the default branch rather than the PR: verify hits at
the task revision. Report local checks unavailable and use actual CI evidence;
never substitute a claimed local run. No local Git credentials does not imply
that the connected GitHub API is read-only.

The PR handoff needs only the behavior changed, checked revision/commands or CI
link, remaining gap and next existing queue step. Do not merge, delete branches,
regenerate Libraries or update source checkouts as a side effect of a docs check.
