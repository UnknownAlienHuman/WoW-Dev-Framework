# Repository maintenance

Current commands include `cargo xtask check` for repository/file policy and
`cargo xtask sync-skill --write` for the four instruction copies. Inspect
`src/main.rs` for other current command dispatch.

`check` validates distributable regular files, forbidden interpreter assets,
JSON syntax/duplicate object keys, local Markdown target paths and GitHub-style
anchors, then checks that the three discovery copies match the canonical skill.
It ignores external URLs and validates local inline/reference/HTML links. See
`src/main.rs::run`, `src/repository.rs::check`, `src/markdown.rs` and
`src/skill.rs::sync`; reading only one owner misses the full command.

This is repository hygiene, not workflow acceptance. Normal addon/source/Library
selection belongs to product operations, and a green check/build does not prove a
working addon, editor or search workflow. Follow [AGENTS.md](../../AGENTS.md) and
[current status](../../docs/CURRENT_STATUS.md) for proportional verification.
