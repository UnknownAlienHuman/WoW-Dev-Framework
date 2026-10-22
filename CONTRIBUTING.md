# Contributing

Read [AGENTS.md](AGENTS.md), the full
[architecture manifesto](docs/ARCHITECTURE.md), the selected route in
[IMPLEMENTATION.md](docs/IMPLEMENTATION.md) and the chosen Issue. The
[source audit](docs/AUDIT.md) is required for architecture/rewrite planning, not
for every small patch.

## What a useful change does

- completes one observable part of Library generation, addon checking or source
  index/search/context;
- removes the displaced code, options, tests and documentation in the same
  change;
- fixes a concrete source/behavior defect without inventing another framework;
- preserves user source/configuration, exact locations, ambiguity, bounded work
  and honest incomplete results.

Existing code, an old test, schema or Issue comment is not justification for a
feature. The current manifesto and queue supersede E1–E7/platform plans.

## Required trace

Before editing, identify:

```text
user input -> actual producer -> handoff -> actual reader -> visible result
```

Do not close a task because one DTO or crate compiles. If a representation is
removed, change its callers and delete exclusive fixtures/docs rather than adding
a compatibility reader.

## Validation

Write code first. Run formatting and strict Clippy for affected crates and
callers. After the path exists, run the smallest existing consumer check that can
fail for the changed behavior. Add a test only for a concrete uncovered risk.
Delete tests for removed obligations. A product claim requires the actual public
workflow; green compilation is only build evidence.

Typical commands:

```sh
cargo fmt --all --check
cargo clippy --locked -p <crate> --all-targets --all-features -- -D warnings
```

Broaden to shared callers or one relevant reopen/protocol case when the change
crosses those boundaries. Do not recreate historical matrices or run an
unrelated full campaign for a bounded change.

## Pull request

State the user problem, replacement, removed path, retained behavior, checks and
remaining limitation. Report production code reduction separately from tests,
docs and generated data. Do not claim runtime/performance improvement without a
measured representative workflow.

## Third-party material

Record provenance and preserve licenses/notices for reused Ketho, EmmyLua,
Blizzard or other input material. Prefer exact source revisions and small
representative fixtures over vendoring another platform.
