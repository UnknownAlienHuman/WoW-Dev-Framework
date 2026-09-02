# E7-B source closure, hermetic build, and reproducibility

**Status:** normative.

## Source validation

`release_source_validate` accepts one exact repository source selector and produces a `ReleaseSourceSnapshot`. It verifies:

```text
repository identity and exact commit/tree
complete recursive path/mode/blob manifest
submodule/vendor/archive identities
Cargo workspace manifests and Cargo.lock
rust-toolchain/toolchain manifest when implemented
forbidden generated/secret/build-output files
line endings/file modes/symlinks under source profile
license/notices/provenance
source retention
```

A Git tag, branch, release name, working-directory state, or “clean” command result is not source identity. A dirty local tree is never silently substituted for the selected commit.

## Materialization

External source/dependency/toolchain acquisition is a separate explicit, authorized, checksummed effect. The normal build consumes a complete verified materialization manifest.

Materialization records:

```text
source and package registry objects
crate index/package checksums and provenance
vendored/native tool inputs
Rust toolchain/components/targets
platform SDK/compiler/linker/runtime inputs
license and redistribution state
mirror/acquisition provenance
retention and offline availability
```

Unknown, floating, unavailable, checksum-mismatched, or license-blocked input prevents the corresponding release target.

## Build plan

`release_plan_validate` freezes:

```text
source snapshot
package and binary targets
Rust target triple, toolchain and linker profile
Cargo features/default-feature policy
locked dependency/materialization manifest
build-script/proc-macro/native code policy
profile/debug/LTO/strip/panic/codegen settings
path remapping and deterministic timestamp policy
environment-variable allowlist and exact values
locale/timezone/umask/file-mode policy
operation registry/schemas/compatibility manifest generation
artifact layout and validation
independent builder requirements
resource and timeout limits
```

No caller-controlled command string, shell fragment, arbitrary environment block, script path, upload callback, or output path derived from source text.

## Build executor port

```text
ReleaseBuildExecutorPort
    validate exact build-plan capability
    execute one allow-listed build plan
    return typed phase/artifact/resource receipts
    cancel safely
    reconcile by OperationId + CanonicalRequestDigest
    close executor resources
```

The executor may internally invoke Cargo, rustc, linkers, packaging libraries and platform tools named by the reviewed plan. Service never receives a generic process API.

## Network policy

Default build profile:

```text
source/dependencies/toolchain already materialized and verified
Cargo locked/offline/frozen behavior
no registry/index/network access during compile/package
no update checks or remote configuration
```

A target that genuinely requires an online platform service uses a separate explicit effect/adapter after unsigned build, never hidden inside compilation.

## Build scripts and native code

Rust build scripts, proc macros and native dependencies are executable supply-chain inputs. The release plan enumerates their package/source/checksum/license/toolchain identities and execution sandbox/profile.

Unreviewed new executable build dependency or changed build script invalidates prior reproducibility/security evidence. It is not treated as ordinary source data.

## Deterministic inputs

Where supported, normalize or explicitly bind:

```text
source paths and remap prefixes
archive member order, names, modes and timestamps
locale/timezone
random seeds
build timestamp/source-date value
linker/build IDs
metadata hashes
file traversal and generated registry/schema order
compression parameters
```

Do not strip meaningful provenance or security metadata merely to force equal bytes. Declared unavoidable wrapper variance is separate from semantic artifact reproducibility.

## Independent builds

Every reproducibility claim uses at least two executions with distinct builder-instance identities under the same exact plan. The profile states which independence properties are required, such as clean work directory, process/container/VM, or separate host.

Comparison occurs before platform signing/notarization:

```text
executable bytes and digest
embedded build/compatibility/registry/schema identities
public schemas/config/docs/license/notices
archive member content/order/modes/timestamps
symbol/debug artifacts when distributed
```

Conclusion states:

```text
Reproducible
ReproducibleWithDeclaredWrapperVariance
Mismatch
NotEvaluated
Failed
```

A mismatch cannot be waived by majority build, newest builder, or signing one output. Root cause and scope are required before a new release candidate.

## Self-description validation

The built `wow` executable must report or expose exact compiled:

```text
version/source tree/build profile/target
service operation registry ID/digest
transport compatibility manifest ID/digest
public schema-set IDs/digests
feature/exposure profiles
store/migration compatibility
license/notices/provenance references
```

These values must match the release plan and external artifacts. The validator does not trust filenames or version text alone.

## Tests during build

The release plan enumerates exact commands/owners for unit, integration, contract, fixture, mutation, platform, client, security and benchmark suites. Each report records pass/fail/skipped/`NotEvaluated` separately.

A skipped required suite blocks release. CI success without exact reports does not satisfy the gate.

## Caching

Build caches are optional accelerators. Reproducibility validation includes clean/cold builds. Cache keys bind exact compiler/tool/dependency/source/profile inputs; cache hit cannot substitute validation or source closure.

## Cancellation and response loss

Build submission/execution uses durable operation identity. Cancellation may stop an executor but does not assume no artifacts or resource effects. A lost executor response is reconciled by exact operation/plan/builder identity; service never starts another build and chooses whichever finishes first.

## Nonclaims

A reproducible build proves repeatability for the exact input/profile. It does not prove absence of vulnerabilities, correct runtime behavior, platform support, WoW compatibility, safe installation, valid signatures, channel eligibility, or future reproducibility.