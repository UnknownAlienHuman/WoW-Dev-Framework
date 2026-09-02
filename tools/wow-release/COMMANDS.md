# `wow-release` E7-B command grammar

**Status:** normative.

## Common options

```text
--config <PATH>
--request <PATH|->
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--operation-id <ID>
--expect-digest <SHA256>
--expect-current-digest <SHA256>
--authorization-reference <ID>
--consumer-profile <ID>
--budget-profile <ID>
```

Every command accepts only its declared fields. One stdin consumer maximum.

## Source and plan

```text
wow-release source validate
    --source-request <PATH|->
    -> release_source_validate

wow-release plan validate
    --release-plan <ID|PATH|->
    -> release_plan_validate
```

Source request must resolve to exact repository/commit/tree content. The tool does not run Git, inspect cwd, choose branch/tag or enumerate files itself.

## Build

```text
wow-release build submit
    --release-plan <ID>
    --operation-id <ID>
    -> release_build_submit

wow-release build get
    --build <ID>
    -> release_build_get

wow-release rebuild compare
    --build <ID> --build <ID>
    --comparison-profile <ID>
    -> release_rebuild_compare
```

No command/Cargo arguments/environment/output-directory options are exposed.

## Artifact and evidence

```text
wow-release artifact validate --artifact-set <ID>
    -> release_artifact_validate

wow-release sbom build --artifact-set <ID> --operation-id <ID>
    -> release_sbom_build

wow-release provenance build --artifact-set <ID> --operation-id <ID>
    -> release_provenance_build
```

Evidence build requests reference exact source/build/test/profile records. Tool cannot invent missing reports or edit evidence.

## Signing

```text
wow-release sign request
    --target <KIND:ID>
    --expect-digest <SHA256>
    --signing-profile <ID>
    --authorization-reference <ID>
    --operation-id <ID>
    -> release_sign_request

wow-release sign validate --signature <ID>
    -> release_signature_validate
```

No private-key, certificate-key, KMS/HSM/vault token, PIN, passphrase, signing socket, platform-signing command or environment secret options exist.

## Bundle and support

```text
wow-release bundle build
    --bundle-request <PATH|->
    --operation-id <ID>
    -> release_bundle_build

wow-release bundle validate --bundle <ID|PATH>
    -> release_bundle_validate

wow-release support validate --support-matrix <ID|PATH|->
    -> release_support_matrix_validate
```

The tool does not zip/extract/repack itself.

## Release candidate

```text
wow-release candidate validate --candidate <ID|PATH|->
    -> release_candidate_validate
```

There is no `--force`, `--ignore-test`, `--skip-signature`, `--assume-supported` or mutate-in-place option.

## Channel

```text
wow-release channel prepare
    --candidate <ID>
    --channel <ID>
    --operation-id <ID>
    -> release_channel_prepare

wow-release channel publish
    --publication-plan <ID>
    --operation-id <ID>
    --expect-current-digest <SHA256>
    --authorization-reference <ID>
    -> release_channel_publish

wow-release channel get
    --channel-record <ID>
    -> release_channel_get
```

Publish accepts one frozen plan and cannot replace candidate/bundle/provider/tag/assets. No direct GitHub API/repository/tag/release/asset options.

## Update manifest

```text
wow-release update-manifest build
    --manifest-request <PATH|->
    --operation-id <ID>
    -> release_update_manifest_build

wow-release update-manifest validate --update-manifest <ID|PATH>
    -> release_update_manifest_validate
```

## Revocation and retirement

```text
wow-release revoke
    --revocation-request <PATH|->
    --authorization-reference <ID>
    --operation-id <ID>
    -> release_revoke

wow-release retire
    --retirement-request <PATH|->
    --authorization-reference <ID>
    --operation-id <ID>
    -> release_retire
```

Deletion/hide of provider assets is not accepted as a substitute for signed records and exact channel/update-manifest effects.

## Reconciliation

```text
wow-release reconcile
    --operation-id <ID>
    --request-digest <SHA256>
    -> release_operation_reconcile
```

No target-changing fields or `--retry` exist.

## Status

```text
wow-release status -> release_status
```

## Forbidden inputs

```text
--latest --newest --previous --first --sole --force
--tag --branch --repository-from-cwd
--command --shell --script --cargo-args --rustc-args --linker-args
--env --token --password --private-key --certificate-key
--kms-token --vault-token --signing-socket
--github-api --provider-api --url --header --upload-callback
--output-directory --installer-command --helper-path
--skip-test --skip-backup --ignore-signature --ignore-revocation
--retry-unknown
```

## One-call rule

After transport parsing, every command invokes exactly one service operation. The release pipeline is intentionally explicit and auditable rather than hidden inside one client script.