# E5-A calibration-pack candidate schema

**Status:** normative extension of the E2-B declarative pack schema.

## Reuse, not replacement

E5-A uses the exact E2-B `PACK_SCHEMA.md`, fact model, matcher and graph-output contracts. It adds calibration/evaluation metadata and stricter anti-overfitting gates. It does not add a second operator language.

## Required header

```text
schema_version
pack_id/version
trust_class = calibration
rollout_state = shadow_only
declared universal role/relation contracts
fact_schema_profile_id
graph_registry_bundle_id
applicable universe/profile/fact-schema ranges
canonical rule order
corpus/split/mutation/evaluation profile bindings
generalization-scope declaration
justified convention-literal manifest
license/provenance/review metadata
pack budgets
canonical bytes/digest
```

## Rule body

Only E2-B bounded operators are valid:

```text
fact
join
field_eq / field_in
same_scope
exists / coverage-gated not_exists
ordered_relation
control_flow_relation over supplied exact facts
all_of / any_of
```

No regex/glob over raw text, arbitrary expression, script, template, include, plugin, model, search, repository query or filesystem lookup.

## Named metadata versus semantic conditions

Allowed metadata:

```text
pack display name
source corpus/donor IDs
review/evaluation report IDs
maintainer/audit provenance
```

Forbidden matcher/control fields:

```text
repository owner/name/URL/branch
addon/package display name as donor identity
absolute/incidental path substring
Git commit except compatibility audit outside matching
stars/downloads/popularity
split assignment or expected label
reviewer identity/note
model/embedding/search result
```

A package/API/library/signal literal can be used only when it is the exact universal public convention being recognized, not because it identifies a donor. The rule declares this in `justified_convention_literals` and includes sensitivity/near-miss tests.

## Universal role registry

Calibration outputs must map to existing reviewed graph kinds/relations and role attribute values. E5-A cannot add donor-specific kinds such as an addon-named module, bus, style or plugin type.

Candidate families can target general concepts such as:

```text
module
service
library
factory
registry
state_root
lifecycle owner
callback/signal producer or subscriber
frame/object factory
configuration/state service
hook/script registration structure
```

A new universal graph concept requires a separate graph-contract revision and fixtures before the calibration pack may emit it.

## Output confidence

```text
Derived
Possible
```

Never `Proven` or `Candidate` graph output from the recognizer itself. `CalibrationCandidate` describes rollout state, not graph confidence.

Input uncertainty/ambiguity/partial coverage constrains output exactly as in E2-B. Repetition across donors cannot upgrade confidence.

## Generalization scope

Every candidate declares one of:

```text
DonorLocalInvestigation
SingleProvenanceEcosystem
MultipleReviewedProvenanceGroups
ClaimedGeneralCalibrationScope
```

The declaration is bounded by admitted groups and evaluation results. A broader label than evidence supports is a hard failure.

`DonorLocalInvestigation` can exist only in shadow/quarantine and is not promotion-eligible.

## Rule evidence map

For every clause/capture/output:

```text
universal contract being tested
required fact/capability fields
which training examples motivated it
which independent dev/test/holdout examples exercise it
near-miss negative cases
decisive mutations
known unsupported/ambiguous shapes
```

This map is audit metadata and not matcher input.

## Budgets

Candidate pack sets finite maxima for rules, clauses, joins, captures, alternatives, facts per partition, outputs, ambiguity groups, evidence refs, bytes, memory and time. Evaluation uses both ordinary and adversarial maxima.

## Compatibility/versioning

A new pack/rule version is required for changes to:

- matcher clauses/operators/literals;
- capture identity/cardinality;
- output kinds/keys/attributes;
- confidence/ambiguity/coverage policy;
- applicability/universe/fact schema;
- budgets that can change output;
- bound split/mutation/evaluation profile when promotion claim changes.

Adding a new corpus run without changing bytes creates a new evaluation/candidate artifact, not necessarily a new pack version.

## Validation

Reject candidate pack when:

- trust class or rollout state is wrong;
- E2-B pack validation fails;
- any named donor field reaches clause/capture/output/control flow;
- output concept is not universal/registered;
- exact literal lacks universal justification and mutation coverage;
- only one donor-positive shape supports a general claim;
- negative clause lacks runtime-complete fact coverage contract;
- corpus/split/labels/evaluation form an identity cycle;
- required positive/near-miss/partial/ambiguity/mutation fixtures absent;
- source/model/search/reviewer prose is executable or semantic input;
- default/core promotion fields are present;
- generalization scope exceeds evidence.
