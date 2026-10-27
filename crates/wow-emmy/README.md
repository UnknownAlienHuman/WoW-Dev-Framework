# EmmyLua adapter

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Exact donor and
owner contracts: [EMMYLUA_UPSTREAM.md](../../docs/EMMYLUA_UPSTREAM.md).

`ConfiguredAnalyzerSession` owns exact Main, explicit Library and optional virtual
snapshots in one Lua 5.1 analysis. It initializes embedded STD, applies caller
diagnostic configuration once, and separates diagnostics-only from explicitly
requested semantic collection.

Library snapshots are canonicalized by exact snapshot ID before root assignment
and registration; compatible-owner admission compares the same set. Project
generation is not part of diagnostics-owner construction or reuse and is supplied
only for explicit virtual semantic reports.

Reuse `configured_session.rs`, `session.rs`, `references.rs`, semantic collectors
and coordinate mapping. Do not add another analyzer owner or diagnostic adapter.
Remaining work is Project-owned selected acquisition/source projection, narrow
chunk/XML/binding context, coherent updates and project-aware frontend reuse.
