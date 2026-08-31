# E3-A coverage, authority, conflicts, and negative claims

**Status:** normative.

## Authority separation

```text
Blizzard UI source snapshot
    exact implementation-source observations for pinned bytes

Reference Pack
    API/type/restriction/profile contract authority

User project snapshot
    exact addon source observations

Runtime observations
    client-session behavior only

External/community/model evidence
    candidate/corroborating evidence only
```

E3-A never promotes source implementation observations into another authority class.

## Coverage dimensions

Record independently:

```text
materialization/inventory
path/byte decoding
package discovery
TOC variant selection and parse
file/dependency resolution
XML/include/template/object/script parse
Lua unit/source-map construction
analyzer snapshot/fact/finding
fact adapter
recognizer rule/input scope
project graph proposals
graph acceptance/rejection/conflict
fingerprint projection
skeleton-input projection
store build/publication/read validation
license/provenance/redistribution
```

A top-level summary references exact underlying coverage records and preserves the worst relevant state.

## Coverage states

Use `wow-core` canonical states and blockers. At minimum distinguish:

```text
Complete
Partial
NotEvaluated
Failed
Cancelled
Truncated
Conflict
NotApplicable
```

Do not turn `Unknown`, parser recovery, unsupported syntax, unresolved dependency, or rejected graph proposal into complete coverage.

## Source-inventory completeness

`Complete` inventory means every entry under configured roots is accounted for by the materialization profile. It does not mean every entry parsed semantically, every dynamic edge resolved, or every runtime behavior known.

## Semantic negative claims

E3-A may make a scoped negative structural statement only when the exact relevant closed scope is complete and unconflicted. Examples:

```text
no selected TOC entry directly lists file X
no exact XML template declaration with key Y exists in the configured source snapshot
no accepted direct relation of kind K exists for exact entity E under all relevant complete producer partitions
```

Even then, the result is limited to the exact static source generation and relation capability. It does not prove runtime absence or absence in another flavor/build/source.

## Non-authoritative absence

Return explicit partial/NotEvaluated when:

- source inventory incomplete;
- TOC variant ambiguous;
- parser/analyzer partition failed or truncated;
- dynamic/generated relation outside capability;
- producer coverage incomplete;
- graph proposal rejected/endpoint unresolved;
- conflict affects the subject;
- query budget truncates traversal;
- source universe/profile mismatch.

An empty SQL/query result is never enough.

## Conflicts

Conflict classes include:

```text
source/build/profile incompatibility
path/case/content identity collision
multiple compatible TOC variants
package/dependency/load cycle or multiplicity conflict
XML template/object target ambiguity
analyzer/source-unit mismatch
overlapping incompatible producer assertions
graph registry/endpoint/attribute conflict
license/provenance disagreement
publication/read-back mismatch
```

Every conflict identifies exact records, evidence, affected capabilities, and resolution state. No first/last/majority selection.

## Confidence

- direct exact source/TOC/XML facts may be `Proven` within the source evidence class;
- deterministic adapters/recognizers are `Derived` under their contracts;
- dynamic/ambiguous plausible relations are `Possible`;
- E3-A does not emit model/external `Candidate` facts.

Source `Proven` does not mean platform-contract or runtime-proven.

## Downstream preservation

Graph, skeleton-input, Project Map, L0/L1, context packs, search, and service results must retain or link the original authority, evidence, confidence, coverage, conflict, and generation. Rendering or ranking cannot upgrade them.
