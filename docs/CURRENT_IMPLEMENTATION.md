# Current implementation state

This file tracks executable state rather than planned architecture.

## Implemented

- deterministic identity, evidence, coverage, generation, finding, and source primitives in `wow-core`;
- immutable `wow-reference` generations and compare-and-swap channel pointers;
- update-oriented Blizzard UI source acquisition with local checkout preference and GitHub fallback;
- source version checking with `auto`, `prompt`, and `never` policies;
- exact per-operation Blizzard source manifests;
- safe normalization of generated Blizzard API documentation into a self-digested reference draft;
- generic optional advisory context disabled and unconfigured by default;
- one canonical WoW-development skill with agent-specific adapters.

## Current package

Generated Blizzard API documentation is converted without executing Lua. Every consumed file is bound to the source manifest, exact Git revision, Git object ID, SHA-256, and source span. Completeness and negative authority remain separate from successful parsing of any individual file.

## Next packages

1. Map the normalized generated API draft into `wow-reference` entity, evidence, coverage, and conflict records.
2. Add Blizzard implementation, XML, and TOC producers as separate evidence lanes.
3. Implement `wow-emmy` against a current compatible upstream API using compatibility probes rather than a permanent revision lock.
4. Compose owners through the remaining E0 project, rules, and service boundaries.

No moving WoW branch, build, Interface value, dependency patch, or toolchain patch is permanent project truth. Exact values belong only to an inspected generation or validation record.
