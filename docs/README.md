# Documentation index

English documentation is the canonical navigation surface.

```text
documentation frontier: E7-A complete
next documentation package: E7-B reproducible packaging, distribution, updates and support
implementation frontier: not started
```

## Read first

1. [Project vision](PROJECT_VISION.md)
2. [Architecture](ARCHITECTURE.md)
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md)
4. [Architecture decisions](DECISIONS.md)
5. [Roadmap](ROADMAP.md)
6. [Launch gates](LAUNCH_GATES.md)
7. [Machine manifest](../crates/MANIFEST.json)
8. [Dependency graph](../crates/DEPENDENCY_GRAPH.md)
9. [Workstreams](../crates/WORKSTREAMS.md)

## Active routes

- [E4-A exact-generation search](../crates/wow-search/e4/README.md)
- [E4-B lineage/migration/static impact](../crates/wow-graph/e4/README.md)
- [E4-C service/CLI](../crates/wow-service/e4/README.md)
- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B review/holdout/submission](../crates/wow-service/e5/README.md)
- [E5-C core-pack publication lifecycle](../crates/wow-service/e5c/README.md)
- [E6-A external semantic-candidate bridge](../crates/wow-cbm/e6/README.md)
- [E6-B external-candidate orchestration/mapping/context](../crates/wow-service/e6/README.md)
- [E7-A frontend session/operation registry](../crates/wow-service/e7/README.md)
- [E7-A `wow` daemon/LSP/MCP host](../apps/wow/e7/README.md)

## E7-A transport model

```text
implemented owner/service capability
-> immutable reviewed FrontendOperationRegistry
-> explicit session + workspace + exact overlay/generation
-> CLI | local daemon | LSP 3.18 | MCP 2025-11-25
-> one service operation
-> bounded progress/cancellation/reconnect/backpressure
-> exact final service result and delivery journal
```

A protocol method/tool/resource, client/editor/model identity, progress event, transport success or response delivery cannot create semantic proof or effect authorization.

The initial MCP profile is fixed and read-only, uses stdio by default, and omits prompts, sampling, elicitation, tasks, arbitrary roots and generic tools. Optional Streamable HTTP is loopback-only and disabled by default. The local daemon is current-user IPC; no default remote listener exists.

## Next package

E7-B must freeze the public release lifecycle:

- supported OS/architecture and client/profile matrix;
- pinned Rust toolchain/dependencies and reproducible build profile;
- exact binary/data/package layout;
- release manifests, SHA-256, signatures, SBOM and provenance attestations;
- portable/install package, install/uninstall/upgrade/rollback and data locations;
- Reference Pack/core-pack/provider-adapter compatibility manifests;
- stable/beta/nightly or other explicitly selected channels and update integrity;
- release rollback, revocation, retirement and incident response;
- privacy, telemetry, logs/crash data and support policy;
- release candidate acceptance over real addon repositories and supported clients;
- CI/release automation only after real executable commands exist and the owner explicitly enables it.

After E7-B documentation, the documentation frontier closes and the next work package becomes E0-A Rust implementation.

Patch-sensitive WoW facts remain in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb); stable contracts link rather than duplicate them.