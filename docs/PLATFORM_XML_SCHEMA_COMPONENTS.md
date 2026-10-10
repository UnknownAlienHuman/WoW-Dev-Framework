# Explicit platform XML schema components

`wow-project::load::schema` admits explicitly selected XSD members from one
genuine retained `AdmittedPlatformSource`. The resulting `AdmittedXmlSchema`
holds an `Arc` to that source, native XML indexes, normalized source observations
and a sealed receipt. It provides the source component prerequisite for later
schema-backed classification; it does not implement ancestry proofs, UI component
classification, XSD instance validation or runtime-class inference.

The profiles are `wow-project/xml-schema-components/1` and
`wow-project/xml-schema-component-policy/1`. No Blizzard schema body, tag inventory
or moving source revision is embedded by this owner.

## Selection and native admission

The public entry points are:

```rust
XmlSchemaSelection::for_source(
    source: &AdmittedPlatformSource,
    paths: &[&str],
    stop: &AtomicBool,
) -> ProjectResult<XmlSchemaSelection>

admit_xml_schema(
    source: &Arc<AdmittedPlatformSource>,
    selection: &XmlSchemaSelection,
    stop: &AtomicBool,
) -> ProjectResult<AdmittedXmlSchema>
```

These are signature summaries. The caller supplies explicit original relative
paths; there is no filename discovery, disk reread, source acquisition or location
following. Each member must be `Included` and declared `PlatformFileKind::Schema`
in the held source. Its requested source-content digest and byte length must
match the actual raw member. Excluded, unsupported, external, conflicted or failed
inventory entries retain the existing raw-owner refusals.

`XmlSchemaSelection` binds `source_snapshot_id`, `profile_digest`,
`content_manifest_digest`, `admission_digest` and its member selections. Each
`XmlSchemaMemberSelection` contains `path`, `content_digest` and `byte_length`.
The request structs support strict Serde decoding, but decoding a request does
not create a native capability. Admission sorts the member list, rejects
duplicates, validates the complete source/member closure and input bounds before
the first parser invocation, and requires UTF-8 bytes.

`AdmittedXmlSchema` has no public constructor or Deserialize route. Its getters
are `source()`, `receipt()`, `documents()`, `components()`, `references()` and
`issues()`. `validate_source(source, stop)` checks the complete source-receipt
binding and every selected member again; equal XSD bytes under a different source
closure do not authorize reuse. The genuine source remains retained through the
original `Arc`; serialized metadata cannot reconstruct that authority.

The Serialize-only `XmlSchemaReceipt` records both profile names, source snapshot
and profile/content/admission digests, canonical selection and component-result
digests, sorted members and component/reference/issue counts. Component identity
binds the selection scope, original document/content digest, native occurrence,
kind and parent address. Local and anonymous components retain their source/owner
addresses rather than receiving invented global names.

## Native normalization and source evidence

The existing bounded XML Reader produces the syntax index in one tokenizer pass.
Normalization consumes that actual index and recognizes grammar by expanded XSD
namespace names. The index's UI-oriented unknown-namespace role is not an XSD
admission verdict. No second parser, temporary project view or analyzer is used.

`XmlSchemaComponent` exposes identity, kind, state, optional expanded name/parent,
document, source-content digest, native occurrence, span and original attributes.
Kinds distinguish global/local/reference elements and attributes, named/anonymous
simple and complex types, particles, groups, attribute groups, content,
extension/restriction, facets, list/union, wildcards and dependency/unsupported
records. States are `Observed`, `Unsupported` or `Invalid` source observations.

After declaration validation, a bounded parent-order pass propagates context
before reference resolution. An otherwise `Observed` child of an Invalid or
Unsupported parent becomes `Unsupported`, including descendants reached through
that propagated state. Existing Invalid/Unsupported child states remain intact.
Thus a parent invalidated by declaration contradictions cannot leave its children
advertised as independently Observed. References from non-Observed contexts,
or to a single non-Observed candidate, retain `UnsupportedContext` rather than
`Unique`; invalid QName outcomes remain explicit. This is context propagation,
not an ancestry or semantic-validity proof.

`XmlSchemaAttribute` preserves the qualified and expanded attribute names,
decoded value, original attribute/value spans and decoded-value digest, with a
bounded typed interpretation where supported. Type, base, ref, substitution,
item/member types and other declared scalar observations remain separate. Unknown
grammar, unsupported attributes/dependencies, malformed semantics, ambiguity and
conflicts retain typed issues and source addresses. An admitted result can contain
these outcomes; admission is not a claim that every component is valid.

Namespace handling follows the native parent/order/span records and current
element's decoded declarations. Element names, attribute names and QName-valued
attributes use separate rules. Default namespaces apply to element/QName-value
interpretation; they do not qualify ordinary unprefixed attribute names.
`targetNamespace` never supplies a missing QName default namespace.

`XmlSchemaQName` retains lexical text, optional expanded name, state
(`Expanded`, `Invalid`, `UnboundPrefix`) and an optional explicit declaration
witness. `XmlSchemaNamespaceBinding` identifies the declaration document,
occurrence, prefix, namespace, attribute/value spans and decoded-value digest.
The reserved `xml` namespace can resolve without a source declaration; absence
of a witness must not be described as a fabricated declaration. Scope checks
cover NCNames, reserved bindings, source order/parent containment and expanded
attribute ambiguity without cloning a complete namespace environment per row.

## Form policy and declared observations

The versioned normalization policy uses `Unqualified` when the schema's
`elementFormDefault` or `attributeFormDefault` is absent. A local element or
attribute uses its explicit `form`, otherwise its respective schema default.
If both are absent, the effective local-name normalization is Unqualified.
Global declaration names use the explicitly declared target namespace, if any.

Policy-derived normalization and source declarations remain distinguishable:

| Origin | Retained evidence |
| --- | --- |
| Explicit local `form` | The local component's original attribute/value spans and digest |
| Explicit schema form default, with local `form` absent | The Schema component's original default attribute and local attribute absence |
| Both declarations absent | The receipt's policy profile and original absence; no synthetic attribute is added |

There is no separate public policy-origin enum or synthesized form attribute in
the current model. Consumers must preserve the original declarations and policy
binding when explaining the normalized name. No implicit base, implicit
`anyType`, runtime virtual/intrinsic flags or other undeclared attributes/types
are manufactured. Explicit source attributes remain observations, including
unsupported ones; they do not establish runtime behavior.

## Symbol spaces and reference outcomes

Global reference candidates are indexed by symbol space, expanded namespace and
local name. Only a declaration whose actual component parent is the parentless
`Schema` root of that same document enters the global index. Invalid nested
schema/type/group observations keep their context issues but cannot compete with
genuine root declarations. Anonymous and local declarations never become global
keys.

| Reference kind | Symbol space |
| --- | --- |
| `Type`, `Base`, `ItemType`, `MemberType` | Shared named simple/complex type space |
| `Element`, `SubstitutionGroup` | Global element space |
| `Attribute` | Global attribute space |
| `Group` | Named group space |
| `AttributeGroup` | Named attribute-group space |

`XmlSchemaReference` retains its owning component, attribute/value evidence,
target QName, candidate IDs and outcome: `Unique`, `Missing`, `Conflict`,
`ExternalNamespace`, `InvalidQName` or `UnsupportedContext`. These describe
membership in the explicitly selected source catalog. `Unique` does not prove
legal derivation, substitution ancestry, instance validity or a runtime class.
The current resolver uses `ExternalNamespace` for an unresolved XSD-namespace
target; other absent selected-catalog targets can remain `Missing`.

Include/import/redefine records retain explicit unsupported dependency outcomes
and inert source locations. No locations are fetched, no imported namespace is
invented, and no recursive content or ancestry proof chain is expanded. Missing
selected-catalog membership, especially with unsupported dependencies, cannot
prove platform-wide absence or a clean negative. A later classifier and ancestry
owner remain separate, unimplemented responsibilities of this slice.

## Finite limits and cancellation

All selected schemas share one operation ledger, in addition to the inherited
raw-source owner limits:

| Bound | Maximum |
| --- | --- |
| Explicit selected documents | 64, at least one required |
| Selected path / source snapshot ID length | 4,096 UTF-8 bytes |
| Original bytes per selected member | 1 MiB |
| Aggregate selected original bytes | 16 MiB |
| Aggregate parsed event records | 32,768 |
| Aggregate indexed elements | 32,768 |
| Normalized components | 32,768 |
| Aggregate indexed attributes | 65,536 |
| Combined retained components + references + issues | 65,536 |
| Charged work units | 1,048,576 |
| Cumulative conservative serialized metadata charges | 16 MiB |

The existing parser also limits nesting to 64, attributes per element to 64,
element/attribute name lengths to 256 bytes and encoded attribute values to
16,384 bytes. Its index has existing finite element/attribute/inline-segment
bounds. XML DTDs and processing instructions are refused; custom entities are
disabled. Schema locations are inert. No XML/Lua source is executed.

Parser scratch is allocated under those existing finite native bounds before
postparse retention charging. The metadata ledger is a conservative cumulative
serialized-size measure, not an exact heap-allocation cap. It charges the native
index and its serde-skipped decoded attribute values before retention, plus
normalizer witnesses/copies/indexes/rows and canonical hash/final-receipt inputs.
Repeated representations consume the same ledger; there is no clipped successful
complete result or per-document reset.

Admission rejects any selected index containing native UI script records before
retaining that index across schemas. This guard prevents a mislabeled UI document
from retaining serde-skipped inline payloads as schema metadata; it does not claim
that the parser made no finite scratch allocation before refusal.

Source/schema-binding and root/namespace failures use
`SourceRegistryInvalid` at Inventory; resource refusal uses
`SourceBudgetExceeded`; wrong selected member kind uses `InvalidFileLanguage`.
Native raw-member and XML parser errors retain their existing codes, including
`InvalidInputInventory` for malformed native XML. Cancellation follows the native
checkpoint precedence and returns `SourceReadCancelled`; no completed capability
is returned after cancellation.

Selected raw schemas are not registered as Main or Library files. This owner
does not change configuration, graph/replay recipes, publication or store
identity. Any later graph consumer must charge the borrowed capability under its
own existing aggregate ledger rather than reuse this admission as a budget reset.

## Validation evidence

The single native synthetic lifecycle passes with the physical source directory
removed. It covers exact selected components, namespaces, source spans and
reference candidates; a different source closure with equal XSD bytes; wrong
member kind, cancellation and mislabeled native UI script refusal; root-only
global symbols; and unsupported descendant resolution after declaration conflicts.
The genuine Main snapshot remains separate from the selected XSD members.

That native case ran before reconciliation with the process/audit head.
During active implementation, the current slice uses the affected `wow-project`
Tier S gate from [the execution model](EXECUTION_MODEL.md). Current exact-head
publication and gate evidence are recorded once in
[canonical Issue #106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

Full XSD/proof/classification, graph integration, real Gethe/Ketho corpus,
performance, exhaustion/fault injection and WoW runtime acceptance remain
NotEvaluated. This component prerequisite does not advance full W17/E3/package
or launch acceptance. Gethe materialization follows product implementation/build.
