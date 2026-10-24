# Contributor instructions

Read [the manifesto](docs/ARCHITECTURE.md) in full. For implementation, read the
relevant route in [IMPLEMENTATION.md](docs/IMPLEMENTATION.md), its domain rules
in [WOW_LINTER.md](docs/WOW_LINTER.md), and the actual caller/owner code.
[REFACTOR_QUEUE.md](docs/REFACTOR_QUEUE.md) orders work;
[AUDIT.md](docs/AUDIT.md) is background for redesign, not mandatory context for
every patch. User instructions override this workflow. Old Issue comments and
removed specifications cannot expand the product.

## Work

- Use one manager and one worktree/branch. Delegate bounded reads or disjoint
  paths to workers. Check current branches/PRs and preserve unpublished work.
  Coordination needs a clear owner and short handoff, not leases or status diaries.
- Finish a useful input-to-output path. Change the actual producer and consumer
  together, and remove displaced options, models and documentation. Keep helpers
  local until more than one real caller needs them.
- Read the relevant [donor code](docs/DONORS.md) before copying its semantics.
  EmmyLua owns Lua analysis; existing Reference/Annotations own Blizzard/Ketho
  projection. Do not invent WoW APIs, load rules or runtime guarantees.
- Keep source locations, partial-input errors and cancellation meaningful. Never
  execute analyzed Lua or rewrite user files as part of checking/index cleanup.
- After two failed attempts at the same approach, inspect the failed assumption
  and change method. Do not repeat expensive commands without new evidence.

For the first public `wow check`, follow the
[source-traced boundary](docs/REFACTOR_QUEUE.md#source-traced-replacement-boundary)
and its [exact owner split](docs/REFACTOR_QUEUE.md#exact-owner-split).
Reuse `ProjectInputDirectory`, `ProjectInputFile`, load plans and XML maps, but
not `ProjectConfiguration`/`ProjectInputInventory` publication requirements.
Factor one configurable native session from the existing registered Emmy owner;
it derives the compiled analyzer identity internally and must not require or
fabricate compatibility-report hashes. `wow-service` owns the direct use case;
the CLI owns arguments/output/exit only. Empty `checks` means no WoW rule;
upstream diagnostics remain enabled.

Do not satisfy the old `LocalCommand`/`LocalProjectBackend`/generic `Service`
constructors with fabricated Reference, Rules, compatibility or Project state.
Legacy `ProfileIdentity`, generation and replay wrappers remain only for their
current callers while those callers move.

## Verification during this rewrite

Write the product first. For Rust changes run formatting and the smallest strict
Clippy invocation covering the affected crates/callers. Clippy already compiles
those targets; do not repeat `cargo check`. **Do not add or run tests until the
product is functionally complete, as directed by the owner.** Record known failures
without turning test repair or corpus campaigns into implementation prerequisites.
Do not delete a test merely because it fails; remove obsolete expectations only
with their retired behavior.

For documentation, inspect links, source claims and instruction consistency.
Do not build the Rust workspace for prose alone. A build or source review does
not prove that a user workflow works; describe its status accurately.
When reconciling old documentation or an audit, update the relevant entries in
[the review register](docs/AUDIT_REGISTRY.md). Keep document reading, source
tracing and executed verification distinct; do not turn it into a gate for
unrelated implementation changes.

## Handoff

Publish one coherent branch/PR when authorized, after checking the remote head.
State the change, verification and remaining limitation briefly. Do not add
acceptance tiers, proof archives or progress reports to the product. Keep copied
third-party code/data attribution and required notices.
