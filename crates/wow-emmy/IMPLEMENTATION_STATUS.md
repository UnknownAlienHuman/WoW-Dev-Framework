# `wow-emmy` implementation status

## Implemented executable boundaries

- Active root Cargo workspace member with an exact compiled upstream dependency.
- Pinned analyzer identity: `EmmyLuaLs/emmylua-analyzer-rust` commit `aaaca68425d9362876228649b0b8d92f07654daa`, tree `9175c01384e650b9a5bd64da69c36f47dbeaaf67`, `emmylua_code_analysis` `0.25.1`.
- Rust validation of compatibility reports before an imported backend identity is accepted.
- Explicit content-addressed in-memory Lua workspaces bound to one backend identity and source universe.
- UTF-8/NUL, path, extension, file-count, per-file byte, total-byte, duplicate-path, and case-collision guards.
- Exact source-text preservation, case-sensitive lookup, deterministic file ordering, and snapshot identity.
- Direct `emmylua_code_analysis` construction from one immutable `LuaWorkspaceSnapshot`.
- Exact caller-supplied file ingestion without workspace scanning, standard-library loading, source execution, or editor configuration.
- Accepted `syntax-error` and `doc-syntax-error` observations with stable framework category, retained upstream code/severity, logical path, exact content digest, and validated UTF-8 byte half-open span.
- Explicit conversion of the pinned upstream's Unicode-scalar LSP-shaped columns to source bytes, including CRLF and multibyte coverage.
- Canonically ordered, content-addressed syntax reports and exact backend-pin mismatch rejection.
- Linux and Windows policy, format, locked build, strict Clippy, debug/release test, rustdoc, updated-dependency, Wasm, parser-compatibility, and semantic-consumer CI evidence for commit `b2441a2b3be847322551fcfcda1d8d8f5cce7a10`.

## Not yet implemented

- One coherent analyzer session containing distinct Main and Library workspaces.
- Annotation-library health and resolution-dependent capability failure isolation.
- Normalized resolved/unresolved global, member, reference, and call facts.
- Local binding, copy, operation, guard, and control-flow facts.
- Mapping analyzer results into complete `wow-core` evidence, coverage, finding, generation, and result envelopes.
- Session actor lifecycle, atomic update batches, incremental invalidation, snapshot publication, cancellation, and multi-generation cache behavior.
- Full P1-P12 behavioral compatibility report and rollback activation gate across analyzer updates.
- Frozen E0-C fixture identities/checksums and complete E0-C acceptance matrix.

The active syntax operation proves only analyzer-owned syntax observations for exact supplied snapshots. It does not claim WoW API absence, Secret status, runtime behavior, project ownership, or a complete/clean semantic analysis result.

## Next package

Implement the smallest combined Main/Library semantic slice:

1. accept distinct explicit Main and Library snapshots with the same compiled backend identity;
2. reject cross-role duplicate/case-colliding paths and mixed backend identities;
3. load the annotation library separately and expose exact library-health state;
4. prove `C_E0Fixture.KnownApi(...)` resolves to Library evidence;
5. prove `C_E0Fixture.RemovedApi(...)` remains an analyzer-unresolved member/call fact without platform-absence wording;
6. retain exact receiver/member/call spans and content identities;
7. prove deterministic output, broken-library isolation, and no host-path/upstream-type leakage.
