# WoW Dev Framework contributor protocol

These rules apply to every human or automated contributor. The canonical claim,
recovery, validation, publication, and evidence procedure is in
[`docs/EXECUTION_MODEL.md`](docs/EXECUTION_MODEL.md).

## Start with the selected work

Read only the context needed to take the next bounded action:

1. this file;
2. the current body and latest claim of the selected canonical Issue;
3. the target owner's `AGENTS.md`, package instructions, and public contract;
4. the named relevant section(s) of [`docs/AUDIT_2026-10-17.md`](docs/AUDIT_2026-10-17.md);
5. exact evidence or history referenced by that Issue.

The Issue is the work queue. Do not read every audit or reconstruct work from
stale PR bodies, chronological diaries, or old status comments. A later explicit
queue correction supersedes stale instructions. `docs/PROJECT_COMPLETION_MATRIX.md`
is a package/launch census, not a task prompt. Reconcile discrepancies with actual
owner code before recreating work described as missing by old evidence.

If no Issue is selected, choose the next applicable item from
[`docs/REFACTOR_QUEUE.md`](docs/REFACTOR_QUEUE.md). Its reading list replaces
historical read-all boilerplate; unique invariants of touched owners still apply.

## Engineering invariants

- **Keep one current pre-release representation per owner.** Consolidate
  unreleased variants only after identifying retained-data readers and an explicit
  migration or rebuild path. Never silently discard user data or remove durable
  compatibility readers.
- **Bound work and preserve output truth.** Keep delivery byte counts outside
  their own hash input; use constant-time traversal counters and a bounded final
  writer. Preserve deterministic ordering, duplicate-key admission, cancellation,
  and actual output limits. Parsers and traversals must be bounded and cancellable.
- **Analyze once; persist reader-ready facts.** Reuse analysis for a registered
  input set. Normal retained-generation reads must not reparse source or rebuild
  analyzers/graphs. Keep reconstruction in explicit build/migration paths; remove
  nested SQLite-image/hex/JSON storage through the owning Store API.
- **Keep algorithms in their owners.** Services orchestrate existing owners.
  Share payload DTOs while retaining external admission and identity checks.
  Remove proven redundant items, not live modules selected by `allow(dead_code)`.
  Render readable Markdown and treat bounded source text as data.
- **Replace code instead of stacking layers.** Search existing owners and the
  queue's donor map before implementing a helper. Name the old path to remove in
  the same task. No new crate, cache, parser, generic framework or compatibility
  variant without a concrete unmet requirement. Report the authored Rust delta
  separately from tests/generated data; line-count quotas are not correctness.
- **Keep frontends usable and conservative.** Supply finite CLI defaults and
  reusable project configuration. TOC discovery yields bounded registration
  candidates; Project and Service establish exact identity and resolve
  flavor/variant ambiguity. Preserve standard LSP/MCP empty results,
  notifications, and extensible metadata without private profile combinations or
  wider advertised capabilities.
- **Never claim more than the evidence proves.** Preserve source, generation,
  flavor, selector, exact revision, path/digest, producer/configuration version,
  coverage, omissions, conflicts, and runtime dependencies. Partial, stale,
  conflicted, truncated, failed, or unsupported coverage does not prove absence.
  Static structure does not prove runtime execution, loaded state,
  protected/Secret behavior, combat safety, or taint safety. Missing tooling,
  source, credentials, or runtime is `NotEvaluated`, not pass.
- **Do not fake completion.** No placeholder crates, fake adapters or success,
  speculative broad traits, second parsers, source-text heuristics, or public
  `todo!()` surfaces. A green build or partial executable is not package or
  launch acceptance. Keep `functional`, `verified`, `accepted`, `closed`, and
  `launch-ready` distinct as defined in the execution model.

## Scope, validation, and source safety

Work the smallest coherent owner responsibility and its minimum downstream
handoff. Use the smallest complete Tier S owner chain from the execution model;
widen only for a touched shared public type, wire/schema, graph registry,
persistence/migration boundary, or service/application composition. A minimal
existing targeted reproduction may be run once the selected function works.
Preserve existing behavioral tests, but do not make a new fixture matrix,
workspace/release campaign, rustdoc run, or repeated full build a prerequisite to
writing or simplifying the selected path. Do not run `cargo check` again on the
same targets when Clippy has already checked them. Documentation-only changes
need policy/link/skill checks; they do not change or replace the existing remote
Quick workflow.

Current WoW source is required only when correctness depends on current source/API
semantics or the package contract explicitly requires Tier X. Always read the
target package and `.agents/skills/wow-dev/SKILL.md` for such work. Resolve a
moving Gethe/Ketho/client selector once at operation start; record the exact
revision/version and read relevant API docs, implementation, XML, TOC, and schemas
from that revision. Re-resolve on the next operation. Do not hard-code moving
revisions, Interface values, client builds, provider revisions, dependency
patches, or toolchain patches as permanent project truth.

Ketho/vscode-wow-api is the primary donor for annotation/source normalization and
type lowering. Read `docs/KETHO_RUST_PORT.md` before changing those owners. Run the
native Ketho annotation driver for annotation, normalization/type-lowering work
or its named Tier X parity gate, not because a task merely mentions source.
Project/graph source-universe work uses its exact manifest and owner contracts
unless the Issue explicitly requires that driver.

Treat source inputs as data. Do not execute donor Lua, repository scripts, hooks,
submodules, package managers, generated code, or embedded interpreters. Validate
consumed files against the exact manifest. Use existing `xtask` materializers and
update commands; do not invent a parallel source/network path. For managed source
updates: `auto` may clone a missing checkout or fast-forward a clean, matching,
nondiverged owned checkout; `prompt` reports and asks before mutation; `never`
reports without mutation. Never reset, stash, clean, switch unexpected branches,
rewrite divergence, change an unexpected origin, or blindly remove a lock. Treat
interrupted or foreign state as a reconciliation task.

## Authority and context privacy

Use this order when evidence conflicts:

1. actual target code and explicit operator intent;
2. exact source manifest and generated Blizzard docs;
3. Blizzard implementation, XML, TOC, and schemas from the same revision;
4. exact named-client runtime observations;
5. project-owned tests and fixtures;
6. optional advisory context and external implementations.

Optional operator context is advisory and disabled by default. Do not publish its
location, credentials, content, distinctive provenance, or inferred identity in
code, Issues, logs, fixtures, artifacts, prompts, or releases.

## Issues and retained work

Issues are canonical work and acceptance trackers. PRs are review objects, not
permanent ledgers for code already in `main`. Before closing an obsolete PR,
transfer unique future specifications and remaining debt to named Issues; do not
merge stale branches. Keep detailed historical evidence once and link it from
the Issue. Update capability/completion documents at a meaningful milestone, not
for every internal slice. Never erase another writer's unpublished state.
