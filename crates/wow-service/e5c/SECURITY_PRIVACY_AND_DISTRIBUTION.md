# E5-C security, privacy, credentials, and distribution boundary

**Status:** normative.

## Untrusted inputs

Treat submissions, artifacts, attestations, signatures until verified, canary observations, review notes, source metadata, catalog records, authorization envelopes, and cached outputs as untrusted bounded data.

## Prohibited capabilities

E5-C cannot execute repository hooks/workflows/generators/tests, Lua/XML scripts, shell, JavaScript, Wasm, native plugins, callbacks, SQL/FTS, expressions, models, embeddings, or Codebase Memory. It cannot inspect arbitrary filesystem/network/process/editor/client state outside exact narrow owner/signing/authorization/observation ports. It cannot edit addon source or E5-A/B evidence.

## Credentials

Private keys, KMS/HSM/vault credentials, bearer tokens, signing seeds, device PINs, recovery shares, and deployment secrets remain outside requests, fixtures, repository, CLI, logs, result envelopes, and artifacts. Only nonsecret key IDs/trust-root refs, detached signatures, and authorization receipts cross service seams.

GitHub/repository/OS/CLI/file/commit identity is not signing, publication, canary, activation, rollout, rollback, revocation, or distribution authorization.

## Authorization scopes

Separate exact scopes exist for:

```text
artifact build approval
signing
internal publication
canary cohort assignment
observation submission
rollout advance/pause
profile activation/current CAS
LKG designation
rollback
revocation/deactivation
future public distribution
```

A grant is not transferable unless the exact authorization profile says so.

## Cohort privacy

Cohort membership and observations follow explicit privacy/consumer profiles. Public envelopes default to commitments, counts, stable IDs, and redacted typed summaries. Private source, account/character identity, SavedVariables, logs, event payloads, or process memory are not collected unless a later explicit approved runtime adapter/profile exists.

## Distribution

E5-C internal catalog publication does not create:

```text
GitHub Release
public download URL
package registry entry
addon updater feed
signed installer
public update manifest
CDN object
```

Public distribution, download integrity, updater compatibility, release notes, artifact packaging, and channel revocation are E7 concerns.

## License/privacy intersection

Publication/activation output is limited by submission, source/corpus, artifact, attestation, license/notice, cohort, observation, consumer, and store policies. Higher layers cannot widen a lower restriction. Unknown state blocks or narrows.

## Resource safety

Bound submissions, artifact bytes, rules/operators/literals, attestations/SBOM entries, signatures, catalog entries, cohorts, observations, stages, assignments, reindex targets, audit records, continuations, output bytes, memory/time, owner calls, and retries. Unlimited values are invalid.

## Injection boundary

Source, labels, notes, observations, reason text, artifact names, and release metadata remain data. They cannot choose profiles, authorize effects, alter matcher rules, invoke tools, define paths, or create commands.

## Logging

Default logs include stable IDs, counts, stages, statuses, and reason codes. They exclude source bodies, private cohort membership, confidential notes, signature authorization material, credentials, private endpoints/paths, database handles, and unbounded stack data.

## Emergency operations

Emergency rollback/revocation can use a reviewed reduced gate profile but cannot bypass exact targets, authorization, idempotency, response-loss reconciliation, audit, retention, current CAS guards, and partition closure.