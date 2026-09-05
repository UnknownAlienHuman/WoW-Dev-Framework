# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor.

## Current implementation frontier

- `wow-core`: executable deterministic boundary primitives.
- `wow-reference`: deterministic reference view plus generated API and UI topology imports; full owner acceptance and persistent channel publication remain incomplete.
- Source producers consume an explicit local Git checkout. GitHub-only input materialization and managed source auto-update are not implemented in these commands.
- Blizzard source manifest: exact per-operation source inventory with file hashes and Git object identity.
- Generated API producer: safe declarative-Lua parser and normalized reference draft.
- Normalized facts are connected to Rust; next implement a real `wow-emmy` analyzer seam.
- Current executable scope and nonclaims: `docs/IMPLEMENTATION_STATUS.md`.

The public repository must remain useful without any operator-only context source.

## Mandatory route

Before any WoW task, read the target package and `.agents/skills/wow-dev/SKILL.md`. Resolve the requested flavor and moving source selector at operation start. Prefer a local Blizzard UI checkout and use GitHub only as fallback. Record the exact revision/version inspected, read all files from that same revision, and re-resolve on the next operation.

Do not hard-code a client build, Interface value, source revision, toolchain patch, dependency patch, or provider revision as permanent project truth. Exact identities belong to one evidence generation; moving selectors remain moving.

## Source updates

- `auto`: clone a missing managed checkout or fast-forward a clean, non-diverged checkout.
- `prompt`: report and ask before updating interactively.
- `never`: report without mutation.

Never reset local changes, rewrite divergence, change an unexpected origin, or switch an operator-owned branch. When network verification is unavailable, report `unverified-current`.

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
python -m unittest
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

For source work, also build and verify a source manifest and generated API draft against a current local checkout. Missing tooling, credentials, network, or WoW runtime is a skip, never a pass.
