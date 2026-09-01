# E4-C migration, static impact, and explicit context handoff

**Status:** normative.

## `migration_candidates`

Acquires one exact lineage snapshot plus exact project/reference/search views required by the selected profile and invokes E4-B candidate generation.

Candidates may use:

- explicit Reference deprecation/replacement/transition records;
- accepted lineage and typed change records;
- exact project source/graph facts;
- Candidate-only E4-A retrieval/shape signals;
- exact coverage/conflict/privacy/license state.

The service preserves candidate confidence/proof ceiling and never describes the top candidate as the replacement unless an explicit accepted replacement assertion exists.

## `migration_validate`

Validates one exact candidate or recipe artifact against the exact governing lineage snapshot and owner generations.

Required recipe closure:

```text
source and target entity identities
accepted governing transition/replacement assertions
applicability scope and preconditions
typed transformation steps
constraints and known unsupported cases
expected postconditions
required static/client/runtime validation steps
proof ceiling, evidence, coverage and conflicts
privacy/license/security state
canonical bytes and schema/profile IDs
```

Validation does not apply the recipe or prove that a future edit succeeds.

## Migration result classes

```text
ExplicitReplacementCandidate
ExplicitDeprecationNoTarget
LineageOnlyNoReplacement
CandidateRecommendationOnly
ValidatedRecipe
InvalidRecipe
NotEvaluated
ConflictBlocked
Unsupported
```

`ValidatedRecipe` means the recipe artifact conforms to its exact static contract. It does not mean the source was edited, compiled, loaded in WoW, taint-safe, combat-safe or runtime-correct.

## `impact_plan`

Validates exact accepted change/assertion roots, exact target graph snapshots and the impact traversal profile. It delegates plan construction to E4-B.

## `impact_run`

Executes the exact plan and returns:

```text
direct affected entities
bounded transitive affected entities
ordered reason paths
relation and confidence classes
evidence/provenance/coverage/conflicts
unvisited/truncated/omitted scopes
budgets/continuation
static-only nonclaims
```

Service does not assign severity, priority, owner, issue status, runtime breakage, performance cost, exploitability or fixability unless supplied as a separate exact owner record under a future contract.

## `impact_continue`

Reopens the exact lineage and target graph snapshots named by the continuation. It preserves total budgets, frontier, visited/result manifests and prior truncation. It never switches to current graphs or recomputes the lineage comparison.

## `impact_explain`

Returns every root/change/lineage assertion and direct edge of each selected reason path. Paths remain paths; service text cannot flatten them into direct dependencies.

## Impact status

- all requested bounded facets complete: `Complete`;
- graph/source/lineage coverage incomplete: `Partial` or `NotEvaluated`;
- only Candidate/Possible paths: `CandidateOnly`/`Partial` as owner reports;
- traversal budget reached: `Truncated` plus continuation when retained;
- conflict blocks a required path/facet: `ConflictBlocked`;
- cancellation/failure preserved.

## Search-to-context handoff

`search_context` is the only E4-C operation that connects search to context. It requires an exact explicit selection receipt.

```text
query-relative search candidate
-> explicit selection receipt
-> exact owner entity root
-> exact E3-C ContextUniverseSet acquisition
-> E3-B map/L0/L1/context operation
```

Hard stops:

- no automatic top-1 or unique-candidate selection;
- no fuzzy query text used as a context root;
- no search score injected as entity confidence;
- no lineage/replacement/migration claim injected into context unless it already exists as an exact E4-B record explicitly requested and supported by the context profile;
- no current generation refresh between selection and context acquisition;
- no context view from an incompatible generation/profile;
- no edit/tool authorization.

## Context selection race

The service validates that the selected entity still belongs to the exact retained owner generation and that the context acquisition uses the same exact publication bindings. If a symbolic-current request advanced after search, the existing exact result remains valid for its retained generation; using the newer current context requires a new search/request.

## Existing context continuation

A search selection cannot be changed during an E3-C context continuation. The service binds the `SearchSelectionReceipt` and exact root into the outer continuation metadata. Context continuation remains owned by E3-C/E3-B.

## Privacy/license

Source/detail/context output is permitted only by the intersection of search result, owner source, migration/impact and context consumer profiles. A metadata-only search candidate cannot be used to smuggle private source into context output.

## No execution

E4-C does not:

```text
apply migration steps
write source
run Lua or WoW
invoke build/test/release scripts
open editor
call network/model/CBM
claim runtime verification
```
