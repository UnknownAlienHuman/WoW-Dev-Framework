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

## Updating an explicitly selected source checkout

```text
cargo xtask update-source CHECKOUT BRANCH --expected-head SHA
```

This command mutates only the explicitly selected clean, standalone checkout. It
checks that `HEAD` and the checked-out branch match the supplied guard, observes
that branch on the checkout's HTTPS `origin`, then applies only a fast-forward to
the exact observed commit. It refuses dirty or detached checkouts and never resets,
rebases, stashes or creates a merge commit. This updates a local source checkout;
it does not update an addon, run donor code or change a pinned donor revision.

Exit 0 means the checkout was already current or was fast-forwarded. Exit 3 means
the selected commit was not a confirmed fast-forward (or local history was
incomplete); review the checkout and history. Exit 4 means remote observation or
fetch failed; on fetch failure the object database may have changed, while the
checked-out revision remains unchanged. Exit 5 means the update outcome needs
reconciliation. A retained `.git/wow-source-update.lock` records the guarded and
selected revisions; inspect the checkout and lock before recovery. Do not blindly
retry, reset the checkout or delete the lock. Invalid arguments and refused
preconditions are command errors (exit 2).

This is repository hygiene, not workflow acceptance. Normal addon/source/Library
selection belongs to product operations, and a green check/build does not prove a
working addon, editor or search workflow. Follow [AGENTS.md](../../AGENTS.md) and
[current status](../../docs/CURRENT_STATUS.md) for proportional verification.
