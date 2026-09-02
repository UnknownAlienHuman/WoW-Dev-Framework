# AGENTS.md — `wow-service` E7-B

## Scope

Implement transport-independent release lifecycle orchestration only. Build, signing, package, installer, channel, and update owners remain behind narrow exact ports. Do not execute arbitrary commands, access credentials, or move release semantics into `apps/wow`, `wow-release`, CI, or a distribution provider.

## Required reading

1. Repository, crate, service, E7-A host, launch-gate, and implementation-handoff instructions.
2. This complete E7-B package.
3. `apps/wow/e7b/` and `tools/wow-release/` contracts.
4. Exact target-platform, dependency, license, signing, installer, channel, and support profiles selected by the release plan.
5. Current external WoW KB routes and exact pinned Blizzard/runtime evidence for any supported WoW profile claim.

## Source and build discipline

- Select one exact source tree/commit and verify clean content identity; branch/tag/display name is not identity.
- Bind `Cargo.lock`, exact Rust toolchain/components/targets, features, dependency/vendor sources, build-script policy, environment allowlist, path/timestamp-remapping, and builder profile.
- Build only through a reviewed `ReleaseBuildExecutorPort`; no shell text or caller-supplied command.
- Network access during build is denied unless the exact materialization profile owns and records it; normal build consumes already verified inputs.
- Unsigned artifact identity freezes before nondeterministic platform signing/notarization.
- Reproducibility claims require independent execution comparison for the exact target/profile.

## Evidence discipline

- Generate SBOM, provenance, license/notices, compatibility manifest, service registry, schemas, checksums, signatures, and validation reports as distinct immutable artifacts.
- Missing dependency identity/license/provenance/toolchain/input is blocked or `NotEvaluated`, never assumed safe.
- A signature proves exact bytes/key/profile binding only.
- A successful build/upload/install does not prove runtime, compatibility, support, or semantic correctness.

## Bundle and channel discipline

- A release bundle references exact artifacts and digests; no floating inputs.
- Reference/core/provider/data packs are separate artifacts unless one explicit offline-bundle profile names exact versions.
- Channel prepare and publish are separate guarded effects.
- Channel pointer updates use exact expected-current CAS.
- Never infer stable/LKG from newest, highest version, age, downloads, or no complaints.
- Upload response loss becomes `OutcomeUnknown`; never republish blindly.

## Installation/update discipline

- Check, plan, download/materialize, verify, stage, backup, migrate, activate, validate, and cleanup are separate recorded states.
- Baseline updates are explicit/opt-in, not hidden startup network activity.
- Never overwrite the running/current installation before exact staging verification and rollback retention.
- Store/schema/config migration uses exact forward/rollback compatibility and backup plans.
- Rollback chooses one exact retained compatible installation, never `previous` by position.
- Failed install/update preserves evidence and the last known runnable installation when the profile requires it.

## Authorization and credentials

- Build, sign, channel publish, revoke, retire, install/update policy, and emergency rollback use separate exact authorization scopes.
- GitHub/OS/CI/CLI/tag/repository identity is not authorization.
- Requests/results contain stable nonsecret references and receipts only; no tokens, private keys, cookies, secrets, endpoints, commands, or environment blocks.

## Lifecycle

- Register `OperationId + CanonicalRequestDigest` before every effect.
- Persist exact owner receipts after each boundary.
- Same ID/different digest fails; response loss blocks blind repeat.
- No public success before retention, audit, read-back/verification, and reverse-order close.
- No detached retry, upload, cleanup, migration, update, or telemetry after return.

## Completion report

Report source/build/toolchain/target/feature/dependency IDs, unsigned and signed artifacts, reproducibility comparison, SBOM/provenance/license, bundle/channel/update/install/rollback/revocation records, authorizations/effect receipts, support profile, tests/benchmarks/platforms, and every blocked/`NotEvaluated` gate.