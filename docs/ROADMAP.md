# Roadmap

**Status: operational**
**Current milestone: E0**

The next milestone is an executable vertical slice, not another architecture rewrite.

## Milestone summary

| Milestone | Outcome | State |
|---|---|---|
| E0 | Merged generic + WoW diagnostics against one deterministic fixture profile | **Current** |
| E1 | Full API reference, raw metadata, annotations, and profile packaging | Planned |
| E2 | Addon project model, TOC/XML/load graph, Project Map | Planned |
| E3 | Blizzard UI graph, package shards, and L0/L1 skeletons | Planned |
| E4 | Exact/historical search, lineage, FTS, ranking explanations | Planned |
| E5 | Universal framework recognizers and managed external corpus | Planned |
| E6 | Optional Codebase Memory MCP bridge | Planned |
| E7 | Production LSP/MCP, installers, packs, releases, and rollback | Planned |

## E0 — executable vertical slice

### Build

- [ ] Create the minimal Rust workspace and transport-independent result types.
- [ ] Pin the checked upstream EmmyLua analysis dependency behind one adapter.
- [ ] Define profile, generation, source-handle, provenance, confidence, coverage, and finding primitives.
- [ ] Add one minimal APIDocumentation fixture.
- [ ] Add one generated Ketho-compatible annotation fixture.
- [ ] Load the annotation fixture as an Emmy library without mutating editor configuration.
- [ ] Normalize one built-in generic Emmy diagnostic.
- [ ] Implement `wow.api.exists` for the fixture profile.
- [ ] Implement one direct `wow.secret.local_operation` rule.
- [ ] Expose a minimal `wow check` CLI path.
- [ ] Add deterministic golden tests and clean negative fixtures.

### Gate

```text
same file receives merged generic + WoW findings
known valid WoW API resolves through annotations/reference facts
unknown current-profile API is detected without grep
one direct Secret-local misuse is detected
all output carries one profile/reference/project generation
dependent rules return NotEvaluated when capabilities are absent
no user editor configuration is mutated
1/2/N repeated runs are byte-identical after canonical sorting
```

### Explicit non-goals

- full Blizzard UI ingestion;
- complete Reference Pack schema;
- LSP or MCP production frontend;
- Codebase Memory integration;
- framework recognizer packs;
- broad interprocedural Secret flow;
- release automation.

## E1 — full API reference and annotations

### Build

- restricted APIDocumentation evaluator;
- all generated API systems, tables, events, widgets, enums, and CVars;
- raw unknown-field preservation;
- restriction facets and predicates;
- Ketho semantic parity and Spartan-style query parity;
- configured historical/current profile packaging;
- annotation and dialect compatibility probes.

### Gate

```text
all declared source files ingested or explicitly diagnosed
negative authority only for complete partitions
annotation pack accepted by the Emmy compatibility probe
raw restriction metadata retained independently of annotations
pack manifest/checksums deterministic and valid
profiles remain isolated
```

## E2 — project model

### Build

- TOC parser and flavor/load-on-demand variants;
- structural XML parser;
- load/dependency/reachability graph;
- API use, event, hook, registration, and state facts;
- direct Secret-local rules over real project files;
- generated Project Map.

### Gate

```text
unreachable and use-before-load fixtures are caught
incremental update changes only affected partitions
Project Map remains within the context budget
zero blocking false positives in the launch project corpus
```

## E3 — Blizzard UI graph and skeletons

### Build

- package-local shards and TOC variants;
- functions, methods, templates, frames, regions, and mixins;
- owner/load/object/call/event/state trees;
- L0/L1 skeleton generation and source handles.

### Gate

```text
historical/current ActionButton lineage fixture resolves
package query touches bounded data
agent reaches a target with no more than three source reads
```

## E4 — search and lineage

### Build

- aliases, deprecations, and replacement journal;
- syntax/signature/neighborhood lineage;
- FTS5 indexes;
- deterministic ranker and explanation signals;
- patch-impact plan generation.

### Gate

```text
explicit replacements outrank fuzzy hits
unknown replacements remain candidates
labeled search benchmark top-3 recall ≥ 0.9
no partial profile produces an authoritative miss
```

## E5 — recognizers and external corpus

### Build

- declarative recognizer DSL over canonical facts;
- core TOC/XML/factory/registry/state/hook recognizers;
- Ace3/oUF/ElvUI/WeakAuras/BigWigs/Details/Plater calibration packs;
- external repository manifests and official GitHub workflow.

### Gate

```text
pack removal changes coverage only
no production code branches on repository name
universal role precision measured per pack
external source license/revision recorded
```

## E6 — Codebase Memory bridge

### Build

- standard MCP client/handoff;
- stable source-handle joins;
- merged local/external search with evidence separation;
- coverage and generation reporting;
- optional DerivedFacts interoperability proposal.

### Gate

```text
bridge absence does not break exact search
semantic candidates are never presented as proven
no direct Codebase Memory database access
measured task benefit over CBM-off baseline
```

## E7 — production packaging

### Build

- thin LSP and MCP frontends;
- installers and configuration bootstrap;
- prebuilt Reference Packs;
- release workflows;
- cargo-deny/audit, SBOM, provenance, signatures where practical;
- migration, rollback, and last-known-good activation.

### Gate

```text
clean install and upgrade paths
public contract compatibility report
least-privilege release automation
corrupt/incompatible pack rejection
rollback tested
security and license artifacts published
```

## Roadmap discipline

- A later milestone may provide fixtures for an earlier one; it may not bypass its gate.
- Architecture changes require an ADR and a concrete failure of the accepted design.
- New components remain experimental until they satisfy the promotion rule in `TEST_STRATEGY.md`.
- The roadmap tracks outcomes and gates, not arbitrary percentages or crate counts.
