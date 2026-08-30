# `wow-recognizers` implementation contract

**Status:** core recognizers deferred to E2; calibration packs deferred to E5.

## Mission

`wow-recognizers` deterministically matches structural conventions in normalized Lua/TOC/XML facts and emits universal graph roles and relations. It captures reusable architecture without hardcoding product behavior by addon or repository name.

## Owned responsibilities

- recognizer definition/pack schema and versioning;
- validation/compilation of declarative structural patterns;
- matching over normalized `wow-emmy` syntax/semantic facts and provided TOC/XML facts;
- universal role/entity/relation emission;
- evidence, confidence, coverage, and ambiguity reporting;
- core recognizers for factories, registrations, hooks, libraries, state roots, and lifecycle patterns;
- named calibration packs as data;
- recognizer mutation/precision evaluation;
- pack enable/disable and producer-version identity;
- bounded matching execution.

## Explicit non-responsibilities

`wow-recognizers` does not:

- parse Lua with another parser;
- inspect repository name to change semantics;
- define Blizzard API truth;
- run diagnostics or decide severity;
- persist graph data directly;
- execute addon code or configuration;
- use an LLM in the correctness path;
- treat an external implementation as platform authority;
- infer dynamic facts as `Proven` when structure only supports `Possible`.

## Input contract

Recognizers consume normalized facts supplied by owning crates, for example:

```text
call/receiver/member/literal arguments
assignments/table fields/metatable/mixin shapes
function/method/lifecycle declarations
TOC package/load/dependency/SavedVariables facts
XML template/frame/inheritance/script facts
known API/event/registry symbols from the selected profile
source handles and generation context
```

They never reopen source files to parse them independently.

## Output contract

A match emits zero or more proposed facts:

```text
entity kind + stable key ingredients
universal role attributes
relation kind + endpoints
source/evidence handles
producer pack/rule/version
confidence and derivation explanation
coverage partition/status
ambiguity/competing matches
```

`wow-project`/`wow-graph` validate and publish those facts. Recognizers do not mutate shared graph state directly.

## Core recognizer families

```text
TOC package/load/dependency/LOD/SavedVariables
XML template/frame/parent/inherits/scripts
CreateFrame factory/template/parent
CreateFromMixins / Mixin assignment
EventRegistry / CallbackRegistry / AceEvent registration
SetScript / HookScript / hooksecurefunc
LibStub and embedded-library use
addon/module/service factories
plugin/region/style/element registries
literal state roots and paths
slash commands and message buses
flavor/edition partitions
Secret guards and known unsafe sinks
lifecycle initialize/enable/disable patterns
```

Each family has a separate rule ID, required capability set, precision corpus, and negative fixtures.

## Required operations

| Operation | Required behavior |
|---|---|
| `parse_recognizer_pack` | Read a versioned data definition with no executable code. |
| `validate_recognizer_pack` | Check schemas, captures, endpoint kinds, capability requirements, and boundedness. |
| `compile_recognizer_pack` | Produce deterministic matcher state independent of repository identity. |
| `match_partition` | Match one normalized fact partition under explicit budgets and generation. |
| `emit_universal_facts` | Produce proposed entities/relations with derivation/evidence; no direct persistence. |
| `classify_match_confidence` | Assign Derived/Possible according to deterministic structural rules; never name popularity. |
| `report_coverage` | Report skipped unsupported facts/patterns and affected rule capability. |
| `enable_pack` | Activate a pack by explicit configuration/profile compatibility. |
| `disable_pack` | Remove only that producer's generation/coverage; core semantics remain unchanged. |
| `explain_match` | Return captures, rule version, input handles, and derivation. |
| `run_mutation_suite` | Rename repository/path/factory identifiers, add structural negatives, and measure hidden overfitting. |
| `build_precision_report` | Produce per-role true/false/unknown counts against labeled fixtures. |

## Declarative pack rules

1. No arbitrary script, regex replacement code, shell command, or dynamic library.
2. Patterns match normalized fact kinds/fields, not raw source text by default.
3. Literal names are allowed only when they are part of a public structural convention and the rule explains why.
4. Repository/addon name may appear in pack metadata/provenance, never in semantic branch conditions.
5. A named pack removal may reduce coverage but cannot alter core fact meanings.
6. Pattern evaluation is bounded by fact count, capture count, recursion/depth, and output count.
7. Ambiguous matches remain competing `Possible` facts.
8. A rule cannot emit a graph kind/relation not declared by `wow-graph`.
9. Rule version changes replace prior producer partitions; stale duplicates are removed.
10. Generated/user script payloads require explicit quarantine/budget policy.

## E2 core scope

Implement only recognizers needed to build basic project facts:

- TOC/package/load/dependency;
- XML template/frame/inheritance/script ownership;
- CreateFrame/CreateFromMixins;
- event/callback registration;
- SetScript/HookScript/hooksecurefunc;
- LibStub/library embeds;
- SavedVariables roots/literal state paths.

Do not add named framework packs until the core matcher and mutation suite prove repository independence.

## E5 calibration packs

Calibration may use Ace3, oUF, WeakAuras, BigWigs, Details, Plater, and other current selected repositories. Every observation must be pinned and treated as implementation evidence. A pack emits universal roles only.

## Required tests

- schema validation and executable-content rejection;
- positive and structurally similar negative fixture per rule;
- repository/path rename invariance;
- pack removal changes coverage only;
- same structural pattern in more than one repository where practical;
- ambiguous/dynamic target remains `Possible`;
- deterministic output under shuffled fact order;
- capability gap reporting;
- output budget/truncation;
- producer-version partition replacement;
- no second parser or raw-text fallback;
- precision/recall report generation.

## Documentation sources

- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/DECISIONS.md`](../../docs/DECISIONS.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [Current external implementation map](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/external/External_Repositories.md)
- [Current WoW addon development workflow](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_DevWorkflow.md)
- [Current event/callback rules](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_EventPatterns.md)

## Definition of done

Core recognizers are ready when they operate solely on normalized facts, emit explainable universal roles, survive repository/path/name mutations, retain ambiguity honestly, and can be removed or upgraded by producer partition without changing graph semantics.
