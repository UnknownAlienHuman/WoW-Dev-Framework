# Useful source context

A result should say where it came from: the selected repository/revision,
flavor, file and range. Keep the document version for an unsaved-buffer result.
Use those existing fields directly; no signed receipt or evidence taxonomy is
required. [Architecture](ARCHITECTURE.md) owns this boundary.

Distinguish three ordinary situations: a supported query with results or a valid
empty result; incomplete/unsupported input with a warning; an operation that
failed. Preserve enough detail to investigate the failure. Do not silently turn
missing analysis into an exact negative, but do not return protocol errors for
an ordinary no-match LSP/search request either.

Use actual Blizzard source for current API facts. Ketho supplies annotation
behavior, and community code supplies examples. Similar names or fuzzy search
hits are suggestions to inspect, not proven API replacements. Static structure
does not establish runtime execution, combat/taint safety or loaded state.
Runtime-specific claims need actual observation; routine source search does not.
