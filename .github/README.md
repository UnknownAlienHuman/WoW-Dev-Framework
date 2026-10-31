# Repository automation

Current workflows:

- [quick-check.yml](workflows/quick-check.yml) always runs repository policy and
  records the checked commit, tree and Rust version. Formatting and strict
  workspace Clippy run for non-documentation-only changes or an unavailable
  comparison base. Only recognized documentation-only changes skip them.
  No tests or separate `cargo check` run. A green result is evidence for those
  checks, not an addon/editor workflow.
- [current-source-bundle.yml](workflows/current-source-bundle.yml) generates Lua
  annotations from selected upstream sources and uploads output/source reports.
  Projection limitations are reported; input failures fail the workflow. Its
  `Mainline` environment default still conflates flavor with API context; see
  [the recorded defect](../docs/AUDIT.md#flavor-is-confused-with-api-environment).
- [branch-hygiene.yml](workflows/branch-hygiene.yml) is manual branch cleanup.
  It defaults to dry-run and checks main revision/eligibility before deletion.

Source generation and branch cleanup are not validation prerequisites. Do not
trigger them to check a documentation PR.

## Reading a check result

Open the run for the current PR revision, then its jobs/steps. Record the checked
commit/tree printed by `Record checked revision`, toolchain and actual conclusions;
`queued`, `in_progress`, `cancelled` and `skipped` are not successful execution.

Despite the historical name `Quick exact-head gate`, checkout has no explicit
`ref`: on `pull_request`, GitHub normally checks the synthetic merge commit,
not the PR head. See [GitHub event semantics](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request).
The workflow verifies `HEAD == github.sha`, meaning the event checkout. Cite that
actual revision; a run associated with a head SHA does not prove that raw head was
checked. Push/manual events can select a different checkout.

Documentation-only detection compares the entire base-to-checkout diff, not just
the latest commit. Earlier Rust/workflow edits in the same PR keep Clippy enabled.
Manual dispatch and an unavailable/unknown base are conservative. Do not weaken
this decision merely to avoid a build on a cumulative PR.

For a failure, inspect the first failing step/log before rerunning. Policy failures
route to [xtask recovery](../tools/xtask/README.md#check-failures); Rust failures
route to [the check matrix](../CONTRIBUTING.md#choose-the-checks). Workflow changes
also require review of event/checkout behavior, permissions, expression-to-shell
quoting and conditional steps. Green Clippy is not YAML/protocol validation.

Contributor instructions are in [AGENTS.md](../AGENTS.md). The live delivery and
ordered work remain [status](../docs/CURRENT_STATUS.md) and
[queue](../docs/REFACTOR_QUEUE.md), not workflow names or old run summaries.
