# Ready to use and release

[Architecture](ARCHITECTURE.md) defines the product.
[Completion](PROJECT_COMPLETION_MATRIX.md) records its current state.
The old R0/A0/A1/B0/V1 and E0–E7 certification chain is superseded by ADR-058.

A release needs the workflows it advertises:

| Claim | Check |
| --- | --- |
| Native annotation generation | Real selected source produces a usable library; relevant types work in the named consumer. |
| Addon checking | Normal root/TOC/library configuration reaches EmmyLua and produces correct located diagnostics. |
| Blizzard UI search/context | Index a real checkout; symbol/text/source queries work after reopen and source changes. |
| Stdio frontend/platform support | Exercise the implemented operations and one normal workflow on that platform/client. |

Build with the existing Cargo/CI path, package the executable with required
notices and concise usage, and state the tested version/platform. A compiled
binary alone is not evidence that a feature worked. Missing runtime/client
checks limit that claim; they do not mandate a new release subsystem.

No signed pack/channel protocol, custom executor, installer, self-updater,
rollback framework or all-history compatibility matrix is required. Publish
releases only with the owner's authorization. Implementation order is the
[queue](REFACTOR_QUEUE.md), not a separate launch roadmap.
