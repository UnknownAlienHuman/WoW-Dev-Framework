# Documentation index

English documentation in this directory is the canonical navigation surface for WoW Dev Framework.

## Document classes

| Class | Meaning |
|---|---|
| **Normative** | Defines behavior, public contracts, invariants, or accepted decisions. Implementation must conform. |
| **Operational** | Defines how contributors, agents, tests, and releases apply the normative contracts. |
| **Research** | Pins external inputs and records verified observations. It is not automatically a product contract. |
| **Candidate** | Contains ideas that require an experiment or ADR before implementation. |
| **Archive** | Retired routing tombstones. Superseded source content is available only through Git history and does not control current routing. |

## Read first

1. [Project vision and boundaries](PROJECT_VISION.md) — normative scope and non-goals.
2. [Architecture](ARCHITECTURE.md) — normative system design.
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md) — normative truth and negative-answer contract.
4. [Architecture decisions](DECISIONS.md) — accepted constraints.
5. [Roadmap](ROADMAP.md) — active implementation gate.

## Component contracts

- [Reference Pack](REFERENCE_PACK.md) — immutable Blizzard reference artifact and builder contract.
- [EmmyLua and diagnostics](EMMYLUA_AND_DIAGNOSTICS.md) — upstream integration, project generations, and rule execution.
- [Graph, search, and planning](GRAPH_SEARCH_AND_PLANNING.md) — typed graph, lineage, skeletons, search, and patch impact.
- [Codebase Memory bridge](CODEBASE_MEMORY_BRIDGE.md) — optional broad-source candidate bridge.
- [Secret Values and restrictions](SECRET_VALUES_AND_RESTRICTIONS.md) — open restriction facets and analysis levels.

## Operating documents

- [Agent workflow](AGENT_WORKFLOW.md)
- [Security model](SECURITY_MODEL.md)
- [Test strategy](TEST_STRATEGY.md)
- [Contributing](../CONTRIBUTING.md)
- [Repository agent instructions](../AGENTS.md)

## Research and planning

- [Research baseline](RESEARCH_BASELINE.md)
- [Candidate ideas](IDEAS.md)
- [Glossary](GLOSSARY.md)

## Archive

- [Retired v8.0 architecture source tombstone](archive/WOW_EMMY_COGNITIVE_PLATFORM_ARCHITECTURE_V8_0_RU.md)

The original v8.0 source was retired from the working tree. Use Git history only for architecture archaeology; the English contracts in this directory are the maintained repository surface.

## Update rules

- A changed public concept must update the glossary and its owning contract.
- A changed architectural invariant must update `DECISIONS.md` through an ADR.
- A changed milestone gate must update `ROADMAP.md` and its tests.
- A new document must be added to this index and linked from the owning document.
- Do not duplicate live WoW patch/security knowledge from the external knowledge base. Link to it and pin only the exact release input required here.
