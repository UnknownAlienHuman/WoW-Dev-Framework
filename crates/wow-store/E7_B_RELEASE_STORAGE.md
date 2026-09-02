# E7-B release, channel, installation, update, and support storage seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-store` provides generic immutable object, append-only state, compare-and-swap, durable effect, lease, retention, backup/restore, and GC mechanisms for E7-B. It does not interpret source/build/signature/channel/install/update/support semantics.

## Logical objects supplied by service/owners

```text
ReleaseSourceSnapshot and materialization manifests
ReleasePlan and build execution receipts
unsigned/signed artifact sets
SBOM/provenance/license/notices/checksums
ReleaseBundle and ReleaseCandidate
support matrices/test/benchmark/security reports
channel/publication/update manifests and provider receipts
installation/staging/backup/current/LKR records
store/config migration plan/receipts
update/rollback/revocation/retirement/incident records
authorization/reconciliation/audit records
```

Producers supply registered schemas, canonical bytes, logical IDs/digests, prepared operations, validation callbacks through narrow ports, and retention edges.

## Generic operations

```text
publish/read exact immutable object
append/read exact catalog/state record
snapshot-bound deterministic list
compare-and-swap channel/current/LKR/session pointers
record/reconcile durable effect
create/renew/release lease
admit/release retention
backup/restore and validate reachability
GC only unreachable eligible objects
```

There is no raw SQL/connection/table/row/object-key/filesystem/transaction callback surface to service/apps/tools.

## State ownership

Store may enforce generic expected-current CAS and append-only transition integrity. It does not decide whether a release is reproducible, signed, supported, installable, revoked, retired or safe. Service/owner validators decide semantics before prepared writes and after fresh read-back.

## Install/data migrations

Physical store schema migrations use the existing registered migration framework. E7-B service supplies an exact migration plan/compatibility/backup/rollback policy; store owns transactions, crash recovery and read-back.

No raw SQL or arbitrary migration scripts cross the seam. A physical commit does not imply complete update/install state.

## Sensitive data

Private signing/distribution/build credentials never enter store objects under this seam. Public signature/trust/provenance artifacts are separate. Private local installation paths/user data are represented only by owner-scoped records under privacy/encryption policy and do not enter portable release identities.

## Retention

Protect all current/channel/LKR/rollback/support/unresolved-effect/incident/legal artifacts and their transitive evidence. GC cannot remove:

```text
published or supported release verification artifacts
current or staged installation members
verified backup and qualified rollback target
active update/channel manifests
unresolved build/sign/publish/install/migration effects
incident/revocation/retirement evidence
```

Cleanup eligibility is an explicit service/owner record; store never infers it from age or path.

## Response loss

Every write/CAS/backup/restore/GC effect reconciles by exact operation/request/object/state identity. Same operation ID/different digest fails. Possible commit with lost response remains unknown; service cannot duplicate or choose newest state.

## Dependency

```text
wow-store -> wow-core
```

No dependency on `wow-service`, `apps/wow`, `wow-release`, signing/distribution/installer adapters or release domain crates.

## Tests

Required tests cover object/catalog corruption, CAS races, crash at every release/install/migration write boundary, backup/restore, lease/retention/GC, cross-release/state substitution, private-record policy, deterministic logical records and exact reconciliation independent of SQLite/WAL/physical layout.