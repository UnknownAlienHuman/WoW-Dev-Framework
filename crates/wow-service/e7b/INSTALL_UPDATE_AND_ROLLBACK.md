# E7-B installation, explicit update, data migration, self-replacement, and rollback

**Status:** normative.

## Separation of states

Keep distinct:

```text
channel manifest discovered
update available
bundle materialized/downloaded
bundle verified
installation plan validated
staging prepared
current installation backed up/retained
executable replacement prepared
store/config/data migration prepared
new installation activated
post-install self-check completed
last-known-runnable designated
old installation cleanup eligible
rollback prepared/applied/validated
```

No earlier state implies a later one.

## Installation owner port

```text
ReleaseInstallationPort
    inspect exact installation state
    validate destination/staging/backup capabilities
    stage exact verified bundle
    prepare executable replacement and data migrations
    guarded activate current installation
    run exact post-install validation entry points
    reconcile by OperationId + CanonicalRequestDigest
    rollback to exact retained target
    close resources
```

The port accepts a typed `InstallationPlan`. Service does not receive raw filesystem/process/registry/service-manager APIs or an arbitrary command runner.

## Installation inspection

`release_installation_validate` reads one explicit installation/data-root owner record and validates:

```text
installed release/bundle/member digests
executable architecture/self-description/signatures
compatibility manifest/operation registry/schemas
store/config/data-pack versions and integrity
current/rollback/LKR records
path/permission/ACL/ownership profile
partial/staged/orphan/failed update state
retention/backup/recovery closure
```

It does not trust a filename, PATH lookup, version text, registry entry, process command line or directory name alone.

## New installation

A new install plan freezes:

```text
exact verified ReleaseBundle and support profile
explicit destination and data-root owner handles
staging/backup paths produced by the installation owner
required runtime prerequisites
configuration bootstrap policy
initial Reference/core/provider data-pack policy
post-install self-check
uninstall and rollback behavior
authorization/budgets/cancellation
```

The installer validates paths, disk, permissions, locks, archive safety and member closure before extracting/writing. It never executes bundle-provided scripts.

Default public config contains no secrets. Existing user data is not imported from arbitrary locations without an explicit migration request.

## Explicit update check and plan

Baseline flow:

```text
user invokes wow update check
-> verify exact signed channel/update manifest
-> return UpdateAvailable or exact nonupdate state

user invokes wow update plan/apply
-> freeze exact current installation and target manifest/bundle
-> materialize/download through narrow distribution-read port
-> verify before staging
-> validate support/migration/rollback compatibility
-> create immutable UpdatePlan
-> apply only after explicit request/authorization
```

An opt-in scheduled check profile may exist later, but check/download/install remain separate and no background install occurs.

## Download/materialization

The distribution reader writes only to an installation-owner staging object/path. It validates maximum size, content length/ranges, digest while streaming, provider receipt and cancellation. Partial content is never executed/extracted as complete.

The service/app never follows arbitrary manifest redirects/URLs outside the exact provider policy. TLS is transport security, not artifact authenticity.

## Verification before staging activation

Verify:

```text
signed update/release manifest and trust roots
bundle/member sizes, digests and detached signatures
platform signature when required
archive/path safety
release candidate/support/target applicability
current version and allowed update edge
revocation/retirement/expiry state
store/config/schema/data-pack migration plan
rollback target and retained bytes
available disk/permissions/process replacement capability
```

Failure leaves current installation unchanged.

## Windows running-executable replacement

The initial Windows profile uses a separately identified minimal replacement helper or parent process strategy included and verified in the release/installer artifact set.

Required protocol:

```text
running wow process validates and stages exact target
-> writes durable update intent/plan/receipts
-> launches exact verified helper with opaque plan reference, not arbitrary paths/commands
-> running process closes service/store/session resources and exits
-> helper validates process identity/closure and plan again
-> atomically swaps staged product directory/current pointer where supported
-> starts exact new wow self-check only if plan permits
-> records activation/self-check
-> deletes nothing needed for rollback
-> exits
```

The helper accepts no shell command, URL, arbitrary executable or unsigned plan. It is independently included in SBOM/provenance/signing/support tests.

If helper response/process handoff is lost, exact durable installation records determine state. The next invocation reconciles; it does not repeat replacement blindly.

## Unix-like replacement

Additional platform profiles may use atomic symlink/directory/current-pointer strategies only after exact filesystem, executable, permission, process and rollback tests. They are not inferred from the Windows design.

## Store and schema migration

Executable and persistent data lifecycles are independent. Migration requires:

```text
exact current store/config/schema versions and integrity
exact target version and migration graph
backup/snapshot strategy and verification
forward migration steps through registered store owner operations
rollback/reverse/restore capability or explicit one-way blocker
resource/disk/time estimates and limits
crash points and recovery states
post-migration read-back validation
```

No raw SQL, arbitrary migration script or application-side file rewriting. `wow-store` owns physical transactions/migrations under registered schemas.

A one-way migration may be allowed only when the channel/install profile explicitly blocks binary rollback and the user/owner accepts that effect; stable baseline should preserve a validated rollback/restore path.

## Configuration migration

Public config schemas are versioned and strict. Migration preserves unknown unsupported user fields only under an explicit extension/loss policy; it never writes secrets into public config or silently changes security/exposure/project/provider settings.

Invalid or ambiguous config migration blocks update and keeps the current config/installation.

## Data packs

Reference Packs, core packs and provider adapters have separate signed compatibility/update records. Executable update does not silently update/activate them.

The update plan may retain compatible installed packs, require an exact separately verified pack update, or mark capabilities unavailable. It never chooses latest/default packs by name.

## Activation

After staging and required migrations, activate the exact installation through expected-current CAS. The new current record binds release/bundle/member digests, support profile, store/config/data state, prior current, rollback target, authorization, and effect receipt.

No partial member mix. If CAS is stale, stop/reconcile; do not overwrite a concurrently changed installation.

## Post-install validation

Run a fixed nonexecuting self-check profile from the exact new binary/install root:

```text
binary self-description and signatures
member/manifest/registry/schema closure
store/config open and migration state
status/check fixture against frozen local data
local IPC host startup/handshake smoke test where profile requires
LSP/MCP initialization/capability smoke test
resource/permission/path sanity
no unexpected network/update/telemetry activity
```

The self-check cannot call arbitrary user projects/providers or mutate unrelated data. Full real-addon/client acceptance belongs to pre-release evidence, not post-install scripts.

## Last-known-runnable designation

Designation requires an exact current installation, successful required self-check, compatible rollback state, retained member bytes/data backup, and authorization. It is append-only and profile-specific.

Previous/current/newest/most-used is not LKR without this record.

## Update failure

Failure before activation leaves current unchanged and marks staging cleanup eligibility. Failure after executable/data activation triggers the exact rollback/emergency policy.

Public result preserves:

```text
which executable/store/config/data effects occurred
current/LKR records
self-check results
rollback eligibility/action
OutcomeUnknown and recovery IDs
retention/audit/cleanup state
```

No “update failed” message may conceal a committed partial effect.

## Rollback

`release_update_rollback` requires:

```text
exact failed/current installation
exact retained qualified rollback target/LKR
expected current digest
bundle/signature/support/revocation validation
store/config/data rollback or backup restore plan
replacement helper/process strategy
authorization/budgets/cancellation
```

Rollback is a new immutable effect and follows stage/verify/CAS/self-check rules. It never rewrites history or relabels the failed release. If data migration cannot be reversed/restored, rollback is blocked rather than applying an incompatible old binary.

## Uninstall

The release profile defines exact uninstall scope:

```text
product binaries/manifests
optional current-pointer/shortcuts/integration
user data/store/config retained or deleted only by explicit option
logs/cache/session data policy
provider/signing credentials never managed as ordinary product files
```

Uninstall does not delete shared/external Reference/core/provider artifacts or user projects unless exact ownership and explicit request permit it.

## Cancellation and response loss

Each materialization, stage, backup, migration, CAS, helper handoff, self-check, cleanup and rollback effect has an exact receipt. Cancellation stops new stages but preserves current effect state. Response loss becomes `OutcomeUnknown`; recovery inspects durable installation/store records and never repeats blindly.

## Nonclaims

A successful update/install proves the exact installation and self-check under its profile. It does not prove all user projects, clients, WoW profiles, provider adapters, runtime scenarios or future updates work.