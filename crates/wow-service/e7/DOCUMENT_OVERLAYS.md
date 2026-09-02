# E7-A document synchronization and immutable overlays

**Status:** normative.

## Purpose

Represent unsaved client document state without mutating the published project, source files, owner stores, or repository.

## Initial synchronization profile

The first LSP profile may support:

```text
open with full text and explicit version
ordered incremental changes with exact version
optional full-text replacement
save observation with or without exact text per negotiated profile
close
```

MCP does not create document overlays in the initial profile unless a later explicit resource-edit/session profile is defined.

## Open

`document_open` validates:

- active session and exact `SessionViewSetId`;
- normalized document URI and workspace authorization;
- language/profile support;
- protocol version presence and initial version policy;
- full content size/encoding/newline constraints;
- owner source mapping or explicit unmapped-overlay policy;
- privacy/license classification;
- no existing open overlay for the same identity unless exact idempotent replay.

It creates the first immutable `DocumentOverlayRecord` and a new complete `DocumentOverlayGeneration` map.

## Change

`document_change` requires:

```text
exact SessionId / SessionViewSetId
exact prior OverlayGenerationId
exact document identity and prior protocol version
a strictly newer document version
ordered change list under the negotiated synchronization profile
position encoding/newline/source-map profile
limits and cancellation
```

Validation occurs against the exact prior full content.

## Incremental edit validation

For each edit:

- range positions decode under the negotiated position encoding;
- start/end fall on valid document boundaries;
- UTF-8/UTF-16 code-unit conversion is exact and overflow-safe;
- ranges are applied in the protocol-defined order;
- no out-of-range line/character;
- invalid surrogate/code-point boundary is rejected;
- size/line/operation limits are enforced after each edit and on final content;
- optional declared range length/checksum is validated where the profile supports it.

The resulting full content is canonicalized only according to the frozen document-content profile and receives a new digest. Source text is never executed.

## Version rules

- Versions are document/session scoped.
- Duplicate exact open/change notification can be recognized only by exact prior/result IDs and payload digest.
- Same version/different payload is conflict.
- Lower or skipped versions follow the exact profile; default is stale/reject rather than guessing.
- A version number alone is not content identity.
- Client restart/new session does not continue a prior overlay chain unless explicit retained-session resumption is implemented later.

## Complete overlay generation

Every accepted mutation creates an immutable complete ordered mapping:

```text
DocumentIdentity -> latest DocumentOverlayId for this session view
```

Normal reads do not replay an unbounded edit history. Edit receipts remain retained for audit/debug according to policy, but the generation identity binds full-content digests.

## Analyzer/project projection

Overlay analysis is supplied through a narrow owner seam:

```text
SessionOverlayAnalysisRequest
    exact base project/analyzer/reference/graph identities
    complete changed-document content/digest set
    unaffected base unit references
    exact analyzer/config/profile and source-map identities
    required capabilities/budgets/cancellation
```

The owner returns an immutable ephemeral result bound to the overlay generation. `wow-service` does not parse/analyze Lua or reconstruct project semantics.

The overlay result cannot be published as a normal ProjectGeneration without a separate owner publication workflow.

## Cross-file effects

A changed document may invalidate other files/facts under the project owner’s dependency graph. E7-A does not assume only the changed file is affected. The owner reports exact affected/reused/partial capabilities.

Unknown invalidation scope widens conservatively or yields `NotEvaluated` according to the owner contract.

## Save observation

`document_save` records:

```text
notification received
optional exact text/digest supplied by client
protocol version/overlay identity
client-provided reason/metadata under bounded schema
```

It does not prove:

- file write occurred;
- disk bytes equal overlay;
- repository commit exists;
- project owner indexed the new bytes;
- current project publication changed.

A separate external materialization/publication event can later produce a new exact project generation and explicit session rebind.

## Close

`document_close` removes the document from the next overlay generation and records closure. It does not write unsaved content. Subsequent requests using the closed overlay ID can complete only if their exact captured generation remains retained.

Closing the final overlay can return to the exact base `SessionViewSet` analysis state without relabeling overlay-generated results.

## Rebind with overlays

Default profile blocks rebind while unsaved overlays exist unless the request explicitly chooses:

```text
RejectRebind
DiscardOverlayWithExplicitReceipt
ForkNewSession
ValidateExactRebase (future, owner-defined)
```

No heuristic patch/rebase/merge.

## Diagnostics invalidation

A new overlay generation invalidates only results whose exact dependency set changed. Result IDs from prior overlay generations remain valid historical references while retained but cannot answer the new generation.

`Unchanged` diagnostics require exact owner proof that the relevant dependency/result manifest is identical.

## Security and limits

Bound:

- open documents;
- document bytes/lines;
- changes per notification and session;
- replacement text bytes;
- position conversion work;
- overlay generations/history retention;
- analyzer/project work and outputs;
- concurrent document mutations.

Reject URI/path traversal, unauthorized workspace mapping, NUL/control abuse, malformed Unicode, decompression/archive inputs, executable directives, source-controlled configuration, and private-data output outside policy.

## Tests

Cover full/open, incremental edits, full replacement, multiple UTF encodings, CRLF/LF, astral code points, stale/duplicate/conflicting versions, overlapping/ordered ranges, oversized edits, cancellation, concurrent mutations, close/reopen, rebind blocking, save nonclaim, cross-file invalidation, exact unchanged proof, hostile source text, privacy, and deterministic overlay digests.
