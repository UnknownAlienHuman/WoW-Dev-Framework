# E3-B L0 container skeleton

**Status:** normative broad structural layer.

## Purpose

L0 provides a compact, deterministic overview of one exact container scope so an agent or later context stage can navigate without reading source bodies.

Supported scope kinds in E3-B:

```text
project
package
selected TOC variant
load unit or phase
file
XML document
namespace
module
service
library
state root
recognized structural group
```

## L0 content

Depending on scope/profile and available evidence:

```text
exact identity/universe/generation
source/package/file/load roles
selected TOC/dependency/load-order facts
direct ownership/containment/dependency edges
top-level declarations and signatures
direct exports/registrations/hooks/state roots/API namespace uses
recognized universal roles with original confidence
direct L0 child/member references and counts
source handles/declaration spans
ReferenceView links for already-resolved API entities
coverage/conflicts/NotEvaluated/omissions
estimated L1 and source-excerpt expansion costs
```

L0 does not contain function/method bodies or an unbounded member list.

## Typed sections

A profile may include ordered sections such as:

```text
Identity
LoadAndDependencies
FilesAndUnits
DeclaredStructure
ExportsAndEntryPoints
SignalsAndCallbacks
HooksAndScripts
StateRoots
ApiUsage
DirectChildren
ConflictsAndCoverage
ExpansionOptions
```

Sections are typed containers, not prose headings with invented summaries.

## Declaration summaries

A declaration summary contains:

```text
exact entity key
entity kind
canonical display/signature facets
source declaration span
owner/container refs
confidence/provenance/coverage/conflict refs
L1 availability/cost
```

Optional counts are derived from exact selected/total-known sets and state their coverage. A count under partial coverage is not presented as total project truth.

## Entry point and role selection

L0 may include a declaration as an entry point or role only when an exact fact/recognizer/graph relation supports the profile rule, for example:

- exported symbol;
- registered event/callback/script handler;
- lifecycle/factory relation;
- TOC/XML load entry;
- exact state root;
- referenced template/mixin/library/module role;
- direct API-using top-level declaration.

Names like `Init`, `Core`, `Manager`, or `Utils` alone cannot assign canonical importance or responsibility.

## Source documents and comments

L0 may include source documentation handles or short exact quotes under explicit source/privacy/license policy. Source prose remains labeled untrusted source content. `wow-context` does not rewrite it into a canonical framework summary.

## Build algorithm

```text
validate exact scope key and map reference
-> load scope identity/source/load facets
-> issue bounded direct graph/project/reference reads
-> build typed sections and declaration summaries
-> deduplicate origins/evidence
-> attach coverage/conflicts
-> calculate mandatory/optional item costs
-> prune optional members deterministically
-> emit omissions, expansion options, and budget report
-> canonicalize and validate
```

## Mandatory L0 closure

- exact scope identity and bound universe set;
- source/package/load interpretation needed for the scope;
- origins for every included declaration/edge/facet;
- coverage/conflicts/omissions/budget state;
- deterministic profile/order version;
- explicit indication when child/member lists are sampled/truncated.

If mandatory closure exceeds the L0 hard budget, return a typed budget failure; do not drop identity or evidence.

## Pagination and member sets

Large containers expose:

- exact total-known count under stated coverage;
- selected page/member IDs;
- stable continuation cursor;
- selection/omission reason;
- no hidden first-N based on DB or source enumeration.

The profile defines stable member ordering.

## L0 nonclaims

L0 never claims:

- runtime initialization success/readiness;
- architectural importance not present in exact facts;
- API/public support from implementation source alone;
- Secret/taint/protected/combat safety;
- performance or hot-path status;
- cross-build continuity;
- remediation or edit plan.

## Validation

- all scope/member/entity/relation/source/reference IDs resolve;
- no source body included as a declaration summary;
- no unsupported role inferred from a name/path;
- counts and member pages reconcile;
- direct vs path relationships remain distinct;
- partial/conflict state retained;
- mandatory items not pruned;
- canonical bytes deterministic.
