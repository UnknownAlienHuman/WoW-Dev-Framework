# E7-B release channels, distribution adapters, update manifests, and publication

**Status:** normative.

## Channel model

A channel is an explicit policy and guarded record namespace:

```text
ReleaseChannelProfile
    channel ID/class
    eligible target/support profiles
    mandatory release-candidate gates
    publication/revocation/retirement authorization
    visibility/rollout policy
    manifest/signature/trust-root policy
    distribution adapter policy
    update-check freshness/expiry policy
    retention/audit/support ownership
```

Baseline classes are `developer-preview`, `beta`, and `stable`. Channel class does not change artifact bytes or evidence.

## Exact channel records

`ChannelReleaseRecord` points to one exact release candidate, bundle and signed release/update manifest. Updating a channel uses exact expected-current compare-and-swap.

```text
expected prior channel record/digest
+ exact ReadyForChannel candidate
+ publication authorization
+ distribution plan
-> prepared channel record
-> materialize/upload exact immutable artifacts
-> validate provider receipts and public read-back
-> publish signed exact manifest
-> guarded channel CAS
-> retain/audit/close
```

The channel name `stable`, a Git tag, a release title or a version string is not eligibility proof.

## Prepare versus publish

`release_channel_prepare` is read-only/evidence-building except for its own immutable plan artifact. It validates all exact candidate/channel/target/provider/support/signature/retention inputs and produces a `ChannelPublicationPlan`.

`release_channel_publish` is effecting. It never repairs a blocked candidate or changes bundle bytes. Any changed artifact/manifest creates a new candidate/plan.

## Distribution provider port

```text
ReleaseDistributionPublisherPort
    validate exact provider/channel capability
    create or reuse immutable artifact objects by exact digest
    publish exact release/update manifests
    query/read back by operation/object/digest
    revoke or hide only under exact plan/authorization
    reconcile response loss
    close resources
```

The port accepts typed plan records, not arbitrary API calls, URLs, paths, repository names, shell commands or JSON payloads.

GitHub Releases can implement this port through a reviewed adapter. Other providers may implement the same owner-neutral contract.

## Provider-specific metadata

Repository, tag, release ID, asset ID, URL, ETag and publication timestamp are distribution provenance/receipts. They do not enter portable artifact identity or independently authorize trust.

Public URLs may be included only in signed manifests or channel records according to policy. Clients never construct asset URLs from untrusted version strings.

## Immutable assets

Release bundle, signatures, checksum manifest, SBOM, provenance and public verification metadata are immutable by digest. A provider that permits replacing an asset under the same name must be constrained by digest-specific read-back validation; replacement under an existing release record is forbidden.

A new artifact creates a new release candidate/channel record. No “fix the zip in place.”

## Signed release/update manifest

The signed manifest freezes:

```text
product/channel/release/bundle IDs
exact target/platform/support applicability
artifact locations/provider object receipts
member sizes/digests/signatures/trust roots
minimum/blocked/revoked current versions
installation/update/migration/rollback profiles
release notes/security/advisory refs
expiry/freshness and key/revocation policy
```

The manifest is validated independently after publication. A client resolves the channel once to this exact object and never mixes assets across records.

## Publication read-back

Before channel CAS/success:

```text
query exact provider objects
read manifest and required small verification artifacts from public path
validate bytes/sizes/digests/content types/provider object bindings
validate signatures/trust/revocation/expiry
validate all locations resolve to the expected immutable objects
record availability/coverage/conflicts
```

Large bundle read-back can use exact ranged or full validation according to profile, but a mere successful upload/HEAD response is insufficient.

## Rollout/visibility

Channel publication may be staged by exact cohort/visibility profile. This is distribution visibility, not E5 recognizer canary and not installation/runtime proof.

Stages are finite and guarded. No automatic promotion from time/download count/no complaints. Each stage records exact audience scope, manifest, observations, blockers and rollback/hide plan.

## Response loss

Upload/object creation, release creation, manifest publication and channel CAS are separate effects. Each uses exact operation/request/object identity.

After a timeout/disconnect:

- do not upload/publish again blindly;
- reconcile by operation/object/digest/provider receipt;
- reuse only exact verified immutable objects;
- quarantine duplicate/conflicting releases/manifests;
- keep channel current unchanged unless exact CAS receipt proves otherwise;
- expose `OutcomeUnknown` when unresolved.

## Revocation

`release_revoke` creates and signs an exact revocation record, updates eligible channel/update manifests by guarded effects, and defines client action:

```text
block installation/update
warn and require explicit action
require rollback/deactivation
remove visibility while retaining evidence
point to exact replacement when validated
```

Deleting provider assets alone is not a complete revocation record and can harm verification/audit. Historical manifests/evidence remain retained according to policy.

## Retirement

`release_retire` ends support or channel eligibility at an exact state/time policy. Retirement can leave artifacts available for verification/rollback while preventing new installations/updates. It is not a security revocation unless a separate revocation exists.

## Client update check

`release_update_check` accepts exact installed release/target/profile/channel policy and an explicit user/opt-in network authorization. It fetches one bounded signed manifest through a narrow read port, verifies it, and returns:

```text
NoUpdate
UpdateAvailable
CurrentReleaseRevoked
CurrentReleaseRetired
TargetUnsupported
ManifestExpiredOrInvalid
Blocked
NotEvaluated
Failed
```

It does not download or install the bundle.

## No hidden network

No startup/background update check, telemetry, remote config or asset prefetch in baseline. Exact opt-in schedules, if later enabled, are independent explicit operations with authorization, limits, cancellation, retention and no silent installation.

## Public verification

Published release pages/manifests must provide enough public data to verify exact bundle/member digests, detached signatures, trust roots, SBOM/provenance/license and compatibility without access to private build/signing/distribution systems.

## Nonclaims

Successful channel publication proves exact provider availability/read-back under the publication profile. It does not prove every download, installation, update, platform/client behavior, runtime correctness, future availability or stable support.