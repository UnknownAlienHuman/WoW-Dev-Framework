# Security model

**Status: normative design**

This document covers threats to the tooling itself. World of Warcraft Secret Values, protected actions, forbidden objects, and secure execution are domain restrictions described separately in `SECRET_VALUES_AND_RESTRICTIONS.md`.

## 1. Trust boundaries

The framework may ingest or communicate with:

- Blizzard UI snapshots from an acquisition provider;
- downloaded Reference Packs;
- local addon repositories;
- external third-party repositories;
- TOC, XML, Lua, JSON, compressed, and SQLite artifacts;
- editor/LSP/MCP/CLI clients;
- an optional Codebase Memory MCP process;
- structured runtime probe records;
- release registries and update metadata.

Only repository-owned configuration, verified Reference Pack manifests, and explicitly configured roots are trusted by default. Source content is untrusted data even when its repository is reputable.

## 2. Primary threats

### Code execution through ingestion

Malicious Lua, repository hooks, build scripts, installers, macros, or generated configuration may attempt to execute during indexing.

**Control:** indexing is parser-based. Arbitrary Lua and repository-local tools are not executed. The APIDocumentation evaluator implements a small allow-listed declarative subset.

### Path traversal and filesystem escape

Archives, symlinks, TOC paths, XML includes, source maps, or client requests may reference paths outside configured roots.

**Control:** normalize paths, resolve roots before access, reject absolute/device/traversal paths, validate symlink targets, and keep source-handle resolution root-scoped.

### Parser and decompression denial of service

Deep XML, huge tables, cyclic includes, compressed bombs, pathological source, or enormous graph queries may exhaust memory/CPU.

**Control:** enforce byte, depth, node, expansion, recursion, time, and output budgets; stream where practical; make truncation and partial coverage explicit.

### Malicious SQLite or pack artifacts

An attacker may supply a crafted database, mismatched manifest, stale signature, or altered compressed object.

**Control:** verify checksums and schema before activation; open released reference stores read-only; import untrusted data into owned schemas rather than mutating it in place; run integrity and migration checks; retain last-known-good.

### Dependency and update compromise

An upstream crate, mirror, oracle, or release channel may change unexpectedly.

**Control:** pin dependencies and source revisions, record digests, generate SBOM/provenance, run compatibility probes, require review for correction sets, and support rollback.

### Evidence spoofing

A source or bridge may claim a stronger authority, profile, or generation than it has.

**Control:** evidence classes are assigned by the owning adapter, not accepted from arbitrary text. Stable handles resolve against registered repositories/generations. Candidate systems cannot set `Proven` directly.

### MCP/LSP request abuse

Clients may request unbounded graph traversals, arbitrary filesystem paths, command execution, or secret leakage.

**Control:** expose a narrow typed service API, validate all parameters, enforce roots and budgets, avoid generic shell tools, redact credentials/paths in diagnostics, and log security-relevant denials without source contents.

### Output or prompt injection

Analyzed source comments, documentation, or external repository text may contain instructions intended for an agent.

**Control:** source content is returned as quoted evidence with provenance, never as repository operating instructions. Agent policy comes from trusted repository files and service contracts. Skeleton generation separates structure from arbitrary prose.

## 3. Reference builder controls

- Never call the WoW Lua runtime or a general Lua interpreter on untrusted source.
- Allow-list registration calls and expression forms.
- Bound table size, nesting, expression steps, and file count.
- Preserve unknown constructs as diagnostics rather than evaluating them.
- Disable XML external entities and network access.
- Record every acquisition provider, input digest, correction, and builder dependency.
- Build in an isolated workspace with no write access outside output roots.
- Compare logical results across providers where available.

## 4. Project/external index controls

- Do not run Git hooks, package managers, tests, generators, or repository scripts automatically.
- Treat submodules and symlinks as explicit external roots requiring policy.
- Ignore or quarantine files outside declared source size/type limits.
- Do not read SavedVariables, logs, or installed addons unless the user explicitly adds that universe.
- Keep first-party, dependency, external example, and runtime universes separate.
- Record license/provenance before retaining external fixtures.

## 5. Database controls

- Use prepared statements and typed query builders.
- Keep reference stores immutable/read-only.
- Use transactions for project generation publication.
- Do not expose raw SQL through MCP/LSP.
- Version schemas and test forward/backward migration behavior.
- Validate JSON payload sizes and schemas before storage.
- Garbage-collect content-addressed objects only when no retained generation references them.

## 6. Bridge controls

- Codebase Memory is accessed only through documented MCP transport.
- The bridge receives a constrained command/configuration; it does not discover arbitrary executables from source repositories.
- No direct database access or mutation.
- Timeouts, cancellation, response size limits, and generation identity are mandatory.
- External candidates remain candidate evidence after transport.

## 7. Runtime evidence controls

Runtime probe imports may contain character names, account paths, chat, combat logs, or other personal data.

- Define minimal structured probe schemas.
- Redact or reject unnecessary personal fields.
- Store raw payloads only when explicitly requested and access-controlled.
- Tie records to build/scenario/probe version and content digest.
- Never treat an unsigned free-form report as a platform-source fact.

## 8. Release controls

Planned E7 controls:

- reproducible or attestable builds;
- checksummed binaries and Reference Packs;
- SBOM and dependency policy;
- least-privilege CI permissions;
- protected release environments;
- no secrets in fork/PR workflows;
- signed release metadata where practical;
- rollback and last-known-good activation;
- security advisory process.

## 9. Security test corpus

Maintain fixtures for:

- path traversal and unsafe symlinks;
- archive bombs and duplicate paths;
- XML entity expansion and excessive nesting;
- huge/cyclic APIDocumentation tables;
- unsupported Lua constructs;
- malformed UTF-8 and line-ending edge cases;
- oversized MCP requests/responses;
- crafted source handles and profile spoofing;
- malicious SQLite metadata;
- source comments containing prompt-injection instructions;
- external candidate attempting to assert proven authority.
