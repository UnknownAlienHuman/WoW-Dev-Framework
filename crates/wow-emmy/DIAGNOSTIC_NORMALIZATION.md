# Generic diagnostic normalization

**Status:** normative E0-C mapping contract.

`wow-emmy` normalizes built-in upstream analyzer diagnostics into framework-owned observations and `wow-core Finding` records. It does not implement WoW-specific diagnostics.

## 1. Two-layer representation

```text
upstream diagnostic
    -> GenericDiagnosticObservation
    -> wow-core Finding
```

The observation preserves adapter/upstream detail. The core finding is the common public diagnostic contract used later by `wow-service`.

## 2. Required retained fields

Every normalized generic diagnostic retains:

```text
framework semantic category
exact upstream repository/commit/crate version
upstream diagnostic code/ID/name
upstream severity
normalized severity
rollout classification
project source handle and canonical byte span
structured message arguments
related source spans when valid
producer ID/version
analyzer snapshot ID
generation context
capability/coverage ID
root-cause key/fingerprint inputs
```

Rendered upstream message text may be retained for debugging, but it is not finding identity or the only contract.

## 3. Stable semantic category

E0-C defines one selected category after the compatibility probe:

```text
emmy.generic.fixture_error
```

Before implementation, the exact upstream diagnostic family and fixture shape remain intentionally unpinned. The probe must choose a category that:

- is reliably produced by the official public analysis API;
- has an exact, stable source span;
- does not prevent the other E0 files from being analyzed because it lives in a dedicated file;
- has a clean counterpart;
- can be normalized without message-text parsing;
- remains representative of a built-in generic Emmy diagnostic.

Candidate forms may include a dedicated syntax or type diagnostic, but the selected upstream ID and fixture are frozen only after probing.

## 4. Upstream diagnostic classification

```text
UpstreamDiagnosticClassification
    accepted
    shadow
    ignored_with_reason
    incompatible
```

### `accepted`

Mapped to a known framework category and allowed in the normalized output.

### `shadow`

Reported in compatibility/evaluation output but not default blocking/user stream.

### `ignored_with_reason`

Excluded only when a documented contract says it is irrelevant/noisy and tests prove required root causes remain visible.

### `incompatible`

Candidate pin cannot activate because the diagnostic behavior violates a mandatory contract.

## 5. Severity normalization

Upstream technical severity and framework blocking policy are separate.

```text
NormalizedSeverity
    error
    warning
    information
    hint
```

```text
RolloutPolicy
    shadow
    advisory
    blocking
```

E0-C records normalized severity but does not silently decide product blocking policy from upstream defaults. The selected generic fixture is normally advisory/blocking only as defined by the E0 integration contract.

## 6. Source location

The primary source is always a Main-workspace project file.

Rules:

- canonical UTF-8 byte half-open span;
- content digest matches snapshot;
- no absolute host path;
- library declaration may appear as related evidence only when needed;
- a library-source diagnostic is classified as library health/root cause, not a first-party project finding;
- stale/invalid span prevents publication.

See [`SOURCE_COORDINATES.md`](SOURCE_COORDINATES.md).

## 7. Message arguments

Prefer structured fields such as:

```text
expected type/kind
actual type/kind
symbol spelling
diagnostic subcategory
related entity/file IDs
```

Do not parse localized/rendered message strings to recover structure when the upstream object/API provides typed data.

If only unstructured text exists for a nonessential argument, retain it as sanitized supplemental text and make that limitation explicit in the probe.

## 8. Finding identity

Finding fingerprint inputs include:

```text
framework category
upstream diagnostic code/version family
primary source handle/content digest/span
structured arguments
project generation
producer version
```

Excluded:

```text
rendered prose
temporary path
memory address
wall-clock timestamp
discovery order
```

Duplicate diagnostics with equivalent structured identity canonicalize deterministically.

## 9. Root-cause behavior

`wow-emmy` may provide deterministic root-cause keys/relationships for analyzer-owned failures, including:

```text
annotation library failed
file parse failed
configuration invalid
upstream diagnostic directly observed
```

It does not perform final cross-crate folding.

Examples:

- broken annotation library -> library-health error/root cause;
- downstream unknown-global symptoms may be related/marked dependent;
- `wow-service` later decides default folding while preserving raw findings.

Grouping by similar message text is prohibited.

## 10. Generic versus WoW findings

Generic finding:

```text
producer: wow-emmy/upstream adapter
provenance: project_source or analyzer observation
rule/category: emmy.*
platform evidence: none
```

WoW finding (later):

```text
producer: wow-rules
project evidence: analyzer/project source
reference evidence: wow-reference
rule: wow.*
```

A generic unresolved-member diagnostic does not become `wow.api.exists` merely because the symbol resembles a WoW API.

## 11. New diagnostics on pin update

The compatibility probe inventories every diagnostic family emitted by the closed fixture set.

For each new/changed family:

1. record exact upstream code/version;
2. identify source files/cases affected;
3. compare prior behavior;
4. assign framework category or shadow/ignore/incompatible classification;
5. add positive/clean/partial fixtures;
6. update normalization mapping and test matrix;
7. do not activate blocking behavior silently.

## 12. Diagnostic capability coverage

Example records:

```text
capability: emmy.file.diagnostics
partition: emmy.file:main/generic-error.lua
status: Complete
```

Failure cases:

- parse/index session unavailable -> `Failed`/`NotEvaluated` as applicable;
- diagnostic budget truncation -> `Partial`, explicit count/budget;
- stale file/snapshot -> request rejected;
- library failure -> diagnostics not requiring library may remain available, resolution diagnostics may be blocked.

An empty diagnostic list under unavailable capability is not a clean file.

## 13. E0-C normative cases

### Clean file

- selected diagnostic category absent;
- diagnostics capability complete;
- no hidden ignored family affecting acceptance.

### Generic-error file

- exactly one expected accepted normalized category;
- exact upstream ID recorded by the probe;
- exact byte span and source handle;
- deterministic structured arguments/fingerprint;
- no WoW platform conclusion.

### Broken annotation library

- library health/root-cause finding or typed session capability failure;
- dependent resolution facts/diagnostics unavailable;
- no flood treated as independent API-removal findings.

### Unexpected new family

- candidate pin report marks unclassified/shadow/incompatible;
- activation gate fails if the family would alter default behavior.

## 14. Required operations

```text
inventory_upstream_diagnostic_families
classify_upstream_diagnostic
normalize_upstream_severity
convert_diagnostic_span
normalize_generic_diagnostic_observation
build_generic_core_finding
derive_generic_root_cause_key
canonicalize_generic_diagnostics
validate_generic_finding
```

## 15. Prohibited operations

```text
parse_message_text_for_identity
map_unresolved_member_to_wow_api_absence
attach_platform_source_evidence
infer_replacement
suppress_all_unknown_globals_after_library_failure_without_root_record
inherit_upstream_blocking_policy_implicitly
```

## 16. Determinism

Equivalent upstream diagnostic sets normalize to identical canonical order/bytes regardless of:

- upstream return order;
- worker scheduling;
- temp roots;
- rendered message wording that does not change structured meaning;
- hash-map insertion order.

A materially changed upstream diagnostic code/category/span is a compatibility difference and must change the probe report/expected contract deliberately.
