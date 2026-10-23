# Repository automation

Current workflows:

- `quick-check.yml`: repository checks, formatting, `cargo check` and strict
  Clippy, plus exact-head/source archive steps. It does not run the test suite.
  The duplicate compile/archive ceremony is not a requirement of the new design.
- `current-source-bundle.yml`: generates and uploads Lua from selected sources,
  rejects input failures and reports partial projection. Its old `Mainline`
  environment default does not solve the API-context defect in the [audit](../docs/AUDIT.md).
- `branch-hygiene.yml`: guarded cleanup of merged ancestor branches.

This documentation rewrite does not change those workflows. Green Quick is
build evidence, not working addon/API coverage. The owner's current validation
policy is in [AGENTS.md](../AGENTS.md). Do not add test/coverage/certification jobs
as a prerequisite to the product-first implementation.
