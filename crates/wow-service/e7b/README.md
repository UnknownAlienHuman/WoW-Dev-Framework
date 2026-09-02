# `wow-service` E7-B reproducible release, distribution, installation, update, and support lifecycle

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `wow-service/e7-b/release-distribution-update-support-lifecycle`

## Mission

Turn one exact reviewed source snapshot and frozen implementation/compatibility profile into reproducible unsigned product artifacts, complete supply-chain evidence, detached signatures, one immutable release bundle, exact channel/update manifests, and guarded install/update/rollback/revocation/retirement effects without exposing build/signing/distribution credentials or bypassing earlier service authority boundaries.

```text
exact source tree/commit and release plan
+ pinned Rust/toolchain/dependency/vendor inputs
+ exact implemented operation registry and compatibility manifest
+ supported target/platform/profile matrix
+ build executor and release authorization ports
-> validate source and all implementation/checksum gates
-> materialize hermetic build inputs
-> build unsigned artifacts on independent builders
-> compare reproducible semantic bytes
-> validate executable/schema/store/protocol self-description
-> build SBOM, provenance, license and notice artifacts
-> sign exact domain-separated digests through external signing ports
-> build and validate immutable release bundle
-> validate install/upgrade/rollback/support/retirement plans
-> publish exact release and update manifests to an explicit channel
-> install/update/rollback only through exact staged verified effects
-> retain/audit/reconcile every effect
```

## Public operations

```text
release_status
release_source_validate
release_plan_validate
release_build_submit
release_build_get
release_rebuild_compare
release_artifact_validate
release_sbom_build
release_provenance_build
release_sign_request
release_signature_validate
release_bundle_build
release_bundle_validate
release_support_matrix_validate
release_candidate_validate
release_channel_prepare
release_channel_publish
release_channel_get
release_update_manifest_build
release_update_manifest_validate
release_installation_validate
release_update_check
release_update_plan
release_update_apply
release_update_rollback
release_revoke
release_retire
release_operation_reconcile
```

## Product and tool boundary

Public product:

```text
wow
    one-shot CLI
    local daemon
    LSP stdio
    MCP stdio
    version/install/update/rollback client operations
```

Internal release tool:

```text
wow-release
    exact build/evidence/sign/bundle/channel operations
```

Both depend on `wow-service` only among framework crates. The release tool is not included in the public product bundle unless an explicit administrative distribution profile says so.

## Release unit

One `ReleaseBundle` contains immutable exact artifacts such as:

```text
wow executable for one target
transport compatibility manifest
service operation registry and public schemas
license and third-party notice set
SBOM and provenance attestations
default nonsecret configuration examples
installation/update compatibility metadata
checksums and detached signature envelopes
bounded user documentation required by the release profile
```

Reference Packs, core recognizer packs, external provider adapters, and private configuration are separate signed artifacts with independent compatibility and update lifecycles. An explicit offline bundle may include exact compatible data packs, but they never become implicit mutable contents of the executable release.

## Reproducibility

The unsigned semantic artifact set binds one exact source snapshot, `Cargo.lock`, Rust toolchain, target, features, dependency/vendor inputs, build scripts/policies, environment allowlist, timestamps/path-remapping profile, and builder implementation. At least two independent build executions are compared for every target advertised as reproducible.

Platform code signing or package notarization may be nondeterministic and is applied after the reproducible unsigned artifact digest is frozen. It cannot change the identity of the unsigned semantic build.

## Selected launch policy

The first supported implementation/release target is Windows x86-64 using an exact MSVC toolchain profile because that is the owner’s primary development environment. Additional Windows, Linux, or macOS targets remain unadvertised until their exact build, install, path, IPC, protocol, security, and rollback suites pass.

The architecture remains portable, but an untested target is not part of the support matrix.

## Channels

Baseline channel classes:

```text
developer-preview
beta
stable
```

A channel pointer is an exact guarded distribution record, not semantic `latest`. Clients resolve it once to an exact signed `ReleaseManifest` and never mix artifacts from different releases.

Stable is not assigned automatically from age, download count, absence of complaints, or successful publication. Promotion requires the exact release-candidate and support gates for that channel.

## Installation/update policy

Baseline automatic update is disabled. The product may check only when the user explicitly invokes `wow update check` or an exact opt-in policy is configured. Download, verification, staging, install, restart, data migration, and rollback are separate states.

Updates use staging and exact verification before replacing the executable. Existing data/store/configuration is backed up or retained according to the exact migration plan. Failure never deletes the last known runnable installation or silently rolls forward.

## Distribution adapters

GitHub Releases may be one distribution adapter profile, but E7-B defines provider-neutral channel/publisher ports. Repository ownership, CI identity, GitHub login, tag name, or successful upload is not release authorization or verification.

## Security boundary

No private signing keys, code-signing credentials, GitHub tokens, package-manager credentials, KMS/HSM/vault material, private endpoints, arbitrary shell commands, environment blocks, or upload callbacks enter canonical requests, fixtures, logs, public configuration, or release bundles.

Build and distribution executors are narrow allow-listed ports with exact plans. There is no generic command runner.

## Completion gate

E7-B implementation is complete only when source closure, dependency/vendor integrity, reproducible unsigned builds, artifact self-description, SBOM/provenance/license/notices, signing and verification, bundle integrity, support matrix, release-candidate evaluation, channel publication, explicit update/install/rollback, revocation/retirement, response-loss reconciliation, security, platform tests, and release documentation pass with all exact profiles/vectors/checksums frozen.