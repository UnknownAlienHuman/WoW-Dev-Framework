# `wow-emmy` implementation status

## Implemented executable boundaries

- Exact pinned `emmylua_code_analysis` adapter at the accepted upstream commit/tree/crate version.
- Rust revalidation of the compatibility report before backend identity is accepted.
- Explicit content-addressed in-memory Lua workspaces bound to one backend identity and source universe.
- UTF-8/NUL, normalized path, extension, file-count, per-file byte, total-byte, duplicate-path, and case-collision guards.
- Exact source preservation, case-sensitive file lookup, and deterministic workspace identity.
- Lua/documentation syntax diagnostics with exact snapshot-relative path, source digest, canonical UTF-8 byte span, stable classification, and deterministic report identity.
- Explicit Main plus Library semantic workspaces for direct static `receiver.member(...)` calls.
- Linked member-reference and call facts with receiver/member/reference/call spans, argument count, colon-call shape, and analyzer-only `resolved`, `unresolved`, or `possible` state.
- Function-local bindings linked to their analyzer declaration identity and, when present, their direct member-call initializer.
- Exact local-use facts for concatenation operands, concatenation operation facts, `canaccessvalue(local)` guard observations, and explicit guard-to-operation `dominates` relations.
- Shadowing-safe analyzer declaration matching; a guard for a different local does not authorize another binding.
- Library parse-health fail-closed behavior, malformed-Main failure isolation, deterministic ordering, canonical fact/report identities, and Library-input-order invariance.
- Linux and Windows workspace tests plus rolling parser/current-source compatibility CI.

## Authority boundary

These outputs are normalized analyzer and control-flow observations only. They do not prove that a WoW API exists or is absent, that a value is Secret, that an operation is safe, or that a client/runtime permits it. `wow-rules` must combine exact project facts with independently authoritative Reference coverage and restriction facets.

## Not yet implemented

- General symbol, definition, reference, type, hover, call-hierarchy, or rename operations.
- Non-call member/reference inventory and dynamic/computed member facts.
- General assignment/copy propagation, interprocedural dataflow, full CFG construction, loops, exceptional control flow, or arbitrary guard semantics.
- Project-generation ownership, persistent analyzer sessions, overlays, concurrent clients, and multi-generation caches.
- Complete mapping into `wow-core` evidence/coverage/result envelopes beyond the exact normalized fact records currently exposed.
- Behavioral compatibility fixtures for the remaining semantic surfaces across analyzer updates.

## Next owner package

`wow-project` E0-D must bind these immutable analyzer reports to one exact first-party file inventory and project generation, reject mixed/stale analyzer state, and publish one atomic read view for `wow-rules`.
