# Native same-file local references

The wow-emmy local references operation resolves the identifier at one UTF-8 byte
offset through the accepted native Emmy semantic model and returns only the
same-file lexical local reference-index spans. It uses
SemanticModel.find_decl with NoTrace, requires the native declaration to be
local and owned by the queried file, then reads that declaration's same-file
reference-index cells.

The dependency remains pinned to the already accepted
aaaca68425d9362876228649b0b8d92f07654daa revision. This feature does not repin
the analyzer and makes no WoW API or client-semantic claim. The native source
references handler is an implementation donor for local-index ranges and
include-declaration behavior; this operation deliberately excludes its global,
alias, module, and cross-file traversal.

The public operation accepts a single Project or Fixture snapshot, byte offset,
include-declaration flag, finite limits, and an atomic stop flag. Its report uses
schema wow-emmy/local-references-result/1 and profile
same_file_lexical_local_references_no_trace/1. It retains snapshot/backend/path/
content identities, query offset, limits, state, optional reason and origin
span, optional declaration span, sorted unique references, and
negative_authority=false. Lua local names and native declaration IDs are never
serialized.

When resolved, references contains native use spans and contains the native
declaration span exactly when include-declaration is true. If the per-file
native index exists but has no entry for this resolved declaration, references
is an observed empty subset (or only the declaration when requested). An empty
references vector has negative_authority=false and does not prove that no use
exists. If the per-file index is unavailable, the report is NotEvaluated and
does not expose declaration_span. The hard reference ceiling is 2048. File,
work, name, or reference budget exhaustion returns a typed Limit error and
discards the whole operation.

Unsupported tokens, unobserved declarations, globals, members, nonlocal
declarations, and unavailable per-file reference indexes remain NotEvaluated.
Malformed source is SourceParseFailed. Invalid byte offsets and cancellation
return typed errors. Native declaration and reference ranges are accepted only
when they are nonempty, in-bounds UTF-8 spans whose source bytes match the
native declaration name.

The report path is the workspace-relative path already admitted by the
snapshot. Service and App consumers apply their negotiated location-privacy and
protocol projection policies; this owner performs no transport conversion.

Focused native tests cover shadowed locals and nested closures, parameters,
local functions, declaration inclusion, unreferenced locals, unsupported
global/member targets, malformed Lua, UTF-8 byte boundaries, cancellation,
reference-limit refusal, deterministic report bytes, and the absence of
negative authority.
