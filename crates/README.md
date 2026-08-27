# Production crates

This directory will contain Rust libraries with independently testable production responsibilities.

Planned responsibility map:

```text
wow-core          IDs, profiles, generations, evidence, findings, handles
wow-store         SQLite schemas, migrations, content-addressed objects
wow-reference     APIDocumentation lowering and Reference Pack read/build
wow-annotations   Ketho-compatible and WoW dialect projections
wow-emmy          upstream Emmy adapter, project actor, diagnostic registry
wow-project       Lua/TOC/XML workspace and incremental generations
wow-graph         entities, relations, lineage, bounded graph queries
wow-recognizers   declarative universal recognizers and calibration packs
wow-search        exact/migration/shape/FTS/graph ranking
wow-rules         API/load/event/Secret/overlay/project diagnostics
wow-cbm           optional Codebase Memory MCP bridge
wow-context       skeletons, Project Map, and context budgets
wow-service       transport-independent use cases
```

This list is architectural decomposition, not a requirement to create every crate immediately. E0 should start with the fewest boundaries that prove the vertical slice. A crate is split only when the responsibility is reusable, independently testable, or has a materially different dependency/security boundary.

Application binaries belong in `apps/`; development-only test/evaluation helpers may live under `tests/` or a future dedicated test crate.
