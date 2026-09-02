# Release artifact boundary

**Status:** documentation-only routing. No generated release artifacts exist.

This directory defines where public release-layout schemas/templates or future checked release metadata may be routed. It must not contain ad hoc binaries, ZIP files, credentials, mutable `latest` assets, private source, local build output, or hand-edited checksums.

## Normative owner

The release lifecycle is owned by:

- [`../crates/wow-service/e7b/README.md`](../crates/wow-service/e7b/README.md)
- [`../tools/wow-release/README.md`](../tools/wow-release/README.md)
- [`../apps/wow/e7b/README.md`](../apps/wow/e7b/README.md)
- [`../docs/WORKSPACE_AND_BUILD_PLAN.md`](../docs/WORKSPACE_AND_BUILD_PLAN.md)
- [`../docs/CONFORMANCE_COMMANDS.md`](../docs/CONFORMANCE_COMMANDS.md)

## Generated release layout

A future exact release output is produced outside the source tree or in an explicitly owned staging root:

```text
wow-dev-framework-<release-id>-<target-profile>/
    bin/wow[.exe]
    manifests/
    config/examples/
    docs/
    LICENSE
    THIRD_PARTY_NOTICES.*
```

The exact layout, members, modes, timestamps, compression, checksums, signatures, SBOM, provenance, support and update manifests are determined by the frozen E7-B `ReleaseBundle` profile.

## Source repository policy

Permitted checked content here, after implementation:

```text
versioned JSON schemas/templates for release manifests
public verification instructions
nonsecret test fixtures
release-layout golden manifests
```

Generated candidate/public artifacts are content-addressed owner objects and distribution-provider assets. They are not committed to `main` merely because a release exists.

## Security

Never commit:

```text
private signing keys or certificates with private material
GitHub/package/cloud credentials
KMS/HSM/vault tokens
session/update capability material
local installation/build paths containing private data
unsigned ad hoc installers/binaries
raw user projects or private logs
```

## Current state

```text
release schemas/templates: pending E7-B implementation
release bundles: 0
signed public releases: 0
supported targets: 0
next target intent: Windows x86-64 MSVC after complete evidence
```