# E2-B recognizer decisions

**Status:** normative.

## RECOG-001 — Normalized facts are the only correctness input

Recognizers never reopen Lua, XML, TOC, generated docs, or repository files.

## RECOG-002 — The matcher is declarative and non-Turing-complete

Core rules use a closed schema of typed fact selectors, joins, bounded predicates, existence clauses, captures, and output declarations.

## RECOG-003 — Repository identity is never a semantic condition

Repository, addon, branch, owner, path, or popularity may appear in corpus provenance only.

## RECOG-004 — Core and calibration packs are separate trust classes

E2-B activates repository-owned universal core rules. Named framework/addon calibration packs remain E5 and cannot redefine core graph meanings.

## RECOG-005 — The recognizer input envelope breaks the project cycle

`wow-project` adapts TOC/XML/project facts into a recognizer-owned `RecognizerFactBundle`; recognizers do not depend on project.

## RECOG-006 — Lua facts retain `wow-emmy` identity

The adapter references exact analyzer fact IDs and does not copy them into a second parser-derived model.

## RECOG-007 — Outputs are proposals, not published graph truth

Recognizers emit `ProposedEntityAssertion` and `ProposedRelationAssertion`; `wow-graph` validates semantic keys, registry compatibility, evidence closure, conflicts, and publication.

## RECOG-008 — Recognizer output confidence is `Derived` or `Possible`

Direct syntax facts may be Proven inputs, but a recognizer-produced role/relation is a deterministic derivation. Ambiguous/dynamic structure is Possible.

## RECOG-009 — No candidate output in the E2 core pack

Semantic/name-similarity candidates belong to later discovery/calibration lanes.

## RECOG-010 — Exact convention literals require review

Names such as `CreateFrame`, `RegisterEvent`, `EventRegistry:RegisterCallback`, `hooksecurefunc`, or `LibStub` are allowed only as profile-bound resolved-symbol/literal constraints with documented semantics and mutation fixtures.

## RECOG-011 — Native and custom signal systems stay distinct

`Frame:RegisterEvent`, EventRegistry frame-event bridges, custom `RegisterCallback`/`TriggerEvent`, and CVar callbacks emit different roles/relations.

## RECOG-012 — Custom callbacks require a producer

Plain `EventRegistry:RegisterCallback` becomes a confirmed custom signal subscription only when an exact compatible `TriggerEvent` producer is found in the bounded analyzed scope. Otherwise the match is unresolved/Possible and cannot be treated as a native event.

## RECOG-013 — Hook facts do not claim safety

`SetScript`, `HookScript`, and `hooksecurefunc` recognizers record hook kind/target/handler structure only. Protected, taint, forbidden, managed-object, combat, and Secret legality remain rule/runtime responsibilities.

## RECOG-014 — Negative clauses require complete coverage

A `not_exists` condition is evaluated only over a closed declared scope with Complete capability coverage; otherwise the rule is NotEvaluated or produces no negative inference.

## RECOG-015 — All competing matches are retained

No first-match, best-name, last-write, or majority-vote resolution.

## RECOG-016 — Match identity is independent of execution order

Rule, pack, input generation/partition, captures, and exact input fact IDs determine stable match identity.

## RECOG-017 — Rule version owns a replaceable producer partition

A rule/pack version update produces a new partition identity; project/graph orchestration atomically replaces old assertions.

## RECOG-018 — Graph registries constrain outputs

A pack can emit only declared graph kinds, relations, attributes, directions, endpoint kinds, and confidence classes.

## RECOG-019 — Paths are not parsed or guessed

Normalized project-relative paths can be evidence/attributes, but path fragments cannot drive semantic role classification except a reviewed format contract explicitly owned by the source fact schema.

## RECOG-020 — SavedVariables roots come from TOC truth

A Lua global name alone does not become a persistent state root. The root must be declared by a normalized SavedVariables TOC fact.

## RECOG-021 — Dynamic state paths are conservative

Literal field/index chains can produce exact state paths. Dynamic keys produce root-level or Possible path facts with exact ambiguity.

## RECOG-022 — No framework lifecycle heuristics in E2 core

`OnInitialize`, `OnEnable`, module factories, plugin/style/element registries, and message buses require calibration packs unless a universal project fact explicitly declares the role.

## RECOG-023 — Evaluation is part of the product contract

Every rule ships with labeled positives, near-miss negatives, partial/ambiguous cases, mutation invariance, and producer-replacement tests.

## RECOG-024 — Precision reports do not hide unknowns

Unknown, NotEvaluated, truncated, and unlabeled results remain separate from true/false counts.

## RECOG-025 — No LLM in the correctness path

Models may later help propose experimental packs or labels, but only reviewed declarative rules and exact fixtures can enter active packs.

## RECOG-026 — No runtime pack loading by default

E2 core packs are repository-owned, versioned, and compiled from frozen artifacts. User/external packs require an explicit later audit/activation policy.

## RECOG-027 — Rule removal reduces coverage only

Disabling a pack/rule removes its producer assertions and reports coverage loss; it cannot mutate other producer facts or redefine graph semantics.

## RECOG-028 — Truncation never publishes complete output

Budget-limited matching returns explicit partial/truncated coverage and cannot authorize authoritative absence or a complete producer partition.
