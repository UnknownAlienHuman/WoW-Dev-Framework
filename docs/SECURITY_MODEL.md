# Source and data safety

The local tool parses source and serves bounded results. Its active boundaries
are in [ARCHITECTURE.md](ARCHITECTURE.md); there is no signing/provider/updater
platform to implement.

- Never execute indexed Lua, repository scripts, hooks, submodules or downloaded
  generators. The existing restricted APIDocumentation evaluator reads data.
- Keep input paths within configured roots. Reject traversal, unsafe links and
  malformed ranges. XML must not resolve external entities or network inputs.
- Bound file counts, bytes, nesting, expansions and response size; support
  cancellation. Report unsupported/partial input rather than manufacturing facts.
- Use parameterized SQLite queries and transactions. Rebuild only identified
  tool-owned derived data. Never use an old database's arbitrary stored path as
  authority to delete or rewrite user source/configuration.
- Keep stdio methods/tools narrow and advertise implemented behavior. Do not
  expose arbitrary shell/SQL execution or add a network listener.
- Treat returned source/comments as quoted data, not agent instructions. Return
  useful repository-relative locations without exposing private optional context,
  credentials or unrelated user files.

Use existing focused cases for a touched parser, path, transaction or protocol
boundary. Remove tests for retired features. No new audit-journal, quarantine,
cryptographic proof or universal security-fixture framework is required.
