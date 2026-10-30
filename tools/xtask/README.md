# Repository maintenance

Current commands include `cargo xtask check` for repository/file policy and
`cargo xtask sync-skill --write` for the four instruction copies. Inspect
[src/main.rs](src/main.rs) for other current command dispatch.

`check` validates distributable regular files, forbidden interpreter assets,
JSON syntax/duplicate object keys, local Markdown target paths and GitHub-style
anchors, then checks that the three discovery copies match the canonical skill.
It ignores external URLs and validates local inline/reference/HTML links. See
[src/main.rs](src/main.rs), [file policy](src/repository.rs),
[Markdown checks](src/markdown.rs) and [skill synchronization](src/skill.rs);
reading only one owner misses the full command. The inventory includes tracked
and nonignored untracked files, so keep scratch output outside the repository.

## Check failures

Run from the repository root. The locked equivalent of the Cargo alias is
`cargo +stable run --locked -p xtask -- check`. Exit 0 means these repository
checks passed; 3 denotes skill-copy drift; command/check errors exit 2.

| Failure | Fix the source, then rerun `check` |
| --- | --- |
| Missing Markdown target or anchor | Follow the reported link; correct its target or inbound references to a moved section. Preserve a useful old anchor when appropriate, not an empty compatibility document. |
| Skill copies differ | Edit `.agents/skills/wow-dev/SKILL.md`; run `cargo xtask sync-skill --write` and inspect the three discovery copies. |
| Missing/nonregular/forbidden inventory file | Inspect `git status --short` and the reported path. An unstaged deletion may still be in the index: stage only intended removals. Do not delete another contributor's untracked files or bypass the policy. |
| Invalid JSON or duplicate decoded key | Correct the named input; parsing into a value that overwrites duplicates is not a repair. |

This checker does not validate external links, Rust symbol names, factual claims,
Actions YAML/shell semantics or runtime behavior. Review those only where the task
changes them; [CONTRIBUTING.md](../../CONTRIBUTING.md) selects the appropriate checks.
Do not add a parallel validator or interpreter runtime for documentation work.

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
