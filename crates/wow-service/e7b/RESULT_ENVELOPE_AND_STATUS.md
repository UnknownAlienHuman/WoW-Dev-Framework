# E7-B result envelopes and conservative release status

**Status:** normative.

Every public operation returns one tagged `ReleaseLifecycleServiceResultEnvelope` containing:

```text
operation/request IDs and operation kind
source/plan/build/artifact/evidence/signature/bundle/candidate IDs
support/channel/update-manifest/install/current/LKR/rollback/revocation/retirement IDs
operation-specific owner result
conservative status and independent gate states
coverage/conflicts/blockers/omissions/NotEvaluated/skipped state
authorization/idempotency/response-loss/reconciliation
privacy/license/security/retention/audit/close/delivery state
mandatory nonclaims
canonical digest
```

## Status set

```text
Complete
NoChange
SourceValidated
BuildSubmitted
BuildComplete
Reproducible
ArtifactValidated
EvidenceComplete
Signed
BundleReady
CandidateReady
Published
UpdateAvailable
InstallPlanned
Installed
Updated
RolledBack
Revoked
Retired
Partial
Truncated
Busy
Blocked
ConflictBlocked
OutcomeUnknown
NotEvaluated
Cancelled
Failed
```

Validation payloads remain:

```text
Valid
Invalid
NotEvaluated
```

## Conservative precedence

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Busy
Truncated
Partial
Revoked
Retired
RolledBack
Updated
Installed
InstallPlanned
UpdateAvailable
Published
CandidateReady
BundleReady
Signed
EvidenceComplete
ArtifactValidated
Reproducible
BuildComplete
BuildSubmitted
SourceValidated
NoChange
Complete
```

Operation-specific payloads control meaning. A complete validation may be `Invalid`; a successful build can remain blocked for reproducibility/signature/license/platform gates; `Published` can coexist with unsupported target/install state; `Updated` requires the exact requested install/migration/self-check closure.

## Independent gates

Report separately:

```text
source and materialization
build execution
unsigned reproducibility
artifact self-description
unit/integration/contract/mutation/security/client tests
SBOM/provenance/license/notices
portable/platform signatures
bundle integrity
support/compatibility
release-candidate readiness
channel/distribution publication/read-back
update-manifest verification
installation/staging/migration/current CAS/self-check
LKR/rollback
revocation/retirement/incident
```

No aggregate status hides a failed mandatory gate.

## Build statuses

- `BuildSubmitted`: exact durable executor request exists; no claim artifacts were produced.
- `BuildComplete`: the exact execution produced retained unsigned artifacts and closed; no reproducibility claim.
- `Reproducible`: exact independent executions satisfy the frozen comparison profile.
- `ArtifactValidated`: self-description/member/target/schema closure passed for the exact artifact set.

## Evidence/signature/bundle

- `EvidenceComplete`: required SBOM/provenance/license/notices/test/security evidence closure completed for the operation profile.
- `Signed`: all required target signatures verify under exact current trust policy; signature is not correctness proof.
- `BundleReady`: exact immutable bundle/member/archive/support metadata validates; not channel-published or installed.
- `CandidateReady`: all target channel mandatory gates passed and authorization state is compatible; not published.

## Publication/update/install

- `Published`: exact provider objects/manifests/read-back and channel CAS completed. It does not prove installation or runtime.
- `UpdateAvailable`: exact signed manifest offers one compatible target; no download/install occurred.
- `InstallPlanned`: exact staged/backup/migration/rollback plan validates; no current change.
- `Installed`/`Updated`: exact current installation CAS, required migration and post-install self-check completed.
- `RolledBack`: exact qualified target became current and required data/self-check closure completed; history remains.

Partial install/migration/closure cannot be rendered installed/updated/rolled back.

## Revoked and retired

`Revoked` includes exact scope, reason/evidence/authorization, effective ordering, required client/channel actions and unresolved coverage. `Retired` includes support/channel scope and retained verification/rollback availability. Neither state deletes history or implies the other.

## NoChange

Requires exact owner proof that the same operation/request already produced the same retained validated effect/state. Equal source tree, version string, artifact digest, release name, provider asset or installed directory alone is insufficient without operation and policy closure.

## OutcomeUnknown

The envelope states:

```text
which build/sign/upload/channel/install/migration/rollback/revoke effect may have occurred
exact operation/request/owner/target/current identifiers
known bytes/state/receipts
unsafe_to_retry = true
required reconciliation operation
retention/cleanup restrictions
```

Frontends cannot fold it into failed/cancelled/success.

## Required nonclaims

As applicable:

```text
build-success-is-not-release-readiness
reproducible-does-not-mean-secure-or-correct
checksum-is-not-authentication
signature-is-not-semantic-or-runtime-proof
bundle-ready-is-not-published-or-installed
channel-published-is-not-installation-success
update-available-is-not-downloaded-or-authorized-to-install
installed-self-check-is-not-all-project-or-runtime-validation
support-is-limited-to-exact-matrix
WoW-profile-support-requires-exact-evidence
rollback-does-not-erase-history-or-incident
revocation-does-not-name-a-safe-replacement-unless-explicit
retirement-is-not-security-revocation
unsafe-to-retry-while-outcome-unknown
```

## Privacy/redaction

Default results/errors expose stable IDs/digests/status/reason codes and public verification data according to policy. They redact private keys, signing/distribution credentials, session capabilities, private endpoints, arbitrary command/environment data, local private paths, user source/data and raw owner handles.

## Canonicalization

Canonical release semantic records exclude local build/install paths, process IDs, CI job IDs, provider timing, upload/download timing, wall duration, retry count, cache state, terminal, user name and transport delivery details unless an explicit operational evidence record owns them.

One-shot CLI adds its defined LF. Publisher/client transports do not alter canonical payloads.