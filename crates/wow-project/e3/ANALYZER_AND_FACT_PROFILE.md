# Blizzard UI analyzer and normalized-fact profile

**Status:** normative E3-A `wow-emmy` and recognizer input seam.

## One Lua analyzer

`wow-emmy` remains the only Lua parser/analyzer. E3-A does not add tokenization, AST parsing, symbol resolution, type inference, call analysis, or diagnostic logic.

## Workspace classes

```text
Main
    physical Blizzard UI Lua files selected by source/package/load profile
    XML external Script files
    XML inline virtual Lua units

Library
    exact generated annotation artifact for the selected ReferenceProfile
    explicitly configured analyzer support libraries
```

Platform source never becomes an annotation library merely because it is vendor code.

## Analyzer plan

```text
BlizzardUiAnalyzerPlan
    exact source/project generation
    analyzer implementation/pin/config/profile
    exact Main unit manifest
    exact Library unit/artifact manifest
    source-coordinate and virtual-unit profiles
    incremental update set
    capability/budget/cancellation policy
    expected snapshot context
    canonical digest
```

## Snapshot validation

The returned `AnalyzerSnapshot` must match:

- exact project/source generation;
- every physical and virtual unit ID/digest/class;
- library artifact/profile;
- analyzer pin/config/profile;
- source coordinate profile;
- fact/diagnostic schema version;
- update/coverage status;
- no omitted or extra unit.

Cross-generation or partial hidden snapshots are rejected.

## Fact partitions

Facts are partitioned at the smallest stable owner scope supported by analyzer/project contracts, for example:

```text
file or virtual unit declarations
symbols/signatures/types
literal/global/table/member access
calls and possible calls
function/source-span relationships
event/hook/library/state-relevant normalized facts
analyzer diagnostics/findings as analyzer-owned records
```

E3-A adapters preserve original fact IDs, evidence/source handles, confidence, coverage, ambiguity, and generation.

## Adapter constraints

- no raw-source fallback;
- no regex/name/path heuristic replacing missing facts;
- every unsupported/lossy mapping emits an adapter-loss/coverage record;
- no mutation of analyzer facts;
- no diagnostic suppression to obtain a clean index;
- no conversion of unresolved/dynamic data into exact literals;
- no repository/package-name condition;
- no API, Secret, taint, protected, runtime, or performance conclusion.

## Global and environment behavior

The analyzer profile must explicitly model the selected WoW Lua environment/annotation library. It cannot read editor settings, installed extensions, global user configuration, or a floating annotation directory.

Unknown/generated/runtime-created globals remain unresolved or use explicit analyzer semantics. E3-A does not inject silent `any` globals to improve coverage.

## Incremental analyzer update

Changes to a unit, virtual-source mapping, load/library classification, analyzer pin/config, or annotation artifact invalidate the exact affected analyzer partitions or conservatively widen when impact is unknown.

Reuse requires identical:

- unit bytes/digest/source coordinate identity;
- analyzer pin/config/profile;
- library artifact and reference profile;
- dependency/fact schema inputs;
- coverage/conflict state.

Removed units leave no target analyzer fact, diagnostic, recognizer input, graph proposal, source handle, or skeleton input.

## Determinism

Equivalent unit/library/profile sets produce identical normalized snapshot/fact/diagnostic manifests under 1/2/N workers and shuffled update order. Scheduler completion order is nonsemantic.
