# Repository automation

Current workflows:

- `quick-check.yml`: repository policy, Markdown local-path/anchor validation,
  synchronized skill copies, formatting, workspace compilation and strict
  Clippy, plus exact-head/source evidence. It does not run the test suite. Green
  Quick is build/document-integrity evidence, not working addon/API coverage.
- `current-source-bundle.yml`: generates and uploads Lua from selected sources,
  rejects input failures and reports partial projection. Its old `Mainline`
  environment default does not solve the API-context defect recorded in the
  [baseline audit](../docs/AUDIT.md).
- `branch-hygiene.yml`: guarded cleanup of merged ancestor branches.

The duplicate compile/archive ceremony is not a product requirement. The current
implementation checkpoint and open handoffs are in
[CURRENT_STATUS.md](../docs/CURRENT_STATUS.md); contributor validation policy is
in [AGENTS.md](../AGENTS.md). Do not add corpus/coverage/certification jobs as a
prerequisite to the owner-directed product-first implementation.
