# `wow-rules`

`wow-rules` evaluates focused WoW diagnostics over facts produced by EmmyLua,
Project, and Reference. It does not parse or index source, select current
projects, mutate source, or claim runtime/client behavior. Service composes
multi-owner requests; this crate evaluates the supplied facts.

## Current route

The implemented providers are `wow.api.exists@1` and
`wow.secret.local_operation@1`. They consume exact project source locations and
reference facts; unsupported or incomplete inputs remain visible instead of
becoming clean results. The normal user path is `wow check` in
[`LOCAL_INPUT.md`](../../apps/wow/LOCAL_INPUT.md). Follow the selected rule in
`src/`, its producer facts, and its caller before changing behavior.

Keep rules small and tied to an actionable developer question. Reuse existing
Emmy/Project/Reference facts. A search candidate, text match, or static rule
result cannot establish a runtime fact, safety property, replacement, or
automatic fix. Do not add general policy, evidence-proof, rollout, or plugin
frameworks to accommodate historical contracts.

The current rule set is useful but incomplete; broader rule/provider machinery
and its cleanup remain pending in the refactoring queue. Existing excess should
not be described as already removed, and old fixture, corpus, or checksum gates
are not prerequisites for a focused change.
