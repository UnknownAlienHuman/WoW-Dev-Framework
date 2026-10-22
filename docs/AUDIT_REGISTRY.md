# Audit coverage registry

Operational ledger for continuing the source audit without repeating work or
mistaking build success for behavioral verification.

- [AUDIT.md](AUDIT.md) owns conclusions and the rewrite architecture.
- This file records **what was inspected, at what depth, with which evidence, and
  what remains unverified**.
- Normal implementation tasks do not need this file unless they continue the
  audit or change an entry below.

**Current repository baseline:** `4f429d5e41147aee5caf8dca2b67294083215562`
(`main`, 2026-10-22).

## Status vocabulary

| Mark | Meaning |
| --- | --- |
| **S** | Source chain traced through producer, handoff, reader and visible result. |
| **R** | The relevant public/runtime workflow was exercised. |
| **P** | Partially inspected; named gaps remain. |
| **U** | Not yet inspected at the required depth. |
| **N/A** | Deliberately outside the product or not applicable. |

`cargo check`, Clippy and a crate-local unit test do not upgrade a row to **R**.
A runtime claim records the exact public entrypoint, representative input and
observable result.

## Coverage matrix

| Area | Source | Runtime | Checked evidence | Remaining gap / next owner |
| --- | --- | --- | --- | --- |
| Workspace/package topology | **S** | N/A | All 16 workspace packages, manifests, dependency direction and public entrypoints; [AUDIT.md](AUDIT.md#5-package-disposition). | Recheck only when crate ownership changes. |
| Root instructions and canonical docs | **S** | N/A | `AGENTS.md`, root/docs README, Architecture, Audit, Implementation, Queue, Completion and active umbrella Issues. | 220 old Markdown files remain unreachable from canonical entries; #112 owns consolidation. |
| Native annotation acquisition | **S** | **U** | `wow-reference-builder` Git/TOC path, `wow-reference`, `wow-annotations`, workflow and old test producer; [Audit §4.1](AUDIT.md#41-generate-a-native-annotation-library). | Run supported builder through its real Library reader after #201 fixes the handoff. |
| Ketho lowering semantics | **P** | **U** | Pinned emitter paths for defaults including false/zero, arrays/stride, callbacks, receivers, ordered params/returns and source maps. | Full selected-catalog/flavor parity and representative real-source run remain unverified. |
| Builder → verifier/Library handoff | **S** | **U** | Builder emits report `/2`; current verifier expects `/1` with embedded Library; tests use another producer; limits disagree. | #201/#200: one producer, one reader, matching finite limits, no compatibility converter. |
| Addon configuration | **S** | **U** | Current CLI/Service materialized inputs and missing ordinary root/TOC/flavor/Library setup traced. | #200/#197: normal configuration, TOC discovery and defaults. |
| Shared Emmy Main+Library owner | **S** | **P** | Project cold path and shared analysis owner traced; #193 delivered one Project analysis. | No public CLI workflow proves a valid/invalid Library call through normal configuration. |
| Diagnostic collection and transport | **S** | **U** | Four-code allowlist, discarded upstream message, forced Error severity and generic LSP text traced. | #201/#198: preserve message/code/severity/range through CLI/LSP. |
| Emmy cancellation | **S** | **U** | Multi-file collector loses the operation stop state; per-file upstream call remains synchronous; #135. | Focused between-file cancellation witness; do not claim per-file preemption. |
| Built-in rules selection/status | **S** | **U** | Requested rules are filtered after provider execution; unrelated partial components can dominate status. | #201/#103: select before work and separate findings from requested-scope completeness. |
| Recognizer/matcher families | **P** | **U** | Every public built-in recognizer producer and its sole production caller under `wow-service/src/graph/build` is mapped; generic pack APIs and reopen reconstruction are traced. | Decide which typed relations survive in direct Graph/SQL consumers; no user workflow has exercised them. |
| Core/Service DTO ownership | **S** | **U** | Root Core/Service exports, exact external imports, local results, Search envelopes, canonical round-trips and self-validation are mapped below. | Remove metadata with its real consumer; frontend operation inventory remains the next audit pass. |
| Project persistence/read | **S** | **U** | Replay hydration reconstructs analysis/graph and recaptures the result; Project Store read path traced. | #131: later-process direct rows without Emmy/Graph reconstruction. |
| Store/Service lifecycle | **S** | **U** | Detailed review below: orphan lifecycle APIs separated from Search/Context/Project consumers. | #111 after #131/#169/#122 switch actual readers. |
| Search storage/query | **S** | **U** | Nested SQLite→bytes→hex→JSON→outer Store, metadata/name FTS and four pagination families traced. | #169/#106: direct body/symbol FTS after restart. |
| Context rendering/handoff | **S** | **U** | Existing excerpt reader, disconnected search path, continuation/pack platform and `\uXXXX` rendering traced. | #122: matching indexed text → readable located excerpt. |
| Source updates | **P** | **U** | Physical delta behavior and Service refusal of TOC/package plans traced. | #109: changed/deleted Lua and load selection, stale-result removal. |
| LSP | **P** | **U** | Stdio implementation and isolated one-file diagnostics traced; standard empty-result issue #198. | Configured addon/Library path, framed no-match→next-request sequence, Windows/client checks. |
| MCP | **P** | **U** | Stdio parser/dispatch and metadata/unknown-notification constraints traced. | #199: initialize→unknown notification→useful request sequence. |
| Security/root/file boundaries | **P** | **U** | Root ownership, no source execution, Git blob regular-file gap and blocking child cancellation traced. | Recheck final builder/index filesystem paths; no full threat/security review claimed. |
| Performance/resource behavior | **U** | **U** | Only structural amplification and existing counters inspected. | #113: representative real workflow, exact revision/input, reached-stage measurements. |
| Full Rust test suite | **U** | **U** | Not part of the architecture audit. | Run only when product implementation reaches the corresponding integration boundary. |
| Real addon workflow | **U** | **U** | Not exercised. | Required by #125 after Phase 1. |
| Real Blizzard UI persisted search | **U** | **U** | Not exercised. | Required by #125 after Phase 2. |
| Full donor corpus / all flavors | **U** | **U** | Not exercised. | Proportional parity only where a concrete semantic gap requires it. |
| Windows/editor runtime | **U** | **U** | Not exercised in this audit. | Required only for the final frontend claims that depend on it. |

## Completed incremental reviews

### 2026-10-22 — Store and Service lifecycle consumer map

**Baseline:** `3c5356f5` (`main`).

Inspected:

- `crates/wow-store/src/{store,model,project,search}`;
- Project lifecycle modules: backup, replacement, quarantine, recovery, release,
  retention, GC, registry, source authority and namespace;
- production callers in `wow-service` and `apps/wow`;
- tests only to distinguish a real product caller from a fixture-only obligation.

Findings:

1. **Backup/restore, replacement, quarantine and source-authority APIs have no
   production caller outside `wow-store`; their external use is test-only.** They
   remain public/exported platform surface, not a dependency of the three target
   workflows.
2. **Physical/current recovery has one product route:** the internal
   `wow project recover` Service/CLI path. It is not needed by ordinary addon
   checking or source search.
3. **Retention roots, publication release and Project GC are not orphaned yet.**
   Search and Context continuations use them to keep replay generations alive;
   LiveProject exposes their administration. Delete them only after direct
   SQLite readers remove those holds.
4. **Generic Store leases, operation journals and object GC are active because
   SearchStore persists a nested search artifact through the generic object
   store.** #169 must switch the real query path before #111 removes this layer.
5. **Project publication/read/namespace records remain widely consumed by
   Project, Graph, Search and Context.** Their replacement is #131's current
   directly readable index, not a mechanical Store-module deletion.
6. The public CLI still exposes Project publish/update/read/reconcile/recover and
   Search build/query/retire/GC administration. This confirms that the internal
   lifecycle platform is still a user-facing surface even though it is outside
   the target architecture.

Decision:

- #111 may remove backup/replacement/quarantine/source-authority first as a
  bounded orphan slice once current `main` is rechecked.
- Retention/GC/release and generic Store lifecycle must move with #131/#169/#122
  consumers; do not replace them with a second hold/journal framework.
- Keep SQLite transactions, locking, failure atomicity and protection of user
  source/configuration.

Runtime boundary: source/caller review only. No backup, recovery, GC, concurrent
reader or crash scenario was executed in this pass.

### 2026-10-22 — Recognizer and rules family map

**Baseline:** `3c5356f5` (`main`).

Inspected:

- every public `recognize_*` entry in `wow-recognizers`;
- generic fact/pack/plan/matcher and graph-handoff APIs;
- `wow-service/src/graph/build/*` callers and reopen validation;
- the complete `wow-rules` registry/provider set.

Family map:

| Family | Useful source meaning | Sole production caller | Generic platform path |
| --- | --- | --- | --- |
| Source calls | resolved calls/API uses | `graph/build/calls.rs` | direct typed recognizer result |
| CreateFrame construction | frame construction | `graph/build/construction.rs` | fixed fact bundle → generated pack → plan → matcher |
| Mixins/assignments | instance/mixin relationships | `graph/build/lua_mixins.rs` | two fixed generated packs and matcher runs |
| XML scripts | handler/receiver bindings | `graph/build/scripts.rs` | direct recognition plus a second assertion path |
| W1/W2/W3/W4/W5 signals/hooks | events, bridges, callbacks, secure hooks | `graph/build/signals.rs` | fixed packs; witness identities are recomputed on reopen |
| Source state/library | state and library relationships | `graph/build/state.rs` / `signals.rs` | library family uses a fixed W6 pack |
| State Core | read/write root/path relationships | `graph/build/state_core.rs` | fixed packs are reconstructed again in Service handoff |
| TOC | load/variant/order relationships | `graph/build/toc.rs` | adapted facts → generated pack → matcher plus assertion path |
| XML | inheritance/script/template relationships | `graph/build/xml.rs` | adapted facts → generated pack → matcher plus assertion path |

Findings:

1. **No built-in recognizer has a direct CLI, diagnostic or search caller.** Every
   public producer is consumed only by Service Graph build; user-facing consumers
   later see generic graph records.
2. **The generic pack engine is not a plugin boundary.** `run_recognizers` and
   `adapt_emmy_direct_calls` have no production caller outside the crate. External
   `parse_recognizer_pack`, `compile_recognizer_plan` and graph handoff use is
   confined to the State Core Service adapter.
3. **Fixed Rust behavior is repeatedly serialized into generic facts and packs.**
   Construction, mixins, W1–W6, State Core, TOC and XML build closed packs in
   code, compile them, execute the matcher and translate the result back into
   typed relations. There is no user-supplied recognizer path to justify this.
4. **Service reconstructs the platform after recognition.**
   `recognizer_handoff.rs` rebuilds State Core packs/plans and adapts matcher
   output to Graph; TOC/XML/scripts also keep parallel recognition/assertion
   forms. This creates additional identity mismatch states without new semantics.
5. **Read-time requalification remains.** `matcher_witness_reopen.rs` reruns
   W1/W3/W4 and library witness recognition when reopening persisted data to
   compare identities. This belongs to the same recomputation pattern as Project
   replay hydration.
6. **`wow-rules` is a general execution/evidence platform for exactly two
   descriptors.** `run_providers` always executes API Exists and Secret Local in
   sequence; selection is applied later in Service. The rule engine therefore
   adds capability/evidence/budget/report machinery but does not provide a real
   extensible rule product.

Decision:

- Preserve the typed relationships actually needed for addon diagnostics, source
  navigation and TOC/XML context.
- Replace fixed pack/plan/matcher execution with direct typed functions, beginning
  with State Core (#195), then each family only when its consumer is named (#103).
- Remove parallel assertion/witness/reopen identity paths as direct consumers move;
  do not introduce another rules DSL or registry.
- Keep ambiguity, incomplete coverage, locations and finite bounds in the direct
  result.

Runtime boundary: source/caller review only. No built-in family was exercised
through the public CLI, persisted search or editor in this pass.

### 2026-10-22 — Core and Service DTO ownership

**Baseline:** `4f429d5e` (`main`).

Inspected:

- every root export in `wow-core/src/lib.rs` and exact production imports outside
  the crate;
- Core canonical, digest, coverage, evidence, finding, generation, profile,
  source and integrity implementations;
- root `wow-service` exports and exact imports outside the crate;
- local Status/Check result construction and CLI serialization;
- Search durable response/journal paths, owner-result handoffs and all four
  pagination families;
- production conversions from owner `canonical_bytes()` back into JSON values.

Findings:

1. **`wow-core` exports 149 root names; 79 have no external production import.**
   This is API-surface evidence, not a claim that every such item is internally
   dead. Module use is concentrated as follows: callable lowering 7/8 exports,
   canonical 4/4, digest 14/21 and source 6/12 are imported externally; budget is
   0/8, error 1/7, coverage 5/19, evidence 7/17, finding 9/17, generation 3/9,
   IDs 9/18 and profile 5/9.
2. **The broadly reused Core primitives and the proof platform are separable.**
   Source paths/spans, selected IDs, content hashes and Ketho callable lowering
   have direct producer/consumer value. Coverage/evidence/finding/generation/
   profile records are concentrated in Recognizers, Rules, Graph, Project and
   Context—the platform consumers already scheduled for reduction.
3. **Core integrity validates records by serializing them to generic JSON.**
   SourceHandle, Evidence, Conflict, Coverage, Finding and Warning validation
   rebuilds JSON projections, removes or copies fields, re-hashes and compares the
   result. Surviving types should validate direct invariants; removed proof types
   must take this self-certification code with them.
4. **`wow-core::OperationId` has no external production import, while Service and
   Store each maintain their own operation identifier.** Similar public parsing
   and comparison helpers have no external production import. Do not preserve
   duplicate identifier families merely because they are re-exported.
5. **The root Service API exports 46 names, but external production code imports
   only eight:** `LocalCommand`, `LocalOperationResult`, `LocalProjectInput`,
   `OperationId`, `ServiceError`, `ServiceErrorCode`, `ServiceResult` and
   `execute_local`. The remaining root DTOs are internal orchestration surface;
   public Context/Search/Frontend submodules are reviewed separately.
6. **Local Check/Status still serialize platform state rather than a user result.**
   `ContextIdentity` carries nine identities; component snapshots carry producer,
   exact/last-good/failed identities and capability maps. Check additionally
   returns a presentation graph, rule evaluations and deferred operations. The
   CLI consumes only `LocalOperationResult`; useful diagnostics and requested
   completeness can be represented without exporting this whole model.
7. **Search still owns an independent durable envelope and response journal.**
   Each result carries schema, operation, operation ID, request digest, result ID
   and payload. This is separate from the already simplified local result path
   and remains coupled to the generic Store operation journal.
8. **Typed owner results are repeatedly serialized, parsed as JSON and serialized
   again.** An exact production-source scan found 32
   `serde_json::from_*(*.canonical_bytes())` conversions in 21 files, primarily
   Search federation/pagination/selection plus MCP Context and recognizer handoff.
   Replace each at its real caller with a typed value; do not introduce a generic
   value transport.
9. **Four `wow-search` pagination implementations run an eight-pass fixed point.**
   Delivered byte count is embedded in the page/cursor being measured. Output
   bounds belong outside a plain page result; #169/#194 own removal with the
   direct query path.
10. **Canonical JSON is too widespread for an indiscriminate purge.** The exact
    import map found `canonical_json_bytes` in 136 production files. Remove it
    where a handoff no longer needs canonical identity; retain deterministic
    persistence/identity only where a surviving consumer can name the need.

Decision:

- Keep a small Core of paths/spans, selected identifiers, finite bounds, content
  identity and callable lowering while their consumers remain.
- Make Service orchestration DTOs private or delete them as the CLI becomes
  user-oriented; preserve diagnostics, accurate locations, requested-scope
  completeness, cancellation and bounded output.
- #201/#198 fixes diagnostic meaning first. #169/#122 replaces Search/Context
  envelopes, JSON round-trips and fixed-point pages. #195/#107/#103 removes the
  evidence/finding/generation consumers, after which Core can be pruned without a
  compatibility layer. #111 removes response journals when direct SQL readers no
  longer require them.
- Serialize a plain result once at the frontend. Durable write transactions may
  keep an internal operation key, but ordinary reads do not need a public
  request/result self-identity envelope.

Runtime boundary: source/import review only. No output compatibility, oversized
response, stored-operation replay or public CLI behavior was executed in this
pass.

## Next audit passes

1. **Frontend operation inventory:** classify each CLI/LSP/MCP operation as one of
   the three workflows, necessary support, or removal candidate (#197/#198/#199).
2. **Old documentation extraction:** move the remaining useful coordinate/TOC/XML
   facts, then delete unreachable E1–E7/platform notes and repair the seven known
   Store links (#112).
3. **Runtime acceptance registry:** as implementation lands, add the exact public
   command/input/result for each **R** upgrade; never infer runtime success from
   compilation or an internal fixture.

## Update rule

Every further audit pass must change this registry in the same PR as its findings:

1. state the exact baseline;
2. name files/owners and the producer→reader chain inspected;
3. record confirmed findings separately from inference;
4. mark runtime work explicitly unverified when it was not run;
5. update the next audit pass instead of appending a second backlog elsewhere.
