# Crate dependency guidance

The active Cargo workspace and each crate's `Cargo.toml` define actual
production dependencies. This document gives responsibility guidance; it is
not a maximum-edge manifest or a requirement that every workflow pass through
Service. Product scope is in [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md).

Keep dependencies pointed toward stable, cohesive owners. Core types belong in
`wow-core`; SQLite details belong in `wow-store`; Emmy parsing and analysis
belong in `wow-emmy`; source/TOC/XML interpretation belongs in `wow-project`;
Ketho source normalization and annotation emission belong in
`wow-reference`/`wow-annotations`. Search, graph, recognizer, rule, and context
code should depend only on the owners whose data or algorithms they actually
use.

`wow-service` may compose existing owner APIs for a multi-owner use case.
A CLI, LSP, or MCP route may call one owner directly when it owns the complete
operation. Do not add service forwarding solely to conform to a historical
service-only application rule. Conversely, do not duplicate an owner's
algorithm in an application or a second crate.

Before adding an edge, check whether an existing public API can support the
real caller. Keep the graph acyclic: lower owners do not depend on Service or
applications, and libraries do not depend on `apps/wow`. Avoid new crates,
generic provider/plugin frameworks, and abstraction layers without a concrete
second use case. Ordinary shared DTOs are appropriate when real callers need
them.

Use actual source dependencies and Cargo metadata when documenting or changing
an edge; do not infer implementation from a plan, README, or former manifest.
