# Repository automation

Current workflows:

- `quick-check.yml` always runs repository policy checks and records the checked
  commit, tree, and Rust version. It runs formatting and strict workspace Clippy
  for Rust/build changes and any other non-documentation-only change, or when the
  comparison base is unavailable. Only recognized documentation-only changes
  skip those checks.
  It does not run tests or a separate compilation step. A green run is policy,
  formatting, and applicable Clippy evidence, not addon workflow coverage.
- `current-source-bundle.yml` generates Lua annotations from selected upstream
  sources and uploads the output and source reports. Projection limitations are
  reported; input failures fail the workflow. The current `Mainline` environment
  default still conflates client flavor with API context; see the
  [recorded defect](../docs/AUDIT.md#flavor-is-confused-with-api-environment).
- `branch-hygiene.yml` is a manually dispatched branch cleanup workflow. It
  defaults to dry-run and checks the main revision and branch eligibility before
  deletion.

Contributor workflow and validation guidance lives in the root
[AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md). The live
implementation status and ordered work queue are in
[CURRENT_STATUS.md](../docs/CURRENT_STATUS.md) and
[REFACTOR_QUEUE.md](../docs/REFACTOR_QUEUE.md).
