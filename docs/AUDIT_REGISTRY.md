# Documentation work register

This is a short navigation and progress note, not a product specification or a
second implementation queue.

## Historical baseline

The full baseline review remains available at the
[immutable 8ab892f snapshot](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/8ab892f040c42a9877836be8bd9818b98b1455d8/docs/AUDIT_REGISTRY.md).
That review inventories 329 Markdown files at baseline commit
[`c36a058`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/c36a0581190a4d08b59c9a19b69ea728b03da5c3).
Its row-by-row dispositions and source evidence are historical records, not active
instructions.

## Work stages

| Stage | State |
| --- | --- |
| Review current code and documentation at `8ab892f` | Complete; current owner paths traced |
| Consolidate canonical contracts and contributor navigation | Complete; implementation, frontend and Emmy rules have one home each |
| Remove stale templates and validation machinery | Complete; current task templates and proportional CI checks |
| Review failure, empty-input, resource and synchronization boundaries | Source review complete; T08/T13/T34/T35/T38/T39 retain open code defects and owner contracts, including diagnostic-budget classification and LSP version/change admission |
| Review generation handoff, source updates and agent entry points | Source review complete; report contents, allocation/cancellation limits and failure/retry behavior corrected in guidance; T10 remains open |
| Make implementation tasks navigable and maintainable | [Map](REPOSITORY_MAP.md) locates owners; [implementation decisions](IMPLEMENTATION.md#small-implementation-decisions) constrain added abstraction/state; [review/check/update matrix](../CONTRIBUTING.md) covers the handoff. Queue steps require connected consumers, not unused scaffolding. |
| Synchronize documentation with each implementation change | Ongoing; update the relevant status, queue, guide, or trace only when its facts change |
| Record ordinary user-workflow observations | Deferred until the product paths are ready |

The current implementation checkpoint is [CURRENT_STATUS.md](CURRENT_STATUS.md);
the ordered work is in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md). Durable domain
requirements live in [WOW_LINTER.md](WOW_LINTER.md),
[IMPLEMENTATION.md](IMPLEMENTATION.md), [EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md),
[KETHO_RUST_PORT.md](KETHO_RUST_PORT.md), and the focused guides they link to.
Git history preserves earlier specifications and review evidence. None of these
sources makes source inspection or a build an observed user workflow.

## The 13 failures reported by Claude

The detailed, source-level discussion is preserved in the
[pinned historical review](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/8ab892f040c42a9877836be8bd9818b98b1455d8/docs/AUDIT_REGISTRY.md#the-13-failures-reported-by-claude).
It does not claim the reported failures were reproduced. Retained behavior belongs
in the current owner guide and queue; obsolete formats do not create compatibility
requirements.
