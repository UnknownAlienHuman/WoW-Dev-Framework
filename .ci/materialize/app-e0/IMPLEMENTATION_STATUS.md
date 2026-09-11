# `wow` implementation status

## Implemented

- `operations`: canonical inventory of registered service operations.
- `rules-evaluate`: bounded strict-JSON input, one immutable service snapshot, one `rules.evaluate@1` execution, and one canonical response.
- Stable exit codes for clean completion, diagnostics, invalid input, and operation rejection.
- stdin or explicit-file input without directory discovery or implicit project selection.
- Deterministic output for identical input bytes after semantic normalization.

## Deliberately outside E0

- Persistent project/reference construction, source discovery, daemon mode, LSP, MCP, editor integration, installation, update, and rollback.
- Live-client, combat, taint, secure-execution, or replacement conclusions.

The next milestone replaces the closed rule-input handoff with persistent Reference/project assembly while preserving the same service operation and response envelope.
