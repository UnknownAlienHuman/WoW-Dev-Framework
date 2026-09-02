# E7-B implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite and release-profile freeze

Before E7-B Rust:

- implement/freeze the selected E0–E7-A product operations and Windows x64 host behavior;
- freeze workspace/source/lockfile/toolchain/dependency/vendor/materialization profiles;
- freeze build executor, sandbox, target, features, deterministic path/time/archive and independent-builder profiles;
- freeze SBOM/provenance/license/notices/checksum/signing/trust-root/platform-signing profiles;
- freeze bundle layout, support matrix, release-candidate/channel/update-manifest profiles;
- freeze Windows installation/helper/store/config migration/update/rollback/uninstall profiles;
- freeze distribution adapter, authorization, response-loss, retention, audit, incident and support profiles;
- freeze canonical fixtures, real project/client/platform evaluations, resource benchmarks and all checksums.

## Phase 1 — release primitives

Implement closed release request/status/error/result types and identities:

```text
ReleaseSourceSnapshot
ReleasePlan
ReleaseBuildExecution
UnsignedReleaseArtifactSet
ReproducibilityComparison
ReleaseEvidenceSet
ReleaseSignatureSet
ReleaseBundle
ReleaseSupportMatrix
ReleaseCandidate
ChannelReleaseRecord
SignedUpdateManifest
InstallationRecord/Plan/Receipt
UpdatePlan/Receipt/RollbackReceipt
Revocation/Retirement/Incident records
```

## Phase 2 — source and materialization validation

Implement exact repository tree/manifest validation, forbidden-file/security scanning, lockfile/workspace checks and retained source/dependency/toolchain materialization catalogs.

Do not run build yet. Tests: `S7B-SRC-*`, `S7B-DEP-*`, `S7B-TC-*`, `S7B-PLAN-*`.

## Phase 3 — narrow build executor

Implement `ReleaseBuildExecutorPort` for the exact first Windows x64 MSVC profile. Executor consumes a typed plan, verified read-only inputs, isolated output/temp roots and explicit environment allowlist. No generic process API crosses service.

Implement durable submission/cancellation/reconciliation and typed artifact receipts.

## Phase 4 — self-description and artifact validation

Generate and embed exact source/build/target/registry/compatibility/schema identities. Validate PE architecture/import/runtime and required bundle members. Add secret/private-path/unexpected-output scans.

## Phase 5 — independent reproducible builds

Run at least two independent clean/offline builds under the same exact plan. Implement byte/member comparison, declared wrapper variance and mismatch reports. Fix reproducibility root causes before proceeding.

## Phase 6 — SBOM, provenance, license and tests

Generate/validate exact SBOM/provenance/license/notices/checksum artifacts. Execute all release-plan-required unit/integration/contract/fixture/mutation/security/client/platform/benchmark suites and retain pass/fail/skipped/`NotEvaluated` reports.

## Phase 7 — portable signing and trust

Implement signing authorization, narrow portable signing and verification adapters with external protected key material. Freeze trust-root bootstrap/rotation/revocation records and offline public verification tooling.

## Phase 8 — platform signing

Implement the selected Windows code-signing profile only after unsigned digest freeze. Validate exact unsigned-to-signed mapping and Windows verification behavior. If credentials/service are unavailable, the gate remains `NotEvaluated`/blocked.

## Phase 9 — deterministic bundle

Build the exact portable archive/layout, manifests, docs/config examples, signatures and evidence. Validate archive safety, member closure, deterministic reconstruction, SBOM/provenance/license and target/support metadata.

Optional native installer and offline data bundle are separate profiles/outputs.

## Phase 10 — support matrix and release candidate

Populate exact Windows editions/builds/runtime/IPC/CLI/LSP/MCP/store/schema/data-pack/WoW profile/resource evidence. Build immutable candidate and apply developer-preview/beta/stable gate matrices.

## Phase 11 — distribution adapter and channel

Implement one reviewed provider adapter, initially GitHub Releases if selected by the owner, behind `ReleaseDistributionPublisherPort` and a narrow read port. Implement immutable upload, manifest publication, public read-back, expected-current channel CAS, response-loss reconciliation and revocation/retirement effects.

No raw token/API surface in service/tool.

## Phase 12 — public update manifest and check

Implement signed target/channel update manifest build/validation and explicit `wow update check`. No startup check, download, telemetry or install.

## Phase 13 — installation inspection and new install

Implement Windows exact installation/data-root records, safe bundle staging/extraction, permissions/path/disk/lock checks, public config bootstrap, current pointer/CAS, self-description/self-check and uninstall behavior.

## Phase 14 — Windows update helper

Implement the minimal separately signed/provenanced replacement helper and opaque plan protocol. Prove it cannot execute arbitrary commands/paths/URLs and survives process handoff/crash/restart scenarios.

## Phase 15 — store/config migration and backup

Implement registered `wow-store`/config migrations, verified backup/restore, crash recovery, forward/rollback compatibility and one-way migration blockers. Do not combine raw SQL/scripts into the app/helper.

## Phase 16 — explicit update apply

Implement check -> materialize -> verify -> stage -> backup -> close -> helper swap -> migrate -> current CAS -> self-check -> LKR designation. Retain exact old installation for rollback according to policy.

## Phase 17 — rollback and recovery

Implement explicit qualified rollback target selection, executable/data restore, expected-current CAS, post-rollback validation and history preservation. Inject failures/response loss at every boundary.

## Phase 18 — revocation, retirement and incidents

Implement signed revocation/retirement records, channel/update-manifest effects, installed status behavior, exact incident/containment/advisory records and support-bundle preview/redaction.

## Phase 19 — internal release tool

Activate `tools/wow-release` only after service request/result bytes freeze. The tool invokes exactly one service operation per command and contains no build/sign/publish/install semantics.

## Phase 20 — user-facing update CLI

Activate `apps/wow/e7b` update/version/install validation commands after service and installer/helper contracts freeze. Preserve explicit network/update policy and one-call transport.

## Phase 21 — cross-platform and additional targets

Only after Windows x64 support is complete, introduce another target through a new exact target/support/install/sign/update profile and its full suite. Do not mark portability intent supported.

## Phase 22 — CI/release automation

Add CI only after real commands exist:

```text
contract/manifest/fixture/checksum validation
workspace tests and lints
Windows target build and client/platform tests
independent reproducibility jobs
SBOM/provenance/bundle validation
release-candidate validation
manual authorized channel publication
```

CI invokes `wow-release` operations and never embeds release semantics or turns skipped jobs into pass. Public publication is manually/explicitly authorized.

## Phase 23 — release candidate rehearsal

Run complete developer-preview rehearsal against an isolated test channel and clean Windows machine/VM:

```text
build twice
verify evidence/signatures/bundle
publish/read back
new install
CLI/daemon/LSP/MCP smoke and real-addon tasks
update from prior candidate
crash/response-loss recovery
rollback
revoke/retire test records
uninstall/data retention
```

## Phase 24 — freeze and publish

Populate every implementation/port/profile/vector/platform/benchmark/member/bundle checksum. Validate one immutable candidate, authorize channel publication, publish/read back and record exact release/support state.

## Deferred beyond first public v1

- unvalidated Linux/macOS/ARM support;
- remote daemon/server transports;
- automatic updates without explicit opt-in policy;
- package manager feeds not selected by a reviewed adapter;
- server-side telemetry/crash upload/remote configuration;
- automatic source edits or agent execution.

No placeholder release artifact, fake reproducibility, unsigned manifest, dummy installer, fake signing adapter, generic executor, or decorative workflow is permitted.