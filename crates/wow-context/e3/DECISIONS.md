# E3-B context decisions

**Status:** normative.

## CTX-001 — Context binds exact immutable views

Every operation starts from one `ContextUniverseSet` containing exact project, graph, platform-source, and reference generation identities. No input view changes during the operation.

## CTX-002 — User project, Blizzard UI source, and Reference Pack remain separate

A context pack may contain items from multiple universes, but identities never merge. Cross-universe relations are explicit graph/reference records.

## CTX-003 — Project Map is a projection, not another graph

The map contains selected typed nodes, edges, groups, and facets linked to graph assertions. It does not own semantic entities, add edges, or replace graph queries.

## CTX-004 — L0 and L1 have distinct purposes

L0 describes bounded container/navigation structure. L1 describes exact entities and local neighborhoods. Source bodies are separate excerpt items.

## CTX-005 — Semantic artifacts precede rendering

`ProjectMap`, skeletons, and `ContextSemanticPack` are structured canonical records. JSON and Markdown are separately identified deterministic renderings.

## CTX-006 — No model inference in the canonical path

No LLM, embedding, natural-language classifier, generated summary, or model ranking determines facts, roots, priorities, omissions, or token counts.

## CTX-007 — Exact roots only in E3-B

Root candidates must already be resolved to exact IDs. Search, fuzzy, and natural-language resolution are later service/search responsibilities.

## CTX-008 — Intent and expansion policies are reviewed declarative profiles

Profiles use a closed non-Turing-complete schema. Source projects and callers cannot provide executable selectors, callbacks, SQL, scripts, regex programs, or model prompts.

## CTX-009 — Every canonical claim has origin closure

A fact, relation, path, or summary facet includes exact input fact/assertion/reference/source IDs, evidence, confidence, provenance, coverage, conflicts, and derivation template/rule where applicable.

## CTX-010 — Rendering cannot create facts

Text emitted by a renderer is a lossless or explicitly loss-declared presentation of semantic items. Free-form generated prose is noncanonical and outside E3-B.

## CTX-011 — Mandatory metadata cannot be pruned

Universe identity, roots, boundaries, claim origins, coverage, conflicts, omissions, budget accounting, and truncation state are mandatory. If they exceed the minimum budget, generation fails.

## CTX-012 — Optional pruning is deterministic and item-level

Priority tiers, tie-breakers, dependencies, and budget costs are profile-defined. No partial structured item or hidden truncation.

## CTX-013 — Bytes are exact; tokens require a pinned profile

Canonical byte counts are exact. Exact token counts require a specific tokenizer implementation, version, vocabulary, and configuration digest. Otherwise only an explicit deterministic estimate, upper bound, or unavailable state is reported.

## CTX-014 — Semantic and rendered budgets are separate

A semantic pack may fit while one renderer does not. Rendering revalidates exact output bytes/tokens and fails or explicitly replans; it never silently cuts output.

## CTX-015 — Progressive expansion must add new evidence or required structure

The no-new-evidence stop condition is based on unseen semantic/evidence IDs, not merely fewer bytes or repeated text.

## CTX-016 — Paths do not become direct relations

Reason paths remain paths. Context rendering cannot flatten reachability into a persisted or stated direct edge.

## CTX-017 — Source excerpts are typed untrusted data

Source text retains exact source handle, digest, range, encoding, privacy, and license identity and is structurally separated from framework metadata and instructions.

## CTX-018 — Instruction-like source remains data

Instruction-looking comments or strings are not removed or followed. Privacy redaction is policy-driven, byte-accounted, and recorded separately.

## CTX-019 — Context coverage is independent

Input/source/graph/reference coverage, selection coverage, budget coverage, excerpt coverage, and rendering coverage are distinct. A valid pack may be explicitly partial.

## CTX-020 — Omission is a first-class record

Every candidate omitted because of profile, confidence, privacy, budget, unsupported capability, conflict, duplication, cancellation, or deferred universe has an omission reason and affected scope.

## CTX-021 — Context cache identity is exact and storage-free

`wow-context` defines cache keys and validation only. Physical cache storage belongs to a higher layer. Cache keys bind exact generations, request, profiles, schemas, and renderer/tokenizer identities.

## CTX-022 — No cross-generation cache relabel

A prior semantic pack or skeleton may not be returned as the target request because content appears similar. Reused subartifacts retain original identity and receive a new validated binding.

## CTX-023 — Ordinary operations are bounded

No whole-source, whole-graph, or unlimited context export. Every map, skeleton, expansion, excerpt, path, and rendering has explicit system and request maxima.

## CTX-024 — Privacy and consumer trust are explicit

Unknown privacy or redistribution state cannot default to external source inclusion. Consumer trust class and source-use policy are exact request/profile inputs.

## CTX-025 — Context does not diagnose or plan edits

It can include existing finding evidence when explicitly rooted, but it does not decide severity, remediation, edit plans, or task completion.

## CTX-026 — Determinism is logical and byte-level for canonical renderers

Equivalent exact inputs and profiles yield the same semantic IDs, canonical JSON bytes, deterministic Markdown bytes, omissions, and budget reports independent of workers, storage layout, cache history, or query completion order.

## CTX-027 — Combined maps reference, not collapse

A combined context map keeps separate project maps and explicit cross-universe links. It never rewrites both universes into one namespace or parent hierarchy.

## CTX-028 — Source slices are fetched last under policy

Selection first resolves structured facts and relations. Exact source excerpts are acquired only for selected items and explicit goals.

## CTX-029 — Consumer format does not alter semantic selection by default

The semantic pack is consumer-neutral. Renderer-specific replanning requires an explicit distinct request/profile and produces a distinct semantic pack ID.

## CTX-030 — No patch-sensitive constants in context algorithms

Current API/build/Secret/event behavior is read from exact Reference/project artifacts and current KB-routed evidence, never hard-coded into map, skeleton, or selection logic.

## CTX-031 — Renumbering creates aliases, not duplicate implementations

Historical `E3-A`, `ContextInputSnapshot`, `ContextBundleCore`, and bundle-operation names map to current E3-B terminology through `MILESTONE_RENUMBERING.md`. Agents must not implement both versions.

## CTX-032 — E3-A platform source is an exact prerequisite

Blizzard UI context can be included only through an exact published `wow-project/e3-a/blizzard-ui-source-index` view and its `SkeletonInputView`. `wow-context` never acquires or indexes that source itself.

## CTX-033 — Existing specialized context documents remain normative

Inherited E3 context documents remain active specialized contracts when interpreted through the current router, renumbering map, and machine manifest. A current machine-contract conflict is blocking and must be resolved before code.

## CTX-034 — Control/effect projections remain closed views

The inherited control/effect model consumes published typed facts only. It cannot become a second parser, CFG, SSA, data-flow engine, or runtime simulator.

## CTX-035 — Metrics and evaluation are later DAG layers

Metrics and evaluation can assess an exact semantic pack and rendering but cannot influence the semantic pack identity or canonical selection in the correctness path.
