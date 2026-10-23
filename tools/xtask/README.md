# Repository maintenance

Current commands include `cargo xtask check` for the native repository/file
policy and `cargo xtask sync-skill --write` for the four instruction copies.
Inspect `src/main.rs` for other current command dispatch.

`check` runs the repository/file checks, then checks that the three discovery
copies match the canonical skill. See `src/main.rs::run` and `src/skill.rs::sync`;
reading `src/repository.rs::check` alone misses that second step. It does not
validate Markdown links or documentation reachability. Normal source/library
selection belongs to the product operation, not a certification/materialization
framework.
Follow [AGENTS.md](../../AGENTS.md) for proportional verification.
