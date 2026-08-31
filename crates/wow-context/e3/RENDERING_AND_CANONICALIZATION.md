# E3-A rendering, canonicalization, and artifact identity

**Status:** normative separation of semantic artifacts, canonical bytes, and consumer renderers.

## Three layers

```text
input domain records
-> semantic context model
-> canonical semantic serialization
-> renderer-specific artifact
```

Only the semantic model and its canonical serialization determine semantic artifact identity. Markdown wrapping, display paths, terminal width, color, and transport envelopes do not.

## Semantic IDs

Domain-separated IDs include exact input snapshot, profile versions, request/root/lane/detail policy, semantic records, evidence/coverage/conflict/loss closure, and canonical ordering.

```text
ProjectMapId
SkeletonId
ContextPlanId
ContextExpansionStepId
ContextBundleId
ContextMetricSetId
ContextEvaluationReportId
```

They exclude:

```text
SQLite row/page/WAL state
absolute paths
wall clock/process/thread/worker identity
query completion order
renderer line wrapping or terminal width
model preference/score
operational lease/connection IDs
```

## Canonical semantic serialization

The initial profile uses repository-owned canonical JSON semantics:

- UTF-8 without BOM;
- fixed schema/version fields;
- no insignificant whitespace;
- object fields in schema-defined order;
- arrays in semantic order, never generic lexicographic resort when order is meaningful;
- map/set projections as ordered arrays with explicit key fields;
- integer and bounded decimal representation only; no NaN/Infinity/platform floats;
- booleans/null exact;
- source-derived strings use the owning normalized semantic value plus source handle; source excerpt bytes are not Unicode-normalized;
- unknown fields rejected unless a compatibility profile explicitly allows and preserves them;
- volatile/operational fields live outside canonical semantic payload.

Tests freeze canonical bytes and SHA-256; implementations do not rewrite golden fixtures automatically.

## Canonical ordering

Ordering comes from each semantic registry/profile:

```text
Project Map section profile order
roots by universe/kind/semantic key/ID
skeleton members by semantic/source ordinal
relations by lane/kind/direction/source/target/qualifiers/assertion ID
paths by length then relation/entity tuple
control/effect nodes by source span/semantic ordinal/kind/ID
evidence by artifact field/provenance/source/ID
loss/omission/stopping by scope/category/reason/ID
frontier by priority/root/lane/path/detail/stable key
```

No hash table, database, filesystem, worker, or completion ordering.

## Semantic strings

Frozen heading/role/status labels are registry values, not source or model prose. Human display labels can accompany exact IDs but cannot determine identity unless the owning domain contract defines them as semantic.

Do not canonicalize distinct identifiers by case folding, Unicode compatibility folding, trimming, punctuation removal, or locale rules unless the owning source/identity contract explicitly requires it.

## Source excerpts

Canonical excerpt identity includes original source content digest, requested/actual half-open byte span, declared normalization/escaping profile, faithful selected bytes, prefix/suffix truncation, provenance/license/security refs, and excerpt digest.

Renderer escaping is recorded separately. A renderer cannot paraphrase source or change the actual span.

## Renderer profiles

Initial renderer classes:

```text
canonical-semantic-json
human-markdown-v1
compact-line-v1
```

A renderer profile defines:

- compatible semantic schema versions;
- fixed section/field order;
- headings/labels/templates;
- source/evidence sidecar placement;
- line ending/indent/wrapping rules;
- Markdown/code/JSON/terminal escaping;
- allowed omission only when already represented by semantic loss/sidecar records;
- byte/output budgets and renderer-specific truncation policy;
- canonical renderer artifact digest.

## Markdown

- fixed heading depth and list/table policy;
- untrusted labels escaped;
- links constructed only from trusted registered routes, never raw source strings;
- source excerpts fenced with a delimiter that cannot be closed by the payload;
- raw HTML disabled/escaped;
- terminal/control escapes neutralized;
- no hidden comments/directives;
- no renderer-added interpretation.

## Compact line format

Each line is one typed record with fixed field order and escaping. It is not ad hoc prose. If a semantic record cannot fit atomically, the renderer emits/retains the semantic truncation sidecar or fails the renderer profile.

## Renderer loss

A renderer cannot delete a mandatory semantic record. When a consumer profile intentionally moves detail to a sidecar, exact sidecar IDs remain reachable and a renderer loss/coverage record states the projection.

`semantic bundle valid` and `renderer artifact valid` are separate statuses.

## Tokenizer subject

Exact token count is over exact final renderer bytes/string under the pinned tokenizer profile. It does not enter semantic bundle identity unless the context request explicitly includes that tokenizer budget/profile; even then, token metrics do not change domain truth.

## Artifact manifests

```text
ContextSemanticArtifactManifest
    semantic schema/profile/input/request IDs
    member record IDs/digests
    canonical semantic bytes digest/length
    coverage/loss/security state

ContextRendererArtifactManifest
    semantic artifact ID
    renderer profile ID/version
    output bytes digest/length/line/scalar counts
    tokenizer result: optional
    renderer coverage/loss/security state
```

## Version compatibility

- additive optional fields require new schema/profile version and compatibility tests;
- changing identity ingredients, ordering, mandatory fields, loss semantics, or source escaping is breaking;
- a continuation binds exact semantic/ordering/profile version;
- a renderer cannot silently accept a newer semantic schema;
- cached artifacts, when later enabled, key exact IDs/digests.

## Validation

Nonrepairing validators recompute:

```text
record reference closure
field/enum/schema constraints
semantic ordering
canonical bytes and IDs/digests
renderer semantic equivalence
escaping/container safety
budget arithmetic
source excerpt faithfulness
coverage/loss/omission/stopping sidecars
tokenizer subject digest
```

## Required tests

- shuffled object/map/fact/query order;
- 1/2/N worker production;
- meaningful member/source order preservation;
- Unicode/case/combining-character identifiers;
- source excerpt original versus normalized renderer bytes;
- unknown/duplicate/missing fields;
- NaN/Infinity/float/locale formatting mutation;
- Markdown fence/link/HTML/terminal injection;
- compact line atomic record boundary;
- semantic bundle equal across renderers;
- renderer hides mandatory blocker mutation;
- renderer sidecar closure;
- tokenizer counts exact renderer digest;
- schema/ordering/renderer version mismatch;
- volatile field accidentally entering ID;
- byte-identical rebuild.

## Hard stops

- no renderer prose as semantic truth;
- no source string controlling structure/template/link/tool behavior;
- no generic sort that destroys semantic order;
- no lossy identifier normalization;
- no semantic ID based on output line/path/row/worker state;
- no unreported renderer omission;
- no exact token count over a different byte subject.
