# E4-A search document and field schema

**Status:** normative.

## Document principle

A `SearchDocument` is a deterministic bounded retrieval projection of one exact owner entity. It is not a source-file copy, a second domain truth record, an analyzer session, a context artifact, or a generated natural-language summary.

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
ExistingFindingEvidenceHandle
```

`ExistingFindingEvidenceHandle` contains exact identity and bounded metadata only when supplied through an owner-public project/graph record. E4-A does not import `wow-rules`.

E4-A does not index `wow-context` Project Maps or skeletons. That would create an undeclared dependency and duplicate source/project/graph projection. Later service-level federation can expose context artifacts separately if a reviewed contract requires it.

## Field definition

Each field definition includes:

```text
field ID/version
data type
allowed document and universe kinds
single/multi-valued and ordering rules
identity/exact/alias/member/prefix/text/shape/filter roles
normalization/tokenizer profile
authority and provenance class
privacy/license class
maximum values/bytes/tokens
stored/indexed/detail behavior
origin requirements
compatibility policy
```

No arbitrary property bag or source-controlled field registration.

## Exact identifier fields

Examples:

```text
entity key/ID
canonical qualified name
canonical short name
namespace/member pair
receiver/method pair
source/package/file/load-unit identifier
API system/name/type/event identifier
explicit alias value and alias-record ID
```

Exact matching preserves case. Frozen Unicode normalization is allowed, but locale-dependent case folding is not exact identifier equality.

## Approximate identifier fields

A separate normalized representation may support:

- ASCII or explicitly frozen Unicode case folding;
- deterministic identifier segmentation;
- separator normalization;
- trigram generation;
- bounded edit distance.

These fields can produce only approximate candidate signals. They never create canonical names or aliases.

## Structured shape fields

```text
entity kind
receiver kind/name key
parameter count/order/name/type/optional/variadic facets
return count/order/type/nilability facets
named type/enum/literal/container facets
restriction/Secret/protected metadata from ReferenceView
package/load/owner/universal-role facets
registration/hook/state/object/template relation kinds
```

Unknown, missing, optional, nilable, unsupported, conflicted, and partial values remain distinct. Shape projection cannot invent `any` or collapse unknown state.

## Approved text fields

Bounded fields can include:

```text
canonical identifiers
explicit aliases
ReferenceView documentation/title/summary fields with exact origins
source documentation comments attached to exact declarations
exact universal-role labels from graph assertions
package/file/load labels
bounded existing-finding message fields carried by owner records
```

Full source bodies, arbitrary repository docs, build logs, SavedVariables contents, runtime payloads, credentials, and unrestricted comments are excluded.

## Text authority classes

```text
CanonicalIdentifier
ExplicitAlias
ReferenceContractDocumentation
ImplementationSourceDocumentation
UniversalRoleLabel
OwnerMetadataLabel
ExistingFindingMessage
```

Text matches retain their class. Source comments and finding messages do not become platform facts.

## Field origins

Every value links exact owner records and transformations. Text origins include:

- raw owner field ID and digest;
- source handle/span where applicable;
- original and normalized text digests;
- normalization/tokenizer profile;
- truncation/redaction/loss record;
- privacy/license decision;
- coverage/conflicts.

## Allowed transformations

- frozen Unicode normalization;
- explicit field-specific case policy;
- deterministic identifier segmentation;
- frozen whitespace/punctuation normalization;
- bounded truncation at declared semantic boundaries with a loss record;
- privacy redaction from exact upstream labels/ranges.

Forbidden transformations:

- model summaries or keyword generation;
- heuristic alias creation;
- source instruction interpretation;
- hidden stemming/synonym dictionaries;
- path/name-based role inference;
- locale/host-dependent normalization;
- dropping origins, conflicts, or coverage to reduce index size.

## Alias fields

An alias requires an explicit owner record containing:

```text
alias value
canonical target entity
scope/profile/generation
alias kind/source/evidence
confidence/provenance/coverage/conflict
```

Fuzzy misspellings, previous search queries, documentation mentions, deprecated names without an explicit alias record, and candidate replacements are not aliases.

## Privacy and license

Local source documentation may be indexed only in a local shard under an exact privacy/license profile. It cannot automatically be returned to an external consumer or copied into a different trust-class shard.

Search snippets are nonauthority presentation records. Exact source retrieval uses owner source handles and a separate permission decision.

## Deduplication

Deduplicate identical values only inside the same document/field/origin-equivalence policy. Preserve all distinct origins and authority classes. Same text in different universes, entities, generations, or source spans remains distinct.

## Validation

- exact owner entity exists and matches universe/generation;
- document and field kinds are allowed;
- origins/evidence/coverage resolve;
- values satisfy type/count/size constraints;
- normalized values reproduce exactly;
- aliases have explicit records;
- no forbidden source body/private field/context dependency;
- no cross-universe identity collapse;
- all loss/redaction/omission transformations are recorded.
