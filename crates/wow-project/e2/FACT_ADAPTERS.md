# E2-C project-to-recognizer fact adapters

**Status:** normative seam to `wow-recognizers`; adapters do not parse or infer extra semantics.

## Purpose

Convert validated project TOC/XML/load records and exact `wow-emmy` fact sets into the E2-B `RecognizerFactBundle` schema while retaining source fact identity, generation, evidence, coverage, and adapter loss.

## Adapter profiles

```text
ProjectRecognizerFactAdapterProfile
    profile_id/version
    accepted source schema versions
    target recognizer fact schema profile
    fact-kind mappings
    field/ID/reference mappings
    loss/unsupported policy
    partitioning and cross-partition dependency rules
    budgets
    canonical digest
```

No mapping is inferred by matching field names dynamically.

## TOC mappings

```text
ProjectPackage/TocDocument/TocVariant -> TocPackageFact
TocFileEntry -> TocFileFact
TocDependencyDeclaration -> TocDependencyFact
TocLoadOnDemand/Bootstrap records -> TocLoadOnDemandFact
TocSavedVariableDeclaration -> TocSavedVariableFact
```

Preserve:

- package/variant/universe;
- semantic ordinal;
- required/optional scope;
- exact path/file resolution;
- source handle/evidence;
- coverage/conflicts/unknowns.

Unknown directives/tags are not converted to known recognizer facts.

## XML mappings

```text
XmlTemplateRecord -> XmlTemplateFact
XmlObjectRecord -> XmlObjectFact
XmlInheritanceRecord -> XmlInheritanceFact
XmlScriptRecord -> XmlScriptFact
```

Preserve exact object/template identity, owner/document/package, ordered refs, source spans, resolution state, append/prepend/inherited flags when represented, and coverage.

Unknown XML records remain adapter loss/unsupported records and can block dependent rule coverage.

## Lua mappings

The adapter references exact `wow-emmy` facts:

```text
ReferenceFact -> LuaReferenceFact
CallFact -> LuaCallFact
LocalBinding/Flow/Use/Operation facts -> corresponding E2-B facts where active rules require them
FunctionScopeFact -> LuaFunctionFact identity/ownership fields
ControlFlowRelation -> LuaControlFlowFact
```

Rules:

- source `wow-emmy` fact ID remains visible;
- no raw CST/upstream analyzer object escapes;
- no re-resolution by spelling;
- no project/platform authority added;
- unresolved/dynamic/ambiguous status preserved;
- Main versus Library roles preserved;
- XML virtual Lua units map back to exact XML source unit/parent spans.

## Ownership mappings

Project can emit `ProjectOwnershipFact` only from exact parsed/declared structure:

```text
package owns selected TOC/document/file
XML document owns templates/objects/scripts
object/template owns script handler unit
file/unit contains exact analyzer function declaration
```

Directory names, table names, file prefixes, comments, or repository identity do not create module/service roles. Those belong to recognizers/calibration packs.

## Partitioning

Bundle partitions are narrow and replaceable, for example:

```text
project.toc:<package>:<variant>
project.xml:<document>
project.lua:<file-or-virtual-unit>
project.signal:<declared-bounded-scope>
project.state:<package-root>
project.load:<package>:<variant>
```

Each partition declares dependencies on exact source/analyzer/project partitions. A recognizer cannot scan outside the declared bundle dependency closure.

## Cross-partition bundles

Rules needing producer/consumer or package/file context receive an explicitly assembled bundle manifest listing every included partition and capability. No implicit project-wide join.

Examples:

- custom EventRegistry producer/subscriber scope;
- XML object plus handler Lua unit;
- SavedVariables declaration plus Lua state accesses;
- TOC load unit plus analyzer function ownership.

## Adapter status

Each selected source record/fact receives:

```text
AdaptedExact
AdaptedWithSidecar
Unsupported
NotApplicable
NotEvaluated
Failed
```

Unsupported/partial source cannot disappear from both bundle coverage and adapter-loss report.

## Confidence

Adapters preserve input confidence. They do not produce recognizer-derived confidence. Exact structural field mapping from a Proven source fact remains Proven as an input observation; recognizer outputs later become Derived/Possible.

## Identity

Recognizer fact ID derives from:

```text
adapter profile/version
target fact kind
exact source fact/record ID
selected mapped fields
project generation and partition
```

Input order, host path, row ID, thread, and rendered text are excluded.

## Budgets

Bound fact count, field/list/string size, source/evidence refs, partitions, cross-partition dependencies, adapter-loss records, and total bytes. Truncation marks the bundle Partial and cannot support complete no-match/negative clauses.

## Operations

```text
validate_project_recognizer_adapter_profile
adapt_toc_facts
adapt_xml_facts
adapt_emmy_facts
adapt_project_ownership_facts
assemble_recognizer_fact_bundle
validate_recognizer_fact_bundle_for_project
build_adapter_coverage_and_loss_report
```

## Tests

- every E2-B target fact kind;
- exact source ID/generation/evidence closure;
- unknown/unsupported fields produce loss;
- unresolved/dynamic Lua status preserved;
- Main/Library/universe separation;
- inline XML Lua source-map closure;
- no path/name/module heuristics;
- explicit cross-partition scope only;
- source removal invalidates adapter facts;
- partial/truncated bundle blocks complete no-match;
- shuffled source/fact order yields identical bundle bytes.
