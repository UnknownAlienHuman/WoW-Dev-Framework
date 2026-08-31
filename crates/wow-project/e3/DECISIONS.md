# E3-B Blizzard UI source decisions

**Status:** normative.

## UISRC-001 — Implementation source is a separate authority class

Blizzard UI implementation source records structure for one exact source snapshot. It is not APIDocumentation, public API policy, runtime observation or user project source.

## UISRC-002 — One exact materialized snapshot

E3-B accepts one closed immutable source snapshot. Floating branches, current installed client directories and online lookups are forbidden inputs.

## UISRC-003 — Acquisition is outside the indexing library

Cloning, downloading, archive extraction and provider authentication belong to an explicit materializer outside `wow-project`. E3-B validates the resulting snapshot and report.

## UISRC-004 — Provider claims are evidence, not truth

Repository/tag/branch/build labels and comments remain provider claims. Build compatibility requires an explicit binding decision and evidence.

## UISRC-005 — Build binding is an independent axis

`ExactBuildMatched`, `ProviderDeclared`, `ContentCorrelated`, `Unverified` and `Mismatch` remain distinct from source ingestion coverage and publication status.

## UISRC-006 — Exact source content outranks revision labels for identity

Provider revision is provenance. Canonical source generation binds admitted logical roots, file identities/content digests, materialization profile and parser/analyzer profiles.

## UISRC-007 — Source roots are declared by profile

Logical root roles and package boundaries are configured and reviewed. Directory names alone do not create semantics.

## UISRC-008 — A complete manifest is mandatory for complete coverage

Scanning only discovered/referenced files cannot claim complete source coverage. The materializer supplies the complete admitted inventory and explicit omissions.

## UISRC-009 — No source execution

Lua, XML scripts, TOC directives, hooks, workflows, build scripts, generators and binaries are never executed.

## UISRC-010 — `wow-emmy` remains the only Lua parser/analyzer

E3-B may adapt normalized analyzer facts but cannot implement a source-specific Lua parser, tokenizer, resolver or evaluator.

## UISRC-011 — Existing TOC/XML/load contracts are reused

E3-B extends E2-C with a source-collection profile; it does not create a second TOC/XML/load semantic model.

## UISRC-012 — Source collection can contain multiple packages and global roots

A Blizzard UI source snapshot is not forced into one addon package. Package, global/root and shared-library units remain typed and explicitly scoped.

## UISRC-013 — Source entities are generation-scoped

Functions, methods, XML templates, frames and state entities at different source generations remain distinct. Cross-build continuity belongs to lineage in E4.

## UISRC-014 — No merge by name, path or signature

An API symbol, UI source function and user project function with the same name remain separate entity keys.

## UISRC-015 — Cross-universe relations preserve endpoint authority

A `uses_api`, `hooks`, `inherits` or other bridge does not convert source structure into reference policy or project truth.

## UISRC-016 — Reference/source bridges require exact endpoints

Bridge proposals require exact reference/source profiles, entity identities, relation schema, evidence and coverage. String equality alone is insufficient.

## UISRC-017 — Project-specific bridges require a project generation

E3-B does not publish a universal assertion that every project name/signature hooks or inherits a source entity. Such bridges require an exact user ProjectSnapshot and later integration/search logic.

## UISRC-018 — Source call sites are not API declarations

A Blizzard implementation call to a symbol may support a source `uses_api` relation but not public availability, signature, restriction or supported-addon status.

## UISRC-019 — Source absence is not API absence

No file/symbol/call in the admitted UI source cannot establish that a public API does not exist.

## UISRC-020 — Static source is not runtime proof

Static source does not prove load success, event delivery, frame existence at a time, callback payload readability, Secret Value state, taint, combat legality, protected/forbidden/managed status or performance.

## UISRC-021 — Comments and documentation have bounded source provenance only

Source comments can be quoted as source text under policy; they do not become platform contracts, graph schema, agent instructions or safety verdicts.

## UISRC-022 — Universal recognizers only in E3-B

Core recognizers may run over normalized source facts. Named Blizzard/product-specific heuristics require separately reviewed calibration rules and cannot depend on paths/repository names by convenience.

## UISRC-023 — Source, recognizer and bridge partitions remain separate

Direct source facts, recognizer-derived structure and reference/source bridges use independent producer partitions so updates/removal affect only the owning producer and coverage.

## UISRC-024 — Graph validation is independent

`wow-graph` validates registry compatibility, endpoints, keys, attributes, confidence, evidence, conflicts and coverage. E3-B cannot repair rejection by weakening identity or hiding the proposal.

## UISRC-025 — Dedicated source publication

Blizzard UI source has its own ProjectId/StoreId/Graph universe/current record. It never shares a user project's current pointer.

## UISRC-026 — E2-D publication protocol is reused

Target generations commit inactive, pass fresh read-back source/graph/bridge validation and activate by stale-base CAS.

## UISRC-027 — Current selection is profile-bound

A current source publication is current only for its exact source-profile selector. No global unqualified “latest Blizzard UI source” exists.

## UISRC-028 — Last-known-good is not relabeled

A failed or incompatible target leaves the prior publication under its original generation/build/profile. Explicit fallback reports the mismatch.

## UISRC-029 — License state is independent from technical validity

A source snapshot can be technically valid but nonredistributable. A redistribution decision cannot be inferred from public visibility or provider popularity.

## UISRC-030 — Derived artifacts need their own redistribution decisions

Facts, source maps, snippets, skeletons, graph exports and packaged stores are classified separately. “Derived” does not automatically mean unrestricted.

## UISRC-031 — Default external output is handle/fact only

Without an explicit positive redistribution decision, source bytes and substantial source reconstructions remain local and are not packaged.

## UISRC-032 — Exact profile incompatibility is a blocker

Source and reference/project profiles that cannot be proven compatible remain separate and bridges become `NotEvaluated`; no nearest-build fallback.

## UISRC-033 — Unknown fields and source forms are preserved

Unsupported directives, XML constructs, source file kinds and analyzer facts are retained as raw/unknown records with narrow coverage impact rather than dropped.

## UISRC-034 — Incremental reuse requires exact dependencies

Unchanged paths, provider revisions or mtimes are insufficient. Reuse requires exact content, profile, parser/analyzer/recognizer/registry, source-map, coverage, conflict and license dependencies.

## UISRC-035 — Removed source has complete derived-data closure

A removed file/entity must disappear from TOC/XML/load/analyzer/recognizer/graph/bridge/source-map/context-index and rendered references in the target generation.

## UISRC-036 — Source generation logical identity excludes physical storage

SQLite rows/pages, WAL/checkpoints, host path, clone directory, worker count, clock and provider checkout mechanics do not enter source/project/graph semantic IDs.

## UISRC-037 — Deterministic source order is contract-defined

TOC/XML/include order and canonical semantic keys define order. Filesystem traversal, Git tree response order, hash maps and worker completion do not.

## UISRC-038 — Full-source context export is not E3-B

E3-B publishes exact source handles and graph/source views. `wow-context` consumes bounded skeleton/excerpt seams and never receives an implicit whole-source dump.

## UISRC-039 — Patch-sensitive interpretation remains routed

Current API/restriction/runtime interpretation uses exact reference/rule/runtime evidence and the external engineering KB router, not hard-coded source-path lore.

## UISRC-040 — Missing real-source evidence blocks implementation release claims

Synthetic fixtures can implement the architecture. Production-current or release-eligible claims require a separately pinned real source snapshot, build-binding proof, license decisions and frozen outputs.
