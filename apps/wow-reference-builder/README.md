# `wow-reference-builder`

This workspace binary currently builds, validates, and compares Reference Pack
outputs through `wow-service`. It includes a private materialization journal,
staging, backup, read-back, retry/reconciliation, and pack eligibility flow.
Those are real existing behaviors, not the product target.

The intended developer path is native Ketho annotation generation from an
explicit Blizzard source revision and flavor, writing an ordinary `.lua`
library into a user-selected output directory. That replacement is tracked by
[#196](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196).
Until that work lands, the current pack-oriented commands and request files
remain the implemented interface; do not describe them as the finished native
workflow.

## Current commands

```text
wow-reference-builder build --request <build-command.json> \
  --source-root <dir> --output <dir> [--json]
wow-reference-builder validate --pack <dir> \
  [--expect <validation-expectation.json>] [--json]
wow-reference-builder rebuild-compare --request <rebuild-command.json> \
  --source-root <dir> --scratch-root <dir> [--json]
```

`src/main.rs` handles signal cancellation and delegates to
`wow_reference_builder::run_with_stop` in `src/lib.rs`. The application currently
depends on Service; source loading, reference facts, annotation semantics, and
pack validation live below that boundary. Consult the implementation for exact
request fields and current exit behavior.

Keep all source handling explicit, rooted, bounded, and non-executing. Preserve
existing user source/configuration and avoid replacing a destination with a
partial output. The target output and generated intermediate files are derived
and rebuildable. Do not extend pack promotion, signing, release, install,
update, or migration lifecycles while completing #196.

For current product scope and task order, see root [`AGENTS.md`](../../AGENTS.md),
[architecture](../../docs/ARCHITECTURE.md), and
[reduction queue](../../docs/REFACTOR_QUEUE.md).
