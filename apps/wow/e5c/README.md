# `apps/wow` E5-C core-pack publication CLI

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `apps/wow/e5-c/core-pack-publication-cli`

The app is a thin adapter over `wow-service`; it imports no other framework crate.

## Commands

```text
wow core-pack status
wow core-pack submission validate
wow core-pack artifact build|validate
wow core-pack sign request|validate
wow core-pack publication publish|get|list|validate
wow core-pack canary plan|start|status|observe|evaluate
wow core-pack rollout plan|advance|pause
wow core-pack activation get|activate
wow core-pack lkg get|designate
wow core-pack rollback
wow core-pack revoke
wow core-pack deactivate
wow core-pack partition-closure validate
```

Each valid command maps to exactly one E5-C service operation. The app does not select latest/best/previous/default targets; sign locally; read private keys; resolve authorization; build cohorts; inspect private observations; mutate catalogs/current records; reindex projects; touch graph partitions; or distribute public releases.

Read `AGENTS.md`, `CLI_COMMANDS.md`, `OUTPUT_EXIT_AND_SECURITY.md`, `TEST_MATRIX.md`, `CONTRACT.json`, and `examples/`.