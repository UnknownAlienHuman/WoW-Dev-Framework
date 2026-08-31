# `wow-reference-builder` E1 application contract

**Status:** implementation-ready documentation; no Rust code yet.

This application is the thin E1 host frontend for the `wow-service` Reference Pack operations:

```text
reference_pack_build
reference_pack_validate
reference_pack_rebuild_compare
```

It does not own ReferenceData, SQLite, annotation projection, pack membership policy, eligibility, or determinism semantics.

## Required reading

1. repository `AGENTS.md` and `apps/README.md`;
2. [`../../crates/wow-service/e1/README.md`](../../crates/wow-service/e1/README.md);
3. [`../../crates/wow-service/e1/APPLICATION_BOUNDARY.md`](../../crates/wow-service/e1/APPLICATION_BOUNDARY.md);
4. this file, [`AGENTS.md`](AGENTS.md), [`CONTRACT.json`](CONTRACT.json), and [`examples/`](examples/README.md).

## Responsibilities

- parse explicit command/arguments;
- load explicit request/config JSON;
- construct typed service requests;
- expose one root-confined materialized-source adapter;
- create and manage isolated staging roots;
- execute service-issued materialization/finalization plans;
- emit canonical JSON or human text projection;
- map typed outcomes to frozen exit codes;
- bounded progress and cancellation;
- no hidden defaults, source discovery, network, shell, editor, or release publication.

## Dependency rule

```text
apps/wow-reference-builder -> wow-service
```

No direct `wow-store`, `wow-reference`, `wow-annotations`, SQLite, analyzer, or source parser dependency.

## Required arguments

### `build`

```text
--request <json>
--source-root <dir>
--output <dir>
--json: optional output projection
```

### `validate`

```text
--pack <dir>
--expect <json>: optional exact expected identities
--json
```

### `rebuild-compare`

```text
--request <json>
--source-root <dir>
--scratch-root <dir>
--json
```

No default source root, output, or current profile.

## Exit codes

```text
0 completed and requested gate passed
2 usage/request/config invalid
3 candidate/partial/blocked requested eligibility
4 validation failed
5 component/build failure
6 cancelled
7 security/path/integrity violation
8 unavailable for milestone/profile
```

## Hard stops

- no arbitrary shell command or repository script;
- no network, download, upload, signing, or publication;
- no editor settings or extension changes;
- no execution of Lua, generated annotations, or source;
- no direct domain orchestration outside service;
- no in-place mutation of existing destination;
- no success exit for candidate when validated-local was requested;
- no CI.

## Fixtures

[`examples/cli-cases.json`](examples/cli-cases.json) freezes commands, argument errors, service outcome mapping, filesystem safety, stdout/stderr classes, and exit codes. Checksums freeze before the first Rust commit.
