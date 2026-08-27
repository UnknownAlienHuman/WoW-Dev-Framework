# Test strategy

**Status: operational design**

Testing is organized around correctness contracts and agent task outcomes. A large test count is not a substitute for proving that the intended path executed.

## 1. Test layers

### Unit tests

Use for canonicalization, IDs, schema lowering, evidence/coverage state machines, path normalization, ranking signals, guard dominance, and deterministic serialization.

### Fixture tests

Small pinned inputs exercise APIDocumentation, TOC, XML, Lua, annotations, restrictions, graph relations, lineage, and diagnostics. Fixtures should be independently understandable and licensed for repository use.

### Golden tests

Golden outputs cover normalized diagnostics, manifests, graph shards, skeletons, search explanations, and patch-impact plans. Output ordering and volatile fields must be canonicalized.

### Differential tests

Compare logical output with Ketho, Numy, LuaLS, historical packs, or another acquisition provider. A mismatch is reported and classified; a test does not silently select one output as truth.

### Integration tests

Exercise a complete use case through the service layer and at least one transport:

```text
source/profile input
→ build/load pack
→ index project
→ query/check
→ evidence-bearing output
```

### Compatibility probes

Pin and test upstream EmmyLua behavior, annotation semantics, database migrations, Reference Pack schema versions, and public CLI/MCP/LSP contracts.

### Agent task evaluation

Real addon tasks measure whether the framework reduces source reads/context and improves accepted changes. This is distinct from conventional code coverage.

### Runtime verification

In-client probes validate dynamic behavior that static analysis cannot settle. Runtime records are external evidence and are never fabricated by automated tests.

## 2. E0 golden path

The first executable test must prove:

```text
pinned Emmy dependency loads
one generated WoW annotation fixture resolves a valid API
one APIDocumentation fixture feeds the exact reference view
one generic Emmy error is emitted
one unknown API error is emitted
one direct Secret-local misuse is emitted
all findings share one profile/project generation
output is byte-identical across repeated sorted runs
```

The fixture also includes clean variants to prevent rules that always fire.

## 3. Negative and mutation fixtures

A regression test should be capable of failing when the implementation is broken. Use mutations such as:

- remove a TOC edge and expect reachability to change;
- change a profile and ensure facts do not leak across it;
- remove an explicit replacement and ensure fuzzy candidates do not become proven;
- introduce an unknown restriction facet and expect `NotEvaluated`;
- alter a correction source digest and expect expiration;
- rename a repository while preserving structure and ensure recognizers still work;
- break a guard dominance relation and expect a Secret finding;
- corrupt a checksum and expect pack rejection.

## 4. Determinism

Repeated `1`, `2`, and `N` worker runs must produce equivalent canonical output. Tests should control:

- filesystem traversal order;
- hash map/set iteration;
- parallel scheduling;
- timestamps and temp paths;
- compression metadata;
- SQLite row ordering;
- source-handle normalization;
- diagnostic and search tie-breaking.

Any intentionally nondeterministic metadata is isolated from the canonical digest.

## 5. Coverage-state tests

For each major rule/query, test:

- complete input and positive result;
- complete input and authoritative negative result;
- partial input and non-authoritative miss;
- failed partition and `NotEvaluated`;
- conflicting evidence;
- stale external generation;
- unsupported profile.

The test must inspect structured status fields, not only message text.

## 6. Profile isolation

Create matrix fixtures for configured profiles. Verify:

- one project generation selects exactly one profile;
- same-named APIs with changed signatures remain separate;
- restriction facets do not leak;
- PTR/beta facts are advisory only when not selected;
- historical comparison is an explicit lineage query, not diagnostic blending;
- caches include profile and generation identity.

## 7. Search and lineage evaluation

Maintain labeled tasks for:

- exact current API;
- alias/deprecation;
- explicit replacement;
- moved package/file;
- unchanged name with changed restriction contract;
- same-name unrelated symbols;
- unknown replacement with plausible candidates;
- natural-language implementation task.

Measure top-1/top-3 recall, explanation correctness, candidate verification cost, and false proven replacements. The E4 target is top-3 recall of at least `0.9` on the labeled WoW task set.

## 8. Recognizer evaluation

Each recognizer pack requires:

- positive examples from more than one repository where practical;
- structurally similar negative examples;
- repository rename/path mutation;
- pack removal test showing coverage loss only;
- precision/recall report for emitted universal roles;
- no production branch on repository identity.

## 9. Secret/restriction tests

Cover always/contextual/ordinary values, guards, aliases, copies, conversions, logging, serialization, table keys, branches, direct call arguments, unknown facets, forbidden objects, protected actions, combat reachability, and runtime-required cases.

Do not encode a permanent spell whitelist as the test oracle.

## 10. Security tests

Use the corpus in `SECURITY_MODEL.md`. Run parsers and artifact loaders with strict resource limits. Security regression tests must not execute malicious fixture content.

## 11. Performance tests

Benchmark representative sizes and update patterns:

- cold Reference Pack load;
- cold project index;
- one-file incremental update;
- package-local UI query;
- exact/migration/FTS/graph search;
- L0/L1 skeleton generation;
- patch-impact query;
- Codebase Memory on/off comparison;
- database size and resident memory.

Report corpus revision, hardware/runtime, warmup, sample count, percentiles, and raw results. No performance claim is accepted from a single anecdotal run.

## 12. Agent task corpus

Maintain at least 30 real tasks across:

```text
find current API
replace removed API
locate Blizzard package/function
find safe attachment surface
trace event to UI update
identify state owner
assess patch impact
compare community implementations
repair Secret misuse
explain load failure
```

Metrics include files read, bytes/tokens delivered, first-patch acceptance, false blocking rate, explicit unknown handling, index latency, database size, and CBM-on/off benefit.

## 13. Promotion rule

A component or rule enters the default path only when it has either:

- a unique correctness responsibility demonstrated by fixtures; or
- a measured benefit on the agent task corpus.

Experimental output remains opt-in or shadow until promotion criteria are met.
