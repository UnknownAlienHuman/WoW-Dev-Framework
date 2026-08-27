# Project vision and boundaries

**Status: normative**

## Purpose

WoW Dev Framework exists to make World of Warcraft addon engineering exact enough for automated agents and efficient enough for routine development.

The framework should reduce three recurring failures:

1. **Stale platform assumptions** — code is written against a different build, flavor, package layout, API signature, event payload, restriction state, or Secret Value model.
2. **Missing structural context** — an agent sees a matching function but not the TOC/XML load path, owner, registry, lifecycle, state root, template lineage, or protected execution context.
3. **Excessive context and weak evidence** — entire repositories are read while conclusions lack pinned source identity, confidence, and coverage.

The product therefore combines exact versioned reference data, project-aware static analysis, a typed WoW graph, historical lineage, compact source skeletons, deterministic query results, and explicit evidence semantics.

## Primary users

- addon developers working on Retail/Midnight projects;
- coding agents editing Lua, XML, and TOC repositories;
- maintainers migrating addons across patches;
- reviewers investigating load-order, performance, hook, state, Secret Value, or protected-surface failures;
- tooling authors who need a stable WoW reference contract without requiring an editor extension.

## Core user outcomes

A user should be able to:

- select one exact WoW build profile and know which source digest backs it;
- query current and historical APIs without grepping full UI trees;
- distinguish an explicit replacement from a similar-name candidate;
- navigate package, load, object, inheritance, registration, state, and call relationships independently;
- receive generic EmmyLua and WoW diagnostics from one coherent project generation;
- identify when a check was not evaluated because a source partition or restriction facet is unknown;
- obtain an L0/L1 skeleton and source handle before reading a full function or file;
- compute a bounded patch-impact study set for an addon;
- compare community implementations only after verifying the current platform contract;
- complete common tasks with fewer source reads and less model context.

## Product boundaries

### In scope

- immutable Reference Packs generated from pinned Blizzard UI snapshots;
- raw APIDocumentation preservation and Ketho-compatible annotation generation;
- EmmyLua-based syntax, semantic analysis, and diagnostics;
- TOC, XML, Lua, package, event, callback, hook, state, restriction, and lineage facts;
- deterministic exact, migration, shape, FTS, graph, and optional semantic search;
- project and reference SQLite stores;
- CLI, MCP, and LSP access to one service layer;
- generated Project Maps and bounded agent context;
- compatibility probes, fixtures, differential oracles, and agent task evaluations;
- optional Codebase Memory handoff through documented MCP transport.

### Out of scope for v1

- a generic multi-language RAG platform;
- an editor extension as the correctness boundary;
- direct modification of Codebase Memory internals;
- execution or injection inside the WoW client;
- a graph database server or vector database;
- full interprocedural proof of every dynamic Lua behavior;
- repository-specific product modes for ElvUI, oUF, WeakAuras, or any other addon;
- redistribution of third-party addon repositories;
- automatic code fixes based only on fuzzy or semantic similarity;
- pretending that static analysis can determine all runtime Secret or protected-state behavior.

## Design values

### Exactness before convenience

A slower explicit `Unknown` is preferable to a fast fabricated answer. Negative authority requires complete coverage for the relevant partition.

### Small sufficient context

The agent should receive the smallest evidence-bearing representation that can support the next decision: summary, relationship neighborhood, L0/L1 skeleton, then exact source.

### Reuse without surrendering contracts

Upstream EmmyLua, Ketho-compatible annotations, Numy differential output, SQLite, and optional Codebase Memory are reused where their boundaries are understood. Product correctness remains defined by this repository's contracts.

### Versioned, reproducible inputs

A profile means an exact build, source digest, schema, builder, correction set, and capability report. “Current” is not a durable identifier.

### Universal structure

Framework-specific examples calibrate recognizers, but emitted facts use universal roles such as module, registry, style, element, service, state root, lifecycle callback, or extension point.

### Explicit uncertainty

Confidence, provenance, coverage, generation, and competing evidence are first-class output, not explanatory prose added later.

## Success criteria

The project is successful when it measurably improves real addon tasks:

- exact and historical search achieves the roadmap recall targets;
- agents read fewer files and receive fewer bytes per accepted patch;
- first-patch acceptance improves on the evaluation corpus;
- blocking false positives remain within a defined budget;
- stale or incomplete profiles never produce authoritative negative answers;
- Reference Pack and project outputs are deterministic;
- patch impact selects a small, relevant study and test set;
- the workflow remains functional when Codebase Memory is absent;
- an upstream dependency update can be probed, classified, and rolled back without redesigning the system.

## Current priority

The next proof is E0: one deterministic `wow check` result that merges an upstream Emmy diagnostic, a WoW API diagnostic, and a Secret-local diagnostic against one pinned fixture profile. Everything else should expand from that vertical slice.
