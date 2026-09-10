# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor.

## Current implementation frontier

- `wow-core`: executable deterministic boundary primitives.
- `wow-reference`: deterministic reference view plus generated API and UI topology imports; full owner acceptance and persistent channel publication remain incomplete.
- `wow-emmy`: active exact-pin analyzer adapter with bounded content-addressed Lua workspaces and normalized Lua/doc syntax diagnostics over caller-supplied bytes. Combined Main/Library semantics, reference/local-flow facts, generation envelopes, and full E0-C acceptance remain incomplete.
- Native source production consumes an explicit local Git checkout. Guarded fast-forward updates of existing standalone checkouts are available through `cargo xtask update-source`; managed cloning and GitHub-only materialization remain incomplete.
- Blizzard source manifest: exact per-operation source inventory with file hashes and Git object identity.
- Generated API input: reference-owned EmmyLua AST evaluation and typed native model.
- Legacy JSON importers remain compatibility readers, not the current source-production path.
- `wow-annotations`: active Rust Ketho emitters plus native source-to-library
  projection; corrections, full ReferenceView publication and semantic consumer
  probes remain incomplete. `examples/native_library.rs` is the native driver.
- Current executable scope and nonclaims: `docs/IMPLEMENTATION_STATUS.md`.

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

Implement the smallest coherent owner responsibility in dependency order. No placeholder crates, fake adapters, fake success, broad speculative traits, or `todo!()` surfaces. Keep parsers bounded and non-executing. One operation uses one source revision. Tests verify fixtures and never silently rewrite them. Update status docs only after executable checks pass. Merge completed work into `main` and delete temporary branches immediately.

Run applicable checks and report pass/fail/skipped/NotEvaluated:

```text
cargo xtask check
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

For source work, build and verify a source manifest with `cargo xtask`, then run the native Ketho annotation driver against the same current local revision. Missing tooling, credentials, network, or WoW runtime is a skip, never a pass.

## Micromodular update boundary

Keep experimental/frequently updated algorithms in small independently buildable
modules behind typed versioned bridges; read `docs/WASM_BRIDGES.md`. Use data-only
updates for compatible Gethe/Ketho resource changes and separately built Wasm for
algorithm changes. Never embed a donor inventory, execute repository scripts or
add a generic plugin capability. Full driver/service routing remains explicit.

## Publication checkpoint

Publish each coherent authorized checkpoint to `main` without force-pushing;
never leave its only copy in a temporary VM. A local commit, detached GitHub
object or downloadable patch is not remote branch publication.

When local Git transport is unavailable, use authorized GitHub API write actions.
Do not infer read-only access from missing VM network or credentials. Re-read
remote `main` and the changed blob identities after publication; report the
verified remote commit SHA, not merely the locally created SHA. Reconcile a moved
remote head without overwriting another contributor's work.

Publication and validation are separate: record unexecuted checks as
`NotEvaluated`, and never claim a checkpoint is tested or the task complete merely
because its commit is published. A failed write must be reported as unpublished.
