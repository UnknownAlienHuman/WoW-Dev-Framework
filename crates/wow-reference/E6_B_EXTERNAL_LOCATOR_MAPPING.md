# E6-B reference-owned external locator mapping seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-reference` owns mapping bounded external locator fields into one exact retained `ReferenceView`. It validates only platform-contract identities represented by the selected reference generation.

## Operations

```text
reference_external_locator_map
reference_external_mapping_validate
```

## Supported mapping classes

As allowed by the exact profile:

```text
API namespace/name
function/method/event/CVar/type/enum/restriction identity
source artifact/path/span in the pinned reference source snapshot
raw and normalized stable keys
explicit owner-known aliases or transition keys
```

Provider labels, scores, summaries, snippets, and inferred relations never create aliases, replacements, deprecations, or platform facts.

## Validation

```text
validate exact ReferenceView/profile/generation
-> validate source snapshot and artifact identity
-> validate namespace/kind/name/stable key
-> validate path/span/digest against owner records when required
-> enumerate exact compatible owner records
-> report checked/unchecked/missing/conflicting fields and coverage
```

## Status

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

A clean no-mapping result requires complete relevant reference partition coverage and negative authority for the requested identity class. Missing generated docs, unsupported kind, unknown upstream fields, partial corrections, or stale profile prevents clean negative authority.

## Hard boundaries

- No floating current/latest ReferenceView.
- No same-name or fuzzy mapping across namespaces/kinds/profiles.
- No automatic deprecation/replacement/lineage inference.
- No runtime truth from static mapping.
- No project-source mapping.
- No direct dependency on `wow-cbm`, `wow-service`, `wow-project`, `wow-context`, or applications.

The boundary consumes an owner-neutral locator projection rather than importing the external-provider crate.

## Evidence and nonclaims

Return exact reference generation/source artifact, mapped owner record/source handles, mapping profile and field comparisons, coverage/negative authority, conflicts/omissions, budgets/cancellation, and canonical digest.

`ExactMapped` means the provider locator identifies an exact reference record under this profile. It does not verify provider interpretation, implementation behavior, current runtime restrictions, replacement safety, or source compatibility outside the selected generation.