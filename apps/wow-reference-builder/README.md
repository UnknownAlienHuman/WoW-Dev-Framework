# `wow-reference-builder` E1 application contract

**Status:** partial executable internal frontend. The binary is a workspace member but is excluded from the default public bundle.

This application is the thin E1 host frontend for the `wow-service` Reference Pack operations:

```text
reference_pack_build
reference_pack_validate
reference_pack_rebuild_compare
```

It does not own ReferenceData, SQLite, annotation projection, pack membership policy, eligibility, or determinism semantics.

## Commands

```text
wow-reference-builder build \
  --request <build-command.json> \
  --source-root <dir> \
  --output <dir> \
  [--json]

wow-reference-builder validate \
  --pack <dir> \
  [--expect <validation-expectation.json>] \
  [--json]

wow-reference-builder rebuild-compare \
  --request <rebuild-command.json> \
  --source-root <dir> \
  --scratch-root <dir> \
  [--json]
```

No source root, output, scratch root, profile, or current generation is selected implicitly.

## Command request schemas

### Build

```json
{
  "schema": "wow-reference-builder/build-request/1",
  "source_config": "relative/path/to/local-project.json",
  "request": { "...": "wow_service::reference_pack::ReferencePackBuildRequest" }
}
```

### Optional validation expectation

```json
{
  "schema": "wow-reference-builder/validation-expectation/1",
  "request": { "...": "wow_service::reference_pack::ReferencePackValidationRequest" },
  "eligibility_target": "candidate"
}
```

Without `--expect`, validation reads exact identities from `manifest.json` and requests candidate eligibility with the default reviewed budgets.

### Rebuild compare

```json
{
  "schema": "wow-reference-builder/rebuild-request/1",
  "source_config": "relative/path/to/local-project.json",
  "request": { "...": "wow_service::reference_pack::ReferencePackRebuildComparisonRequest" }
}
```

## Filesystem behavior

Build writes only into a deterministic private staging directory beside the requested output. Every service-declared member is created with create-new semantics, synchronized, reread, hashed, and independently validated. Existing destinations must already be valid candidate packs. Replacement uses a same-parent backup and rename sequence; final output is reopened and independently validated before the previous destination is removed. Failed final read-back attempts rollback to the prior destination and quarantine the failed candidate when possible.

Before any staging or rename effect, the app opens a private sibling SQLite journal and registers an exact materialization operation through `wow_service::reference_pack_materialization`. The request binds the output, deterministic staging/backup/quarantine paths, pack/plan/validation identities, and the original destination pack. Durable checkpoints are written before and after backup, install, final read-back, cleanup, and rollback. Re-running the same command observes the exact filesystem state and either resumes a proven-safe step, returns the completed receipt without repeating effects, restores the prior destination, or stops with `OutcomeUnknown`; blind retry is prohibited for ambiguous observations.

Pack validation rejects traversal, non-UTF-8 member paths, case-insensitive collisions, symlinks/reparse points, special files, member-count overflow, and byte-budget overflow. Rebuild comparison writes its canonical report to an isolated scratch session, reopens it, and removes the session.

Current limitation: the durable state machine is implemented but process-kill/response-loss fault injection and Windows-specific sharing/reparse/rename evidence have not yet been executed.

## Exit codes

```text
0 completed and requested gate passed
2 usage/request/config invalid
3 candidate/partial/blocked requested eligibility
4 validation failed
5 component/build/output failure
6 cancelled
7 security/path/integrity violation
8 unavailable for milestone/profile
```

Machine-readable JSON is authoritative. Human text is a projection of the same typed result.

## Dependency rule

```text
apps/wow-reference-builder -> wow-service
```

No direct `wow-store`, `wow-reference`, `wow-annotations`, SQLite, analyzer, or source-parser dependency.

## Explicit nonclaims

- no network, upload, signing, release publication, or global activation;
- no source, Lua, generated annotation, repository script, or shell execution;
- no claim that process-kill, power-loss, Windows reparse, or cross-version pack replacement acceptance has passed;
- no `ValidatedLocal`, E1 package, or launch-gate advancement from workspace activation alone.
