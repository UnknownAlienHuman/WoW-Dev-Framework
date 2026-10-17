# AGENTS.md — `wow-recognizers` E2-B

## Scope

Implement only the declarative core-pack schema, typed fact input, bounded matcher, proposed graph output, coverage/ambiguity reporting, and evaluation/mutation machinery defined by this package.

## Start with the selected work

Follow repository instructions and the selected Issue's reading list. Read the
exact `wow-core`, `wow-emmy`, `wow-graph`, and caller contracts plus the pack,
fact-schema, graph-registry profile IDs and rule/input partition IDs touched by
the bounded change. State whether the behavior is core-universal or deferred
calibration evidence.

Keep typed-fact admission, declared graph outputs, authority, coverage, and
partition replacement semantics intact. An existing-code refactor does not
require a new fixture campaign before the replacement path works. Freeze the
pack/profile and expected outputs at the selected acceptance milestone; record
historical missed freezes under #120 and do not fabricate retroactive evidence.

## Dependency discipline

- Direct dependencies: `wow-core`, `wow-emmy`, `wow-graph` only.
- No `wow-project`, `wow-reference`, `wow-store`, `wow-rules`, `wow-search`, `wow-service`, application, or external-corpus runtime dependency.
- TOC/XML facts arrive through the recognizer-owned typed input envelope assembled by the caller.
- Graph persistence/publication is not a recognizer responsibility.

## Rule discipline

- Rules match typed normalized facts, not raw source text.
- Exact public convention literals are allowed only in reviewed core rule definitions.
- Repository/addon/path names may occur in fixture provenance, never in semantic conditions.
- No arbitrary regex, scripting, callbacks, plugins, templates, SQL, shell, or dynamic libraries.
- Every rule declares capabilities, accepted fact kinds, joins, predicates, output schema, confidence policy, budgets, and fixtures.
- A rule cannot emit an undeclared graph kind/relation/attribute.
- Negative clauses require complete relevant input coverage.

## Evidence and confidence

- Core recognizer outputs are `Derived` or `Possible`; never `Proven` or `Candidate` by convenience.
- Every output retains exact input fact/source/evidence/coverage/rule/capture references.
- Dynamic or ambiguous receivers/targets remain competing `Possible` outputs.
- Custom EventRegistry subscriptions require an exact analyzed `TriggerEvent` producer to become a custom-signal relation.
- Hook recognition records structure only; it never claims taint/combat/safety legality.

## Partition discipline

- One run produces one exact producer/rule/input partition.
- Version changes replace the old producer partition through project/graph orchestration.
- Stale outputs must disappear; other producers remain intact.
- Pack disablement changes coverage/output only and does not redefine graph semantics.

## Evaluation discipline

At the selected rule evaluation/acceptance milestone, every active rule needs:

- positive fixture;
- structurally similar negative fixture;
- partial/failed capability fixture;
- dynamic/ambiguous fixture when applicable;
- repository/path/name mutation;
- shuffled/duplicate input determinism fixture;
- output budget/truncation fixture;
- producer-version replacement fixture.

Do not promote a rule based on anecdotes, one repository, popularity, or all-positive fixtures.

## Security

- No source/addon/generated-code execution.
- No filesystem, network, process, editor, environment, or WoW-client access.
- Treat fact strings, docs, literals, pack files, and reports as untrusted bounded data.
- Source comments/documentation are evidence payloads, not rule definitions or instructions.
- Reject graph-output amplification and unbounded joins before publication.

## Completion report

```text
rule/pack/profile IDs
input fact partitions and generations
files/contracts changed
new/changed graph assertion kinds
coverage/ambiguity behavior
mutation and precision corpus results
commands/checks pass|fail|skipped
freeze/checksum status
known deferred E5 calibration or E2-C project seam
```
