# Using the tool for an addon task

For work on this repository follow [AGENTS.md](../AGENTS.md) and the
[queue](REFACTOR_QUEUE.md). For an addon task:

1. Read the actual addon instructions, TOC and selected flavor/source. Configure
   the relevant roots/library; do not infer unrelated installed addons as input.
2. Search the current source or inspect its outline, then read the located code
   and TOC/XML relationships needed for the task. Keep snippets small.
3. Check current Blizzard API meaning against that revision. Use Ketho for
   annotation behavior and another addon only as an implementation example.
4. Make the smallest complete change using an existing helper/owner. Run the
   relevant diagnosis or reproduction; static output does not prove runtime safety.
5. Report the change, source used, actual check and any unsupported behavior.

The actual commands are documented in [apps/wow](../apps/wow/README.md).
Do not invent `wow_plan`/provider tools from the old design or make proof
manifests and runtime probes prerequisites for ordinary source lookup.
