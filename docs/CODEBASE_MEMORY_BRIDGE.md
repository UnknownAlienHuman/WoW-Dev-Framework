# Codebase Memory bridge

**Status: normative optional integration design**

## 1. Purpose

Codebase Memory is useful for broad repository indexing, BM25/semantic discovery, generic definitions/calls, architecture summaries, traces, and cross-repository candidates.

WoW Dev Framework remains responsible for exact WoW contracts:

- TOC/XML/load facts;
- Blizzard API and UI ownership;
- callbacks, events, hooks, and registries;
- state paths;
- restriction and Secret facets;
- profile isolation;
- historical lineage and patch impact.

The bridge combines the two without pretending they are one evidence system.

## 2. Hard boundaries

Prohibited:

- direct writes to Codebase Memory SQLite or other internal storage;
- generated fake Lua files whose only purpose is to trick its parser;
- patching a vendored Codebase Memory language specification during installation;
- treating generic Lua `CALLS` edges as exact WoW call proof;
- making exact search unavailable when Codebase Memory is absent;
- upgrading semantic similarity to a proven API replacement.

## 3. Join contract

```text
Codebase Memory result
    repository + revision + path + symbol/span
→ StableSourceHandle
→ resolve against project/external source registry
→ attach exact WoW graph facts when available
→ merged result with separate evidence classes
```

A candidate that cannot be resolved to a stable handle remains an external semantic candidate and cannot participate in exact impact or autofix logic.

## 4. Planned client boundary

```rust
trait CbmBridge {
    async fn ensure_index(
        &self,
        repo: &RepositoryIdentity,
    ) -> Result<CbmGeneration>;

    async fn semantic_candidates(
        &self,
        query: &str,
        scope: &RepoScope,
    ) -> Result<Vec<CbmCandidate>>;

    async fn trace_candidates(
        &self,
        seed: &StableSourceHandle,
    ) -> Result<Vec<CbmRelation>>;

    async fn coverage(
        &self,
        scope: &RepoScope,
    ) -> Result<CbmCoverage>;
}
```

The implementation communicates through standard MCP transport with a user-configured command. It does not install, own, or update the external server.

## 5. Evidence separation

Merged output labels:

- exact local/reference facts as `Proven`, `Derived`, or `Possible` with their source provenance;
- Codebase Memory results as `semantic_candidate + Candidate`;
- generic Codebase Memory call/trace relations as candidate relations unless independently verified;
- unavailable or partial Codebase Memory coverage separately from local coverage.

A higher semantic score does not alter the evidence level.

## 6. Failure behavior

When the bridge is unconfigured, unavailable, stale, or times out:

- exact reference/project lookup still works;
- search reports that the semantic lane was not used;
- no local capability is marked failed;
- cached external candidates remain tied to their old external generation and are not presented as current;
- the request may return a structured optional-integration warning, not a product error.

## 7. External repository workflow

External source is cloned/indexed on demand and recorded by manifest:

```text
repository URL
commit SHA
license/SPDX when detected
retrieved_at
external index generation
source digest
recognized universal roles
```

Source is not redistributed in release artifacts. License is recorded before copying any material into fixtures or implementation.

## 8. Potential upstream interoperability

A future `DerivedFactsPack` proposal may define:

```text
provider and schema version
repository/revision identity
nodes with path/span/stable key
edges with relation/confidence/evidence
custom relation namespace
replace-by-generation semantics
```

If Codebase Memory adopts a supported import ABI, the sidecar graph may optionally be projected into it. This repository's correctness and data ownership do not depend on that outcome.

## 9. Evaluation

The bridge is promoted only when an agent task evaluation demonstrates benefit over local exact/FTS/graph search. Measure:

- additional top-3 recall;
- accepted task outcomes;
- extra source reads and bytes;
- candidate verification cost;
- false or stale candidate rate;
- CBM-on versus CBM-off latency.
