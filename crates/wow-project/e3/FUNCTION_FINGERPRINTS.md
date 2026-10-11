# Native comparison components

Issue [#118](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118)
owns implementation and acceptance. The first selected recipe consumes the
genuine retained Main function report and its accepted native direct graph
assertions. It produces comparison inputs, not cross-generation identity or
lineage. An unsupported entity has no generic hashing fallback.

The exact owner key retains the original assertion scope, accepted node ID and
owner key. Separate bindings retain project/analyzer/reference/configuration,
source snapshot/manifest/admission, package and function-report identities.
Those fields do not enter the comparable source-span digest. The direct graph
predecessor remains identified separately from the service's final composed
graph; output does not relabel a predecessor node as a final node.

## Exact source-span recipe

`wow-project/physical-function-source-span/1` hashes exact UTF-8 bytes from the
existing registered Main file and exact physical function span. The component
domain is `wow.project.fingerprint.physical-function.source-span.v1`; its
canonical projection contains only source-content SHA-256 and byte length.
The component retains both its domain-separated digest and the raw content
digest. It does not include path, offsets, labels, function IDs, proposal IDs,
graph IDs, generation, profiles or storage records.

Whitespace, comments, BOM and newline changes inside the span change this exact
content component. Identical bytes at another offset or path have the same
component under this recipe, while their exact owner/support bindings differ.
A whole-file Chunk may supply file content; a closure without an exact supported
byte span reports `missing_exact_span`. The component makes no declaration-shape
or semantic-signature claim.

Unselected `signature` and `declaration_shape` retain the legacy strings
`missing_signature` and `missing_shape`; the selected recipes below supply only
their explicitly reviewed components.
Direct neighborhood is `not_selected`; package/load role stays `not_selected`
unless explicitly selected below. Incoming ownership is eligibility/evidence
support, not a complete evaluated neighborhood. There is
no composite fingerprint while required components are unavailable. Manifest
coverage is Partial even for an empty explicit selection.

## Selected declared-header recipe

The explicit producer recipe `wow-emmy/physical-function-declaration-syntax/1`
collects declarations in the existing Main session and function-call sidecar.
Configuration, report identity and cold/retained/replay consumers bind the recipe;
absence preserves their previous bytes and identities. The selected manifest uses
`wow-project/function-fingerprint-records/declared-header/1` and
`wow-project/physical-function-components/declared-header/1`. Each selected record
adds an owned `declared_header`, retaining the original observation/support
separately from comparison bytes, under component profile
`wow-project/physical-function-declared-header/1`.

Canonical comparison bytes are domain
`wow.project.fingerprint.physical-function.declared-header.v1`, a zero byte,
declaration-form tag (local=0, named=1, closure=2), explicit colon flag and ordered
parameter count as big-endian u64. In original order, each name has tag 0,
big-endian u64 UTF-8 byte length and exact name bytes; vararg has tag 1.
Parameters are never sorted and implicit `self` is never inserted. Declaration
names, paths, spans, content, generation, graph and storage identities are excluded.
No inferred parameter/return types or body equivalence are claimed.

Native unavailable/unsupported reasons remain typed outcomes. A containing native
scope conflict suppresses the digest without asserting a declaration conflict for
a particular function. A Chunk has no declaration. Missing facts never trigger
source parsing or a generic hash. Observations/output use a cumulative metadata
ledger; the older recipe separately checks the final envelope against the same
cap. Cancellation/refusal returns no successful truncated manifest. The exact
source-span component stays separate.
Semantic signature, whole declaration shape and overall coverage remain Partial.

## Typed declaration/body shape

`DeclaredSyntaxAndBodyShapeV1` selects
`wow-emmy/physical-function-declaration-syntax-and-body-shape/1` in the same
registered Main session as the function report. `SourceFunctionFact.shape` is
a separate optional native observation populated only under this producer.
Header-only `wow-emmy/physical-function-declaration-syntax/1` keeps its exact
declared-header recipe; shape does not replace header observation or comparison.
The exact source-span component remains independent.

The shape represents the complete ordered body as `Enter(node)`, `Leave(node)`
and `Token(tag)` events, including nested closures. Explicit finite owned tags
retain nesting, child order, operators and punctuation. Names are anonymous;
literals retain only Number/String/Boolean/Nil categories, not their spellings
or values. Built-in special call node kinds normalize to `Call`.
Whitespace/comments, paths/spans, function/proposal/generation IDs and support
identities do not enter comparable shape bytes. Events are never sorted.
Parameter names remain significant in the separate existing header component;
comments/whitespace remain significant in exact source-span content.

The pinned parser omits Block for a valid empty ordinary closure. Its observed
header and native closing end normalize to an empty balanced Block; missing
closing syntax remains unavailable. No source-text test supplies that result.

Dialect syntax, unknown nodes/tokens and attributes are unsupported. They cannot
supply an Available shape through a skipped descendant, prefix, generic enum
or Debug hash. Native observations/support and exact bindings remain separately
retained; decoded comparison data cannot reconstruct AST authority.

Project `declaration_shape` returns `FunctionDeclarationShapeComponent`:
`Available` carries a typed digest; `MissingShape`, `Unavailable`, `Unsupported`
and `Conflict` remain explicit outcomes without an available shape digest.
The normalization recipe is
`wow-project/physical-function-declaration-body-shape/1`. Digest framing is domain
`wow.project.fingerprint.physical-function.declaration-body-shape.v1` plus a zero
byte; form:u8 (local=0, named=1, closure=2); colon:u8; parameter count:u64 BE;
ordered parameter kinds:u8 (anonymous Name=0, Vararg=1); event count:u64 BE;
then each event kind:u8 (Enter=0, Leave=1, Token=2) and owned tag:u16 BE.
`FunctionShapeNode` explicitly assigns tags 1–36; `FunctionShapeToken` assigns
1–57. Parameter count/order/kinds remain significant, while parameter and body
name spellings are omitted; the separate header keeps exact parameter names.
No upstream enum ordinals, Debug text or serialized AST enter the digest.
`MissingShape` retains the legacy JSON string `"missing_shape"`; absent native
`SourceFunctionFact.shape` is omitted. The combined native manifest uses
schema `wow-project/function-fingerprint-records/declared-syntax-and-body-shape/1`
and profile `wow-project/function-fingerprint/declaration-body-shape/1`.

Traversal, normalization and owned output obey finite selected work/byte limits
and cancellation. Retained representation is charged before owned custody. Only
the combined body-shape recipe also charges the complete final manifest, including
framing, to that same cumulative metadata ledger; older recipes retain their
separate final-output check against the same cap. Resource refusal or cancellation
returns no successful truncated shape.

A shape digest describes only this normalized body structure. It supplies no
semantic type/signature, inferred return, body equivalence, runtime behavior or
lineage claim. Overall function fingerprint coverage stays Partial; Available
header and shape do not produce a complete composite.

Configuration/report and cold/retained/replay consumers bind the exact selected
producer. The old header comparison encoding remains exact. Full component and
corpus acceptance remains open; the named schema/profile and shape outcomes are
not acceptance evidence. See
[platform input](../../../apps/wow/PLATFORM_INPUT.md).

## Selected indexed-type signature

Native `PhysicalFunctionSignatureRecipe::IndexedTypesV1`, wire
`wow-emmy/physical-function-indexed-types/1`, observes the held Main session's
signature index. Local configuration selects `physical_function_signatures`;
configuration/report/replay identities bind that exact recipe. The current
`--fingerprints` query needs no new flag or signature selector.

Comparable parameters preserve ordinal order, exact `backend_nullable` flags and
colon definition; returns preserve the primary row followed by ordered alternative
rows and ordered types. Backend nullable is not parameter-omissibility or source
annotation proof. Resolved empty return rows stay empty, never synthesized Nil
or proof that runtime returns nothing. The vocabulary is Nil/Boolean/Integer/
Number/String/Thread/Userdata/Never, arrays with native length, ordered tuples and
normalized unions. Unions flatten/deduplicate and collapse single members; the
native owner orders members by its fixed typed SHA-256 keys and refuses unequal
members sharing a key. This ordering key is not a signature fingerprint.

Missing indexes/parameter types, unresolved returns, unknown/dynamic/nominal/
callback/generic/literal/object/variadic or unsupported types, generic signatures,
callable overloads, varargs and unmatched parameter documents remain unavailable.
`FunctionSignatureComponent` exposes Available typed digest, Unavailable reasons,
IncompleteSourceHealth or Conflict; absence keeps `"missing_signature"`. Native
observations/support remain separate. Overall coverage is Partial, without full
semantic, runtime, equivalence, parity or acceptance claims.

Project comparison recipe `wow-project/physical-function-indexed-types/1` uses
that domain plus NUL; colon:u8; parameter count:u64 BE; ordered parameters
(nullable:u8, type); return-row count:u64 BE; each ordered row (type count:u64 BE,
ordered types). Owned type tags 1–11 are respectively Nil, Boolean, Integer,
Number, String, Thread, Userdata, Never, Array, Tuple, Union. Array length is tag
0 for unspecified or tag 1 plus i64 BE maximum, then element type; tuples/unions
carry u64 BE counts and ordered children. Names, paths, spans, IDs/generations,
return-resolution and async labels are excluded from comparison bytes. The
native finite work/byte limits, cancellation and cumulative final manifest
metadata charge still apply. Final #118 acceptance is pending.

## Selected physical Lua package/load role

Select this optional query component with
`FunctionFingerprintSelection::with_package_load_roles(FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1)`.
The wire field `package_load_roles` carries
`wow-project/physical-function-package-load-role/1`. This selects comparison
output; native configuration, analyzer report and replay remain unchanged.

The selected manifest schema is
`wow-project/function-fingerprint-records/selected-components/1`. Its explicit
`selected_components` receipt retains optional native `physical_declarations`
and `physical_signatures`, plus optional `package_load_role`, each with its exact
recipe. With signatures and load-role selection both absent, previous manifest
bytes remain; an unselected role retains `not_selected`.

Comparable data is static selected-package root status, known LoadOnDemand state,
package phase/reachability and physical Lua role groups with exact occurrence
multiplicity. Groups retain phase/reachability/bootstrap; multiplicity is neither
function count nor runtime execution count. Exact package/selected-TOC variant,
Main-file, load-unit and plan receipts remain witness data, outside comparison.
There is no runtime-loaded, relative-order or filename/flavor inference.

Incomplete native closure, scope conflicts, unknown LoadOnDemand or missing
physical-file/load-unit joins remain unavailable with explicit coverage. The
existing cumulative metadata ledger and cancellation cover inputs, retained
witnesses/components and final output. Overall fingerprints remain Partial;
final #118 acceptance is pending.

## Selection, support and bounds

The query names strictly increasing exact current native function IDs and finite
positive record, cumulative source-byte and metadata-byte limits. Native
membership and ID length are checked before caller ordering comparisons, with
cancellation checkpoints. Duplicate, unordered or foreign IDs refuse the query.
The query never expands a caller's selected set.

Each record validates the accepted `lua_source_function` and exact incoming
`source_declaration_owns` assertion, source-file endpoint, native source handle,
evidence/context, Main digest and span. Scope coverage and conflicts stay visible.
Serialized row cost is checked before owned retention; complete envelope framing
must fit the same metadata cap. Hashing checks cancellation between chunks.
The native manifest retains the exact finite limits alongside its records, so a
candidate consumer does not reconstruct execution policy from service/store rows.
Refusal/cancellation produces no truncated successful manifest.

Source-span query selection changes the service request/result identity. This read-only observation
does not change source, analyzer, project configuration, graph or native replay
bytes. Existing exact native replay supplies the source-free recomputation input;
serialized comparison output cannot reconstruct native authority. The selected
declaration producer additionally changes semantic configuration/report identity
and binds its named replay/storage consumers in `REPLAY_PUBLICATION.md`.

## Selected XML schema-class component

The `--xml-fingerprints <selection.json>` query uses schema
`wow-service/xml-fingerprint-selection/1`. It requires genuine selected native XML classification and
`direct_capabilities_v1` containing `xml_schema_classification_v1` with its
existing dependencies. Decoded reports, tag names and class names supply no
native classification authority.

Targets name exact scoped `occurrence_proposal_id` values as `occurrence_id`,
with each matching native `class_proposal_id` as optional `class_id`. Targets
must be distinct and strictly increasing by occurrence ID, not node IDs or
unscoped occurrence labels. A classified occurrence requires its exact class ID.
For an unclassified native occurrence, omit `class_id`; canonical request
serialization omits the absent field. Explicit `class_id: null` is refused by
the existing strict input scanner; this query does not widen that decoder.
`max_records` and `max_metadata_bytes` are required positive finite integers.

The finite recipe is
`wow-project/xml-schema-class-component/named-proof-witnesses/1`. Comparable
bytes contain only the reviewed class category, expanded declaration QName and
sorted distinct qualified positive Type/Base/Element/SubstitutionGroup proof
targets, retaining reference kind, named target kind and expanded QName.
Supported target kinds are named simple/complex types and global elements;
missing named type support stays unavailable. This proof-target set does not
describe witness order, complete XSD topology or whole-type shape.

Original observations, accepted occurrence/class/relation support, raw schema
evidence and exact source/profile/catalog/generation bindings stay separate.
Proposal/node IDs, paths/spans, content digests, selection limits and stored rows
do not enter comparable bytes. Unknown, conflict, missing, ambiguous and
unsupported proofs retain explicit unavailable outcomes. There is no second
parser or generic hash fallback. An absent expanded-name namespace is already
omitted by the native `XmlExpandedName` serializer; no comparable serializer
change is introduced.

The selected handoff stays Partial over genuine retained classification, even
with Available components or an explicit empty selection. It never expands to
every occurrence of a class or proves absence, runtime identity, equivalence or
lineage. Cumulative metadata accounting covers input, native indexes,
normalization and retained support/output; final framing fits the same selected
cap. Existing outer guards remain. Refusal/cancellation yields no successful
truncated manifest.

Exact targets/limits bind service request/result identity. This observation
query does not change source, analyzer, project configuration or graph identity,
native replay or the existing classification catalog. It adds no replay/catalog
version and decoding output cannot construct native authority. See the
[CLI query contract](../../../apps/wow/GRAPH_BUILD.md).

Remaining #118 component coverage, whole XML type shape, the genuine
#116 candidate/Skeleton handoff, mutation freeze and selected real-corpus Tier A
remain required by #118. A verified source-span checkpoint does not accept them.
