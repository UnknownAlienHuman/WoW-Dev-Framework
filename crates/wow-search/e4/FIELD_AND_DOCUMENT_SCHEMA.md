# E4-A search document and field schema

**Status:** normative.

## Document principle

A `SearchDocument` is a deterministic bounded retrieval projection of one exact owner entity. It is not a copy of the source file, a new domain fact, or a generated natural-language summary.

## Initial document kinds

```text
ProjectEntity
ProjectFileOrLoadUnit
BlizzardUiEntity
BlizzardUiPackageOrFile
ReferenceApiEntity
ReferenceTypeOrEnum
ReferenceEventOrCallback
ReferenceRestrictionFacet
ProjectMapOrSkeletonProjection
ExistingFindingEvidenceHandle (optional exact metadata only)
```

ProjectMap/L0/L1 text/facets are indexed only when exact E3-B artifacts are supplied and their profile/generation/coverage identities are retained. Search does not build context artifacts.

## Field registry

Every field definition includes:

```text
field ID/version
data type
allowed document/universe kinds
single/multi-valued and ordering
identity/exact/alias/prefix/text/shape/filter roles
normalization/tokenizer profile
authority and provenance class
privacy/license class
maximum values/bytes/tokens
stored/indexed/detail behavior
origin requirements
compatibility policy
```

No arbitrary JSON blob or source-controlled field registration.

## Exact identifier fields

Examples:

```text
entity key/ID
canonical qualified name
canonical short name
namespace/member pair
source/package/file/load-unit identifiers
API system/name/type/event identifiers
explicit alias value and alias record ID
```

Exact normalization is profile-specific and preserves the original value plus normalized key. Locale-dependent case folding is forbidden unless explicitly frozen.

## Structured shape fields

```text
entity kind
receiver kind/name key
parameter count/order/name/type/optional/variadic facets
return count/order/type/nilability facets
named type/enum/literal/container facets
restriction/Secret/protected metadata from ReferenceView
package/load/owner/role facets
registration/hook/state/object/template relation kinds
```

Unknown/unsupported fields remain explicit. Shape projection cannot invent `any` or collapse optional/nil/missing.

## Searchable text fields

Bounded approved fields can include:

```text
canonical and short identifiers
explicit aliases
ReferenceView documentation/title/summary fields with exact origins
source documentation comments near exact declarations
exact role labels emitted by universal recognizers
Project Map/L0/L1 typed labels and exact source documentation fields
selected migration-note text only in future E4-B shards
```

Full source bodies, arbitrary repository docs, build logs, SavedVariables, runtime payloads, credentials, and unrestricted comments are excluded by default.

## Text authority classes

```text
CanonicalIdentifier
ExplicitAlias
ReferenceContractDocumentation
ImplementationSourceDocumentation
RecognizerRoleLabel
ContextProjectionLabel
ExistingFindingMessage
```

A text match retains its class. Source comments and finding messages do not become platform facts.

## Field origins

Every value links exact owner records and transformations. For text:

- raw source/reference field ID and digest;
- source handle/span when applicable;
- original and normalized text digest;
- normalization/tokenizer profile;
- truncation/redaction/omission record;
- privacy/license decision;
- coverage/conflicts.

## Bounded transformations

Allowed deterministic transformations:

- frozen Unicode normalization/case policy;
- identifier segmentation profile;
- whitespace normalization for text indexing;
- exact field-specific punctuation/token policy;
- bounded truncation at declared semantic boundaries with loss record;
- privacy redaction from exact upstream labels/ranges.

Forbidden:

- model summary/keywords;
- heuristic alias creation;
- source instruction interpretation;
- hidden stemming/synonym dictionary;
- path/name-based role inference;
- locale/host-dependent normalization;
- dropping conflict/coverage/origin to reduce index size.

## Aliases

Alias fields require an explicit owner record with:

```text
alias value
canonical target entity
scope/profile/generation
alias kind/source/evidence
confidence/provenance/coverage/conflict
```

Fuzzy misspellings and deprecated/replacement candidates are not aliases unless the owning current contract explicitly records them. Cross-generation alias/lineage is E4-B.

## Project/source text privacy

Local project source documentation can be indexed in a local shard under an exact privacy profile. It cannot automatically be returned externally or copied into a different trust-class shard. Search result snippets remain presentation data; exact source uses owner handles.

## Deduplication

Deduplicate identical field values only within the same document/field/origin-equivalence policy. Preserve all distinct origins and authority classes. Same text in different universes/entities/spans remains distinct.

## Validation

- exact owner entity exists and matches universe/generation;
- field allowed for document kind;
- origins/evidence/coverage resolve;
- values within type/count/size limits;
- normalized values reproduce deterministically;
- alias records explicit;
- no source body/private data/unsupported field;
- no cross-universe identity collapse;
- all loss/omission transformations recorded.
