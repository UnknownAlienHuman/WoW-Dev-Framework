# EmmyLua integration and diagnostics

**Status: normative design**

## 1. Upstream relationship

The framework embeds a pinned upstream `emmylua_code_analysis` dependency behind one adapter. EmmyLua is not forked by default.

The checked upstream source provides project analysis, configuration, workspaces, VFS updates, indexing, file identity, built-in diagnostics, and semantic model access. It does not currently expose the external diagnostic registration surface required by this product.

The v1 solution is therefore a host, not a fork:

```text
wow-emmy-check
    batch CLI/CI/agent checker

wow-emmy-ls
    thin LSP frontend

wow-emmy-core
    project actor, upstream adapter, generation publisher,
    diagnostic provider registry, result normalization
```

An optional upstream compile-time provider trait may be proposed later. Product correctness cannot depend on its acceptance.

## 2. Project actor and generations

One project actor owns one mutable Emmy analysis instance and the mutable project index. It serializes writes and publishes immutable `ProjectGeneration` snapshots.

A generation records:

- selected Reference Pack profile and generation;
- workspace roots and normalized file identities;
- file content digests;
- Emmy dependency/version and compatibility report;
- project graph/index generation;
- enabled providers and rule versions;
- capability/coverage state.

Readers obtain a generation snapshot or lease. One response never mixes diagnostics or semantic facts from different generations.

## 3. Workspace assembly

```text
main workspace
    first-party addon Lua files

library workspace
    generated WoW annotation pack
    project-declared libraries
    narrow Blizzard stubs referenced by the project

project side inputs
    TOC and XML files

excluded from normal Emmy library
    full Blizzard UI implementation tree
    unrelated installed addons
    generated source bodies not required for inference
```

Generated analysis configuration is written under:

```text
.wow/generated/<profile>/<generation>/emmyrc.json
```

The framework never mutates user or workspace editor settings to achieve correctness.

## 4. WoW dialect profile

Lua 5.1 alone is not a sufficient environment description. A versioned dialect profile includes:

```rust
struct WowDialectProfile {
    id: ProfileId,
    interface: u32,
    build: String,
    parser_version: String,
    allowed_std_globals: Vec<String>,
    removed_std_globals: Vec<String>,
    blizzard_globals: Vec<String>,
    require_like_functions: Vec<String>,
    nonstandard_symbols: Vec<String>,
    restricted_globals: Vec<String>,
    secure_environment_globals: Vec<String>,
    evidence_digest: Digest,
}
```

The profile configures both Emmy and WoW-specific analysis. A mismatch between the two is a `dialect_gap`; it is never silently resolved by disabling a diagnostic.

## 5. Provider boundary

The internal provider contract is capability-driven and side-effect-free:

```rust
pub trait WowDiagnosticProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn required_capabilities(&self) -> CapabilitySet;
    fn check(&self, ctx: &WowCheckContext<'_>, out: &mut Vec<WowFinding>);
}

pub struct WowCheckContext<'a> {
    pub file_id: FileId,
    pub source_path: &'a Path,
    pub source_text: &'a str,
    pub emmy: &'a EmmyLuaAnalysis,
    pub reference: &'a dyn ReferenceView,
    pub project: &'a ProjectSnapshot,
}
```

Providers read one immutable context and emit findings. They never mutate Emmy state, the project graph, or other providers.

Before execution, the host compares `required_capabilities` with the generation's capability report. Missing requirements produce `NotEvaluated`, not a speculative pass or error.

## 6. Diagnostic pipeline

```text
file update
→ normalize and update Emmy VFS/index
→ run built-in Emmy diagnostics
→ normalize generic findings
→ obtain exact syntax/semantic facts for affected files
→ update affected project graph/index partitions
→ run enabled WoW providers
→ merge duplicate/downstream findings by root cause
→ sort deterministically
→ publish one result envelope
```

Generic and WoW diagnostics share one generation and source coordinate system.

## 7. Initial provider families

Planned first rules:

```text
wow.api.exists
wow.api.deprecated
wow.api.arguments
wow.event.exists_payload
wow.widget.method
wow.toc.reachable
wow.load.use_before_load
wow.secret.local_operation
wow.secret.unsafe_log
wow.overlay.direct_blizzard_override
wow.framework.duplicate_registration
```

E0 implements only the minimum vertical set:

- one generic Emmy diagnostic;
- one current-profile API existence diagnostic;
- one direct local Secret misuse diagnostic.

Additional rules are not added until the result envelope, profile isolation, and deterministic golden output are proven.

## 8. Finding contract

A normalized finding includes:

```text
rule ID and version
severity and default policy
root-cause group
message and structured arguments
primary source span
related source handles
profile/reference/project generation
provenance and confidence
coverage partition/status
required capability failures
optional safe fix or investigation plan
```

A diagnostic message must not be the only machine-readable contract.

## 9. Root-cause folding

One missing or stale capability can cause many apparent errors. The host should group findings when a common root cause is known.

Examples:

- missing selected profile → suppress downstream API-not-found noise;
- failed TOC partition → report one load-graph root cause and mark reachability rules `NotEvaluated`;
- unknown Secret facet → do not emit contradictory safe/unsafe findings;
- broken annotation projection → distinguish analyzer library failure from a platform API removal.

The raw findings may remain inspectable, but the default user/agent stream prioritizes actionable root causes.

## 10. Severity and rollout

New diagnostic families begin in `shadow` or evaluation mode unless they enforce a directly proven invariant with established fixtures.

Promotion requires:

- representative positive and negative fixtures;
- false-blocking measurement on the launch corpus;
- explicit capability and coverage requirements;
- deterministic output;
- documented remediation;
- compatibility behavior across configured profiles.

A new upstream Emmy diagnostic family is also shadowed until classified. Dependency updates must not silently change the blocking surface.

## 11. Compatibility probe

Every candidate Emmy update is probed for:

- public configuration keys and actual behavioral effects;
- LuaCATS tags that influence inference;
- diagnostic IDs and default severities;
- incremental update semantics;
- deterministic pull results;
- large annotation-library performance;
- WoW fixture noise and false positives;
- source-span stability required by source handles.

Loss of a mandatory capability blocks activation. The previous pinned dependency remains last-known-good.

## 12. Autofix policy

A provider may return an automatic edit only when:

- the triggering fact is `Proven` or a deterministic `Derived` fact from complete inputs;
- the edit precondition is exact and checked at application time;
- the replacement is profile-valid and not selected by fuzzy similarity;
- source coordinates still match the generation digest;
- a post-edit diagnostic/test verifies the intended path.

Otherwise return a structured plan or ranked candidates.

## 13. Incrementality

The project actor should invalidate only affected files and graph partitions where correctness permits. Last-known-good data may remain available for unaffected partitions, but stale facts must carry their original generation and may not masquerade as current.

A parse failure in one file should not corrupt the whole project generation. The failed partition is explicit, and dependent diagnostics become `NotEvaluated`.
