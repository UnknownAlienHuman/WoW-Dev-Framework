# E7-B release signing, trust roots, verification, and key lifecycle

**Status:** normative.

## Independent scopes

Keep separate:

```text
release candidate authorization
portable artifact signing
platform executable/package signing
update-manifest signing
channel publication authorization
revocation/retirement authorization
E5 core-pack signing
provider-adapter signing
```

One authorization, key, signature or successful verification never implies another scope.

## Signing ports

```text
ReleaseSigningAuthorizationPort
    authorize exact target kind/ID/digest, signing profile, key reference and purpose

ReleaseSigningPort
    sign one exact domain-separated digest
    reconcile by OperationId + CanonicalRequestDigest

ReleaseSignatureVerificationPort
    validate algorithm, signature, key/trust-root, scope, time, expiry,
    revocation, transparency/evidence and exact target digest

PlatformSigningPort
    apply/verify target-specific code/package signing under a separate profile
```

Service and tools receive only nonsecret key/trust-root references, detached signatures and bounded receipts. They never receive private keys, seeds, passphrases, certificates with private material, KMS/HSM/vault credentials, device PINs, recovery shares, signing-agent sockets or arbitrary callbacks.

## Domain separation

Different artifact classes use distinct exact signing domains, for example:

```text
wow-release-unsigned-artifact-v1
wow-release-bundle-manifest-v1
wow-release-checksum-manifest-v1
wow-release-update-manifest-v1
wow-release-revocation-record-v1
wow-release-retirement-record-v1
```

The exact byte encoding/profile is frozen. A signature for one domain, target, channel, product, profile or key purpose is invalid for another.

## Signing graph

The release profile defines an acyclic graph, typically:

```text
unsigned artifacts
-> member checksum manifest
-> SBOM/provenance/license evidence
-> release bundle manifest
-> detached signatures over required digests
-> release candidate manifest referencing signatures
-> signed update/channel manifest
```

A signature artifact is not included in the digest it signs. Circular/self-referential signing is invalid.

## Portable signatures

Portable detached signatures allow offline verification of exact bundle/manifests independent of the distribution provider. The release includes:

```text
algorithm/profile
key ID/version and trust-root reference
signed target kind/ID/digest
signature bytes
verification instructions/tool compatibility
issuance/expiry/revocation evidence required by profile
```

Trust roots are distributed through an explicit release/bootstrap policy. A key ID in the same untrusted bundle is not independently trusted by itself.

## Platform signing

Windows code signing or another platform signature is a separate wrapper/effect over the reproducible unsigned binary/package. The profile records:

```text
exact unsigned input digest
platform signing identity/reference
signature/notarization receipt
signed output digest
verification policy and supported platform behavior
```

Platform signature variance does not invalidate the unsigned reproducibility claim when the exact mapping is recorded, but signed output remains its own immutable artifact.

## Key lifecycle

Key/trust-root state:

```text
Active
Rotating
Expired
Revoked
Compromised
Retired
Unknown
```

Unknown, expired, revoked, compromised, purpose-mismatched, unsupported or unverifiable state blocks the corresponding gate. Trusted time/order evidence is explicit; local wall clock alone is insufficient where policy requires stronger evidence.

Key rotation creates new signature envelopes and release validation records. It never mutates unsigned artifacts or historical signatures.

## Revocation

A release revocation record can target:

```text
release/bundle/update manifest
specific unsigned or signed artifact
signing key/version/trust root
channel/profile/target applicability
```

It contains evidence, reason, authorization, effective ordering, required client/channel actions, replacement guidance if known, audit and signatures. It does not retroactively erase that a signature once verified under an earlier state; it changes current eligibility.

## Verification order

Consumers verify before execution/extraction/install:

```text
trusted bootstrap/trust-root policy
-> signed update/release manifest
-> exact bundle/member sizes and digests
-> portable detached signatures
-> platform signatures when required
-> SBOM/provenance/license/compatibility closure
-> archive/path safety
-> target/support/install applicability
```

Distribution TLS/provider metadata, GitHub release status, tag, download count or filename never substitutes for artifact verification.

## Response loss

If signing or platform signing may have occurred but the response is lost:

- enter `OutcomeUnknown`;
- do not request another signature blindly;
- reconcile by exact operation/target digest/key/profile;
- preserve quota/audit effects;
- quarantine conflicting signatures/receipts.

Multiple valid deterministic or randomized signatures may be allowed only when the signature profile explicitly defines set semantics; a lost-response retry cannot create an untracked extra signature.

## Transparency and timestamping

Transparency log or trusted timestamp evidence is optional unless the selected channel/signing profile requires it. When used, it is a narrow external effect with exact request/receipt, privacy policy, response-loss reconciliation and independent validation. It is not semantic/runtime proof.

## Verification tooling

The public `wow` binary may verify release/update artifacts using embedded or configured public trust roots under the release profile. It never signs. The internal `wow-release` tool requests signing through service ports and never reads private material.

## Nonclaims

A valid release signature proves exact bytes, key/trust-root and signing policy at a defined time/state. It does not prove source correctness, reproducibility, vulnerability absence, platform compatibility, installation success, runtime behavior, WoW profile support, stable-channel eligibility, or authorization to execute/update.