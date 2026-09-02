# E7-B release security, supply-chain, credentials, and distribution trust

**Status:** normative.

## Threat model

Treat as untrusted until exact validation:

```text
source repository/tree and generated files
dependency registry/index/package/vendor objects
Rust/native toolchains, SDKs and build tools
build scripts, proc macros and native code
build executor output/logs
SBOM/provenance/license metadata
signatures/certificates/trust data until verified
archives/installers/update manifests/provider receipts
channel/download URLs and redirects
existing installation/store/config/data
client/user paths and release notes
```

Threats include source/dependency substitution, compromised build/signing/distribution systems, malicious archive paths, credential leakage, rollback/downgrade attacks, manifest mix-and-match, partial installs, data loss, unsafe migration, stale/revoked update, and response-loss duplication.

## Credential boundaries

Separate protected adapters own:

```text
source/dependency registry access where needed
build infrastructure authorization
portable signing keys
platform code-signing/notarization credentials
distribution provider credentials
update/channel publication authorization
incident/revocation authority
```

Service, `wow`, `wow-release`, repository, public config, fixtures, logs, crash/support bundles and release artifacts contain only stable nonsecret references, public certificates/trust roots where appropriate, and bounded receipts.

Forbidden public inputs/outputs:

```text
private keys/seeds/passphrases
KMS/HSM/vault tokens, PINs or recovery material
GitHub/package registry/cloud credentials
passwords/cookies/session tokens
private endpoints and arbitrary URLs
signing-agent sockets/process handles
unrestricted environment blocks
```

## No generic execution

There is no release operation that accepts arbitrary:

```text
shell/PowerShell/cmd command
script/plugin/callback/dynamic library
Cargo/rustc/linker arguments outside an exact reviewed plan
SQL/database query or migration script
HTTP request/body/header
GitHub API method/JSON
installer command line
model/tool prompt
```

Build/sign/publish/install adapters implement closed typed plans and allow-listed capabilities. Runtime negotiation may narrow but never widen them.

## Source integrity

Validate exact repository/tree/manifests and reject:

- untracked/dirty local substitution;
- submodule/vendor pointer mismatch;
- symlink/submodule/path escape;
- case/Unicode/path collision;
- forbidden credentials/build output/private source;
- generated registry/schema drift;
- source file mutation after validation;
- branch/tag-only identity;
- shallow/incomplete tree presented complete.

Source acquisition and release build use retained exact objects. A later force-pushed branch cannot alter the snapshot.

## Dependency integrity

`Cargo.lock` and materialization manifest bind exact packages/checksums/sources. The build cannot update the lockfile or select a different registry/git revision.

For registry, git, path, vendored, native and toolchain dependencies, record source/provenance/license and executable build behavior. Ambiguous duplicate package identity, checksum mismatch, yanked/revoked/advisory state required by policy, or unavailable source/license closure blocks release.

## Build isolation

The build profile enforces:

```text
verified read-only source/dependency/toolchain inputs
isolated writable target/output/temp directories
explicit environment allowlist
no ambient repository/Git/editor/user secrets
no network in normal compile/package phase
bounded CPU/memory/disk/time/process count
no privileged host/device access
captured typed executor receipts
output allowlist and secret scan
```

The exact sandbox implementation is platform/profile-specific. An unavailable sandbox requirement is `NotEvaluated`/blocked, not silently skipped.

## Build-script/proc-macro risk

These execute at build time and are supply-chain code. New/changed executable build dependencies require source/checksum/provenance/license review, sandbox compatibility and tests. Their output is bounded and validated; they cannot write arbitrary release/source/data-root paths.

## Artifact scanning

Profiles may require:

```text
secret/credential/private-path scan
unexpected binary import/rpath/dependency scan
archive member/path/link/device scan
known malware/defense scan
vulnerability/advisory scan over SBOM
license/notice policy validation
reproducibility/mismatch analysis
```

Scanner databases/tools/results are exact time/snapshot/profile evidence and cannot prove permanent absence. False positives/unknowns/conflicts remain explicit.

## Signature and trust bootstrap

Portable verification depends on trust roots distributed/installed through an explicit bootstrap policy. A public key shipped only inside the same untrusted bundle cannot authenticate that bundle without an independently trusted binding.

Trust-root rotation/revocation has exact signed transition records, overlap/emergency policy, client compatibility and rollback behavior. It never occurs through unsigned remote configuration.

## Distribution security

The publisher and reader enforce allow-listed provider/channel/repository/object scopes. Upload/read uses exact object size/digest and read-back. Client downloads verify signed manifests and content independently of TLS/provider status.

Reject:

```text
unsigned/unknown/expired/revoked manifest
redirect outside provider policy
HTTP downgrade or mixed channel
same-name asset with wrong digest
partial/range-confused content
zip/tar bomb or path traversal
cross-target/cross-channel/cross-product substitution
rollback to blocked/revoked release
manifest replay outside freshness policy
```

## Installer/update security

Installation owner validates destination/staging/backup paths, permissions, reparse/symlink/device/UNC/ADS behavior, disk limits, existing ownership, process locks and exact bundle members.

The Windows replacement helper accepts only an opaque exact signed/validated local plan reference and fixed operation. It cannot execute arbitrary path/command/URL. Helper/binary mutual identity and expected process/current state are verified.

No bundle-provided scripts, post-install shell commands, unsigned plugins or dynamic code are executed.

## Downgrade and rollback protection

Signed update manifests define allowed edges, minimum/blocked versions, revocations and rollback targets. An older but validly signed release is not automatically eligible.

Emergency rollback still requires exact retained qualified target, signature/trust validity, compatibility with current data/store/config, authorization, CAS and audit.

## Data protection

Before migration/update, validate backup/snapshot creation and restoration capability according to profile. Private project/store/config data never enters release artifacts, logs or remote systems by default.

Failed update/rollback preserves data and exact recovery state. Cleanup cannot delete backups or LKR evidence prematurely.

## Release metadata injection

Version strings, release notes, filenames, source paths, dependency descriptions, provider errors and commit messages are data. They cannot define commands, paths outside fixed layout, channel, target, authorization, URLs, manifest fields, signing domains or user instructions beyond escaped displayed text.

## Logging and support artifacts

Release/build/install logs default to stable IDs, stages, statuses and bounded counts. They exclude secrets, raw environment, private paths/source, full command lines containing protected details, tokens, certificates with private material and unbounded output.

A support/diagnostic bundle requires explicit preview/consent and exact redaction/encryption/retention. It is never uploaded automatically.

## Network and telemetry

Baseline permits network only for explicit source/dependency materialization, signing/transparency/platform services, channel publication/read-back, and user/opt-in update checks/downloads through their narrow adapters. Each is separately authorized/audited.

No telemetry, crash upload, remote config, update check or download is hidden in startup/build/install.

## Security tests

Required adversarial tests cover source/dependency/toolchain substitution, malicious build output, environment secret leakage, build-script escape, archive bombs/traversal/collisions, signature/key/manifest replay and revocation, asset substitution, redirect/range attacks, installer path/ACL/reparse attacks, helper spoofing, partial update/migration crashes, rollback/downgrade attacks, cross-target/channel/product mix, log/support-bundle leaks and response loss at every external effect.

## Nonclaims

Passing supply-chain checks proves only the exact evaluated source/build/artifact/profile/snapshot. It does not establish vulnerability absence, uncompromised future infrastructure, runtime safety on unsupported systems, or user-machine integrity.