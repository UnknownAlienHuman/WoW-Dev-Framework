# EmmyLua upstream compatibility

`wow-emmy` has one Lua semantic backend: the upstream EmmyLua analyzer. The framework does not carry a second correctness-path Lua parser.

The upstream is rolling, not permanently pinned. A moving branch is resolved at operation start, one exact revision is used for that operation, and the revision is retained in the resulting compatibility and analysis evidence. The next operation checks the branch again.

`Cargo.lock` may record the dependency revision used by one reproducible framework build. It is not a claim that the revision is permanently current. Scheduled compatibility checks and intentional dependency updates move that build input forward.

## Local clone first

The default checkout location is platform cache storage and can be overridden with `WOW_EMMY_SOURCE_DIR` or `--source`.

```bash
python scripts/check-emmylua-version.py ensure \
  --update auto \
  --json
```

The default public upstream and branch are configurable. Embedded credentials are rejected. Existing Git credentials, SSH agents, or credential helpers remain owned by Git and the operator; this tool never stores secrets.

A managed or existing checkout is updated only when all of these are true:

- it is a Git worktree;
- `origin` matches the configured upstream;
- `HEAD` is on the configured branch;
- the worktree and index are clean;
- the local commit is an ancestor of the current remote branch;
- the update can be applied as a fast-forward.

Dirty, wrong-origin, wrong-branch, ahead, and diverged checkouts are never reset, force-updated, stashed, cleaned, or silently switched. Clone and merge operations disable repository hooks. Submodules and repository scripts are not executed.

Update policies:

- `auto`: clone a missing checkout or fast-forward a safe stale checkout;
- `prompt`: offer the same operation only in an interactive terminal;
- `never`: report status without changing the checkout.

When the network is unavailable, an existing checkout is reported as `unverified_current`, not current. Exact local analysis can continue, but no freshness claim is made.

## Version and surface report

```bash
python scripts/check-emmylua-version.py probe \
  --update auto \
  --output .wow-dev/emmylua-compatibility.json \
  --json

python scripts/check-emmylua-version.py verify \
  --report .wow-dev/emmylua-compatibility.json \
  --json
```

The report records:

- exact commit and tree identifiers;
- branch relation and remote head observed by that operation;
- workspace resolver, edition, Rust version, and license declarations;
- the discovered `emmylua_code_analysis` package path and manifest metadata;
- a deterministic inventory and SHA-256 digest of public Rust symbols;
- required adapter symbols and any missing names;
- explicit limitations and a report self-digest.

The textual symbol inventory is a change detector, not a substitute for Rust compilation or behavioral tests. Once the adapter owns concrete upstream calls, its required symbol set and compile fixtures become the authoritative compatibility probe.

## CI policy

The scheduled compatibility workflow:

1. clones the current upstream branch into an ephemeral local checkout;
2. builds and verifies the compatibility report;
3. compiles the current upstream `emmylua_code_analysis` package with its own lockfile;
4. runs framework manager tests on Linux, Windows, and macOS.

The job has read-only repository permissions and no secrets. It deliberately compiles upstream code as a compatibility test; source acquisition and ordinary indexing do not execute upstream build scripts.

A changed upstream surface is not automatically a framework regression. A missing required adapter seam, compilation failure, or behavioral fixture failure is. The repair is made in the adapter or compatibility layer, then a normal dependency update records the new reproducible build input.
