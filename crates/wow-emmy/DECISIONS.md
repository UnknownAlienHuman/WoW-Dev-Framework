# `wow-emmy` E0-C decisions

**Status:** normative for the E0-C adapter slice.

## EMMY-001 — One upstream adapter, no default fork

**Decision:** all upstream EmmyLua Rust integration is isolated behind one framework adapter.

**Consequence:** downstream crates never import upstream analyzer types directly, and pin updates affect one boundary.

## EMMY-002 — Documentation does not select the final pin

**Decision:** E0-C contract preparation records pin/probe requirements but does not promote the historical research revision automatically.

**Consequence:** the implementation agent must inspect current upstream source, choose an exact commit, and commit a compatibility report before code activation.

## EMMY-003 — Mandatory compatibility probe precedes activation

**Decision:** no candidate pin becomes active solely because it compiles.

**Consequence:** configuration, workspaces, updates, diagnostics, source spans, semantic facts, incrementality, determinism, and fixture behavior must pass the probe.

## EMMY-004 — Previous pin remains last-known-good

**Decision:** a new candidate that loses a mandatory capability or changes unclassified blocking behavior is rejected.

**Consequence:** dependency upgrades are reversible and cannot silently alter the analyzer contract.

## EMMY-005 — `wow-emmy` depends directly only on `wow-core`

**Decision:** E0-C does not depend on `wow-reference`, `wow-project`, `wow-rules`, or service/transports.

**Consequence:** facts and findings are transport-independent and platform-neutral; higher layers perform joins/orchestration.

## EMMY-006 — Project generation is supplied, not invented

**Decision:** the adapter validates and carries a caller-supplied `ProjectGenerationId`; it owns only analyzer-session/snapshot identity.

**Consequence:** `wow-project` remains the future owner of coherent repository generation publication.

## EMMY-007 — One mutable actor publishes immutable snapshots

**Decision:** writes to one upstream analysis instance are serialized by an actor/owner. Readers use immutable snapshot views.

**Consequence:** one response cannot mix pre/post-update facts or diagnostics.

## EMMY-008 — Main and library workspaces are distinct

**Decision:** first-party source and annotation/library source are registered with distinct roles.

**Consequence:** library declarations aid resolution without being treated as project-owned implementation files/findings.

## EMMY-009 — Full Blizzard UI is excluded from the ordinary library

**Decision:** E0-C loads only the narrow fixture annotation library.

**Consequence:** performance/noise/context remain bounded and the analyzer does not treat implementation bodies as ordinary project libraries.

## EMMY-010 — No editor mutation

**Decision:** analyzer correctness is derived from explicit adapter configuration; user/workspace editor settings are never changed.

**Consequence:** CLI/tests/agents/editors observe the same analysis contract.

## EMMY-011 — Canonical coordinates are UTF-8 byte half-open ranges

**Decision:** all public source spans use `[byte_start, byte_end)` against the exact content digest.

**Consequence:** upstream/LSP coordinate systems are converted in one tested boundary and cannot leak into consumers.

## EMMY-012 — Public facts are normalized and upstream-independent

**Decision:** `wow-emmy` emits owned fact types for references, calls, bindings, operations, guards, and control-flow relations.

**Consequence:** downstream code does not bind to upstream CST/semantic-model internals.

## EMMY-013 — Facts do not contain WoW authority conclusions

**Decision:** analyzer facts state what source resolves/contains/does syntactically or semantically; they do not state build availability, Secret status, hook safety, or replacement.

**Consequence:** `wow-rules` must join analyzer facts with exact reference/project evidence.

## EMMY-014 — Generic diagnostics may become core findings

**Decision:** built-in upstream diagnostics are normalized into `wow-core Finding` records with project/analyzer evidence only.

**Consequence:** generic and WoW findings can later share one envelope without conflating provenance.

## EMMY-015 — Stable framework category and upstream diagnostic ID coexist

**Decision:** normalized diagnostics retain both the framework semantic category and exact upstream diagnostic code/version.

**Consequence:** message/ID drift is observable, while the framework can test stable behavior categories.

## EMMY-016 — Diagnostic text is not identity

**Decision:** finding identity/order/deduplication use structured category/code/span/arguments/context, not rendered message text.

**Consequence:** localization/wording changes cannot silently change semantics.

## EMMY-017 — New upstream diagnostics start unclassified

**Decision:** a candidate pin cannot introduce new blocking behavior automatically.

**Consequence:** new families are shadowed/reported until fixtures and policy classify them.

## EMMY-018 — E0 generic diagnostic uses a dedicated file

**Decision:** the selected generic diagnostic fixture is separate from API/Secret-flow files.

**Consequence:** a deliberate parse/type error cannot prevent facts needed by other E0 rules.

## EMMY-019 — Secret metadata remains outside annotations

**Decision:** the E0 annotation fixture declares ordinary Lua signatures only; canonical `secret.return` comes from `wow-reference`.

**Consequence:** analyzer type projection cannot become platform restriction authority.

## EMMY-020 — Local-flow facts are bounded to the current function

**Decision:** E0-C extracts only direct local producer/binding/use/guard/control-flow facts needed by the E0 Secret rule.

**Consequence:** no broad interprocedural or dynamic-callback flow engine is built in E0.

## EMMY-021 — Guard facts are structural, not safety verdicts

**Decision:** the adapter may report a recognized guard call and proven dominance relation, but not that the use is safe.

**Consequence:** `wow-rules` applies selected-profile restriction semantics and required-capability policy.

## EMMY-022 — Unresolved reference is not platform absence

**Decision:** `C_E0Fixture.RemovedApi` yields an unresolved/member-reference observation only.

**Consequence:** only `wow-reference` can establish authoritative absence in the selected profile.

## EMMY-023 — File parse failure is partitioned

**Decision:** a malformed file has failed parse/fact capabilities; unaffected files may remain usable only if the snapshot proves their currency.

**Consequence:** no fake facts from malformed source and no unnecessary total-workspace failure.

## EMMY-024 — Annotation-library failure is a root cause

**Decision:** library load/index failure blocks dependent resolution facts and is represented separately from downstream unknown-global symptoms.

**Consequence:** service can fold noise under the actual root cause.

## EMMY-025 — Session corruption is fatal

**Decision:** upstream panic/poisoned state/contract violation invalidates the session snapshot.

**Consequence:** uncertain data is never published as partial success.

## EMMY-026 — Incremental reuse requires proof

**Decision:** facts/diagnostics from an older snapshot may be reused only when dependency/invalidation analysis proves they remain current.

**Consequence:** “last known good” never masquerades as the new generation.

## EMMY-027 — Canonical outputs exclude volatile state

**Decision:** wall-clock time, memory addresses, temp paths, worker scheduling, and hash iteration do not enter canonical facts/findings/snapshot digests.

**Consequence:** repeated runs serialize byte-identically.

## EMMY-028 — E0-C is synchronous at the domain boundary

**Decision:** the actor may use internal synchronization, but the domain contract does not require async/network/process behavior.

**Consequence:** no async runtime is introduced without an actual E0 need.

## EMMY-029 — No external diagnostic plugin system in E0

**Decision:** WoW providers remain higher-layer code invoked after analyzer facts/diagnostics.

**Consequence:** product delivery does not depend on an upstream plugin API or dynamic Rust ABI.

## EMMY-030 — Fixture byte freeze precedes implementation

**Decision:** closed workspace/fact/diagnostic examples list member files now; actual canonical byte digests are frozen after E0-A canonicalization exists and before first Rust code.

**Consequence:** documentation does not invent unverifiable hashes, but implementation cannot begin with mutable fixtures.

## EMMY-031 — Current KB routes remain external

**Decision:** current WoW patch/security rules are linked, not copied into analyzer contracts.

**Consequence:** `wow-emmy` stays generic while current game guidance evolves independently.
