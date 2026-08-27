# Provenance, confidence, and coverage

**Status: normative**

Correctness depends on keeping several independent questions separate:

1. Where did a claim come from?
2. How strongly does the evidence support the relation or conclusion?
3. Was the relevant source partition completely analyzed?
4. Which profile and generation produced the result?
5. Is there competing evidence or a known dynamic gap?

A single boolean such as `found`, `valid`, or `safe` cannot carry this contract.

## 1. Evidence provenance

Recommended public provenance classes:

| Class | Meaning |
|---|---|
| `platform_source` | Pinned Blizzard API documentation, UI Lua/XML/TOC, transition source, or exact Reference Pack fact. |
| `project_source` | First-party addon code, TOC, XML, configuration, fixture, or generated project fact. |
| `runtime_probe` | Structured observation from an exact client build, restriction state, and scenario. |
| `curated_correction` | Reviewed correction bound to an expected upstream source digest and evidence set. |
| `differential_oracle` | Ketho, Numy, LuaLS, or another comparison result; useful for disagreement detection, not canonical by itself. |
| `external_implementation` | Pinned third-party repository behavior; implementation evidence only. |
| `semantic_candidate` | Candidate returned by Codebase Memory or another broad discovery system. |
| `historical_record` | Older build/source material used for lineage or regression reconstruction. |
| `model_inference` | Explicitly labeled reasoning not directly established by source. Never platform authority. |

The WoW engineering knowledge base may use human-facing labels such as source-confirmed, project-confirmed, runtime-confirmed, implementation-derived, historical, or unverified. Importers map those labels to provenance without changing their original meaning.

## 2. Confidence

Confidence describes the relation between evidence and conclusion, not source prestige.

| Level | Contract |
|---|---|
| `Proven` | Direct structural or explicit contractual evidence establishes the fact for the selected profile/generation. |
| `Derived` | A deterministic rule derives the fact from proven inputs; the derivation is identified and reproducible. |
| `Possible` | Static structure permits the relation, but dynamic dispatch, aliasing, load state, or incomplete resolution prevents proof. |
| `Candidate` | The item is selected for investigation by name, text, semantic, or external implementation similarity. |

Examples:

- An exact `APIDocumentation` registration is `platform_source + Proven`.
- A TOC load order computed from parsed manifests is `project_source + Derived`.
- A dynamic callback target inferred from a registry shape may be `project_source + Possible`.
- A Codebase Memory semantic hit is `semantic_candidate + Candidate` until exact verification.

No LLM judgment or name similarity upgrades a fact to `Proven`.

## 3. Coverage

Coverage applies to a named partition and capability.

Recommended statuses:

| Status | Meaning |
|---|---|
| `Complete` | All declared inputs for the partition were ingested and validated for the requested capability. |
| `Partial` | Some inputs or constructs were skipped, rejected, truncated, or unsupported. |
| `Unknown` | Completeness cannot be established from the available manifest or process state. |
| `Failed` | The partition could not be built or read reliably. |
| `NotApplicable` | The capability does not apply to this entity/query. |
| `NotEvaluated` | A rule intentionally did not run because one or more required capabilities were unavailable. |

Coverage partitions should be narrow enough to preserve unaffected functionality, for example:

```text
apidoc.system:C_UnitAuras
apidoc.events:UNIT_AURA
ui.package:Blizzard_ActionBar
ui.xml:Blizzard_AuraFrame
project.toc:Mainline
project.file:Core.lua
restriction.facet:secret.return
lineage:120001→120100
```

## 4. Negative authority

A negative result is authoritative only when:

1. the query was normalized against a known profile;
2. the relevant entity kind and partition are known;
3. the partition coverage is `Complete` for the required capability;
4. the generation is current for the requested operation;
5. no conflicting source record remains unresolved.

Otherwise return one of:

- `not_found_with_partial_coverage`;
- `profile_unavailable`;
- `partition_failed`;
- `not_evaluated`;
- `candidate_only`;
- `conflict`.

Never collapse those states into “API does not exist” or “code is safe.”

## 5. Generation identity

Each result carries the identities needed to reproduce it:

```rust
struct GenerationContext {
    profile_id: ProfileId,
    reference_generation: GenerationId,
    project_generation: Option<GenerationId>,
    external_generations: Vec<ExternalGeneration>,
    schema_versions: SchemaVersions,
    tool_versions: ToolVersions,
}
```

Readers must not combine facts from different project generations inside one answer. Long-running requests may obtain a generation lease or immutable snapshot handle.

## 6. Stable source handles

A stable handle identifies source without embedding an entire file:

```text
repository identity or reference-pack identity
revision/build/profile
normalized path
byte span and line span when known
content digest
symbol/entity key when known
```

A handle can be resolved to L0, L1, or L2 detail. External bridge results must be converted to this form before merging.

## 7. Evidence record

A relation, finding, or search signal should be able to reference:

```rust
struct EvidenceRecord {
    provenance: ProvenanceClass,
    source: SourceHandle,
    producer_id: String,
    producer_version: String,
    confidence: EvidenceLevel,
    coverage_partition: CoveragePartition,
    coverage_status: CoverageStatus,
    generation: GenerationContext,
    competing_evidence: Vec<EvidenceRef>,
    note: Option<String>,
}
```

The final public schema may use IDs and normalized tables rather than embedding every record. The semantic fields are mandatory.

## 8. Conflicts

When two sources disagree:

1. retain both evidence records;
2. identify the authority and profile of each source;
3. determine whether the disagreement is expected projection loss, stale input, parser coverage, or a true upstream inconsistency;
4. downgrade dependent results or mark them `NotEvaluated`;
5. create a reproducible triage fixture;
6. apply a curated correction only when it is digest-bound and reviewed.

Differential oracles never silently overwrite canonical source facts.

## 9. Rule capability declaration

Every diagnostic or query path declares required capabilities, for example:

```text
wow.api.arguments
    requires: apidoc.signature.complete

wow.secret.local_operation
    requires: restriction.facets.readable + emmy.expression_facts

wow.load.use_before_load
    requires: project.toc.complete + symbol.use_index
```

If a capability is absent, the rule returns `NotEvaluated` with the missing set. It does not emit a clean result or a speculative error.

## 10. Search explanations

A search hit reports the lanes and signals that affected ranking:

- exact canonical match;
- alias/deprecation/replacement edge;
- lineage evidence;
- entity kind and namespace;
- receiver/signature/restriction shape;
- package/load affinity;
- graph-neighborhood overlap;
- FTS score;
- semantic candidate score.

The explanation distinguishes authoritative signals from candidate-generating signals.

## 11. Runtime evidence

Runtime probes are structured evidence, not anecdotes. A probe record includes:

```text
client build and region/flavor
selected Reference Pack profile
addon revision
restriction/combat/group/scenario state
probe version and raw output digest
observed result
reproduction steps
```

Runtime evidence may confirm behavior for that scenario. It does not imply global static completeness and must not be used to freeze data-driven Secret status permanently.
