# E2-B normalized fact input model

**Status:** normative consumer boundary over `wow-emmy` and future `wow-project` TOC/XML facts.

## Principle

Recognizers consume typed facts whose source, generation, coverage, and producer identity are already validated. They do not inspect source text or infer missing parser facts.

## Common requirements

Every fact supplies:

```text
source fact ID and kind
exact GenerationContext
universe/scope/partition
producer ID/version
source handle and byte/span identity when applicable
evidence and confidence
capability/coverage records
stable structured references to related facts/entities
```

Facts with stale file digests, mixed project/reference generations, invalid source handles, or unsupported schema versions are rejected before matching.

## Lua adapter facts

These adapt exact `wow-emmy` facts; source IDs remain visible.

### `LuaReferenceFact`

```text
reference kind
spelling and receiver/member spelling
resolution status
resolved symbol key when exact
containing function/file/scope
```

An unresolved symbol is not platform absence.

### `LuaCallFact`

```text
callee reference fact ID
call kind
receiver/value identity when known
ordered argument fact/value/literal references
result binding/value key when known
containing function/control-flow region
```

### `LuaAssignmentFact`

```text
assignment kind = local | field | index | table_field
source value/expression/call refs
target binding/table/key refs
ordered source span
```

### `LuaTableFieldFact`

```text
table/value identity
literal or resolved key
value/call/function refs
field declaration/update kind
```

### `LuaFunctionFact`

```text
function/symbol identity
owner/receiver identity when resolved
parameters
source scope
method/function/chunk kind
```

### `LuaOperationFact`

References exact operation/control-flow facts when a rule needs ordered or local structural relationships. It does not carry a legality verdict.

### `LuaControlFlowFact`

Only relations already proven/represented by `wow-emmy`, such as dominance or source order. The matcher never constructs its own CFG.

## TOC/project adapter facts

Produced by future `wow-project` E2 parsing contracts.

### `TocPackageFact`

```text
addon/package identity
TOC variant/flavor/interface metadata
manifest source handle
ordered file/dependency/directive facts
```

### `TocFileFact`

```text
package/variant
normalized project-relative path
file kind/order/conditions
bootstrap marker when explicitly parsed
```

### `TocDependencyFact`

```text
source package
target package name/resolved identity
required | optional
semantic ordinal/source handle
resolution state
```

### `TocLoadOnDemandFact`

Exact normalized directive/value. It does not imply when a particular frame exists.

### `TocSavedVariableFact`

```text
package/variant
variable name
account/character scope
semantic ordinal/source handle
```

A declared variable can seed a `state_root`; a Lua global alone cannot.

## XML adapter facts

Produced by future `wow-project` streaming XML parser.

### `XmlTemplateFact`

```text
template identity/name
element/object kind
virtual flag when explicit
inherits references
source handle
```

### `XmlObjectFact`

```text
object identity/type/name
parent object/template references
inherits/template references
ownership/file/package
source handle
```

### `XmlInheritanceFact`

Exact child/parent template or object reference with resolution state and ordinal.

### `XmlScriptFact`

```text
object/template identity
script kind/name
handler function/reference/source span
inherit/append/prepend semantics when explicit
```

No XML text or embedded Lua is executed.

## Project ownership fact

```text
ProjectOwnershipFact
    owner entity identity
    owned file/function/object/template identity
    ownership kind and exact producer
```

This can anchor universal roles without guessing from directory names.

## Literal/value contract

Allowed literal values:

```text
nil
boolean
bounded integer/finite number
bounded UTF-8 string
normalized identifier/symbol key
enum/tag from frozen schema
```

No arbitrary source AST, code fragment, documentation body, executable expression, or unbounded table.

## Resolved public convention symbols

Core rules may require exact normalized keys for public structural surfaces, for example:

```text
CreateFrame
CreateFromMixins
Mixin
Frame:RegisterEvent
Frame:RegisterUnitEvent
EventRegistry:RegisterFrameEventAndCallback*
EventRegistry:RegisterCallback
EventRegistry:TriggerEvent
CVarCallbackRegistry:RegisterCallback
SetScript
HookScript
hooksecurefunc
LibStub
```

These keys come from the exact analyzer/library/profile fact environment. The recognizer does not prove their current API contract and does not silently fall back to spelling similarity.

## Scope and joins

Bundles declare whether joins may cross:

```text
same fact partition
same file
same function
same package/TOC variant
same XML document/load unit
explicitly listed dependent partitions
```

A rule cannot scan an undeclared repository-wide universe. Cross-partition inputs are enumerated and budgeted by the caller.

## Coverage

Each fact family has exact capability IDs, such as:

```text
emmy.fact.references
emmy.fact.calls
project.toc.package
project.toc.dependencies
project.toc.saved_variables
project.xml.templates
project.xml.objects
project.xml.scripts
project.ownership
```

Missing, Partial, Failed, Unknown, or truncated capability state is propagated to the rule outcome. Empty facts are not a clean negative by themselves.

## Adapter rules

- adapters are deterministic projections, not new parsers;
- no field inferred from comments/path/repository name;
- no platform authority upgrade;
- no silent lossy projection: unsupported source facts produce adapter coverage/loss records;
- input order does not affect bundle identity;
- duplicate exact facts canonicalize without losing source/evidence refs;
- source fact removal invalidates dependent match/output IDs in the next generation.
