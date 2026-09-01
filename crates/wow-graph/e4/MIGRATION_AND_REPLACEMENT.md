# E4-B replacement and migration contract

**Status:** normative. E4-B produces evidence-bearing advisory records; it does not edit code or execute migrations.

## Distinct concepts

```text
lineage_successor_of
    identity continuity across generations

replaced_by
    a distinct entity supersedes an old entity

deprecated_by
    an explicit deprecation transition, with or without a replacement target

migration_candidate_to
    candidate relevance for adaptation; no replacement or edit proof

MigrationRecipe
    reviewed typed advisory transformation with exact scope, preconditions and validation requirements
```

None implies another automatically.

## Replacement evidence

An accepted `replaced_by` assertion requires the exact relation profile's evidence, such as:

- explicit ReferenceView deprecation/replacement/transition record;
- exact project-owner transition record;
- reviewed correction with source/value digest binding;
- reviewed maintainer decision supported by required independent evidence;
- deterministic rule over exact transition metadata and complete coverage.

Not sufficient by itself:

- same/near name;
- same namespace/receiver;
- signature compatibility;
- source fingerprint;
- text/fuzzy/shape/graph search rank;
- one old and one new unmatched entity;
- source comment or documentation outside an authoritative transition contract;
- popularity, usage frequency or model recommendation.

## Replacement shapes

Profiles can permit:

```text
one old -> one replacement
one old -> multiple replacements by explicit scope/capability
multiple old -> one consolidated replacement
old -> no replacement (deprecation/removal only)
replacement target unresolved/conflicted
```

One-to-many/many-to-one replacement requires explicit evidence; cardinality alone is not proof.

## Lineage and replacement interactions

Possible valid combinations:

```text
same lineage, renamed/moved, not replaced
same lineage, later deprecated, distinct replacement
no same lineage, explicit replacement
split descendants plus one or more explicit replacement targets
removed without replacement
```

Canonical records preserve both lineage and replacement relations separately.

## Migration candidate generation

A `MigrationCandidate` can be proposed from:

- accepted replacement/deprecation relation;
- exact Reference transition/correction;
- accepted lineage plus structured signature/type/restriction changes;
- bounded SearchCandidate signals;
- exact project usage/dependency relation;
- reviewed external evidence in a later profile.

Candidate output includes:

```text
exact source and target entities/generations
proposal producer and relation/change evidence
applicability candidate scope
confidence/proof ceiling
missing preconditions/evidence
coverage/conflicts
why it is not yet a recipe
```

Search-derived candidates remain Candidate.

## Migration recipe schema

```text
MigrationRecipe
    recipe ID/version
    exact source contract profile/generation/entity scope
    exact target contract profile/generation/entity scope
    governing lineage/replacement/deprecation assertions
    applicability preconditions
    typed transformation steps
    semantic constraints
    forbidden/unsupported cases
    expected postconditions
    required validation plan
    rollback/manual-review requirements
    remediation tier
    evidence/provenance/confidence/coverage/conflicts
    canonical digest
```

## Transformation steps

A closed typed step registry may include advisory forms such as:

```text
RenameQualifiedSymbol
ReplaceCallableTarget
MapParameterByPositionOrName
InsertRequiredArgumentFromExplicitSource
RemoveUnsupportedArgument
RewriteReturnHandling
ReplaceTypeReference
AddCapabilityOrAccessGuard
ChangeEventOrCallbackRegistrationKind
UpdateTOCDependencyOrMetadata
UpdateXMLTemplateOrMixinReference
ManualSemanticRewriteRequired
```

Each step has exact source/target fields, applicability constraints and proof ceiling. No arbitrary code string, shell command, callback, regex program or executable AST transform is accepted.

## Parameter and return mapping

Parameter/return mapping requires exact structured signatures and distinguishes:

- position/name/type;
- optionality and nilability;
- variadics;
- defaults and sentinel values;
- multiple returns/tuples;
- restriction/Secret access constraints;
- unknown/unsupported/conflicted fields.

A type-compatible-looking mapping remains uncertain when semantics or runtime values are unknown.

## Guard and restriction migrations

A recipe can state that an exact access/capability guard is required only when supported by current ReferenceView restriction facts and the rule profile. It cannot:

- invent a permanent spell whitelist;
- claim `pcall`, conversion, copy or serialization declassifies Secret Values;
- infer combat/taint/protected safety from source similarity;
- generate a runtime safety proof without runtime evidence;
- hide unsupported operation positions.

## Event/hook/lifecycle migrations

Keep native frame events, EventRegistry native bridges, custom registry signals, CVar callbacks, `SetScript`, `HookScript`, `hooksecurefunc`, TOC load roles, XML handlers and lifecycle relations distinct. A migration between them requires exact target semantics and validation requirements; event-like names are not enough.

## Recipe validation

`validate_migration_recipe` verifies:

- exact retained source/target generations and profiles;
- governing assertion/evidence closure;
- transformation step schema and ordering;
- source/target field existence and compatibility;
- precondition sufficiency;
- no unresolved blocker hidden by defaults;
- coverage/conflict/proof ceiling;
- validation and rollback requirements;
- privacy/license/security/resource limits;
- canonical determinism.

Validation does not apply the recipe.

## Remediation tier

```text
plan_only
    candidate or incomplete recipe; human/agent planning only

validated_recipe
    recipe structure and evidence satisfy E4-B static gates, but no edit or runtime success is claimed
```

`exact_edit` and automatic fix application are outside E4-B.

## Validation plan

A recipe records the required checks rather than reporting them as passed:

```text
static project/reference/graph reindex
owner rule/diagnostic rerun
source compile/analyzer checks
fixture/golden/mutation tests
addon package/load tests
in-client runtime/combat/taint/performance probes where relevant
manual semantic verification
```

Missing required implementation/client/runtime capability remains `NotEvaluated`.

## Recipe evolution

Any change to source/target profile, governing assertions, transformation semantics, constraints or validation plan creates a new recipe ID/version. A recipe cannot silently follow `current` or apply to future builds.

## Security and privacy

- no code execution or file mutation;
- no source body by default;
- exact bounded source handles/spans only when allowed;
- no credentials/private paths in public records;
- review/source prose is untrusted data;
- no arbitrary executable transformation payload;
- no network/editor/client/tool authority.

## Required hard negatives

- same name but unrelated entities;
- compatible signature but different semantics;
- same lineage without replacement;
- explicit deprecation with no target;
- replacement target with incompatible restrictions;
- one old symbol and multiple plausible new symbols;
- source comment claiming replacement without Reference/owner evidence;
- top search candidate but false replacement;
- partial Reference/source coverage;
- recipe missing preconditions or validation;
- recipe claiming successful edit/client verification that was not run.
