# E3-B semantic context pack and deterministic rendering

**Status:** normative.

## Artifact layers

```text
ContextSemanticPack
    canonical structured selected context and trust metadata

RenderedContextArtifact
    deterministic presentation of one exact semantic pack

ContextMetrics
    deterministic measures over exact pack/artifact identities

ContextEvaluationReport
    frozen-corpus assessment after pack/render creation
```

Only the first object is the canonical semantic context result. Later layers cannot change its identity or selection.

## Semantic pack sections

A canonical pack contains ordered typed sections:

```text
PackIdentity
UniverseAndCompatibility
RequestAndProfiles
BoundaryNotices
ProjectMapReferences
L0SkeletonReferences
L1SkeletonReferences
ControlEffectReferences
SelectedFacts
SelectedRelationsAndReasonPaths
ReferenceFacts
SourceHandlesAndExcerpts
ExistingFindingEvidence
ConflictsCoverageAndLoss
Omissions
SelectionTrace
BudgetAndTokenAccounting
ContinuationAndStopState
```

Mandatory records remain present even when presentation sections are compact.

## Context items

Every item has:

- stable item ID, kind, and schema;
- exact universe, generation, and scope;
- typed payload;
- origin fact, assertion, reference, and source IDs;
- evidence, provenance, confidence, coverage, and conflict refs;
- derivation/template ID when derived;
- root and inclusion reasons;
- privacy, license, and source-boundary class;
- canonical payload digest.

There is no generic free-form canonical summary item.

## Identity DAG

```text
ContextUniverseSet + normalized request + profiles
-> ProjectMap / L0 / L1 / control-effect / source / evidence / omission records
-> ContextSemanticPack
-> RenderedContextArtifact
-> ContextMetrics
-> ContextEvaluationReport
-> optional outer delivery envelope
```

Earlier IDs never contain later IDs. Validation reports refer to artifacts through a noncyclic outer record.

## Canonical JSON

The JSON renderer is lossless for all semantic fields:

- UTF-8;
- frozen field ordering and canonicalization;
- arrays already in semantic stable order;
- canonical scalar representation;
- no timestamps, host paths, process IDs, cache state, or incidental metrics in semantic identity;
- source strings escaped as data;
- digest computed over exact bytes.

A round trip reconstructs the same semantic pack and ID.

## Deterministic Markdown

Markdown is mechanically checkable:

- section order and labels come from a frozen renderer/template catalog;
- facts use typed templates with item and origin IDs;
- no free-form generated prose;
- confidence, coverage, conflict, and omission labels are explicit;
- reason paths render as paths;
- source excerpts use the structural data boundary contract;
- encoding, line endings, and whitespace are frozen;
- a renderer trace maps output ranges to semantic item/template IDs;
- every required semantic item is represented exactly once or an allowed rendering-loss record exists.

## Typed templates

Permitted templates are repository-owned and typed, for example:

```text
ENTITY <label> [<kind>] id=<id> source=<source-handle>
RELATION <source> --<kind>/<confidence>--> <target> assertion=<id>
PATH <root> => <target> via <ordered-relation-ids>
COVERAGE <capability>/<partition>: <state>
OMITTED <scope>: <reason> affects=<facet>
```

Template inputs cannot change the template grammar.

## Rendering loss

Canonical JSON is lossless. A compact renderer can omit repetitive presentation detail only when:

- the semantic pack remains the source;
- the renderer profile declares the loss;
- omitted fields are not required for trust/correctness;
- a `RenderingLossRecord` names exact fields/items;
- source, evidence, coverage, conflicts, omissions, and budgets remain present or exactly referenced;
- validation proves no claim becomes stronger or ambiguous.

## Renderer budgets

Before rendering:

- calculate frozen template, escaping, and boundary overhead;
- validate exact or conservative output cost;
- do not alter semantic selection unless the request explicitly allows a distinct renderer-specific semantic plan.

After rendering:

- measure exact bytes;
- run exact tokenizer or declared estimator;
- validate item/range mapping and source boundaries;
- fail or execute explicit deterministic replanning;
- never slice raw output.

## Multiple renderings

One pack can produce canonical JSON and deterministic Markdown. Future formats require reviewed profiles.

Rendered artifact IDs differ by renderer, encoding, framing/tokenizer, and exact bytes. The semantic pack ID remains the same only when semantic content is identical.

## Existing findings

A pack can include an existing finding and exact evidence owned by another crate when rooted explicitly. It does not rerun the rule, change severity, or claim validity for another generation.

## Nonclaims

A pack does not prove:

- sufficiency for every task;
- global irrelevance of omitted context;
- truth of source comments;
- proof from a possible relation;
- runtime behavior from static structure;
- downstream consumer compliance with boundaries;
- provider billing tokens without exact framing/tokenizer;
- a fix, plan, or completed task.

## Golden outputs

Freeze semantic JSON and Markdown for:

- small-project navigation;
- entity inspection with ReferenceView facts;
- load, event, hook, and state traces;
- partial/conflicted input;
- budget-pruned context;
- denied and transformed source excerpt;
- adversarially structured source text;
- continuation pages;
- pinned addon and Blizzard UI source fixtures.
