# E3-B coverage, conflicts, confidence and authority ceilings

**Status:** normative.

## Independent axes

Keep these states separate:

```text
provider/revision verification
build/profile binding
materialization/root/file coverage
TOC/XML/load parsing coverage
Lua analyzer/fact coverage
recognizer rule/partition coverage
graph proposal/publication/query coverage
reference/source bridge coverage
license/redistribution coverage
store/read-back integrity coverage
publication eligibility
```

A complete axis cannot overwrite an incomplete or conflicted axis.

## Source ingestion coverage

Partitions:

- provider object/revision enumeration;
- configured root admission;
- file path/content materialization;
- file kind/encoding classification;
- package/global-unit boundaries;
- symlink/submodule/LFS/archive decisions;
- license/redistribution evidence;
- build-binding evidence.

`Complete` means complete for the exact profile's intended admitted universe, including explicit reviewed exclusions. It does not mean every file that Blizzard might possess or every client runtime artifact is present.

## Parser/analyzer coverage

Each TOC/XML/Lua unit records:

- parsed/analyzed capability partitions;
- unsupported constructs and raw records;
- syntax/semantic/source-map failures;
- partial/truncated/cancelled state;
- downstream fact/recognizer/graph capabilities affected.

A malformed unit cannot silently disappear from the source universe.

## Recognizer coverage

Recognizer coverage states which exact rule versions and fact partitions were evaluated. `EvaluatedNoMatch` under complete closed scope is distinct from `NotEvaluated`, `Partial` and `Failed`.

Removing or disabling a recognizer removes only its output partition and reduces coverage; direct source facts remain.

## Graph coverage

Graph coverage distinguishes:

- proposal validation;
- accepted/rejected assertion partitions;
- registry kind/relation support;
- endpoint/index closure;
- graph conflicts;
- publication/read query validation;
- bounded query truncation.

A successfully persisted graph does not make incomplete source/analyzer/recognizer coverage complete.

## Bridge coverage

For each bridge rule/partition:

- source endpoint fact coverage;
- reference endpoint lookup/negative-authority coverage;
- profile/build compatibility;
- alias/namespace/member resolution coverage;
- ambiguity/conflict state;
- graph validation/publication coverage.

A missing bridge can mean no match, partial inputs, incompatible profile, ambiguity, conflict or unsupported rule. Those states are not collapsed.

## Confidence

```text
Proven
    direct source or reference fact under its own authority class

Derived
    deterministic source/graph/bridge conclusion over adequate exact facts

Possible
    structurally plausible but unresolved/ambiguous/dynamic

Candidate
    later discovery hypothesis; excluded from production E3-B source graph by default
```

Cross-universe bridges are normally `Derived` at best. Aggregation cannot promote confidence.

## Negative authority table

### “Source entity is absent”

Allowed only for the exact admitted source universe/entity kind/query when root/file/parser/analyzer/graph coverage is complete and no conflict/truncation applies.

It does not imply the entity cannot exist in runtime-generated code, another build, omitted source, native code or another universe.

### “Public API is absent”

Never derived from UI source. Only exact reference negative authority may support this claim.

### “Source does not use API X”

Allowed only under exact complete source/analyzer/call-reference/graph coverage for the declared closed scope. It says nothing about runtime/native/dynamic usage outside that capability.

### “Project does not hook/inherit UI entity X”

Not an E3-B claim. Requires an exact user project generation, project bridge coverage and bounded/complete relation scope.

### “Behavior is safe/allowed at runtime”

Never an E3-B static source negative/positive claim. Requires owning reference/rule/runtime evidence.

## Conflict classes

```text
provider-revision-or-content-conflict
build-binding-conflict
logical-path-or-root-collision
package-or-TOC-variant-conflict
encoding/source-byte-digest-conflict
source-map-conflict
analyzer-semantic-key-conflict
recognizer-ambiguity-or-incompatible-assertion
source-graph-identity-or-relation-conflict
reference-source-endpoint-or-profile-conflict
license-or-redistribution-conflict
publication-manifest-or-store-integrity-conflict
```

Every conflict links exact evidence/assertions/records and affected capabilities. No first/last/majority/provider-popularity resolution.

## Build-binding conflicts

Examples:

- provider labels revision as build A while independent manifest binds build B;
- source-generated version markers disagree;
- source profile targets a reference generation with incompatible interface/build evidence;
- content manifest matches no reviewed build-binding record;
- two provider sources claim the same revision but have different bytes.

The result is `Mismatch` or explicit conflict, not nearest compatible build.

## License conflicts

If root/file license evidence conflicts or redistribution scope is unclear:

- technical indexing may continue locally under policy;
- source excerpt/pack/database/export capabilities become restricted or `NotEvaluated`;
- no artifact is labeled redistributable;
- current publication eligibility may be blocked if policy requires complete licensing;
- the conflict remains attached to every derived artifact decision.

## Capability summary

A source publication exposes a conservative summary with references to original records:

```text
source_inventory
package_toc_xml_load
lua_semantics
structural_recognizers
source_graph
reference_source_bridges
source_excerpt_local
source_excerpt_redistributable
pack_or_database_redistributable
build_exactness
current_publication_eligibility
```

Summary cannot hide the worst relevant partition or replace detailed records.

## Publication eligibility

A policy may require:

- minimum build-binding state;
- complete mandatory source roots and file content;
- complete mandatory parser/analyzer/graph capabilities;
- no unresolved identity/build/store conflicts;
- resolved license state for the intended local/release channel;
- successful inactive read-back validation.

A candidate can be valid for local exploratory analysis but ineligible for production-current or redistribution.

## Partial publication

A `PartialCandidate` can publish only to an explicitly partial/local channel whose selector and current record expose all incomplete/conflicted state. It cannot replace an exact production-current publication unless policy explicitly accepts every blocker.

## Last-known-good disclosure

If a selected exact target is unavailable and a caller explicitly permits fallback, return:

- requested profile/build/source selector;
- actual last-known-good source publication IDs;
- exact mismatch/build-binding/license/coverage reasons;
- unavailable bridge/query capabilities.

Never relabel or silently substitute.

## Authority explanation

Every source/bridge query can return:

- exact universe/generation/profile;
- input source/reference entities;
- source/evidence/provenance/confidence;
- coverage/conflicts/truncation;
- build-binding and license states;
- producer partition and validation report;
- whether any negative statement is authoritative for the exact capability.
