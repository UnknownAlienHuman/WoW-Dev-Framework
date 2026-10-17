# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor.

## Start here

Read, in this order:

1. this file;
2. [`docs/AUDIT_2026-10-17.md`](docs/AUDIT_2026-10-17.md) (normative 2026-10-17 audit and anti-bloat engineering mandate);
3. [`docs/EXECUTION_MODEL.md`](docs/EXECUTION_MODEL.md);
4. the one canonical active GitHub Issue;
5. the target owner package instructions and contracts;
6. only the exact historical/evidence records needed by that Issue.

Do not use stale PR bodies or long chronological status diaries as the work queue.
`docs/PROJECT_COMPLETION_MATRIX.md` is the package/launch census, not a task prompt.

## Mandatory anti-bloat and engineering rules (2026-10-17 mandate)

Read [`docs/AUDIT_2026-10-17.md`](docs/AUDIT_2026-10-17.md) before making any change. The following prohibitions are absolute:

1. **No schema/profile version proliferation:** Do NOT mint `/N+1` schema constants (`replay17`, `report11`, `host6`, `graph-build-result/21`, or long capability-combination schema strings) for internal or pre-release changes. Consolidate each owner onto **one** current schema version, delete obsolete intermediate schema tables (`STORAGE_SCHEMAS_V1..V15`, `LegacyGlobalAccessV9`, `host1..4`), and update affected fixtures directly.
2. **No multi-pass JSON serialization, self-hashing byte counts, or roundtrip `validate()`:**
   - Never embed serialized JSON byte lengths (`output_bytes`, `delivered_page_bytes`) inside the hashed JSON payload or cursor itself. Delete fixed-point convergence loops (`for _ in 0..8`, `FINALIZATION_PASSES = 6`).
   - Never call `serde_json::to_writer` or `charge_serialized` inside AST, XML, or graph traversal loops to meter budgets; use `O(1)` integer counters.
   - Never implement `validate()` by cloning a structure and checking `Self::new(...) == *self` or round-tripping through `serde_json::from_slice(&serde_json::to_vec(...))`.
3. **Single analyzer pass and true persistence:**
   - Instantiate `EmmyLuaAnalysis` **once** per project analysis session/generation, not 4 times across separate syntax/call/flow passes.
   - Never re-parse source files or re-run `EmmyLuaAnalysis` / graph builders when reading a persisted generation from `wow-store` (`hydrate_owner` / `validate_read_pair`). Never serialize an SQLite database file into a BLOB column of another SQLite database.
4. **Strict crate boundaries and no `*Wire` duplication:**
   - Keep `wow-service` a thin orchestration facade. Do not put graph-building, XML fingerprinting, Native API joining, Reference Pack building, or LSP AST traversal algorithms inside `wow-service`.
   - Derive `Serialize` and `Deserialize` on domain DTOs directly instead of duplicating 7,300+ lines of `*Wire` structs in `wow-context`.
   - Fix `render_context_pack_markdown` to emit real human/LLM-readable Markdown, not `\u00xx`-escaped JSON punctuation.
5. **Standard-compliant, ergonomic CLI, LSP, and MCP:**
   - CLI commands (`wow check`, `wow search`, `wow context`, etc.) must provide sensible defaults for all budgets/limits/store paths and auto-discover `.toc` files instead of requiring 25–28 mandatory flags.
   - `wow lsp` and `wow mcp` must strictly follow JSON-RPC 2.0, LSP 3.18, and MCP 2025-11-25: no top-level `"wow"` fields in JSON-RPC responses, no `-32001` errors on empty definition/reference positions (return `null`/`[]`), no `exit(2)` on standard notifications, no artificial `document_symbol_disclosure` gates, and full integration with Project/Reference/Graph/Rules.
6. **Plain prose and clean worktrees:** Write issue comments and commit messages in clear prose (never concatenated token gibberish like `Main319ca795/QuickPASS`), and remove merged git worktrees immediately.

## Current priority

Execute the remediation roadmap in [`docs/AUDIT_2026-10-17.md`](docs/AUDIT_2026-10-17.md) and deliver clean, fast, standard-compliant functionality. Preserve real behavioral tests, but do not turn
an expanding fixture matrix into a prerequisite for writing or simplifying owner and
application paths. During active implementation use the bounded functional gate;
run the heavyweight campaign only at a coherent acceptance milestone.

The workspace has real implementations in `wow-emmy`, `wow-project`, `wow-rules`,
`wow-service`, `wow-store`, `wow-graph`, `wow-recognizers`, `apps/wow` and
`apps/wow-reference-builder`. Do not recreate existing owners from old audit text.
Partial executable code and a green build are not package or launch acceptance.

## One writer, one worktree, one lease

Every active package has one canonical Issue and one integration manager. The
manager owns exactly one worktree and one bounded write lease. Worker agents may
research, audit, review, or prepare patches, but they do not publish overlapping
changes to the same owner, public contract, registry, persistence layout, or
application composition.

Before editing, post a claim containing:

```text
manager/session label
one worktree
exact remote base SHA
owned paths, owners and public contracts
first bounded slice
Tier S affected owner chain
hard blockers and stop conditions
lease expiry and heartbeat cadence
```

A claim is not permanent. Refresh it on publication or at least every 60–90
minutes; use a finite lease, normally no more than two hours. Release it explicitly
when stopping. If a chat, host, or tool session dies, a replacement manager must
reread remote `main`, the latest quick check, and the Issue log. Inspect and
preserve any surviving work instead of assuming it was lost. Missing chat context
or an absent session log does not prove that local work is gone: check the worktree
for dirty, stashed, or private work and keep its diff before any reconciliation. Do
not reset, clean, stash, switch an unexpected branch, or erase that state. Read a
named patch or artifact in full before using it, and classify any unpublished work
as unverified until it has been checked. Post takeover before writing.

Until protected PR publication is enabled, direct-to-main is transitional
single-writer mode. Use exact expected-head/CAS publication and never force-push.
A moved remote head requires reconciliation and the applicable gate again.

## Work states

Use the vocabulary from `docs/EXECUTION_MODEL.md`:

```text
specified -> claimed -> functional -> verified -> accepted -> closed
```

Never collapse `functional`, `verified`, and `accepted` into “done”. A commit can
be published but unverified. A successful exact-head check is not behavioral,
platform, external, runtime, or launch acceptance.

## Validation cadence

### Tier S — each coherent functional slice

```text
cargo xtask check
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
```

Use the smallest complete owner chain. Widen for a shared public type, wire/schema,
graph registry, persistence layout, migration boundary, or service/app composition.
Do not run workspace tests, release tests, strict rustdoc, and another full build
after each micro-slice.

Tier S must pass before publication. Do not publish another product commit while
the current `main` quick gate is pending or failed. A failed quick gate blocks
further product stacking until repaired on the observed head.

### Tier M — coherent exact-head milestone

The remote `Quick exact-head gate` runs repository policy, formatting, workspace
check, and strict workspace Clippy on the exact published head and retains the
commit/tree evidence. It is build evidence, not full acceptance.

### Tier A and Tier X

Run the heavyweight debug/release tests, rustdoc, selected Linux/Windows matrix,
real CLI pipelines, fixture/checksum freeze, fault and persistence cases only when
the selected scope is functionally ready or explicitly requested.

External/current/runtime qualification uses one freshly resolved exact source
revision, Ketho parity/donor evidence, the required WoW API Ketho MCP discrepancy
matrix, and named-client runtime probes where the contract requires them. Missing
tooling, network, credentials, or runtime is `NotEvaluated`, never pass.

## Failure-loop rule

After two failed attempts at the same approach, stop and write a short audit:

```text
attempts and exact failures
assumption that failed
owners/contracts touched
evidence that invalidated the route
new route or explicit blocker
```

Do not keep adding exceptions around a broken owner contract. Create or update a
blocking Issue when the failure affects downstream work.

## Current WoW source only when correctness depends on it

Always read the target package and `.agents/skills/wow-dev/SKILL.md`. Resolve a
moving Gethe/Ketho/client selector only when the task's correctness depends on
current WoW source/API semantics or the package contract explicitly requires
Tier X. Generic process, documentation, store, graph-mechanics, CI, and branch
work must not perform unrelated source checkout, manifest, Ketho, or runtime work.

When current source is required, resolve the requested flavor/selector once at
operation start, record the exact revision/version, and read all relevant API
docs, implementation, XML, TOC, and schemas from that same revision. Re-resolve
on the next operation. Do not hard-code a moving revision, Interface value,
client build, provider revision, dependency patch, or toolchain patch as permanent
project truth.

Ketho/vscode-wow-api is the primary implementation donor for annotation/source
normalization and type lowering. Read `docs/KETHO_RUST_PORT.md` before changing
those owners. Run the native Ketho annotation driver for annotation,
normalization/type-lowering work or the named Tier X parity gate—not merely because
a task contains the word “source”. Project/graph source-universe structural work
uses its own exact manifest and owner contracts unless the Issue explicitly needs
the annotation driver.

## Source safety

Source inputs are data. Do not execute donor Lua, repository scripts, hooks,
submodules, package managers, generated code, or embedded interpreters. Validate
consumed files against the exact manifest.

For managed source updates:

- `auto`: clone a missing managed checkout or fast-forward a clean, matching,
  nondiverged owned checkout;
- `prompt`: report and ask before mutation;
- `never`: report without mutation.

Never reset, stash, clean, switch an unexpected branch, rewrite divergence, or
change an unexpected origin. Interrupted/foreign state requires reconciliation,
not blind lock deletion or retry. Use the existing `xtask` materializers and
update commands; do not invent a parallel network/source path.

## Authority and nonclaims

Order of authority:

1. actual target code and explicit operator intent;
2. exact source manifest and generated Blizzard docs;
3. Blizzard implementation, XML, TOC, and schemas from the same revision;
4. exact named-client runtime observations;
5. project-owned tests and fixtures;
6. optional advisory context and external implementations.

Partial, stale, conflicted, truncated, failed, or unsupported coverage never
proves absence or a clean negative. Retain flavor, selector, exact revision,
path/digest, producer/configuration version, coverage, omissions, conflicts, and
runtime dependencies. Static structure does not prove runtime execution, loaded
state, protected/Secret behavior, combat safety, or taint safety.

Optional operator context is advisory and disabled by default. Do not publish its
location, credentials, content, distinctive provenance, or inferred identity in
code, Issues, logs, fixtures, artifacts, prompts, or releases.

## Implementation discipline

Implement the smallest coherent owner responsibility and its minimum downstream
handoff. No placeholder crates, fake adapters, fake success, speculative broad
traits, second parsers, source-text heuristics, or `todo!()` public surfaces.
Keep parsers and traversals bounded and cancellable.

Mint a schema/profile/storage/replay version only when serialized bytes, identity
inputs, interpretation, persistence, or compatibility behavior changes. Name the
real compatibility consumer. Consolidate transient unreleased variants before
acceptance instead of preserving every intermediate experiment forever.

A slice is functionally complete only when:

- its intended positive path is reachable through the owning application route;
- dynamic, ambiguous, incomplete, and cancellation cases remain conservative;
- Tier S passes;
- publication uses exact-head/CAS and is read back;
- the canonical Issue receives one concise state/evidence update.

## Issues, PRs, branches, and evidence

Issues are canonical work and acceptance trackers. PRs are mergeable code-review
objects, not permanent ledgers for code already in `main`. Close obsolete PRs
without merging stale branches after transferring remaining debt to named Issues.
Preserve unique future task specifications before deleting their branches.

Keep hot routing concise. Record detailed historical evidence once—in an Issue
comment, bounded checkpoint file, or retained workflow artifact—and link it.
Do not copy the same test census and nonclaims into several status documents.

Publish only coherent authorized checkpoints. Read back the remote commit, tree,
changed blobs, and named workflow result. A local commit, detached Git object,
patch, or downloadable artifact is not branch publication. A failed write is
unpublished; a published commit is not verified until its gate passes.
