# Contributor instructions

## Product and reading order

Build a practical WoW development tool: **upstream EmmyLua, Ketho's annotation
pipeline ported to Rust, editor-independent use, and Blizzard UI repository
search**. The owner's 2026-10-18 scope decision in
[ARCHITECTURE.md](docs/ARCHITECTURE.md) supersedes the earlier E0–E7 platform plan.
Older package contracts describe existing code; they cannot require discarded
features, legacy support, cryptographic envelopes, or acceptance bureaucracy.
Update a conflicting description when changing that path.

Read this file, the selected Issue/current claim, the relevant architecture
section, then the actual implementation and callers. Read a crate's local
instructions when present. Choose work from [REFACTOR_QUEUE.md](docs/REFACTOR_QUEUE.md).
Do not read every package, audit, checkpoint or fixture manifest before coding.
[EXECUTION_MODEL.md](docs/EXECUTION_MODEL.md) owns coordination and verification.

## Write less code

- Trace the user action through producer and consumer before changing it. Reuse
  the current native path and the donor; do not rebuild an implemented feature.
- Keep one current internal representation. Remove old schema/profile readers,
  writers, fixtures and options with their replacement. Derived indexes, caches
  and annotations can be regenerated. A historical fixture is not a supported
  product. No compatibility adapter or migration framework is required.
- Preserve user source, configuration and another contributor's work. Rebuild
  only the tool's identified output/cache. Never reinterpret old bytes as the
  new format; report that a rebuild is needed if inputs are unavailable.
- Remove unused features instead of moving them, hiding them behind flags or
  storing copies in an archive. Git retains history. Remove callers, tests and
  docs belonging only to the deleted feature in the same change.
- Keep functions and types local until a real second caller needs them. Prefer
  concrete types and ordinary Rust/SQLite operations. No new crate, parser,
  generic framework, plugin system, result-proof protocol or speculative trait.
- Service coordinates multi-owner use cases. A single-owner caller may use that
  owner directly. Do not add a wrapper solely to preserve the old crate diagram.
- Analyze a source input once; reuse Emmy's tree/session. Normal queries read
  stored facts, not reconstructed analyzers. Serialize a response once at the
  output boundary; keep byte limits outside any payload identity.
- Report code removed and added, separately from tests/generated data when
  relevant. Moving files, deleting useful tests or minifying code is not a
  reduction in implementation complexity. No arbitrary line-count target.

## Keep the useful correctness boundaries

Use one selected flavor and source revision for a result. Preserve file paths,
source spans, meaningful errors and visible partial/unsupported results. A
search miss or static analysis does not prove runtime safety or execution.
Keep finite defaults, bounded input/output, cancellation, parameterized SQL and
transactional writes. Do not add a budget flag for every internal counter.
LSP/MCP must follow their standards and advertise implemented capabilities only.

Read [KETHO_RUST_PORT.md](docs/KETHO_RUST_PORT.md) for annotation/type changes and
[EMMYLUA_UPSTREAM.md](docs/EMMYLUA_UPSTREAM.md) for analyzer/editor changes.
Resolve a moving source selector once when the task depends on current WoW
semantics; record the revision and flavor. Use actual Blizzard docs, Lua, XML
and TOC from that source. Generic storage or documentation changes need no fresh
WoW corpus. Keep donor attribution; no Lua, Node, Python or VS Code runtime in
the product and no second Lua parser.

Treat indexed source as data. Never execute donor/addon Lua, repository hooks,
submodules, package managers or generators while indexing. Keep file access
within configured roots. Reuse source materializers; preserve dirty/diverged
checkouts and respect the user's update mode. Private optional context stays
private and advisory; do not copy its content or location into public artifacts.

## Finish a bounded task

Implement the complete selected path and delete its superseded path. Use the
smallest relevant existing check or reproduction after it works. Add a test
only for a concrete regression risk not already covered; remove tests for
removed requirements. No new fixture factory, compatibility matrix, full
corpus campaign or test-count target. Check shared-type callers when affected.
State what works, what was checked and what remains; do not invent measurements
or claim implementation from documentation. Out-of-scope work is retired, not
mandatory debt to carry forever.
