# E7-B release artifacts, bundle layout, SBOM, provenance, license, and notices

**Status:** normative.

## Required public artifact classes

For each exact target release profile, produce at least:

```text
unsigned wow executable
transport compatibility manifest
service operation registry
public request/result/error schemas
release/support/update/install compatibility manifests
LICENSE
third-party notices and source/license manifest
SBOM
source/build provenance attestations
artifact/member SHA-256 manifest
detached signature envelopes and trust-root references
verification instructions
required user installation/configuration/troubleshooting documentation
```

Optional debug symbols, native installer/package, offline Reference/core-pack bundle, and source archive are separate explicitly identified artifacts with independent policy.

## Artifact identity

Every artifact binds exact:

```text
release plan/source/build/artifact-set IDs
target triple/platform/profile
content bytes/size/digest/media type
file mode and executable policy when relevant
license/privacy/distribution policy
producer implementation/profile
canonical manifest entry
```

Filename, extension, GitHub asset name, MIME guess, archive order, or URL is not identity.

## Bundle layout

The portable bundle has one deterministic top-level directory:

```text
wow-dev-framework-<release-id>-<target-profile>/
    bin/wow[.exe]
    manifests/release.json
    manifests/compatibility.json
    manifests/operations.json
    manifests/schemas.json
    manifests/checksums.sha256
    manifests/sbom.*
    manifests/provenance.*
    manifests/signatures/
    config/examples/
    docs/
    LICENSE
    THIRD_PARTY_NOTICES.*
```

Exact names come from the bundle profile and release ID, not untrusted source metadata. Archive member traversal, absolute paths, devices, alternate streams, links outside root, duplicate/case-colliding names, unsupported modes, and ambiguous Unicode normalization are rejected.

The archive profile freezes format, ordering, path separator, compression, timestamps, modes, ownership fields, symlink policy, metadata and maximum sizes. Archive extraction is never required to validate unsafe names first; validation uses a bounded manifest reader.

## Executable validation

Validate the executable against the exact target/profile:

```text
file format/architecture/subsystem
expected imports/runtime dependencies
no unexpected rpath/search path or bundled secret
embedded build/source/registry/compatibility identities
version output and self-check contract
reproducible unsigned digest
platform signature/notarization state when required
malware/defense scan reports when required by profile
```

Scanner results are time/tool/signature-database scoped and do not create proof of future safety.

## SBOM

The SBOM covers all distributed and executable build dependencies relevant to the release:

```text
Rust crates and exact checksums/source provenance
build dependencies/proc macros/build scripts
native libraries/tools/runtime redistributables
embedded schemas/templates/config/data
offline bundled Reference/core/provider artifacts
installer/bootstrapper/updater components
licenses, notices and known package identifiers
relationships from source to artifact where supported
```

The selected SBOM format/version is an exact profile input. Unknown package identity or license remains explicit and can block the channel.

## Provenance

Provenance attestations bind:

```text
source repository and exact tree
release plan and materialization manifest
builder implementation/independence
Rust/toolchain/target/dependency inputs
build commands as reviewed plan IDs, not arbitrary shell
unsigned artifact digests
reproducibility comparison
test/benchmark/security reports
SBOM/license/notice artifacts
signing and channel intent
```

Attestation predicates and schemas are pinned. Provenance cannot claim inputs or tests not actually observed.

## License and notices

The release candidate requires explicit decisions for:

- framework code under MIT;
- Rust/native dependencies;
- generated schemas/manifests/documentation;
- external provider adapters;
- optional Reference/core/data packs;
- Blizzard or addon-derived fixtures/source excerpts;
- installer/updater/runtime redistributables;
- source-offer/attribution/notice obligations.

Querying or indexing a source does not grant redistribution rights. Private/project source is never placed in a release bundle. A license block cannot be waived by omitting attribution metadata while retaining the content.

## Offline data bundles

An optional offline bundle may include exact separately signed artifacts:

```text
Reference Pack
core recognizer pack
provider adapter/configuration without credentials
starter fixtures
```

The bundle manifest records each independent artifact identity, compatibility, signature, license, update policy and uninstall/rollback relationship. The executable release remains validly identifiable without the offline members.

No mutable `current` data pack is copied into a release.

## Checksums

The public checksum manifest covers every distributed member except signatures that sign the checksum/release manifest according to the exact acyclic signing graph. It specifies algorithm/profile and canonical ordering.

Checksums detect byte mismatch; they do not authenticate without an independently trusted signature.

## Documentation artifacts

The release bundle includes only the bounded end-user subset required for use/verification. Full architecture/agent/internal test planning can remain in the source repository. User docs are generated or copied from exact source paths and included in checksums/SBOM/provenance where applicable.

## Bundle validation

`release_bundle_validate` verifies:

```text
exact candidate/target/profile
member closure and no extras
member bytes/sizes/digests/modes/paths
executable self-description
manifest/schema consistency
SBOM/provenance/license/notice closure
signature/trust-root policy
support/install/update/rollback metadata
archive safety and deterministic reconstruction
privacy/secret scan
retention and audit
```

It never repairs a bundle. Changed content creates a new artifact/bundle/candidate identity.

## Nonclaims

A complete bundle does not prove installation success, platform support outside the matrix, runtime correctness, current WoW compatibility, absence of vulnerabilities, channel publication, user adoption, or safe automatic update.