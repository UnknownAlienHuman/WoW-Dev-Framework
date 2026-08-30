# Upstream pin and compatibility probe

**Status:** normative gate for selecting the E0-C upstream EmmyLua dependency.

## 1. No automatic pin

The revision recorded in [`../../docs/RESEARCH_BASELINE.md`](../../docs/RESEARCH_BASELINE.md) proves that a usable public Rust API existed when the architecture was researched. It is not automatically the implementation pin.

Before writing adapter code, inspect the current official repository:

- [EmmyLuaLs/emmylua-analyzer-rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust)

Select one exact commit and record:

```text
repository
commit SHA
commit date
workspace/crate names and exact versions
Cargo features enabled/disabled
Rust edition and MSRV/toolchain assumptions
license/SPDX and relevant notices
public APIs used by the adapter
transitive dependency changes relevant to security/build size
```

A tag or branch may aid discovery but cannot replace the commit SHA.

## 2. Candidate pin record

```text
CandidatePin
    repository
    commit_sha
    commit_date
    crate_versions
    cargo_features
    rust_toolchain
    license
    source_tree_digest or lockfile identity
    probe_contract_version
```

The pin record is committed as data before or with the implementation. It contains no local paths or credentials.

## 3. Mandatory public API capabilities

The compatibility probe must demonstrate an equivalent supported path for:

```text
create one analysis/session instance
apply explicit analyzer configuration
register main workspace/source files
register library workspace/source files
add/update/remove file content
build or refresh indexes
obtain stable file identity/URI mapping
run built-in diagnostics for a file/snapshot
obtain syntax facts/source tree
obtain semantic resolution/model facts needed by E0
convert upstream source ranges exactly
repeat updates without hidden editor state
```

Concrete upstream symbols may differ. The adapter contract is semantic, not a demand for one exact upstream function name.

## 4. Probe fixture

Use the closed E0-C fixture workspace under [`examples/`](examples/README.md):

```text
main/clean.lua
main/generic-error.lua
main/missing-api.lua
main/secret-local.lua
library/C_E0Fixture.lua
```

The probe must not read the full Blizzard UI tree or user editor settings.

## 5. Probe sections

### P1 — build and public API

Verify:

- selected commit builds on the supported toolchain;
- required crates/public items are accessible without forking;
- enabled features are explicit;
- adapter does not require private/internal module access;
- no dynamic plugin ABI is required.

### P2 — configuration isolation

Verify:

- configuration is supplied programmatically/through owned data;
- no `.vscode`, user config, or workspace settings are mutated;
- main/library roles are distinct;
- full external editor extension is unnecessary.

### P3 — workspace and file lifecycle

Verify:

- all fixture files register deterministically;
- update and remove operations take effect;
- file identity remains stable according to adapter policy;
- path normalization does not leak absolute host paths;
- duplicate roots/files are rejected/classified.

### P4 — annotation library

Verify:

- `C_E0Fixture.lua` loads as library content;
- `KnownApi` and `SecretText` resolve from main source;
- library declarations are not reported as first-party findings;
- a broken library fixture produces a root capability failure.

### P5 — generic diagnostics

Verify:

- the selected `generic-error.lua` produces one stable semantic category;
- exact upstream diagnostic code/ID and severity are recorded;
- source range maps to canonical bytes;
- rendered message text is not used as identity;
- clean file remains clean for the selected diagnostic family;
- new/unexpected diagnostic families are reported unclassified.

### P6 — reference/call facts

Verify:

- `KnownApi` yields an exact member/reference/call fact;
- `RemovedApi` yields unresolved/unknown analyzer facts without platform-absence wording;
- receiver/member/call spans are exact;
- library source and project source are distinguishable.

### P7 — local-flow facts

Verify:

- `SecretText()` producer call is identified;
- local binding/value identity is stable inside the function;
- concatenation/comparison/branch/log use is represented for the selected fixture;
- `canaccessvalue(value)` guard call and proven control-flow relation can be extracted where supported;
- facts contain no Secret verdict.

### P8 — source coordinates

Verify exact conversion for:

- ASCII LF;
- CRLF;
- multibyte UTF-8 before/inside span;
- empty file;
- EOF insertion/error;
- file update changing byte offsets;
- any upstream UTF-16/LSP position conversion used by the implementation.

### P9 — incrementality

Verify:

- changing one fixture file updates its diagnostics/facts;
- unchanged file facts remain byte-identical only when proven current;
- removed file facts disappear;
- library change invalidates dependent resolution;
- update order does not change final canonical output.

### P10 — determinism

Run equivalent logical inputs repeatedly with varied:

- file discovery order;
- update order ending at same contents;
- worker/test scheduling;
- temporary root path;
- hash-map insertion order.

Canonical analyzer outputs must be byte-identical.

### P11 — scale sanity

E0 does not benchmark full Blizzard source. It must still measure the closed fixture and a bounded synthetic library expansion to detect pathological per-file/session behavior.

Record:

```text
file count and bytes
cold session/index latency
single-file update latency
diagnostic/fact counts
peak/resident memory when practical
```

These are compatibility observations, not production performance claims.

### P12 — failure isolation

Verify:

- one malformed main file does not fabricate facts;
- unrelated clean file diagnostics/facts remain usable only under a coherent snapshot;
- broken annotation library blocks resolution-dependent capabilities;
- upstream panic/session corruption does not publish partial success;
- cancellation/budget failure has typed state where supported.

## 6. Probe report

```text
CompatibilityProbeReport
    candidate_pin
    probe_contract_version
    case_results[]
    observed_upstream_diagnostic_families[]
    source_coordinate_report
    incremental_update_report
    determinism_report
    performance_observations
    mandatory_capabilities
    missing_or_changed_capabilities
    activation_decision
    rollback_pin
```

`activation_decision`:

```text
accepted
accepted_with_adapter_change
rejected_missing_capability
rejected_unclassified_diagnostics
rejected_nondeterministic
rejected_source_coordinate_failure
rejected_security_or_private_api_dependency
```

## 7. Diagnostic policy gate

A candidate pin cannot activate if it introduces a new built-in diagnostic family into default output without classification.

For every observed family record:

```text
upstream code/ID
semantic category
default severity
fixture files affected
normalization mapping
rollout policy = accepted | shadow | ignored-with-reason
```

Ignoring a family requires a documented reason and tests; it cannot hide a required root cause.

## 8. Last-known-good and rollback

- Retain the previously accepted pin and probe report.
- Do not overwrite fixture expectations before understanding a candidate difference.
- If a mandatory capability is lost, keep the old pin active.
- Adapter changes and pin changes must be reviewable separately where practical.
- No dependency auto-update process may activate an unprobed commit.

## 9. Upstream contribution boundary

A small compile-time external diagnostic provider API may be proposed upstream later. E0-C does not depend on it.

Prohibited for E0:

- dynamic Rust plugin ABI;
- permanent fork solely to register WoW diagnostics;
- patching vendored upstream internals in place;
- editor-extension-only correctness.

## 10. Probe acceptance gate

The candidate pin is accepted only when:

```text
all mandatory capabilities have passing cases
main/library isolation works
selected generic diagnostic normalizes exactly
required source/fact spans are exact
local E0 fact slice is obtainable
incremental updates are coherent
determinism passes
no editor mutation or arbitrary source execution occurs
new diagnostics are classified
license/toolchain/dependency record is complete
rollback pin is documented
```
