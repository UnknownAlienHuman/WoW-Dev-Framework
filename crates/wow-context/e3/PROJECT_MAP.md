# E3-B deterministic Project Map

**Status:** normative compact navigation projection.

## Purpose

A Project Map answers, within explicit bounds:

```text
what exact project/source universe is this?
which packages, TOC variants, load units, files, and typed structural groups exist?
how are they directly owned, contained, loaded, and related?
where should an exact deeper L0/L1 request start?
what is incomplete, conflicted, omitted, or outside the profile?
```

It is not a full graph dump, prose architecture report, search index, runtime model, or replacement for graph queries.

## Map scope

Build one `ProjectMap` for one exact project/graph universe. A semantic pack can reference:

- one primary user-project map;
- zero or one Blizzard UI platform-source map;
- explicit cross-universe link records selected from exact graph/reference relations.

Maps remain separately identified.

## Node classes

The profile can select bounded classes from:

```text
project
package
selected_toc_variant
load_phase
load_unit
file
xml_document
template_or_object_group
namespace
module
service
library
state_root
signal_event_callback_group
hook_group
api_namespace_reference_group
entity_group
coverage_conflict_group
```

A group is a deterministic projection with exact member IDs, counts, and selection rule. It is not a new semantic entity unless the graph already owns it.

## Node facets

Allowed typed facets include:

- exact identity and universe/generation;
- source/package/file/load role;
- selected TOC variant and direct dependency role;
- exact recognizer-provided universal role with original confidence;
- declaration/export/registration/state/API-use counts and member references;
- direct source span/handle references;
- input/graph/reference coverage and conflicts;
- L0/L1 availability and deterministic cost class;
- omission and continuation state.

Forbidden facets:

- model-generated responsibility descriptions;
- guessed importance or popularity;
- hidden product/framework labels;
- runtime readiness, safety, or performance;
- inferred cross-build history;
- facts created solely from a display name or path.

## Edge classes

Map edges are selected exact graph relations or explicit path references, for example:

```text
contains
owns
loads
loads_before
depends_on
optional_depends_on
includes
references_template
inherits
mixes_in
registers_event
subscribes_callback
reads_state
writes_state
uses_api
```

The profile defines allowed classes and directions. Every edge links underlying relation/assertion IDs. A reason path remains a path record.

## Build algorithm

```text
validate exact project/graph view
-> enumerate exact configured roots under budget
-> select profile-declared node classes
-> create deterministic node projections
-> load only direct profile-declared relations
-> form deterministic groups from exact member sets
-> attach source/evidence/coverage/conflict indexes
-> apply mandatory and optional map tiers
-> prune optional nodes/edges at item boundaries
-> emit omission and budget records
-> canonicalize and validate
```

No whole graph is loaded to build a small map when bounded owner queries exist.

## Mandatory content

- exact project/source/graph/reference binding;
- root project node;
- selected package and TOC/load identity needed to interpret included source;
- every included node's exact underlying key;
- every edge's underlying graph records;
- coverage/conflict/partial/truncation state;
- omitted-profile and budget summaries;
- deterministic ordering/profile/schema IDs.

## Grouping

Profiles use closed typed rules such as:

```text
same exact package
same exact file
same universal role key from accepted recognizer evidence
same direct state root
same exact native/custom signal class
same exact API namespace
```

Repository/addon/provider names, arbitrary path substrings, popularity, or learned clusters cannot form canonical groups.

Groups preserve member IDs and confidence. Mixed/conflicted role evidence remains conflicted.

## Ordering

A frozen order can use:

```text
project/universe order
package selected-load order
TOC/load ordinal
source file canonical key
node kind order
canonical entity/group key
relation kind/source/target/assertion ID
```

Never filesystem enumeration, SQL row ID, hash iteration, worker completion, or model rank.

## Size and continuation

Profiles set hard limits for nodes, edges, groups, members, evidence refs, bytes, and optional tokens. At a limit:

- mandatory closure remains;
- optional items are pruned deterministically;
- counts state selected versus total-known under coverage;
- omissions identify scopes/classes;
- continuation binds exact snapshot/request/profile/ordering;
- a truncated map cannot claim complete map projection coverage.

## Combined map

A combined map is an outer reference structure:

```text
CombinedProjectMap
    primary user ProjectMapId
    optional Blizzard UI ProjectMapId
    exact cross-universe relation/path items
    compatibility, coverage, conflict, omission, and budget records
```

It does not copy both maps into one namespace or generic hierarchy.

## Map updates and cache

Map identity binds exact project, graph, reference, context profiles, and selected roots. Any relevant change creates a new ID.

Physical cache reuse is external. A prior map is never relabeled.

## Validation

- every node, edge, member, and facet origin resolves;
- no cross-universe key collapse;
- no unsupported class;
- no direct edge backed only by a path;
- group membership/counts close;
- confidence/coverage/conflict preserved;
- budget/omission totals reconcile;
- deterministic bytes under randomized input order;
- no free prose or source text in framework-control facets.
