# E5-C signing, provenance, SBOM, license, and notices

**Status:** normative.

## Separation

```text
semantic artifact validation
publication authorization
signing authorization
signature generation
signature verification
provenance attestation
license/notice validation
activation authorization
```

These are independent. A signature cannot repair semantic invalidity or authorize activation.

## Signing ports

```text
CorePackSigningAuthorizationPort
    authorize exact artifact/attestation digest, algorithm/profile, key reference and purpose

CorePackSigningPort
    sign exact digest under authorized scope
    reconcile by exact operation/request identity

CorePackSignatureVerificationPort
    verify detached signature, key/trust-root/revocation/expiry/profile binding
```

Service sees nonsecret key IDs/trust-root refs and detached signature bytes. It never receives private key, seed, KMS/HSM/vault credential, bearer token, PIN, recovery material, or unrestricted signing callback.

## Detached signatures

Sign exact domain-separated digests for:

```text
CorePackArtifact
CorePackAttestationSet
optional publication manifest
```

The signature envelope includes algorithm/profile, key ID/version, trust-root refs, authorization receipt, issuance/expiry/revocation evidence where required, exact signed digest, signature bytes, and verification report.

Do not sign floating catalogs/current pointers as semantic artifact identity.

## Key lifecycle

- Unknown, expired, revoked, unsupported, untrusted, scope-mismatched, or replayed keys/signatures block the required gate.
- Revocation after publication creates an explicit revocation/re-evaluation requirement; it does not rewrite historical validation.
- Key rotation does not mutate an artifact. A new signature envelope/publication validation record is created.
- Local clock alone is insufficient when trusted time is required.

## Provenance attestations

Attest at least:

```text
exact submission/candidate lineage
artifact canonical bytes and build profile
builder implementation/toolchain/dependency identities
reproducibility classification and comparison report
input corpus/split/label/holdout/review report references
registered operator/rule/graph schema/profile identities
source/license/notice/privacy decisions
```

Attestations are evidence records, not claims that all dependencies or environments are safe.

## SBOM

The SBOM identifies exact packaged rule/operator/schema/config/template dependencies and any executable/library dependencies required by the build/runtime implementation profile. Unknown dependency identity or license state blocks publication under a profile requiring closure.

A rule pack is data, but its interpreter/runtime implementation remains a dependency and must be pinned separately.

## License and notices

Publication requires explicit decisions for:

- framework-owned files;
- copied/adapted rule or fixture content;
- corpus-derived metadata;
- generated artifacts;
- third-party schema/runtime dependencies;
- Blizzard or addon source references/quoted material;
- redistribution and attribution scope.

Local analysis permission is not public redistribution permission. E5-C internal publication cannot silently broaden license/privacy rights.

## Reproducibility

Classify separately:

```text
semantic artifact bytes reproducible
attestation bytes reproducible except authorized operational fields
signature bytes deterministic or algorithmically variable but verifiable
catalog/store physical bytes noncanonical
```

1/2/N worker and shuffled-order builds must produce identical canonical semantic artifact bytes. Signature nondeterminism cannot change the signed digest.

## Response loss

If signing may have occurred but the response is lost, enter `OutcomeUnknown` and reconcile by exact operation/artifact/digest/key/profile. Never request another signature blindly under a new operation identity.

## Nonclaims

A valid signature or SBOM does not prove runtime correctness, semantic completeness, absence of vulnerabilities, current WoW compatibility, canary success, activation authorization, or public release eligibility.