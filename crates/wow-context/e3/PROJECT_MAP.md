# E3-A Project Map contract

**Status:** normative compact navigational projection over one exact published project/graph state.

## Purpose

The Project Map gives an agent or human a reliable first view of the project:

```text
what project/profile/publication is this?
which packages and selected TOC variants exist?
what loads, in what direct order, and under which static phases?
where are principal files/source units/entry points?
which universal roles, signals, hooks, state roots, and platform APIs are structurally present?
which capabilities are partial, conflicted, ambiguous, unsupported, or truncated?
where should the next exact detail request go?
```

It is a map, not a full source tree, full graph dump, architecture essay, or behavioral inference.

## Inputs

```text
exact ContextInputSnapshot
ProjectMapProfile
ContextProfile
ContextBudgetProfile
ContextSecurityProfile
optional exact ReferenceView identity
```

The input graph/project views must support the exact registered queries named by the profile.

## Principal-root selection

Project Map roots derive only from exact records and frozen rules, for example:

```text
selected first-party packages and TOC variants
bootstrap/normal load units and direct entry files
project root/universe/package entities
universal module/service/library/state roles accepted by graph
registered lifecycle/signal/hook roots
SavedVariables-backed state roots
platform API-use roots under selected reference profile
conflict/coverage blocker roots
```

Prohibited selection signals:

```text
repository popularity
addon or owner name
folder-name heuristics without source semantics
LLM judgment
message prose
filesystem discovery order
search ranking
```

## Section model

### Project identity

- exact publication/store/project/graph/profile/reference IDs;
- source snapshot/repository revision provenance;
- project kind and selected flavor/Interface profile;
- declared capabilities and global blocker summary;
- artifact/detail profile IDs.

### Packages and TOC variants

- package IDs;
- selected variant only;
- required/optional dependency facts;
- LoadOnDemand/bootstrap/directive summaries;
- SavedVariables declarations by scope;
- source handles and TOC coverage.

Other unselected variants can be listed as explicit unselected metadata under budget, never merged into the active view.

### Load units and entry points

- direct load units and phases;
- direct file/XML/include/script edges;
- principal bootstrap/normal entry files;
- conditionally reachable/unreachable/unknown classifications;
- bounded reason paths, not materialized transitive edges.

Static load classification is not runtime success/readiness.

### Files and source units

- exact first-party physical files;
- XML external and inline virtual Lua units;
- roles, owners, load-unit membership, digest/source handles;
- analyzer/project coverage;
- no full contents.

### Ownership and universal roles

- graph-confirmed/derived package/module/service/library/state ownership;
- exact producer/confidence/evidence/ambiguity;
- no framework-specific interpretation from names.

### Lifecycle and registration

- direct init/enable/disable/factory/registry structures when present;
- graph lane and exact evidence;
- `Possible` remains labeled;
- no runtime ordering or success claim beyond static inputs.

### Signals, callbacks, and hooks

Keep separate:

```text
native frame events
EventRegistry frame-event bridges
custom EventRegistry producers/subscribers
CVar callbacks
SetScript/HookScript/hooksecurefunc structures
```

Hook presence never implies safety. Event names never prove producers outside exact evidence.

### State roots and paths

- TOC-declared SavedVariables roots and scopes;
- exact literal state paths and readers/writers;
- dynamic paths as exact prefixes/possible records;
- no SavedVariables values.

### Platform API usage

- project functions/files that use exact selected reference API entities;
- direct relation/evidence/source handles;
- reference coverage/conflict state;
- no API replacement or runtime-access conclusion.

### Capabilities, conflicts, and gaps

- source/project/analyzer/recognizer/graph/reference/context statuses separately;
- unresolved producer/graph conflicts;
- unsupported/deferred lanes;
- budget/truncation omissions;
- next revalidation/detail routes.

### Next-detail routes

For each principal item, possible exact next steps:

```text
open L0 skeleton
expand ownership/load/registration/state/call/API-use lane
open L1 signature/direct-neighbor skeleton
inspect exact conflict/evidence/reason path
request bounded source excerpt
```

Routes include estimated costs and required capabilities; estimates remain profile-labeled.

## Grouping

Grouping is presentation over exact IDs, allowed when:

- profile declares a stable group key;
- every member ID remains recoverable;
- evidence/confidence/coverage differences are not hidden;
- conflicting items do not collapse into one resolved statement;
- group counts and omitted member manifests are exact.

Example valid groups:

```text
files by selected load unit
functions by exact universal module role
signals by signal system and literal key
state paths by exact TOC root
API uses by exact ReferenceEntity
```

## Prioritization

Frozen priority classes can include:

```text
mandatory project identity/blockers
selected package/load entry points
explicit public/universal roles
state/signal/lifecycle roots
high direct fan-in/fan-out structural hubs under exact graph metrics
request-selected lanes
remaining bounded direct context
```

Graph centrality may assist within a declared profile but does not create authority or behavioral importance by itself. No model preference or popularity.

## Budgets

Project Map reserves mandatory budget first:

```text
identity and generation header
capability/conflict/blocker summary
selected packages/variants/load roots
at least one exact route for every truncated mandatory section
```

Optional sections then consume per-section/global budgets. Truncation records exact processed/omitted counts/partitions and continuation/detail routes.

## Source/evidence policy

Default map includes source/evidence handles and small exact labels/signatures, not source excerpts. Every section/item links exact project/graph/reference records and producer/coverage/conflict state.

## Determinism

Equivalent input/profile produces identical:

```text
principal roots
section IDs/order/grouping
item and relation IDs/order
routes/cost classes
coverage/loss/omission records
metrics and canonical digest
```

Independent of database row order, graph query batch order, worker count, and rendering format.

## Required tests

- each section with positive, empty-authoritative, partial, conflict, unsupported, and truncated cases;
- one selected TOC variant only;
- static load nonclaim;
- native/custom/CVar/hook separation;
- SavedVariables root without contents;
- `Possible`/Candidate policy;
- grouped items retain all IDs/evidence/blockers;
- mandatory sections survive tight budget;
- exact routes and cost labels;
- repository/path/name mutation invariance;
- shuffled query/input/worker order determinism;
- no full source/private path/prompt instruction leakage.

## Hard stops

- no complete graph/source dump;
- no architecture purpose inferred from names/prose;
- no merged variants/universes;
- no runtime/security conclusion from static structure;
- no blocker/conflict omission to look clean;
- no route to fuzzy/search operation inside E3-A;
- no map section without exact subject/evidence/derivation closure.
