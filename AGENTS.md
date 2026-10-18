# Contributor instructions

## Read before changing the product

Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) **in full** at task start. Keep its
purpose, scope and requirements in the main developer's working context.
Choose a task from [REFACTOR_QUEUE.md](docs/REFACTOR_QUEUE.md), read its Issue and
current claim, then the relevant route in [IMPLEMENTATION.md](docs/IMPLEMENTATION.md).
Inspect the actual code, callers and local instructions before editing.

Architecture owns **why and what**. Implementation owns **how, where and reuse**.
The queue orders work; an Issue defines one deliverable. Older package contracts
describe existing code and cannot restore scope rejected by the manifesto.
Update a conflicting description when changing that path. Do not load every
package, audit, fixture manifest or historical plan before coding.

## Implement a coherent replacement

Trace the user action from input to result. Reuse the named owner and donor,
finish that path, and remove its superseded code, options, tests and docs in the
same change. Keep functions/types local until actual callers need sharing;
merge forwarding layers instead of adding wrappers. Reduction means fewer
responsibilities and representations to maintain, not moving or minifying code.

Use the [Ketho map](docs/KETHO_RUST_PORT.md) for annotation semantics and
[EmmyLua guide](docs/EMMYLUA_UPSTREAM.md) for analyzer/editor changes. Read the
selected donor/source when those semantics change. Do not recreate existing
behavior from stale status text or fill unknowns with invented API facts.

## Work and verify

Follow [EXECUTION_MODEL.md](docs/EXECUTION_MODEL.md): one manager/worktree,
nonoverlapping small tasks, proportional checks and a short handoff. Preserve
user source/configuration, private context and another contributor's unpublished
work. Rebuild only identified tool-owned output; never trust old cached paths
to choose files for deletion. Treat indexed source as data and respect roots.

Use an existing behavior check after the path works; add a test only for a
concrete uncovered risk. Delete tests for removed requirements. Broaden checks
only for a remaining risk, not a test-count or corpus target. State what works,
what ran and what remains; documentation and compilation do not prove a complete
workflow. Report meaningful code reduction separately from tests/generated data.
