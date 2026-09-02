# E7-B compatibility policy, support lifecycle, retirement, and incident response

**Status:** normative.

## Support ownership

Every public channel/release has an exact support owner/profile that defines:

```text
supported targets/platforms/clients/protocols
supported store/config/schema and data-pack ranges
supported WoW profiles and evidence freshness policy
security/advisory intake and triage
response and publication authority
support window and retirement rules
rollback/revocation/update obligations
public communication artifacts
retention/audit/privacy policy
```

A repository owner, committer, CI actor, distribution account or signing key holder is not automatically the support authority.

## Compatibility manifests

Compatibility is machine-readable and release-bound. The release ships exact manifests for:

```text
binary/target/platform requirements
transport protocol and client capability profiles
service operation registry and schema sets
store/config/data migration ranges
Reference/core/provider artifact ranges
installation/update/rollback edges
WoW profile/client evidence ranges
known exclusions and NotEvaluated capabilities
```

Clients and installers validate exact manifests; they do not infer compatibility from version string, file name, tag or successful process startup.

## Compatibility changes

A change to a public request/result/error schema, operation semantics, store/config format, protocol mapping, artifact layout, trust root, install/update behavior or support matrix requires:

```text
new exact version/identity
compatibility or migration analysis
old/new fixture vectors
forward and rollback behavior
client/tool impact
release notes and deprecation policy
```

No silent semantic change under the same version/digest.

## Deprecation

Deprecation retains availability for an exact window/profile while directing consumers to a reviewed replacement when one exists. It records:

```text
deprecated operation/schema/protocol/artifact/release
introduced/deprecation/retirement release IDs
replacement identity or explicit none
migration/compatibility guidance
warning behavior
support and security policy
```

Deprecated does not mean insecure, removed or replaced. A replacement is not implied by name similarity.

## Retirement

Retirement ends ordinary support/new installation/channel eligibility according to policy. A `ReleaseRetirementRecord` binds exact release/bundle/channel/target scope, reason, effective ordering/time evidence, authorization, user action, retained verification/rollback availability and public notice.

Retired artifacts may remain downloadable for reproducibility/rollback when policy allows. Retirement does not erase history or create a security revocation.

## End-of-support behavior

After retirement:

- update checks return explicit retired/unsupported state;
- stable/beta channel pointers no longer select the retired release for new installs;
- existing installations retain exact identity and can still verify artifacts;
- security-critical policy may require a separate revocation/update;
- telemetry/network behavior does not change automatically;
- data export/uninstall/rollback guidance remains available according to policy.

## Security revocation

A revocation is used when an artifact/release/key/manifest is unsafe or ineligible. It defines exact scope and client/channel/install actions. Revocation can coexist with support retirement but is not inferred from it.

A revoked current installation does not silently update or delete itself. The next explicit check/status operation reports exact required action; an opt-in managed policy may perform only separately authorized update/rollback operations.

## Incident record

```text
ReleaseIncidentRecord
    incident ID/class/severity
    affected exact release/bundle/artifact/key/channel/target/profile ranges
    evidence and confidence
    discovery/triage/decision ordering evidence
    containment actions
    revocation/channel/update/rollback records
    user data/privacy impact classification
    public advisory and remediation refs
    unresolved questions/NotEvaluated state
    authorization/audit/retention
    canonical digest
```

Incident severity, scope and certainty remain separate. A report is not confirmed evidence until validated under the incident profile.

## Incident classes

```text
artifact corruption or substitution
signing key/trust-root compromise
malicious or vulnerable dependency/build input
release process credential exposure
installer/update/rollback data-loss defect
store/config migration corruption
remote/local transport security defect
source/privacy/license disclosure
incorrect compatibility or WoW profile claim
critical semantic/runtime defect
provider/channel outage or manifest inconsistency
```

## Triage

Triage freezes exact evidence and determines:

```text
affected identities and uncertainty
whether distribution/current installs remain safe
whether channel freeze, revocation, update or rollback is required
whether data backups/restoration are required
whether credentials/trust roots rotate
whether public disclosure is required and permitted
what tests/evidence are needed to close scope
```

Unknown scope is not treated as unaffected. Conversely, an unverified report does not automatically revoke every release; containment can be conservative and exact.

## Containment

Possible exact actions:

```text
pause channel publication/update visibility
publish signed revocation/advisory
rotate signing/distribution credentials through owner systems
publish exact fixed release candidate
require explicit update or rollback
block a target/profile/operation through a new compatibility manifest
provide data backup/restore instructions
retain forensic build/distribution/install records
```

Containment effects use exact authorization, durable operation identity, response-loss reconciliation, public read-back and audit.

## Remediation release

A remediation release is built from a new exact source snapshot through the complete E7-B pipeline. It cannot reuse old reproducibility/test/evidence/signatures as if bytes were unchanged. The release notes bind the incident and exact corrected scope without overclaiming root cause closure.

## Public advisory

The advisory states:

```text
affected exact versions/targets/profiles
known impact and uncertainty
how to verify installed identity
required update/rollback/uninstall/data action
fixed or replacement release identity
signature/checksum/trust instructions
support contact/process where defined
timeline evidence under the disclosure policy
```

It excludes secrets, private source/user data, exploit-enabling operational details not appropriate for the policy and unsupported claims.

## Recovery and closure

Incident closure requires evidence that required channel/revocation/update/rollback/data/trust actions completed for their claimed scope. It does not prove every installation was reached unless exact coverage exists.

Post-incident review updates contracts/tests/profiles only through normal review. It preserves the original incident/effect records and does not rewrite failed release history.

## Support requests and diagnostics

User support artifacts should contain bounded product/build/manifest/store/profile/status/error IDs rather than private projects/source/secrets. A diagnostic bundle requires explicit generation, preview and consent; it is not silently uploaded. Its contents, redaction, encryption, retention and deletion are exact profile inputs.

## Privacy incidents

Potential private source, project metadata, credential, endpoint or user-data disclosure triggers the privacy/incident policy independently of semantic correctness. Logs/crash/support bundles are included in scope.

## Service availability

Distribution-provider outage, update-manifest unavailability or support service outage does not stop the installed local product from operating. Exact local/offline capabilities remain available; update status is unavailable/`NotEvaluated`.

## Nonclaims

A support window does not guarantee compatibility with future WoW/client/platform changes. Incident closure does not prove vulnerability absence. Retirement does not imply insecurity, and revocation does not identify a safe replacement unless one exact validated release is named.