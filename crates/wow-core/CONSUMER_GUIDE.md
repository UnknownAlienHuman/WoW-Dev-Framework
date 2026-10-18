# `wow-core` consumer guide

Use `wow-core` only for values shared by more than one current owner.

- Construct the specific ID, digest, source, evidence, finding, coverage, budget or error type you need.
- Validate untrusted input at the boundary where it enters.
- Return the caller's own typed result; do not wrap it in a generic core envelope.
- Keep parser, graph, search, storage, transport and WoW semantics in their owning crate.
- Do not preserve removed schemas or compatibility readers. Rebuild derived data from current source/configuration.
- Add a shared type only after finding two real callers that require the same invariant.

Canonical JSON and typed digest helpers remain available for IDs that actually use them. They are not a requirement for every operation or result.
