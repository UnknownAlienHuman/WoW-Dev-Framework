# E7-B support matrix, release-candidate gates, compatibility, and nonclaims

**Status:** normative.

## Support is exact

A release is supported only for an exact combination recorded in `ReleaseSupportMatrix`:

```text
release/bundle/target profile
OS family/edition/version range and architecture
required runtime libraries and platform security features
filesystem/path/permission profile
local IPC named-pipe/Unix-socket behavior
CLI/LSP/MCP protocol and client capability profiles
store/object/schema/migration versions
Reference Pack/core-pack/provider-adapter compatibility
WoW flavor/build/Interface/reference profile classes
feature/exposure/configuration profiles
resource envelope and addon/project size classes
installation/update/rollback mechanism
support and retirement policy
```

Unlisted or untested combinations are unsupported or `NotEvaluated`, not implicitly compatible.

## First target

The initial support candidate is:

```text
x86_64-pc-windows-msvc
Windows x86-64 exact release profile
named-pipe local daemon
one-shot CLI, LSP stdio and MCP stdio
```

The exact minimum/maximum supported Windows editions/builds, runtime redistributables, code-signing behavior, path/ACL semantics and terminal/editor clients remain freeze-gate values until tested. Documentation does not advertise them prematurely.

Linux/macOS/ARM targets require independent build, package, endpoint, filesystem, process replacement, signature/notarization, client, update and rollback evidence.

## Compatibility dimensions

Keep separate:

```text
binary ABI/runtime compatibility
on-disk store/schema compatibility
public service schema compatibility
operation-registry compatibility
CLI output/exit compatibility
local-daemon protocol compatibility
LSP/MCP protocol and mapping compatibility
project/reference/core-pack artifact compatibility
configuration schema compatibility
installation/update manifest compatibility
WoW profile/client data compatibility
```

Compatibility on one dimension does not imply another.

## Version relationships

The release profile defines exact semantics for:

```text
release version ordering
schema compatibility ranges
upgrade edges
rollback edges
minimum supported current version
blocked/revoked versions
channel eligibility
```

A semantic-version-looking string alone does not prove compatibility. Every allowed edge is backed by an exact migration/validation record or a frozen compatibility rule with tests.

## Release candidate input closure

`release_candidate_validate` requires exact:

```text
ReleaseSourceSnapshot and ReleasePlan
independent build executions and reproducibility comparison
unsigned/signed artifact sets
SBOM/provenance/license/notices/checksum evidence
transport compatibility manifest and operation registry
unit/integration/contract/fixture/mutation/security/platform/client reports
performance/resource benchmark reports
bundle validation
support matrix
install/update/migration/rollback plans and validation
release notes/changelog/source provenance
channel target and authorization state
known blockers/incidents/advisories
retention/audit/reconciliation closure
```

No field is inferred from Git tags, branch names, CI badges, checksums alone, upload success, issue count, download count, age, or prior release behavior.

## Gate matrix

Mandatory gate classes:

```text
SourceClosure
DependencyAndToolchainClosure
BuildSucceeded
UnsignedReproducibility
ArtifactSelfDescription
ContractAndFixtureTests
SecurityAndSupplyChain
SBOMProvenanceLicenseNotices
PortableAndPlatformSignatures
BundleIntegrity
PlatformAndClientConformance
StoreAndMigrationCompatibility
InstallUpdateRollbackValidation
PerformanceAndResourceLimits
WoWProfileEvidence where claimed
DocumentationAndVerificationInstructions
AuthorizationRetentionAuditRecovery
```

A channel profile declares which are mandatory and exact thresholds. Required `Partial`, `Conflict`, `Truncated`, `NotEvaluated`, skipped, stale, expired or failed state blocks readiness.

## Test reports

Each report binds:

```text
suite/implementation/profile/version
exact source/build/artifact/target inputs
test corpus and fixture bundle digests
commands as reviewed operation IDs
pass/fail/skipped/NotEvaluated counts
failures/conflicts/flakes/retries
resource/timing results
logs/artifacts under redaction
canonical digest
```

A retry that eventually passes does not erase previous failure/flakiness; channel policy decides eligibility. A required suite skipped because the platform/tool/client is unavailable is not pass.

## Real addon evaluation

Developer-preview/beta/stable profiles may require admitted real addon repository evaluations. Each uses exact pinned revisions, source/license/privacy decisions, exact Reference/WoW profiles and bounded nonexecuting analysis.

Evaluate representative small/medium/large and adversarial projects for:

```text
indexing/diagnostic/context/search correctness
clean-negative honesty and NotEvaluated behavior
memory/CPU/latency/disk envelopes
cancellation/recovery/incremental overlays
LSP/MCP/daemon client behavior
no source execution or data leakage
```

Community addons remain implementation evidence, not platform API authority.

## WoW profile support

A release may be binary-supported on Windows while specific current WoW profiles remain unavailable. WoW profile support additionally requires:

```text
exact ReferenceProfile/ReferenceGeneration
pinned Blizzard source/build inputs
current external KB routing review
required analyzer/recognizer/rule/core-pack compatibility
runtime probe evidence for runtime-dependent claims
known upstream/client defects and restrictions
```

Patch-sensitive state is not baked into release marketing from stale documentation.

## Channel readiness

```text
developer-preview
    selected core operations work on the first target;
    limitations and unsupported capabilities are explicit.

beta
    broader real-project/client/platform evidence;
    recovery/update/rollback and critical security gates complete;
    known issues bounded.

stable
    all stable-profile mandatory gates pass;
    support/incident/retirement policy active;
    no unresolved release-blocking defect.
```

No automatic time-based promotion.

## Blocker classes

```text
SourceOrDependencyUnclosed
BuildOrReproducibilityFailed
ArtifactOrRegistryMismatch
RequiredTestsFailedSkippedOrNotEvaluated
SecurityOrCredentialBoundaryFailed
LicensePrivacyNoticeBlocked
SignatureOrTrustInvalid
BundleOrArchiveInvalid
TargetPlatformUnsupported
StoreMigrationOrRollbackInvalid
InstallUpdateSelfCheckFailed
PerformanceBudgetExceeded
WoWProfileClaimUnsupported
DocumentationOrVerificationIncomplete
AuthorizationRetentionAuditOrRecoveryIncomplete
KnownCriticalIncidentOrRevocation
```

Aggregate success rates cannot override a blocker.

## Release notes

Release notes are exact source-derived artifacts. They distinguish:

```text
implemented operations/capabilities
behavior/schema/protocol changes
migration/update/rollback requirements
security fixes and known limitations
deprecations/revocations/retirement dates
WoW profile support changes
NotEvaluated or deferred features
```

They never claim tests, platforms, compatibility or runtime behavior absent from evidence.

## Candidate immutability

Any change to source, lockfile, toolchain, target, features, dependency input, build profile, artifact, registry/schema, evidence, signature, bundle, support matrix, install/update plan, release notes or channel target creates a new candidate identity. Existing candidates remain retained.

## Nonclaims

A validated release candidate does not mean channel publication, download availability, installation success on every machine, vulnerability absence, runtime correctness, future WoW compatibility, automatic update eligibility, or long-term support beyond the exact matrix/policy.