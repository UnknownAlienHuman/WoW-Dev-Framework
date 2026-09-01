# E5-A corpus splits, sealed holdout, and leakage prevention

**Status:** normative.

## Split classes

```text
Train
    pack authors may use labels/results for rule construction

Dev
    iterative threshold/ambiguity/budget tuning under recorded runs

Test
    fixed evaluation after candidate freeze; using results for changes consumes the test generation

SealedHoldout
    membership digest known; labels/results hidden until pack bytes and evaluation request freeze

Challenge
    explicit adversarial/mutation suite; not counted as independent natural examples unless declared

Quarantine
    unresolved source/license/provenance/label/coverage/security cases
```

## Atomic split groups

The unit of assignment is `CalibrationProvenanceGroup`, not file/function/example. All related items stay together:

- forks/upstream descendants;
- copied/vendor/shared library code;
- generated/template variants;
- near-identical content/structure under different names;
- mutations derived from one source example;
- multiple versions of the same project unless the profile explicitly treats chronological generalization as one group held entirely in one split;
- fixtures authored from the same donor source.

## Group graph

Provenance grouping is a conservative graph closure. If A shares copied code with B and B is a fork of C, the connected component cannot cross splits unless an exact field-level exclusion proves the evaluated examples are independent and the split profile explicitly supports it.

Unknown grouping state blocks sealed-test/generalization claims.

## Same-owner repositories

The initial user repository set supplies breadth but not automatically independent ecosystem evidence. Current common ownership is recorded; upstream origin/fork/copy lineages must be audited. Generalization scope can be limited to the admitted provenance groups until independent external lineages are available and licensed.

## Split creation

```text
validate admitted corpus and group graph
-> freeze group-key profile
-> sort canonical group IDs
-> assign explicit group IDs using a reviewed split manifest
-> verify no connected provenance component crosses partitions
-> freeze membership and visible-label policy
-> seal holdout member/label manifests under exact digests
-> publish leakage-analysis report
```

Random percentage splitting is not canonical. If deterministic pseudorandom assignment is used later, algorithm/seed/input order must be frozen and explicit group overrides remain reviewable.

## Label visibility

Pack authors can see Train/Dev labels. Test labels can be visible only according to the frozen process; after a test result influences a change, the next pack version records that test generation as development evidence, not untouched test proof.

Sealed holdout:

- exact member manifest digest is fixed;
- labels and expected outputs are withheld from pack authors;
- evaluator access is separated;
- candidate pack bytes, implementation/profile IDs and run request freeze before unsealing;
- results are immutable;
- any subsequent candidate change requires a new candidate and a new untouched holdout generation to make the same claim.

E5-A defines artifact semantics. E5-B later owns authorization, durable workflow and unsealing audit.

## Leakage classes

```text
RepositoryOrForkLeakage
CopiedOrVendoredCodeLeakage
GeneratedTemplateLeakage
ChronologicalVersionLeakage
MutationFamilyLeakage
LabelOrExpectedOutputLeakage
RecognizerOutputLeakage
SearchOrModelSuggestionLeakage
ReviewerCrossContamination
ProfileOrThresholdTuningLeakage
SourceExcerptIdentityLeakage
```

## Leakage tests

- rename donor metadata and verify split/labels do not depend on it except audit IDs;
- detect identical/near-identical fact subgraphs across groups;
- detect shared vendor/generated source-handle families;
- detect same upstream commit/tree ancestry when available;
- verify mutation parent/child stay together;
- verify test/holdout labels absent from candidate construction inputs;
- verify expected labels never appear in pack clauses/metadata used by matching;
- verify evaluation result IDs are not pack inputs;
- verify a rerun after tuning is labeled contaminated/consumed.

Similarity detectors produce leakage candidates, not automatic proof; unresolved candidates conservatively block independence claims.

## Chronological evaluation

When testing update stability across project versions, all versions stay in one provenance group. The profile may designate earlier versions as construction inputs and a later version as a chronological challenge only if no future source/label information was used. This is a specialized challenge, not ordinary random holdout.

## Corpus expansion

Adding examples creates a new corpus/split manifest. Existing candidate evaluation remains bound to the old split. New examples cannot be silently placed into holdout after their labels or pack behavior are known and then described as sealed.

## Weighting

Weights do not alter group separation. They can normalize reporting across large/small projects but cannot hide mandatory case failures or count correlated examples as independent repositories.

## Split validation output

```text
exact corpus/split/group IDs
group membership and reasons
connected-component closure
visible/hidden label policy
leakage findings/conflicts/NotEvaluated
independent group counts by claimed scope
consumed test/holdout history
status and canonical digest
```

## Hard failures

- same provenance component crosses train/dev/test/holdout;
- pack/label output used to assign favorable split after observation;
- holdout labels accessible before candidate freeze;
- consumed holdout described as untouched;
- unresolved duplicate/fork leakage ignored;
- individual files/functions counted as independent donor repositories;
- Unknown examples treated as negative to improve precision;
- split membership changed without new identity.
