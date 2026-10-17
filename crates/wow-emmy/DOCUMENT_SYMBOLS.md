# Native document outline

`document_symbols::document_symbols` consumes one exact physical Project or
fixture `LuaWorkspaceSnapshot`, explicit limits and a caller stop flag. It uses
the accepted compiled native analysis setup, declaration/type/signature indexes
and typed Emmy parser AST. It acquires no source and returns no upstream AST,
URI, LSP object or owner handle.

The algorithm is adapted from the pinned EmmyLua LS
`handlers/document_symbol/{mod,builder,stats,expr,comment}.rs`. The exact donor
revision is the compiled `EMMYLUA_REVISION`, recorded with the selected Issue190
evidence; it is not a moving-source freshness claim. The adapted source retains
the MIT attribution and full [`DOCUMENT_SYMBOLS_DONOR_LICENSE`](DOCUMENT_SYMBOLS_DONOR_LICENSE).

The outline includes the donor's local/global functions, native declarations
from assignments, static table fields, parameters and assigned closures,
numeric/generic loops, branches, do scopes, expression traversal and region
comments. Kinds and details are native outline observations. Intentional
non-declaration assignment and non-static-field omissions follow the named
outline recipe; this is not complete Lua semantic, dependency, WoW or runtime
authority. No second parser or source-text recognition heuristic is used.

The normalized report carries exact backend, input snapshot, logical file,
content digest, limits, state, work and canonical digest. Records use UTF-8 byte
half-open `SourceSpan`s and explicit backward `parent_index` values in native
preorder. The synthetic file holder is omitted. A parent range may expand to
the envelope of its actual AST binding/value children; the original selection
range remains exact. Empty/blank display titles use a static placeholder while
retaining `original_name`; display names are not symbol identity or edit authority.

`Ready` requires a complete qualified traversal. A genuinely empty outline
retains that report; missing prerequisites and parse failure remain
`NotEvaluated` or `SourceParseFailed`. Failed traversal never publishes its
partially collected records. Cancellation and budget failures reject the whole
operation. The strict Core digest omits absent optional parents and rejects null
identity material rather than weakening canonicalization.

Limits separately bound input bytes, charged work/node visits, symbol count,
AST depth, outline depth, names, details and report bytes. Own loops and ancestor
walks check cancellation. Captured metadata and transient detail copies are
charged conservatively; these counters are not measured process heap. Native
registration/analysis calls remain synchronous, with checks before and after;
there is no preemption or hard runtime deadline guarantee.

Service is the concrete consumer. It holds the Project overlay, checks the
returned identities and asks Project to project the exact byte spans into the
negotiated encoding. Source-derived labels/details require the separate explicit
host disclosure policy. The native report itself authorizes no disclosure,
workspace registration, persistence, edit, lineage or runtime conclusion.
