# `wow-core` decisions

**Status:** normative for E0-A; implementation has not started.

These decisions refine the repository-wide architecture for the smallest shared crate. A coding agent may improve names, but it must not change the semantics below without updating this file, the affected contract, the machine manifest, and the tests in the same change.

## CORE-001 — `wow-core` is pure boundary logic

`wow-core` performs validation, normalization, comparison, conservative aggregation, canonical ordering, and deterministic hashing over values supplied by callers.

It has no filesystem, network, process, editor, database, clock, random-number, logging, telemetry, or WoW-client access. It does not require an async runtime. Any operation that needs external state belongs to the owning higher crate.

## CORE-002 — identifiers are typed and text-safe

Stable identifiers use family-specific canonical grammars. Identity-like strings are never accepted as arbitrary free text.

- Administrative identifiers such as profile, rule, producer, capability, operation, and schema IDs use lowercase ASCII.
- Source paths and entity keys preserve case and exact UTF-8 text where their domain is case-sensitive.
- Reserved floating aliases such as `current`, `latest`, `live`, `head`, `default`, and `auto` are forbidden as identity segments.
- Parsing may return a suggested canonical form, but it must never silently remove whitespace, path traversal, control characters, or ambiguous separators.

## CORE-003 — a profile label is not profile proof

`ProfileId` is a stable label. `ProfileIdentity` is the evidence-bearing structured identity.

`wow-core` validates structural consistency only. It does not know whether a particular Interface number truly belongs to a Blizzard build; that platform validation belongs to `wow-reference` and a pinned Reference Pack.

Fixture and release identities are distinct. A fixture identity cannot be promoted into a release identity merely by changing a flag.

## CORE-004 — generation IDs are deterministic and type-tagged

Reference, project, and external generations use separate ID families. They are derived from canonical generation material, not wall-clock time, process order, database row IDs, UUIDs, or random values.

A generation ID alone is not enough to combine data. The associated profile and relevant tool/schema versions must also be compatible.

## CORE-005 — SHA-256 is the E0 canonical digest algorithm

E0 uses full lowercase SHA-256 digests for content, handles, fingerprints, contexts, generations, and result digests.

- Truncated hashes are display-only and never identity.
- Git object IDs remain source revisions, not `ContentDigest` values.
- A future algorithm requires a contract/version update and migration vectors; it is not added speculatively in E0.

## CORE-006 — source identity uses repository-relative UTF-8 paths

The canonical source path is slash-separated, repository- or artifact-relative, valid UTF-8, and case-preserving.

Core path normalization:

- converts `\` separators to `/`;
- removes empty and `.` components;
- rejects `..`, absolute paths, drive/device/UNC prefixes, NUL, and control characters;
- does not resolve symlinks or inspect a filesystem;
- does not Unicode-normalize path text, because that could alter repository identity.

Non-UTF-8 repository paths are explicitly unsupported by the E0 public handle contract rather than converted lossily.

## CORE-007 — UTF-8 byte ranges are canonical spans

A canonical source range is zero-based, end-exclusive UTF-8 byte offsets.

Line and column data are presentation output outside the canonical E0 `SourceHandle`, because transports use different column encodings. Transport adapters derive LSP or editor coordinates from exact source content.

The explicit span states are:

```text
unknown
whole_file
byte_range [start, end)
```

An empty byte range is valid. An inverted range is invalid.

## CORE-008 — provenance, confidence, and claim scope remain separate

An evidence record states:

- where the evidence came from;
- which claim scope it addresses;
- how strongly it supports that claim;
- which producer created the record;
- which exact generation and coverage state apply.

Known candidate-only provenance cannot claim `Proven` or `Derived` confidence. Combining evidence records never upgrades confidence. A deterministic higher-layer derivation creates a new `Derived` record with explicit input evidence IDs and producer identity. Evidence conflicts are separate records over evidence IDs; evidence records do not point back to conflicts.

## CORE-009 — coverage and evaluation are related but not interchangeable

The public vocabulary keeps the repository-wide statuses:

```text
Complete
Partial
Unknown
Failed
NotApplicable
NotEvaluated
```

For implementation semantics:

- source/index partitions normally report `Complete`, `Partial`, `Unknown`, `Failed`, or `NotApplicable`;
- a requested rule or operation reports `NotEvaluated` when required capabilities cannot be established from those partition states;
- every `NotEvaluated` record carries the exact missing capabilities and blocking partitions;
- `NotEvaluated` is never a clean pass.

## CORE-010 — conflict is an orthogonal denial condition

An unresolved evidence conflict does not become another coverage status. It is recorded separately and denies negative authority for affected claims even when source ingestion coverage is otherwise `Complete`.

## CORE-011 — negative authority is computed, never asserted naked

The framework may state that an entity or fact is absent only after evaluating:

- exact profile and generation identity;
- relevant entity kind and query scope;
- required capability coverage;
- unresolved conflicts;
- candidate-only evidence;
- operation evaluation state.

The decision returns a typed reason. No caller stores `not_found = true` without this evaluation.

## CORE-012 — messages are projections of structured findings

A finding owns a stable rule ID, finding code, structured identity arguments, source/evidence references, severity, rollout policy, coverage, root-cause key, and remediation class.

Rendered prose is optional transport/UI output. It is not the only contract and does not define deduplication or causal grouping.

Severity and rollout policy are separate:

```text
severity: error | warning | information | hint
policy: shadow | advisory | blocking
```

## CORE-013 — semantic identity and presentation are hashed separately

Core defines separate deterministic values:

- source-handle ID;
- evidence ID;
- finding fingerprint within a generation-independent semantic scope;
- finding ID bound to one generation context;
- generation-context ID;
- canonical result digest.

Rendered/display text, notes, line hints, elapsed time, timestamps, host paths, trace IDs, and source excerpts never enter these identities or the canonical E0 envelope. They are transport presentation outside the hashed result.

## CORE-014 — canonical results contain no volatile fields

The E0 canonical result envelope itself is byte-stable. It does not contain timestamps, elapsed durations, random trace IDs, temp paths, thread counts, or host metadata.

A transport may add noncanonical telemetry outside the canonical envelope, but it cannot change the canonical digest or be mistaken for result evidence.

## CORE-015 — dynamic maps are normalized as sorted entries

Identity-critical serialized data does not use arbitrary object keys from external input. Dynamic maps such as tool versions, schemas, capabilities, and message arguments are represented as arrays of typed entries with explicit deterministic sort keys.

This avoids language-dependent map iteration and Unicode key-order differences.

## CORE-016 — no generic unbounded extension bag in E0

E0 core contracts do not contain unrestricted `extras`, `metadata`, or arbitrary JSON-value maps.

Unknown fields in the internal E0 serialized contract are rejected with a schema error. Forward-compatible public extension points may be designed later under an explicit schema/version policy.

## CORE-017 — budgets and truncation are part of truth

Limits and observed usage are structured. Truncation is explicit, identifies the affected collection/capability, and may downgrade coverage or operation completeness.

A producer must not silently drop findings, evidence, partitions, or source handles to fit an output budget.

## CORE-018 — errors, findings, and `NotEvaluated` are different channels

- **Error:** the requested boundary operation could not accept or maintain its contract.
- **Finding:** analysis completed and reported a code/project condition.
- **NotEvaluated:** analysis intentionally did not run because required capabilities were unavailable.

No conversion between these channels occurs merely to simplify transport handling.

## CORE-019 — E0 uses strict internal schema handling

Before a public interchange schema is released, E0 uses strict internal contract versions and known fixtures. Unsupported major or unknown required fields fail explicitly.

Public backward/forward compatibility policy is introduced only with the first external consumer schema under `schemas/`.

## CORE-020 — core exposes no domain workflows

`wow-core` may define generic value contracts used by API, graph, Secret, search, or runtime features, but it does not know those algorithms. It must not contain API-name lookup, Secret Value rules, graph traversal, search ranking, Reference Pack parsing, or project indexing logic.

## CORE-021 — the canonical reference graph is acyclic

Identity-bearing references are layered so a valid result can be constructed and validated without hash fixed points:

```text
source handles
→ evidence
→ conflicts
→ coverage records
→ capability summaries and NotEvaluated
→ findings and warnings
→ envelope
```

Evidence uses semantic coverage references rather than `CoverageId`, because coverage may itself reference conflicts derived from evidence. Derived-evidence input edges must also be acyclic.

## CORE-022 — coverage records and capability summaries are distinct

A `CoverageRecord` is one producer's context-bound statement about one capability partition. A `CapabilitySummary` is a conservative derived view over exact coverage records.

Summaries never replace source records, erase blockers, or acquire negative authority by themselves. Evidence names the exact capability/partition/producer statement it relied on; `NotEvaluated` and negative-authority decisions retain the exact blocking coverage records.

## CORE-023 — source location and evidence authority are separate

A `SourceHandle` identifies an immutable artifact/span. It does not, by itself, state why that source is authoritative for a claim. `EvidenceRecord.provenance`, `claim_scope`, producer identity, and coverage refs carry that role.

A finding's `primary_source_handle_id` is the location being reported, normally the project use site. Platform/reference evidence may and often must point to a different handle. For example, an API-absence finding derives from both:

```text
project-source observation of the API use
+ platform/reference catalog evidence with complete exact-lookup coverage
```

Core validates handle existence, context coherence, and record structure. Because `origin_id` is an opaque source-registry identity, the owning producer/source registry must validate that a handle is eligible for the claimed provenance before constructing the evidence record. No caller may treat the finding's primary project span as platform-contract proof.
