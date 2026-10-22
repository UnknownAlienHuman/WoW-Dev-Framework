# Contributor instructions

## Read before changing the product

Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) **in full** at task start. Keep its
purpose, scope and requirements in the main developer's working context. Choose
a task from [REFACTOR_QUEUE.md](docs/REFACTOR_QUEUE.md), read its current Issue,
then the relevant route in [IMPLEMENTATION.md](docs/IMPLEMENTATION.md). Inspect
the actual producer, reader, callers and local instructions before editing.

[AUDIT.md](docs/AUDIT.md) is the complete source/architecture basis for the
rewrite. Read it for architecture, subsystem removal or documentation rewrite;
do not load it into every small implementation task. Architecture owns **why and
what**. Implementation owns **how and where**. The queue orders work; an Issue
defines one deliverable. Old package contracts and Issue comments are history
and cannot restore rejected scope.

## Implement a coherent replacement

Trace the user action:

```text
input -> actual producer -> handoff -> actual reader -> visible result
```

Finish that path and remove its superseded code, options, tests and docs in the
same change. Keep functions/types local until real callers need sharing; merge
forwarding layers instead of adding wrappers. Reduction means fewer maintained
responsibilities and representations, not moving or minifying code.

Use the [Ketho map](docs/KETHO_RUST_PORT.md) for annotation semantics and
[EmmyLua guide](docs/EMMYLUA_UPSTREAM.md) for analyzer/editor changes. Read the
selected donor/source when those semantics change. Do not recreate delivered
components from stale status text or invent unknown WoW API facts.

## Work and verify

Follow [EXECUTION_MODEL.md](docs/EXECUTION_MODEL.md): one manager/worktree,
nonoverlapping bounded work and a short handoff. Preserve user source/config,
private context and another contributor's unpublished work. Rebuild only
identified tool-owned output; never use old cached paths to choose user files for
deletion. Treat indexed source as data and respect roots.

Write code first. Run formatting and strict affected-crate Clippy. Use an existing
behavior check after the path works; add a test only for a concrete uncovered
risk. Delete tests for removed requirements. Broaden checks only for a remaining
risk, not a test-count/corpus target. State what works, what ran and what remains.
Documentation and compilation do not prove a workflow. Report production code
reduction separately from tests/generated data.
