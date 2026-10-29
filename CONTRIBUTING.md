# Contributing

Follow [AGENTS.md](AGENTS.md), which links the current status, active queue,
relevant owner guidance, and source to read for the task.

A useful change makes an addon check, annotation generation or source lookup work
better and removes the displaced implementation. Keep normal inputs and helpful
output; do not add a framework to preserve obsolete internal contracts.

For Rust changes, follow the formatting and affected-crate strict Clippy policy
in [AGENTS.md](AGENTS.md); tests are deferred until functional completion. For
documentation changes, inspect factual claims and current guidance.
`cargo xtask check` checks local Markdown paths and anchors and synchronized skill copies; it
does not validate those claims.

When opening a PR, explain the problem, the change, what you verified, and any
material gap. Existing tests can help locate behavior, but their presence does
not make obsolete requirements current. Preserve notices for reused code and
data, and preserve other contributors' unpublished work.
