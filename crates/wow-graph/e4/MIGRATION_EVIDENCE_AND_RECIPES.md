# E4-B migration evidence and recipe candidates

**Status:** normative static migration contract. E4-B does not edit source.

## Purpose

Migration output answers:

```text
what exact old surface/use changed?
what exact new surfaces or transitions are supported?
which evidence establishes identity, replacement, or applicability?
which project consumers are statically exposed?
what typed migration operation is justified, at what proof ceiling?
what remains runtime-, project-, or human-validated?
```

It does not generate or apply a patch.

## Evidence set inputs

```text
exact ordered reference/source generation pair
accepted/candidate lineage snapshot
exact GenerationChangeSet
exact old and new ReferenceView facts
exact project-use/source/graph records from a published project generation
optional exact existing findings and rule evidence
migration profile and requested capability scope
coverage/conflicts/omissions/budgets
```

Blizzard implementation source, Reference Pack contract, user project use, runtime observation, and community/KB evidence retain separate authority classes.

## Migration evidence set

A set contains:

- old endpoint and exact contract/source facts;
- accepted continuity/removal/deprecation/replacement assertions;
- candidate alternatives and ambiguity groups;
- new target endpoint facts;
- exact changed facets;
- exact project use sites and typed relation paths;
- applicability and precondition records;
- contraindications/conflicts;
- required runtime/client/human validation;
- proof ceiling and completeness state.

No prose recommendation can exist without the structured evidence set it summarizes.

## Recipe proof classes

### `ExactMechanical`

A deterministic transformation shape is fully specified by exact old/new owner contracts and the local project use is within a closed supported syntax/semantic slice.

Requires:

- explicit/accepted transition at sufficient proof;
- exact old/new signatures/types/restrictions;
- exact use-site form and complete required analyzer facts;
- no ambiguity/conflict/dynamic target;
- transformation operation registry entry;
- exact validation plan;
- no runtime-only precondition unless the recipe remains plan-only.

Even this class is not applied by E4-B.

### `ValidatedTemplate`

A reviewed migration template applies to a bounded structural family but requires project-specific parameterization and post-change validation.

### `PlanOnly`

Evidence supports a migration direction and required steps, but source edits or runtime decisions are not uniquely determined.

### `CandidateOnly`

A potential target/recipe is suggested by candidate lineage/search/shape evidence. It cannot be described as the replacement.

### `NotEvaluated`

Required owner facts, coverage, analyzer capability, lineage proof, or runtime requirement is unavailable.

## Recipe operation registry

Typed nonexecutable operations may include:

```text
RenameReference
MoveImportOrNamespace
ReplaceCallableReference
ReorderOrMapArguments
AddRequiredArgumentPlaceholder
RemoveUnsupportedArgument
UpdateReturnHandlingPlan
UpdateTypeOrEnumReference
ChangeEventOrCallbackRegistrationPlan
ChangeTemplateOrMixinReferencePlan
ChangeLoadDependencyOrTOCPlan
AddAccessGuardPlan
ReplaceUnsafeValueUseWithDocumentedSinkPlan
RemoveObsoleteUsePlan
ManualReviewStep
RuntimeProbeStep
ValidationStep
```

These are structured plans/constraints, not text edits or shell commands.

Each operation defines:

- supported old/new endpoint kinds;
- required exact facets and proof class;
- supported project-use fact shapes;
- parameters and source-handle references;
- preconditions/contraindications;
- required follow-up diagnostics/tests/probes;
- maximum recipe proof class;
- security and budget limits.

## Replacement target eligibility

A target can be stated as an exact replacement only when an accepted `replaced_by` assertion or exact owner transition supports the requested capability/scope. Otherwise:

```text
possible migration target
candidate target
no supported target
NotEvaluated
```

Top search result, same signature, same docs words, or nearest source location is insufficient.

## Identity-preserving changes

For accepted `same_stable_identity` plus changed facets, recipes can reference the same logical surface with updated:

- canonical name;
- source namespace/location;
- signature;
- type/enum members;
- restriction metadata;
- load/package contract.

The recipe still validates exact local use shape. Identity continuity does not mean call compatibility.

## Removed surface without replacement

When removal is authoritative and no replacement exists, valid output can be:

```text
RemoveObsoleteUsePlan
ManualReviewStep
feature retirement/conditional compatibility plan
NotEvaluated due runtime requirement
```

Do not invent a target to avoid a negative result.

## Split and merge migrations

A split/merge recipe must represent all endpoints and capability partitioning:

```text
old capability facets -> exact target endpoint(s)
selection conditions
project-use classification
unmapped facets
coverage/conflicts
```

It cannot choose one target globally when different use sites require different targets.

## Signature migration

Parameter and return operations use exact ordered typed facets. Preserve:

- optional versus required;
- nilability versus missing;
- variadic state;
- overload selection;
- callbacks/tuples/multiple returns;
- restriction/Secret-capable positions;
- unknown/unsupported fields.

A conversion is not assumed safe merely because types have compatible display names.

## Secret/restriction changes

Migration evidence can include exact ReferenceView restriction changes and current KB-routed rules. It must not claim:

- a copied/converted/stringified value is declassified;
- `pcall` makes a forbidden use safe;
- a permanent spell whitelist is valid;
- a static guard is sufficient in every runtime context;
- source implementation proves runtime accessibility.

Recipes involving restricted values are normally `PlanOnly` unless the exact supported local rule and sink/guard contract is proven, and still require named runtime validation where the platform contract demands it.

## Events, callbacks, hooks, and lifecycle

Recipes preserve distinctions among native frame events, EventRegistry native bridges, custom registry signals with exact producers, CVar callbacks, scripts, hooks, load phases, and runtime readiness. A migration cannot swap these systems based only on similar names.

Hook availability never automatically proves taint/combat/protected safety.

## Project use closure

For a recipe to cover a use site, exact project facts must identify:

- source handle/span;
- resolved old endpoint or candidate state;
- operation/call/registration/use shape;
- owner/load/context relations required by the recipe;
- analyzer/recognizer/rule capability coverage;
- conflicts/dynamic resolution;
- existing finding evidence when present.

Dynamic/unresolved use stays candidate/NotEvaluated.

## Validation plan

A recipe includes typed required checks, for example:

```text
re-run exact project analysis
re-run specific diagnostics
verify no old endpoint uses under complete coverage
verify new signature/type facts
verify TOC/XML/load graph
run named runtime probe on exact client build/context
run in-client smoke scenario
manual review of dynamic callsites
```

E4-B records requirements; E4-C/service or later edit tooling orchestrates them.

## Recipe rendering

Any human-readable steps are deterministic templates over typed operations/evidence. They cannot:

- add an unrecorded target;
- omit candidate/coverage/conflict labels;
- state runtime success;
- include executable shell/code payload as authority;
- conceal required validation.

## Negative and partial outcomes

Valid outcomes include:

```text
exact migration evidence, no recipe needed
exact removal, no replacement
candidate targets only
multiple applicable recipes by use-site class
recipe blocked by ambiguity/conflict
runtime validation required
partial project-use coverage
NotEvaluated
```

An empty recipe list is not evidence that no migration work exists unless exact requested scope and coverage authorize it.

## Determinism

Equivalent exact inputs/profiles produce identical:

- evidence-set membership;
- target eligibility;
- recipe operation parameters/order;
- proof classes;
- applicability/contraindication records;
- validation requirements;
- omissions/conflicts;
- canonical IDs/bytes.

No model wording, source order, worker timing, or top-rank instability may affect canonical output.

## E4-B nonresponsibilities

- source editing/patch generation;
- applying/refactoring code;
- selecting a candidate target without evidence;
- user/business severity;
- runtime execution/proof;
- automatic dependency installation;
- releasing or updating addons;
- LSP code actions.
