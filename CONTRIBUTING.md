# Contributing

Follow [AGENTS.md](AGENTS.md). Read the live
[current status](docs/CURRENT_STATUS.md), the full
[manifesto](docs/ARCHITECTURE.md), the relevant
[implementation route](docs/IMPLEMENTATION.md) and its actual code and donor.

A useful change makes an addon check, annotation generation or source lookup work
better and removes the displaced implementation. Keep normal inputs and helpful
output; do not add a framework to preserve obsolete internal contracts.

The owner's current validation policy is product first, formatting and affected
strict Clippy for Rust, with tests deferred until functional completion. For
prose, `cargo xtask check` validates local Markdown paths/anchors and skill
synchronization; still inspect source claims and current-status consistency. A
compiler result is not workflow acceptance.

A PR needs the problem, what changed, what was verified and any material gap.
Use the existing tests as source references where useful; do not restore old
requirements because their fixtures exist. Preserve notices for reused code and
data and preserve other contributors' unpublished work.
