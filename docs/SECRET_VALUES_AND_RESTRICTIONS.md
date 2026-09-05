# Secret Values and restrictions

**Status: normative design**

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

## 1. Open facet model

Restriction behavior is represented by an open, versioned facet registry. Initial facets include:

```text
secret.return
secret.argument
secret.conditional
secret.predicate
secret.aspect
protected.action
hardware_event.required
combat.restriction
secure_hooks.allowed
forbidden.aspect
private_script_object
private_partition_member
```

Unknown upstream facets are preserved raw. A rule that depends on an unknown facet returns `NotEvaluated`.

## 2. Canonical versus projected data

Raw Blizzard metadata is canonical. Generated annotations are a convenience projection for static analysis and editor compatibility.

For linting, annotations may use nominal analysis types:

```lua
---@class WowSecretValue
---@class WowSecretNumber: WowSecretValue
---@class WowSecretBoolean: WowSecretValue
```

Possible projections:

- always-secret return → nominal Secret type;
- contextually secret return → ordinary/Secret union plus facet metadata;
- never-secret return → ordinary type.

This does not claim that the runtime wraps values in Lua objects.

## 3. Analysis levels

### Level 0 — contract facts

Track API argument/return facets, predicates, protected actions, combat restrictions, forbidden aspects, secure hook allowances, and build applicability.

### Level 1 — direct local flow

Within one function, track:

- Secret-producing expressions;
- access predicates and guard dominance;
- arithmetic, comparison, concatenation, table-key/index, and branch use;
- serialization, formatting, logging, and SavedVariables sinks;
- calls into arguments that reject or cannot accept Secret values;
- loss of an established access guard.

This is the MVP static rule level.

### Level 2 — bounded interprocedural summaries

Propagate only through stable direct calls and explicit summaries. Dynamic dispatch, unknown callbacks, or incomplete callees remain `Possible` or `NotEvaluated`.

### Level 3 — runtime evidence

Correlate structured client probes, taint/restriction errors, or known scenarios with exact build and context. Runtime evidence never becomes a permanent global whitelist.

## 4. Guard modeling

Recognized guards and predicates come from the selected Reference Pack and project facts, including the relevant `canaccessvalue`, `canaccessallvalues`, `issecretvalue`, `C_Secrets` predicate family, and known secure/insecure bridges for that profile.

A guard is valid only for the values and control-flow region it actually dominates. Copying, converting, `pcall`, or serializing a value does not automatically establish access.

## 5. Sinks and operations

Rules classify operations by facet and profile rather than by generic Lua type alone. Candidate unsafe uses include:

```text
arithmetic and comparison
concatenation and formatting
branch conditions
string/number conversion
serialization or logging
table keys and indexes
SavedVariables persistence
passing to a forbidden argument
inspecting a forbidden or private object as a state oracle
mutating protected attributes/scripts in an unsafe context
```

A sink may be secret-safe for one runtime/widget contract and unsafe for another. The Reference Pack stores the actual facet rather than a blanket allow/deny list.

## 6. Overlay and hook context

A callback attached to Blizzard code may receive Secret values, run in a protected chain, or touch a managed/forbidden object. Findings can combine:

- selected extension point and hook kind;
- API argument/return facets;
- local use of callback arguments;
- combat or hardware-event reachability;
- object/template restriction facets;
- load timing;
- required runtime confirmation.

Static analysis does not label a hook safe merely because `hooksecurefunc` exists. It evaluates the selected symbol and context.

## 7. Data-driven state

Spell secrecy and other runtime classifications may change through game data or hotfixes without a corresponding Lua API surface change. Therefore:

- source contracts describe mechanisms and available predicates;
- a Reference Pack may record build-scoped known data only with explicit provenance;
- runtime predicates remain authoritative for the observed build/context;
- caches require profile-appropriate invalidation events;
- static code never embeds a permanent universal spell whitelist.

Current field guidance and probe procedures belong in the external knowledge base.

## 8. Finding examples

A useful finding states:

```text
rule: wow.secret.local_operation
profile/build/generation
source and operation span
producer API/facet evidence
missing or satisfied guard
unsafe operation/sink
confidence and coverage
safe remediation class or required runtime probe
```

When the producer facet or predicate partition is incomplete, the result is `NotEvaluated`, not “safe.”

## 9. Fixtures

The launch corpus should include:

- always-secret, conditionally secret, and ordinary returns;
- dominated and non-dominated guards;
- aliases and copied values;
- nested branches and early returns;
- safe and unsafe sinks;
- unknown upstream facet preservation;
- profile changes in the same API contract;
- forbidden object access;
- protected action/combat reachability;
- false-scrub patterns such as conversion, copying, serialization, and `pcall`;
- deterministic diagnostics under partial coverage.

## 10. Scope limit

The framework cannot statically prove all dynamic Lua dispatch, secure execution, combat state, game-data flags, or client internals. It must state the gap and produce the smallest decisive runtime scenario rather than inventing certainty.
