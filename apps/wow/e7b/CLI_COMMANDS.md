# E7-B public release/update CLI grammar

**Status:** normative.

## Common options

```text
--config <PATH>
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--operation-id <ID>
--expect-digest <SHA256>
--expect-current-digest <SHA256>
--consumer-profile <ID>
--budget-profile <ID>
```

No implicit configuration or project/install discovery is performed by the app. Exact local installation lookup, when permitted, is a typed service selector resolved once and returned with a resolution receipt.

## Version and status

```text
wow version
    -> release_status

wow release status
    -> release_status
```

`version` uses a compact output projection but the same exact service operation. It reports compiled source/build/release/target/registry/compatibility IDs and installation state if available. It performs no network access.

## Bundle verification

```text
wow release verify bundle
    --input <PATH>
    [--expect-digest <SHA256>]
    -> release_bundle_validate
```

The path is transport input only. Service/owner validates archive/member/signature/trust/support closure. The app does not extract, follow manifest URLs or execute content.

## Installation validation

```text
wow installation validate
    [--installation <ID>]
    [--expect-current-digest <SHA256>]
    -> release_installation_validate
```

An omitted installation ID is allowed only when the service profile has one explicit local installation binding and returns an exact resolution receipt. The app does not inspect PATH, registry, cwd or executable directory to infer it.

## Update check

```text
wow update check
    --channel <ID>
    [--installation <ID>]
    [--network-policy <ID>]
    --operation-id <ID>
    -> release_update_check
```

Explicit invocation authorizes only the exact bounded check under service policy. It does not authorize download, install, telemetry or remote configuration.

The result is one of the exact service states such as `NoChange`, `UpdateAvailable`, revoked/retired/unsupported/blocked/`NotEvaluated`/failed.

## Update plan

```text
wow update plan
    --installation <ID>
    --update-manifest <ID|PATH>
    --operation-id <ID>
    [--expect-current-digest <SHA256>]
    -> release_update_plan
```

The plan validates exact target bundle/support, materialization, staging, backup, migrations, helper, self-check, LKR and rollback. It does not modify the installation.

## Update apply

```text
wow update apply
    --update-plan <ID>
    --operation-id <ID>
    --expect-current-digest <SHA256>
    -> release_update_apply
```

No replacement fields are accepted. The app cannot choose another manifest/bundle/channel/helper/migration/rollback target. Service invokes the exact installation owner and returns/hands off according to the Windows profile.

## Update rollback

```text
wow update rollback
    --installation <ID>
    --rollback-target <ID>
    --operation-id <ID>
    --expect-current-digest <SHA256>
    -> release_update_rollback
```

`--previous`, `--newest`, `--last`, directory path and version-only selectors are invalid. The rollback target must be an exact retained qualified installation/LKR record.

## Reconciliation

```text
wow update reconcile
    --operation-id <ID>
    --request-digest <SHA256>
    [--installation <ID>]
    -> release_operation_reconcile
```

This reads/reconciles the exact existing operation. It accepts no target replacement and never repeats build/download/install/migration/helper/rollback effects.

## Forbidden options

```text
--latest
--newest
--previous
--last
--best
--force
--ignore-signature
--ignore-revocation
--ignore-compatibility
--skip-backup
--skip-self-check
--allow-downgrade
--run-script
--command
--shell
--url
--token
--credential
--private-endpoint
--helper-path
--provider-api
--retry-unknown
--auto-update
```

An automatic-update policy, if ever supported, is a separate exact reviewed configuration/service operation, not a convenience flag that bypasses explicit stages.

## Input files

Strict bounded JSON/manifest/bundle inputs are accepted only by commands that declare them. One stdin consumer maximum. No include, interpolation, environment expansion, archive extraction in app, script/plugin execution or URL fetching.

## One-call rule

After transport parsing each valid command invokes exactly one service operation. The app cannot compose check -> plan -> apply or apply -> rollback automatically.