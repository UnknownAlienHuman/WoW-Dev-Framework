# `wow-reference` E0-B decisions

**Status:** normative for the E0-B fixture slice. A later change must update this register and affected fixtures in the same commit.

## REF-001 — E0-B uses a closed synthetic fixture catalog

**Decision:** the E0-B model uses the project-owned `C_E0Fixture` catalog, bound to an explicit Retail build context.

**Reason:** the vertical slice must test exact reference semantics without freezing a large or hotfix-sensitive subset of live Blizzard data into unit fixtures.

**Consequence:** fixture facts prove implementation behavior, not broad API currency.

## REF-002 — Fixture profile and release profile are distinct classes

**Decision:** `profile_kind = fixture` is part of profile identity and cannot satisfy a release-grade profile requirement.

**Consequence:** applications and tests cannot accidentally present the minimized E0 catalog as a complete Reference Pack.

## REF-003 — Raw canonical records precede typed lowering

**Decision:** supported source registrations first become canonical raw-value records. Typed API/facet facts are projections from those records.

**Consequence:** unknown fields survive even when the typed model cannot interpret them.

## REF-004 — Arbitrary Lua is never executed

**Decision:** the evaluator accepts a small declarative allow-list and rejects/quarantines every other construct.

**Consequence:** no Lua runtime, file IO, module loading, metatable execution, or source callback is required or permitted.

## REF-005 — Input order and semantic registration order are separate

**Decision:** filesystem/enumeration order is canonicalized, while explicitly declared TOC/registration order remains semantic input.

**Consequence:** randomized file discovery cannot change results, but a deliberate registration-order fixture can.

## REF-006 — Reference facts are profile and generation bound

**Decision:** every model, view, symbol, facet, evidence record, conflict, and coverage record belongs to one exact fixture profile and reference generation.

**Consequence:** cross-profile lookup is rejected rather than treated as a miss.

## REF-007 — Exact lookup has no fallback lanes

**Decision:** `lookup_symbol_exact` performs only canonical exact-key resolution for the selected profile.

**Consequence:** alias, fuzzy, lineage, semantic, and replacement discovery cannot alter E0 lookup outcomes.

## REF-008 — A lookup result is never a bare optional

**Decision:** exact lookup returns a typed state containing profile, generation, coverage, evidence/conflict references, and negative-authority status.

**Consequence:** `None` cannot be misinterpreted as proof of absence.

## REF-009 — Negative authority is delegated to `wow-core`

**Decision:** `wow-reference` emits exact coverage/conflict inputs and invokes the core authority decision contract.

**Consequence:** the crate does not maintain a second, weaker absence boolean.

## REF-010 — Coverage is partitioned narrowly

**Decision:** coverage records are emitted per producer, capability, and partition.

**Consequence:** one unsupported record can degrade its system/facet partition without invalidating unrelated fixture facts.

## REF-011 — Complete ingestion does not resolve conflicts

**Decision:** source ingestion may be complete while the normalized contract remains conflicted.

**Consequence:** affected lookups return `Conflict` or dependent evaluation becomes `NotEvaluated`; input order never chooses a winner.

## REF-012 — Duplicate equality and duplicate conflict differ

**Decision:** byte/logically equivalent duplicate registrations are classified separately from incompatible registrations.

**Consequence:** harmless duplication may be retained as provenance, while conflicting duplication blocks dependent authority.

## REF-013 — `secret.return` is first-class normalized data

**Decision:** the E0 Secret producer stores an explicit restriction facet independent of annotation text.

**Consequence:** `wow-rules` can consume a typed facet with exact evidence/coverage, and annotation generation remains deferred.

## REF-014 — E0 Secret semantics are synthetic and unconditional

**Decision:** `C_E0Fixture.SecretText` has an unconditional fixture-only `secret.return` facet.

**Consequence:** E0 tests guard local-flow plumbing without claiming real runtime spell/API secrecy.

## REF-015 — Unknown fields are preserved, then classified

**Decision:** every unknown field round-trips in raw canonical form and receives a capability-impact classification.

**Consequence:** unaffected typed facts may remain usable, while dependent capabilities become partial/unknown as appropriate.

## REF-016 — Reference evidence never owns project locations

**Decision:** reference source handles point only to registered reference/fixture inputs.

**Consequence:** addon use-site spans are supplied by `wow-emmy`/`wow-project`; `wow-rules` joins them with reference evidence.

## REF-017 — E0-B has no persistence

**Decision:** the fixture model/view is in-memory or loaded from closed fixture files. `wow-store` is not activated.

**Consequence:** no SQLite schema, migration, WAL, or object-store abstraction is introduced in E0-B.

## REF-018 — E0-B has no correction engine

**Decision:** correction APIs are absent or explicitly unsupported in E0-B.

**Consequence:** a placeholder correction path cannot return fake success. E1 must implement digest-bound reviewed corrections as a separate slice.

## REF-019 — Canonical output excludes volatile metadata

**Decision:** timestamps, temporary paths, worker IDs, and discovery order do not participate in canonical fixture/model identity.

**Consequence:** repeated builds of equivalent logical input are byte-identical.

## REF-020 — Closed fixture checksums are normative

**Decision:** the fixture bundle and lookup cases include declared SHA-256 digests/checksums.

**Consequence:** implementation tests reject accidental fixture drift unless contracts and expected digests are intentionally updated together.

## REF-021 — E0-B does not create findings

**Decision:** the crate returns reference lookup/facet/coverage data only.

**Consequence:** diagnostic severity, project primary spans, root-cause folding, and result envelopes remain owned by higher layers.

## REF-022 — E1 behavior is documented but inactive

**Decision:** README references full Reference Pack responsibilities, but E0 implementation exposes only the fixture seam.

**Consequence:** agents must not prebuild acquisition, full APIDoc ingestion, FrameXML, corrections, annotations, storage, or lineage.

## REF-023 — Current KB remains the live WoW route

**Decision:** patch-sensitive API/security claims remain in the separate knowledge base and pinned release inputs, not copied into this fixture contract.

**Consequence:** the fixture remains stable while live guidance can evolve independently.
