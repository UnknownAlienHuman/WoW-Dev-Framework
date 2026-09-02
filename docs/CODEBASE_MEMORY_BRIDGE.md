# External Codebase Memory bridge

**Status:** normative integration overview subordinate to the E6-A and E6-B package contracts.

## 1. Purpose

A Codebase Memory provider can contribute broad repository discovery, semantic candidates, generic definition/call candidates, architecture summaries, and trace candidates. WoW Dev Framework remains responsible for exact WoW and project authority:

```text
TOC/XML/load facts
Blizzard API and UI ownership
callbacks/events/hooks/registries
state and restriction facets
profile/generation isolation
lineage/migration/static impact
exact project/reference source identity
context and diagnostic evidence
```

The systems remain separate evidence universes.

## 2. Normative routes

- E6-A provider descriptor/state/query/normalization owner: [`../crates/wow-cbm/e6/README.md`](../crates/wow-cbm/e6/README.md)
- E6-B session/result/mapping/selection/context orchestration: [`../crates/wow-service/e6/README.md`](../crates/wow-service/e6/README.md)
- Project mapping owner: [`../crates/wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](../crates/wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- Reference mapping owner: [`../crates/wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](../crates/wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- Exact context handoff: [`../crates/wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md`](../crates/wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md)

This overview does not override those contracts.

## 3. Authority ceiling

Every accepted provider result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider labels such as `exact`, `verified`, `authoritative`, `complete`, or `high confidence`; top rank; sole result; repeated result; stable external generation; and numeric score do not raise authority. Scores/ranks remain provider-local.

A zero result means only that no accepted candidates were returned for one exact provider/state/query/profile/page under reported coverage. It is not proof that an API, symbol, source entity, implementation, relation, bug, or relevant context is absent.

## 4. Provider and transport boundary

E6-A receives an already-acquired `ExternalCandidateTransportPort` exposing only reviewed typed operations:

```text
provider_status
provider_capabilities
provider_generation
candidate_query
candidate_continue
candidate_explain
```

E6-B resolves one exact provider configuration, obtains a nonsecret credential-use authorization receipt, acquires a narrow session through a host adapter, registers durable operation identity, and invokes E6-A.

Forbidden throughout E6:

```text
generic MCP/tool/RPC calls
raw SQL or provider database access
shell/script/plugin/model-prompt escape hatches
provider install/start/configure/index/import/delete effects
raw credentials/private endpoints/process handles
path/URL following or provider-returned source execution
hidden fallback to another provider/cache/model/web/local search
```

Provider/index lifecycle may be defined later only through a separate explicit host/provider-owner contract. It is not an E6-A or E6-B semantic operation.

## 5. External state

Every query binds one explicit state class:

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Stable state requires a sufficient immutable provider generation/index/corpus receipt. Mutable state binds one observation/session episode. Opaque state is explicitly nonreproducible. Timestamp, uptime, repository name, same query, same top result, same count, or provider `current/latest` prose is not generation identity.

Continuation and cache bind exact descriptor, capability, state, query, profiles, prior result, and cumulative budgets. Cache never converts stale to fresh, opaque to stable, partial to complete, or Candidate to verified.

## 6. Locator and owner mapping

Provider repository/revision/path/URI/symbol/span/digest fields become `UnverifiedProviderLocator`. E6-A does not open or resolve them.

E6-B submits an owner-neutral bounded locator projection to one exact retained project or reference generation. Only the owning crate may return:

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

`ExactMapped` proves only locator-to-owner-record identity under the selected profile. It does not verify provider summaries, relations, traces, ranking, lineage, replacement, migration safety, impact, or runtime behavior.

`NoMappingWithOwnerAuthority` requires complete relevant owner coverage. Service cannot manufacture it from provider zero results or failed matching.

## 7. Explicit selection and context

After mapping, the caller supplies an explicit `Selected`, `Rejected`, or `Deferred` decision for exact candidate/mapping IDs. Service never chooses top, sole, highest-score, nearest, same-name, newest, or first candidates.

Context handoff requires retained `ExactMapped` plus `Selected`. Service reacquires exact project/reference/graph views and invokes one existing `wow-context` operation with the exact mapped root.

Output composition remains separated:

```text
normal exact ContextSemanticPack / rendered artifact
+ separate ExternalCandidateSidecar
```

Provider snippets, summaries, labels, ranks, scores, generic traces, and unverified claims never become exact context or graph facts.

## 8. Durable effects and failure behavior

Provider dispatch, response receipt, result publication, mapping, selection, context publication, retention, and audit are separate effects. Each uses `OperationId + CanonicalRequestDigest` where effecting.

A lost response after possible dispatch/publication becomes `OutcomeUnknown`. Blind retry is forbidden until the exact provider/store/owner effect is reconciled. Provider failure disables only the optional external lane and cannot lower exact local reference/project/graph/search/context/diagnostic capability.

## 9. Storage and license

`wow-store` persists generic immutable objects, catalogs, effect receipts, retention edges, and audit references. It does not interpret provider semantics, map locators, select candidates, or build context. Direct writes to provider storage remain forbidden.

Ability to query, index, map, or inspect external source does not grant redistribution rights. Source/snippet retention requires exact provenance, license, notice, privacy, consumer, and redistribution decisions. Metadata-only output remains possible when source retention is denied.

## 10. Evaluation and enablement

The external lane may ship disabled. Before enabling a provider adapter, measure:

```text
additional top-3 candidate recall
accepted developer-task outcomes
verification cost and extra source reads
false/stale/ambiguous candidate rate
provider-on versus provider-off latency and resource use
partial/zero/outage behavior
credential/privacy/license exposure tests
```

A provider enters the default path only when a frozen evaluation shows unique task benefit. Its absence must never block the exact local product.