# E7-A workspace and document lifecycle

**Status:** normative. The initial document model is nonmutating and generation-bound.

## Workspace roots

A workspace root is accepted only through an explicit initialization/bind request and an exact URI/root policy.

```text
ProtocolWorkspaceBinding
    normalized workspace URI
    exact configured root identity
    resolved project publication/generation
    profile/reference/source binding
    permissions/privacy/license profile
```

The protocol adapter does not scan cwd/home/editor/Git/WoW installations or arbitrary filesystem roots.

## URI policy

The profile freezes:

- allowed URI schemes;
- URI normalization and case behavior per platform;
- percent encoding and Unicode normalization;
- authority/host policy;
- workspace containment checks;
- virtual/untitled document policy;
- symlink/reparse/device/UNC/ADS behavior;
- maximum URI bytes/components.

A URI is an identifier, not permission to open an arbitrary resource. Provider/source URLs are not document URIs by default.

## Document states

```text
Absent
Open
Changing
OpenSynchronized
Closing
Closed
Desynchronized
Failed
```

Every open document has an exact overlay generation, version, content digest, language ID, position encoding, and workspace owner.

## Open

`protocol_document_open` requires:

```text
active exact session binding
normalized document URI
language ID under profile
initial document version
full text bytes/string under encoding profile
content length/digest
privacy/license classification
resource limits
```

Validation:

- URI belongs to exactly one allowed workspace/virtual namespace;
- document not already open unless exact idempotent replay profile applies;
- language/profile supported;
- text encoding and size valid;
- no NUL/control/normalization ambiguity beyond profile;
- version valid;
- content digest canonical.

Open creates a new overlay and `SessionBindingGeneration` for future requests. It does not write disk or editor state.

## Position encoding

The exact negotiated position encoding is session capability state. All ranges/positions and optional range lengths are interpreted under that encoding and validated against the expected prior text.

No hard-coded UTF-16/UTF-8 assumption. A position from another encoding/profile is invalid.

Validation includes:

- line and character bounds;
- code-unit/code-point boundaries;
- no split inside invalid sequence/surrogate representation under profile;
- start <= end;
- optional range-length consistency;
- maximum range/change counts;
- deterministic newline model.

## Change

A change request binds:

```text
exact active session binding
exact document URI and current overlay ID
expected prior version and content digest
strictly permitted new version
ordered change list
position encoding/sync profile
```

Supported sync classes are profile-selected:

```text
FullText
IncrementalRanges
```

The initial implementation can enable one or both only after frozen conformance vectors. No implicit conversion between them.

## Incremental application

For an ordered change list:

1. validate the request against the exact prior overlay;
2. apply changes in protocol-defined order to the evolving intermediate text;
3. validate every range against the text state to which it applies;
4. bound total intermediate/final bytes and operations;
5. compute exact final text digest;
6. create one immutable target overlay and binding generation;
7. publish target binding for future requests only.

If the exact profile defines all ranges relative to one original version instead, that behavior must be separately pinned; no guess.

## Version rules

The profile defines valid version type/range and monotonicity. By default:

- a new change version must be greater than the current version;
- equal version is accepted only as exact idempotent replay with identical request digest;
- lower/out-of-order version is rejected;
- missing version is unsupported unless the exact protocol profile permits and defines synchronization consequences.

A version is transport synchronization metadata, not project generation identity.

## Desynchronization

Enter `Desynchronized` when:

- expected version/digest mismatches;
- change range invalid;
- change loss/order ambiguity occurs;
- required prior overlay is unavailable;
- transport drops a lifecycle message in a way that cannot be reconciled.

While desynchronized, bound document requests are rejected or return explicit `NotEvaluated`. Recovery requires an explicit full-text resynchronization operation under the profile. The server never reads disk and guesses.

## Save

`protocol_document_save` records exact notification/request metadata and optionally supplied full text/digest under the profile.

Save does not by itself prove:

- disk write occurred;
- disk bytes equal the overlay;
- repository/source publication changed;
- project generation advanced;
- analysis reindex completed.

A profile can trigger an explicit owner refresh/rebind after independently materializing/verifying source. That creates a new binding with exact receipts.

## Close

Close requires the exact current document overlay/version guard where the profile supplies it. It:

- stops future request admission against that overlay;
- creates a new binding without the overlay;
- returns to the exact underlying retained project/source state when available;
- does not write the overlay to disk;
- retains old overlay while in-flight requests/results/reconciliation reference it;
- records close receipt.

Duplicate/unknown close is classified explicitly, not silently accepted as success unless exact idempotent replay.

## Document-to-project composition

The service passes a typed immutable overlay manifest to the existing project owner seam. `wow-project` remains sole owner of Lua/XML/TOC analysis and project generation semantics.

```text
base exact ProjectSnapshot/source publication
+ exact ordered document overlays
+ analyzer/reference/profile config
-> explicit overlay project candidate/generation binding
```

Protocol code never parses Lua/XML/TOC or creates findings/graph facts.

## TOC/XML and non-Lua documents

Document language/kind profiles determine whether an overlay is supported. XML/TOC changes remain bounded nonexecuting inputs to the project owner. Unknown language/kind yields unsupported/`NotEvaluated`; no generic text parser fallback.

## Privacy and source boundaries

- Document text is private session source by default unless the exact project/privacy profile states otherwise.
- Text is not logged/transcribed by default; transcripts store digest/length and permitted bounded metadata.
- Source text cannot create protocol messages, methods, tools, capabilities, settings, or instructions.
- Diagnostics/context output follows exact source disclosure policy.
- Client content never becomes a provider credential or tool permission.

## Resource limits

Bound:

```text
workspace roots
open documents
URI/language ID bytes
document bytes/lines
change count/range count/inserted bytes
intermediate/final bytes
version values
position conversions
binding generations and retained overlays
analysis requests per document
```

Exceeding limits rejects the transition before activating a partial overlay.

## Tests

- open/full change/incremental change/save/close;
- exact UTF-8/UTF-16/UTF-32 or selected encodings under frozen profiles;
- multibyte, combining, newline, invalid boundary cases;
- duplicate/out-of-order/lost versions;
- overlapping changes under defined order;
- stale expected digest;
- full resynchronization after desync;
- request in flight while document changes/closes;
- save falsely treated as disk publication;
- URI root escape/symlink/reparse/device cases;
- oversized document/change bomb;
- source text containing protocol framing/instructions;
- no disk/editor mutation.
