# E7-B decisions

**Status:** normative.

## E7B-001 — Release is a distinct governed lifecycle

A compiled binary is not a release. Source, build, evidence, signing, bundle, channel, install/update, support, revocation, and retirement are separate exact states.

## E7B-002 — One public executable

The baseline public product ships one `wow` executable containing the E7-A modes. Internal build/publisher tooling is not part of the default public bundle.

## E7B-003 — Windows x86-64 is the first support target

The first supported target profile is exact Windows x86-64 MSVC. Other targets remain unadvertised until their complete platform suites pass.

## E7B-004 — Unsigned semantic artifacts define reproducible build identity

Platform code signing/notarization occurs after unsigned artifact digests freeze. Nondeterministic signature containers cannot redefine source-build reproducibility.

## E7B-005 — Two independent builds are required for reproducibility claims

A single successful rebuild is insufficient. Exact independent builder executions must compare according to the target artifact profile.

## E7B-006 — Build inputs are closed

Source tree, lockfile, toolchain, targets, features, dependencies/vendor objects, build scripts, environment allowlist, path/timestamp policy, schemas, registry, and profile are exact inputs.

## E7B-007 — Build executor is narrow

The release service sends a typed frozen build plan to an allow-listed executor. There is no arbitrary shell/command/environment callback.

## E7B-008 — Offline verified inputs are preferred

Network materialization is a separate authorized effect. The normal build consumes already verified source/dependency/toolchain objects and cannot silently download replacements.

## E7B-009 — Supply-chain evidence is first-class

SBOM, provenance, dependency/source attestations, license/notices, checksums, compatibility manifest, schemas, registry, signatures, and validation reports are immutable release artifacts.

## E7B-010 — Signature scopes remain separate

Executable/bundle/update-manifest signing is distinct from E5 core-pack signing and from channel publication authorization. One key/grant never implies another scope.

## E7B-011 — Private release material stays outside the repository

Signing, code-signing, distribution, package-manager, CI and endpoint credentials never enter public config, requests, fixtures, logs, bundles, or canonical results.

## E7B-012 — Data packs have independent lifecycles

Reference Packs, core recognizer packs and provider adapters are separately identified/signed/compatible. The binary release does not silently bundle or update them.

## E7B-013 — Offline bundles are explicit

An offline/starter bundle may include exact compatible data packs only under a separate bundle profile that records every artifact/digest/license/update relationship.

## E7B-014 — Channel pointer is not semantic latest

A channel record points to one exact release manifest by guarded CAS. Clients resolve it once. Newest/highest version/tag is never selected by inference.

## E7B-015 — Stable is an evaluated state

Age, upload success, download count, absence of complaints or a tag named stable cannot satisfy stable-channel gates.

## E7B-016 — Updates are explicit by default

Baseline performs no startup update check. User invocation or a precise opt-in policy is required for network/channel access.

## E7B-017 — Update discovery, download, verification and installation are separate

A newer channel manifest does not authorize download or install. Each stage has exact policy, authorization, effect and failure state.

## E7B-018 — Install is staged and verified

The running/current installation is not replaced until exact bundle verification, platform compatibility, disk/path/security checks, rollback retention and staging validation pass.

## E7B-019 — Data migration is independent from executable replacement

Store/config/schema migration has exact forward/rollback compatibility, backup and recovery. A binary swap cannot imply a successful data migration.

## E7B-020 — Last-known-runnable installation is explicit

Rollback target qualification is an exact retained installation/bundle validation record, never simply the previous directory or version.

## E7B-021 — Rollback is a new immutable effect

Rollback creates new installation/current/audit records and does not rewrite the failed release or erase incident evidence.

## E7B-022 — Revocation and retirement are distinct

Revocation addresses unsafe/ineligible artifacts or manifests; retirement ends support/channel eligibility according to policy. Neither deletes historical evidence.

## E7B-023 — GitHub Releases is an adapter, not architecture

Channel/publisher contracts are provider-neutral. GitHub tag, release, repository ownership or API success is not release proof or authorization.

## E7B-024 — Update manifests are signed exact objects

Clients accept only exact supported signed manifests with target, channel, bundle, compatibility, revocation and rollout policy closure.

## E7B-025 — No self-update while active without an exact platform strategy

The Windows baseline uses a staged replacement/helper strategy defined by the installer profile; the running process never overwrites itself ad hoc.

## E7B-026 — Support claims are exact

OS/architecture/protocol/store/schema/WoW-profile support exists only where the release support matrix and required tests pass. Portability intent is not support.

## E7B-027 — Patch-sensitive support is revalidated

WoW profile/client support requires current KB routing plus exact pinned Blizzard/reference/runtime evidence where the capability needs it.

## E7B-028 — Release candidate is an immutable evidence closure

A release candidate binds exact artifacts, all gates and blockers. It cannot be repaired in place after signing; changed input creates a new candidate.

## E7B-029 — CI is an executor after commands exist

CI may run exact real release commands after implementation/freeze. It cannot define semantics or turn skipped/missing tests into pass.

## E7B-030 — Channel publication is not installation success

A published bundle can still be unsupported or fail installation on a target. Channel, download, install, self-check and runtime evidence remain independent.

## E7B-031 — Response loss is never no effect

Build submission, signing, upload, channel CAS, install, update, rollback, revoke and retire require exact reconciliation.

## E7B-032 — No hidden network behavior

Update checks, downloads, telemetry, crash upload and remote configuration are disabled unless an explicit reviewed policy enables each independently.

## E7B-033 — Public verification is possible without secrets

Release consumers receive all public checksums, signatures, trust-root references, SBOM, provenance, license/notices, compatibility and verification instructions required to verify the bundle.

## E7B-034 — Documentation completion hands off to implementation

After E7-B and global consistency/freeze planning, the next project frontier is E0-A Rust implementation, not another speculative architecture package.