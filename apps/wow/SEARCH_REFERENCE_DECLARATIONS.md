# Native Reference source declarations

Set `"native_documents": "source_declarations_v1"` in the existing exact
source-reference input to index original named Structure, CallbackType,
Enumeration, Constants and ScriptObject type declarations alongside callables
and events. The existing `search reference` build/query/page/retire lifecycle
and `search index gc` commands apply. Omitting the selector and selecting
`callables_and_events_v1` preserve their admitted profiles and bytes.

The same normalization pass captures a sealed `NativeDeclarationSignature` and
the Partial partition `reference.native.apidoc.declaration`. Keys have the form
`declaration:<kind>:<original Name>`, with kinds `structure`, `callback`,
`enumeration`, `constants` and `script_object`. Search copies only that owner
port. Original Name is the identifier; definition namespace is metadata and
does not create qualification, namespace membership or aliases. ScriptObject
methods and receiver/runtime behavior are outside this type-declaration view.

Each selected document has a `declaration` member carrying kind/name, optional
definition namespace, ordered fields, callback arguments/returns, enum/constant
values and original source ranges. Callable vectors on the outer document are
empty and do not describe declaration shape. Values retain boolean/string data,
exact numeric lexemes (including hex and wide integers), or an explicit
`unresolved_digest` of the raw source value. Expressions are never evaluated or
assigned inferred types. Full raw records remain with the Reference owner.

Identity, exact name, prefix, literal FTS and bounded identifier similarity use
the original name and name the actual owner port in explanations. Structured
selectors are closed and kind-specific:

```json
{"shape":{"declaration":{"kind":"structure","field_count":2,"fields":[
  {"position":1,"original_type":"table","nilability":"allows_nil"}
]}}}
```

Callback selectors support `argument_count`, `return_count`, `arguments` and
`returns`; each field filter uses zero-based position, original type and optional
nilability. Enumeration and Constants selectors support `member_count` and
`member_name`. ScriptObject supports the kind selector alone. Incompatible
kind/field combinations and mixing declaration, callable or event shape fields
reject. An older corpus reports declaration shape as unavailable. These shape
matches and text/similarity signals remain Candidate.

The Search profile/manifest are
`wow-search/native-reference-source-declarations-profile/1` and
`wow-search/native-reference-source-declarations-shard/1`; the native projection
is `wow-reference/native-source-declarations-view/1`. The existing document
handle carries the exact selection. It cannot be changed into another component.
The query profiles bind the new name/shape interpretation. No physical Store
schema or signature-cache reconstruction is introduced.

All selected records count toward the whole-corpus bound. Conflicts retain every
candidate in Reference and never select a Search winner. Mixed coverage stays
Partial, with no authoritative absence. The existing atomic ReferenceView/Shard
publication and independent three-root result holds support source-absent
queries/continuation, ordinary retirement, last-hold GC and historical replay.

Donor evidence for this operation is Ketho `luasrc/annotate/init.lua` at
`d0b5b51fac4c52c493371b9b18e66ce604ea4326`, blob
`1f0b902a809472c30b28a0e220b70235b29235b8`, freshly resolved and read as data.
Its Structure/CallbackType dispatch and original `apiTable.Name` informed this
projection. Parsing, annotation/type lowering and scalar resolution were not
changed. This is bounded functional Search evidence; full E4, current-WoW,
Ketho/Tier X, named-client runtime and launch acceptance remain separate in #169.
