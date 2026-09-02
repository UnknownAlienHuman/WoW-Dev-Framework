# E7-B release lifecycle acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E7-B.

## Source, dependency, toolchain, and plan

| ID | Case | Expected |
|---|---|---|
| `S7B-SRC-001` | Exact complete source tree/commit manifest | SourceValidated |
| `S7B-SRC-002` | Branch/tag/working tree substituted for exact tree | reject |
| `S7B-SRC-003` | Dirty/untracked/forbidden secret/build output | reject |
| `S7B-SRC-004` | Symlink/submodule/path/case/Unicode collision | reject |
| `S7B-SRC-005` | Force-pushed branch after snapshot | snapshot unchanged |
| `S7B-DEP-001` | Exact locked verified dependency materialization | pass |
| `S7B-DEP-002` | Lockfile update/floating git or registry input | reject |
| `S7B-DEP-003` | Checksum/source/provenance/license mismatch | blocked |
| `S7B-DEP-004` | New build script/proc macro/native input unreviewed | blocked |
| `S7B-TC-001` | Exact Rust/toolchain/target/SDK profile | pass |
| `S7B-TC-002` | Host-installed latest toolchain selected | reject |
| `S7B-PLAN-001` | Complete bounded release plan | valid |
| `S7B-PLAN-002` | Arbitrary command/environment/network callback | reject |
| `S7B-PLAN-003` | Unbounded targets/features/resources | reject |
| `S7B-PLAN-004` | Registry/schema/compatibility inputs mismatched | reject |

## Build and reproducibility

| ID | Case | Expected |
|---|---|---|
| `S7B-BLD-001` | Submit exact typed build plan | BuildSubmitted |
| `S7B-BLD-002` | Executor exposes generic process/shell | architecture fail |
| `S7B-BLD-003` | Normal build attempts network access | fail |
| `S7B-BLD-004` | Build reads ambient user/repository credentials | fail |
| `S7B-BLD-005` | Same operation/digest retry | same build/effect |
| `S7B-BLD-006` | Same operation/different digest | reject |
| `S7B-BLD-007` | Response lost after executor dispatch | OutcomeUnknown/reconcile |
| `S7B-BLD-008` | Blind second build chosen by newest/first | reject |
| `S7B-BLD-009` | Unexpected/missing output or secret/private path leak | fail |
| `S7B-BLD-010` | Built binary self-description matches plan | pass |
| `S7B-REP-001` | Two independent unsigned builds byte-match | Reproducible |
| `S7B-REP-002` | Only one build | NotEvaluated/blocked |
| `S7B-REP-003` | Executable/member bytes mismatch | blocked |
| `S7B-REP-004` | Majority/newest output selected after mismatch | reject |
| `S7B-REP-005` | Declared supported wrapper variance only | explicit scoped result |
| `S7B-REP-006` | Platform signature variance changes unsigned identity | reject |
| `S7B-REP-007` | Cache-only build used as reproducibility proof | reject |
| `S7B-REP-008` | 1/2/N workers and clean/cold cache profile | same unsigned artifacts |

## Artifacts, evidence, and bundle

| ID | Case | Expected |
|---|---|---|
| `S7B-ART-001` | Exact target executable/manifest/schema/member set | ArtifactValidated |
| `S7B-ART-002` | Filename/version text substitutes identity | reject |
| `S7B-ART-003` | Unexpected import/rpath/runtime/secret | fail |
| `S7B-ART-004` | Embedded registry/build/source IDs mismatch | fail |
| `S7B-SBOM-001` | Complete distributed/build/runtime dependency SBOM | valid |
| `S7B-SBOM-002` | Missing proc macro/build/native dependency | blocked |
| `S7B-PROV-001` | Provenance binds observed exact inputs/outputs | valid |
| `S7B-PROV-002` | Attestation claims unobserved test/input | reject |
| `S7B-LIC-001` | Complete license/notices/redistribution decisions | pass |
| `S7B-LIC-002` | Private/addon/Blizzard/provider content unlicensed | blocked |
| `S7B-TST-001` | All channel-required reports pass | pass |
| `S7B-TST-002` | Required suite skipped/NotEvaluated | blocked |
| `S7B-TST-003` | Retry hides earlier failure/flakiness | reject |
| `S7B-BND-001` | Deterministic safe portable bundle layout | BundleReady |
| `S7B-BND-002` | Absolute/traversal/device/link/case/Unicode collision | reject |
| `S7B-BND-003` | Archive bomb/size/depth limit | bounded reject |
| `S7B-BND-004` | Missing/unexpected/member digest mismatch | reject |
| `S7B-BND-005` | Bundle bytes changed under same ID | reject |
| `S7B-BND-006` | Mutable/current data pack copied implicitly | reject |
| `S7B-BND-007` | Exact offline data pack set with signatures/licenses | pass |
| `S7B-BND-008` | Rebuild deterministic archive | byte-identical under profile |

## Signing and trust

| ID | Case | Expected |
|---|---|---|
| `S7B-SIGN-001` | Authorized portable detached signature | verify |
| `S7B-SIGN-002` | Wrong domain/target/digest/profile/key purpose | reject |
| `S7B-SIGN-003` | Expired/revoked/compromised/untrusted key | reject |
| `S7B-SIGN-004` | Private key/token/KMS/HSM/vault material exposed | fail |
| `S7B-SIGN-005` | Signature used as build/security/runtime proof | reject |
| `S7B-SIGN-006` | Signing graph cycle/self-reference | reject |
| `S7B-SIGN-007` | Response lost after possible signature | OutcomeUnknown/reconcile |
| `S7B-SIGN-008` | E5 core-pack signing grant reused for release | reject |
| `S7B-SIGN-009` | Platform signing maps exact unsigned to signed artifact | pass |
| `S7B-SIGN-010` | Trust root supplied only by same untrusted bundle | blocked |
| `S7B-SIGN-011` | Key rotation mutates old artifacts/signatures | reject |
| `S7B-SIGN-012` | Revocation transition exact signed and current | pass |

## Support and release candidate

| ID | Case | Expected |
|---|---|---|
| `S7B-SUP-001` | Exact Windows x64 support matrix with all required evidence | valid |
| `S7B-SUP-002` | Untested OS/architecture advertised | reject |
| `S7B-SUP-003` | Protocol compatibility inferred from version string | reject |
| `S7B-SUP-004` | Store/config/data-pack migration edge unproved | blocked |
| `S7B-SUP-005` | WoW profile support without exact current evidence | blocked |
| `S7B-SUP-006` | Required resource benchmark exceeded | blocked |
| `S7B-RC-001` | Complete exact candidate closure | CandidateReady |
| `S7B-RC-002` | Missing/partial/conflict/skipped/NotEvaluated required gate | blocked |
| `S7B-RC-003` | Aggregate success hides mandatory blocker | reject |
| `S7B-RC-004` | Tag/age/downloads/no complaints grants stable | reject |
| `S7B-RC-005` | Candidate input changes after signing | new candidate required |
| `S7B-RC-006` | Release notes overclaim tests/support/runtime | reject |
| `S7B-RC-007` | Real addon evaluation executes repository code | fail |

## Channel, distribution, and update manifest

| ID | Case | Expected |
|---|---|---|
| `S7B-CH-001` | Prepare exact candidate/channel publication plan | valid |
| `S7B-CH-002` | Publish exact immutable objects/manifests/read-back | Published |
| `S7B-CH-003` | Publication mutates bundle/candidate | reject |
| `S7B-CH-004` | Stale channel expected-current CAS | reject |
| `S7B-CH-005` | Latest/newest/highest/tag selected implicitly | reject |
| `S7B-CH-006` | Asset replaced under same release/name | detect/reject |
| `S7B-CH-007` | Upload response lost | OutcomeUnknown/reconcile |
| `S7B-CH-008` | Duplicate/conflicting provider release | quarantine |
| `S7B-CH-009` | GitHub/repo/CI identity authorizes publication | reject |
| `S7B-CH-010` | Public read-back checks only successful HEAD | insufficient |
| `S7B-UPM-001` | Signed exact update manifest | valid |
| `S7B-UPM-002` | Cross-target/channel/product/bundle substitution | reject |
| `S7B-UPM-003` | Unsigned/expired/revoked/replayed manifest | reject |
| `S7B-UPM-004` | Redirect/range/provider scope violation | reject |
| `S7B-UPM-005` | Startup hidden update check/download | reject |
| `S7B-UPM-006` | Explicit update check only | exact status, no download |
| `S7B-UPM-007` | Distribution outage stops local product | reject |

## Install, migration, update, and rollback

| ID | Case | Expected |
|---|---|---|
| `S7B-INS-001` | Inspect exact installed members/current/store/config | valid state |
| `S7B-INS-002` | Directory/version/PATH alone defines installation | reject |
| `S7B-INS-003` | Exact new install plan/staging/rollback policy | InstallPlanned |
| `S7B-INS-004` | Archive extracted before safe member validation | reject |
| `S7B-INS-005` | Bundle script/post-install command executed | reject |
| `S7B-INS-006` | Destination/path/ACL/reparse/device/lock invalid | reject |
| `S7B-UPD-001` | Exact explicit compatible update plan | InstallPlanned |
| `S7B-UPD-002` | Download partial/digest/signature mismatch | fail; current unchanged |
| `S7B-UPD-003` | Running process overwrites itself ad hoc | reject |
| `S7B-UPD-004` | Exact verified Windows helper handoff | pass |
| `S7B-UPD-005` | Helper accepts arbitrary path/command/URL | reject |
| `S7B-UPD-006` | Executable swap implies data migration success | reject |
| `S7B-MIG-001` | Registered store/config migration with verified backup | pass |
| `S7B-MIG-002` | Raw SQL/script/app-side migration | reject |
| `S7B-MIG-003` | One-way migration silently permits binary rollback | reject |
| `S7B-MIG-004` | Crash at every migration boundary | exact recovery/rollback |
| `S7B-ACT-001` | Exact current install CAS + self-check | Installed/Updated |
| `S7B-ACT-002` | Stale current CAS | reject |
| `S7B-ACT-003` | Partial member mix called installed | reject |
| `S7B-ACT-004` | Self-check fails but update called success | reject/rollback policy |
| `S7B-LKR-001` | Explicit retained qualifying LKR | pass |
| `S7B-LKR-002` | Previous/newest/directory inferred LKR | reject |
| `S7B-RB-001` | Exact compatible retained rollback target | RolledBack |
| `S7B-RB-002` | Old binary incompatible with migrated data | blocked |
| `S7B-RB-003` | Rollback rewrites history/relabels failure | reject |
| `S7B-RB-004` | Rollback response lost after CAS | OutcomeUnknown/reconcile |
| `S7B-CLN-001` | Cleanup after retention/LKR gates | pass |
| `S7B-CLN-002` | Cleanup deletes current/backup/LKR/evidence | fail |
| `S7B-UN-001` | Uninstall exact product scope, retain user data by default | pass |
| `S7B-UN-002` | Uninstall deletes projects/shared data packs implicitly | reject |

## Revocation, retirement, incidents, security, and lifecycle

| ID | Case | Expected |
|---|---|---|
| `S7B-REV-001` | Exact authorized signed revocation | Revoked |
| `S7B-REV-002` | Revoked artifact remains eligible in manifest/channel | fail |
| `S7B-REV-003` | Provider asset deletion used as complete revocation | reject |
| `S7B-RET-001` | Exact support/channel retirement | Retired |
| `S7B-RET-002` | Retirement treated as security compromise | reject |
| `S7B-INC-001` | Exact incident scope/evidence/containment/advisory | valid |
| `S7B-INC-002` | Unknown affected scope treated unaffected | reject |
| `S7B-INC-003` | Remediation mutates old release evidence | reject |
| `S7B-INC-004` | Support bundle uploaded without preview/consent/redaction | reject |
| `S7B-SEC-001` | Generic shell/script/SQL/HTTP/GitHub API executor | absent/reject |
| `S7B-SEC-002` | Build environment leaks secrets/private paths | fail |
| `S7B-SEC-003` | Malicious archive/path/link/device/collision/bomb | bounded reject |
| `S7B-SEC-004` | Manifest/asset/key replay/downgrade | reject |
| `S7B-SEC-005` | Release metadata controls path/command/channel | data only |
| `S7B-SEC-006` | Hidden telemetry/update/remote config | absent |
| `S7B-IDEM-001` | Same operation/digest at every effect | same effect |
| `S7B-IDEM-002` | Blind retry under OutcomeUnknown | reject |
| `S7B-IDEM-003` | Conflicting duplicate effect | quarantine |
| `S7B-LIFE-001` | Retention/audit/read-back/close before success | pass |
| `S7B-LIFE-002` | Mandatory close failure returned success | reject |
| `S7B-LIFE-003` | Cancellation at every effect boundary | exact state/no background work |
| `S7B-LIFE-004` | Startup recovery starts new build/upload/update | reject |
| `S7B-DET-001` | Scheduling/host/path/cache/provider timing changes | same semantic records |
| `S7B-FIX-001` | Null pins while implementation not started | allowed |
| `S7B-FIX-002` | First Rust commit with required nulls | fail |
| `S7B-FIX-003` | All source/build/evidence/sign/install/platform checksums frozen | pass |

## Acceptance

E7-B is incomplete until every nondeferred case executes with real source/dependency/toolchain materialization, independent builders, release executor, signing/trust, distribution provider, installer/helper, store/config migration, supported Windows platform/client profiles, update/rollback/revocation recovery, security tests, real project evaluations, and measured resource limits. Documentation fixtures, a CI badge, an uploaded archive, or a valid signature alone are not release evidence.