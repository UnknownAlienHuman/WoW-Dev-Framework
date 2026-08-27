# Public schemas

This directory will contain versioned machine-readable contracts consumed outside a single crate.

Expected schema families:

- Reference Pack manifest and capability report;
- source handles and generation context;
- evidence, confidence, coverage, and findings;
- entities, relations, lineage, and restriction facets;
- search, skeleton, plan, and patch-impact results;
- external repository manifests;
- structured runtime probe records;
- optional DerivedFacts interoperability packs.

Rules:

1. No schema becomes public merely because an internal Rust struct exists.
2. Every schema has an explicit version and compatibility policy.
3. Unknown fields are preserved or rejected according to the owning contract; they are never silently discarded.
4. Breaking changes require a version bump, migration fixtures, and a compatibility report.
5. JSON Schema is expected for interchange contracts; SQLite migrations remain owned by `wow-store`.
6. E0 should define only the minimal result/evidence primitives required by the vertical slice.
