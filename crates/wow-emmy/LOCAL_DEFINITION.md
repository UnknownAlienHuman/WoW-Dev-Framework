# Exact lexical local-definition observation

`navigation::local_definition` accepts one exact in-memory Main
`LuaWorkspaceSnapshot`, an original UTF-8 byte offset, finite
`LocalDefinitionLimits` and a stop flag. It returns the actual accepted native
semantic model's lexical declaration, normalized into framework-owned types.
It performs no source acquisition or editor/protocol work.

The implementation reuses the existing registered analyzer builder. It obtains
the native token and calls `SemanticModel.find_decl` with `NoTrace`, then admits
only an actual local/explicit-parameter declaration from the same supplied file.
Local function bindings are local declarations. No annotation source-location,
member/type/global, initializer/call-origin or cross-file fallback is followed.
At a shared token boundary the left identifier is selected, matching the
accepted donor's explicit cursor convention; this is not a nearest-name search.

The input must be one Project/Fixture Main file. Libraries, source generations
and dependencies are not fabricated. Parse errors produce `SourceParseFailed`;
unsupported/nonlocal/unobserved targets produce `NotEvaluated` with a reason.
They never produce a clean-negative or a fallback location. A target is returned
only as `ResolvedLocal`, with exact original origin/declaration byte spans and
the native local/parameter kind. The target name is bounded metadata; original
source bodies and absolute paths are excluded. `negative_authority` is always
false, including on a successful local observation.

The report preserves the exact backend/workspace/path/content/query, limits,
capability profile, coverage and state. Its analysis ID hashes canonical report
data with `analysis_id` excluded; it contains no wall clock, native handle or
transport position. Positions are converted only by Project against its leased
immutable overlay. The owner does not mint a saved Project generation or claim
WoW API, runtime, load, replacement or cross-file binding authority.

Limits bound file bytes, charged work and emitted name bytes. Cancellation is
checked before/after native registration and lookup and before return; one
synchronous upstream call is not preempted. This endpoint uses the already
accepted compiled dependency from Cargo.toml/lockfile. A new upstream revision
still requires its own compatibility qualification; this capability is not a
rolling/current-donor acceptance claim.

The actual Service consumer is the opt-in same-document LSP definition path in
[`apps/wow/LSP.md`](../../apps/wow/LSP.md). Default host-v1 remains syntax-only.
Focused native tests exercise shadowing, parameters, local functions, original
multibyte/CRLF spans, malformed/nonlocal input, invalid boundaries, cancellation
and finite budgets. General definition/reference/type/document-symbol services
and complete E0/E7-A acceptance remain open.
