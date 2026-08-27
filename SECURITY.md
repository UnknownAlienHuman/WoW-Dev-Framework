# Security Policy

WoW Dev Framework processes untrusted Lua, XML, TOC, SQLite, compressed artifacts, repository metadata, and MCP responses. Source analysis must never imply permission to execute the analyzed project.

## Supported versions

No production version has been released. Security fixes apply to the current default branch until a release policy is defined.

## Reporting a vulnerability

Do not disclose a vulnerability in a public issue. Use GitHub private vulnerability reporting or a private security advisory for this repository when available. Include:

- affected commit or planned component;
- attack preconditions;
- minimal reproduction;
- impact and trust boundary crossed;
- whether untrusted repository content is required;
- suggested containment, if known.

## Security invariants

- Arbitrary Lua is never executed to ingest APIDocumentation or addon projects.
- XML parsing must disable external entities and enforce depth, size, and expansion bounds.
- Archive extraction must reject absolute paths, traversal, device paths, and unsafe symlink targets.
- External repositories are indexed as data; their hooks, installers, build scripts, and local tools are not run by default.
- Reference Packs are checksum-verified and tied to a manifest, source digest, schema version, and builder version.
- SQLite files from outside the trust boundary are not opened as writable project databases without validation or import.
- MCP and LSP clients are untrusted peers. Requests are bounded, validated, and isolated from filesystem paths outside configured roots.
- Codebase Memory is accessed only through documented transport; its database is never mutated directly.
- Secrets, access tokens, private repository URLs, and local paths must not be embedded in generated packs, diagnostics, fixtures, or logs.
- Secret Values in World of Warcraft are a game-runtime restriction model, not an authorization mechanism for this tooling. The two concepts must remain separate.

The detailed threat model is in `docs/SECURITY_MODEL.md`.
