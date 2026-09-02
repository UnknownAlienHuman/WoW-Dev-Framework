# `apps/wow` E7-B release identity, verification, explicit update, and rollback client

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `apps/wow/e7-b/release-update-client`

The public `wow` executable exposes a narrow user-facing subset of E7-B. It depends only on `wow-service` among framework crates and never implements download, verification, installation, migration, replacement or rollback semantics itself.

## Commands

```text
wow version
wow release status
wow release verify bundle --input <PATH>
wow installation validate
wow update check
wow update plan
wow update apply
wow update rollback
wow update reconcile
```

The exact command-to-service mapping is in `CLI_COMMANDS.md` and `CONTRACT.json`.

## Default network policy

`wow version`, `release status`, `release verify`, and `installation validate` are local-only.

`wow update check` performs network/channel access only when explicitly invoked or when an exact opt-in update policy is configured. It does not download or install.

`update plan`, `apply`, and `rollback` are separate explicit operations. No startup/background check, download, install, telemetry, crash upload or remote configuration.

## Update behavior

The app passes one strict request to service. Service/installation owners:

```text
resolve and verify exact signed update manifest
materialize/download exact bundle into staging
verify members, signatures, support and compatibility
validate backup/migration/rollback plan
close running service/session resources
hand off to the exact verified Windows replacement helper
reconcile activation/migration/self-check
retain last-known-runnable rollback state
```

The app never overwrites its running executable, executes bundle scripts, invokes shell commands, edits store/config files, chooses newest/previous versions, or deletes backup/data.

## Verification output

Every command preserves exact release/bundle/manifest/target/support/install/current/LKR IDs and statuses. Text never equates build, signature, publication, update availability, installation or runtime correctness.

`OutcomeUnknown` is always nonzero and unsafe to retry. `update reconcile` retrieves the exact retained effect state; it is not a retry command.

## Current state

```text
documentation frontier: E7-B
implementation frontier: not-started
first target intent: Windows x86-64 MSVC, unadvertised until complete tests
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```