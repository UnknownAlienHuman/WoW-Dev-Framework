from __future__ import annotations

from pathlib import Path
import subprocess


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


lib = read("crates/wow-core/src/lib.rs")
for line in [
    "pub mod contract;\n",
    "pub mod envelope;\n",
    "mod envelope_decode;\n",
    "pub use contract::E0_OPERATION_IDS;\n",
    "pub use envelope_decode::E0DecodeLimits;\n",
]:
    if lib.count(line) != 1:
        raise SystemExit(f"lib.rs missing {line!r}")
    lib = lib.replace(line, "", 1)
lib = replace_once(
    lib,
    """pub use envelope::{
    E0CheckResultDraft, E0CheckResultEnvelope, E0OperationErrorEnvelope, ResultStatus,
    SchemaCompatibility, canonical_result_digest, canonical_result_order, finalize_result_envelope,
    validate_result_envelope, validate_schema_version,
};
""",
    "",
    "lib envelope exports",
)
write("crates/wow-core/src/lib.rs", lib)

write(
    "crates/wow-core/README.md",
    """# `wow-core`

`wow-core` contains small deterministic value types shared by the framework.
It has no filesystem, database, parser, graph, search, editor, network or process behavior.

## Owns

- validated identifiers, digests, profile and generation values;
- source handles and byte spans;
- evidence, findings, coverage and budget records;
- canonical JSON and typed digest helpers used by real callers;
- typed errors for those values.

## Does not own

- CLI or transport result envelopes;
- operation replay, schema registries or compatibility readers;
- publication proofs, retention histories or lifecycle orchestration;
- WoW semantics, parsing, indexing, rendering or storage.

A caller returns its own typed result. Core validates only the values it owns.
When a shared type no longer has a real caller, delete it with its exclusive tests and documentation.

Implementation is the source of truth. Read the selected module and its direct tests; do not reconstruct the retired E0 contract platform.
""",
)

write(
    "crates/wow-core/CONSUMER_GUIDE.md",
    """# `wow-core` consumer guide

Use `wow-core` only for values shared by more than one current owner.

- Construct the specific ID, digest, source, evidence, finding, coverage, budget or error type you need.
- Validate untrusted input at the boundary where it enters.
- Return the caller's own typed result; do not wrap it in a generic core envelope.
- Keep parser, graph, search, storage, transport and WoW semantics in their owning crate.
- Do not preserve removed schemas or compatibility readers. Rebuild derived data from current source/configuration.
- Add a shared type only after finding two real callers that require the same invariant.

Canonical JSON and typed digest helpers remain available for IDs that actually use them. They are not a requirement for every operation or result.
""",
)

write(
    "crates/wow-core/examples/README.md",
    """# `wow-core` test data

These JSON files are synthetic inputs for direct tests of records such as coverage, evidence, findings and budgets. They are not a public result format, compatibility promise or release artifact.

Tests may read only the fields they exercise. New code should construct the owned type directly instead of adding another all-purpose fixture or checksum manifest.
`HASH_VECTORS.json` covers the canonical JSON and typed digest helpers that remain in use.
""",
)

write(
    "crates/wow-core/tests/e0_budget_conformance.rs",
    """//! Direct budget and truncation validation.

#[path = "budget/limits.rs"]
mod limits;
#[path = "budget/support.rs"]
mod support;
#[path = "budget/truncation.rs"]
mod truncation;
""",
)
write(
    "crates/wow-core/tests/e0_finding_conformance.rs",
    """mod finding {
    mod arguments;
    mod records;
    mod support;
}
""",
)
write(
    "crates/wow-core/tests/e0_coverage_join_conformance.rs",
    """#[path = "coverage_join/evaluations.rs"]
mod evaluations;
#[path = "coverage_join/support.rs"]
mod support;
""",
)
write(
    "crates/wow-core/tests/e0_evidence_conformance.rs",
    """//! Direct evidence, conflict and derivation validation.
#[path = "evidence/conflicts.rs"]
mod conflicts;
#[path = "evidence/derivation.rs"]
mod derivation;
#[path = "evidence/records.rs"]
mod records;
#[path = "evidence/support.rs"]
mod support;
""",
)

budget = read("crates/wow-core/tests/budget/support.rs")
budget = replace_once(
    budget,
    """    Budget, BudgetLimits, CanonicalResult, ContentDigest, CoreErrorCode, CoreResult,
    E0CheckResultDraft, E0CheckResultEnvelope, GenerationContext, NegativeAuthorityDecision,
    TruncationEntry, TruncationState, canonical_json_bytes, domain_separated_digest,
    evaluate_negative_authority,
""",
    """    Budget, BudgetLimits, CoreErrorCode, CoreResult, GenerationContext,
    NegativeAuthorityDecision, TruncationEntry, TruncationState, evaluate_negative_authority,
""",
    "budget imports",
)
marker = "\npub fn draft(value: &Value) -> TestResult<E0CheckResultDraft> {"
if budget.count(marker) != 1:
    raise SystemExit("budget envelope helpers not found")
write("crates/wow-core/tests/budget/support.rs", budget.split(marker, 1)[0].rstrip() + "\n")

evidence = read("crates/wow-core/tests/evidence/support.rs")
evidence = replace_once(
    evidence,
    """    BudgetLimits, ClaimScope, ConflictId, CoreErrorCode, CoreResult, E0CheckResultDraft,
    E0CheckResultEnvelope, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    GenerationContextId, ProvenanceClass, ResultStatus, TruncationState,
""",
    """    ClaimScope, ConflictId, CoreErrorCode, CoreResult, EvidenceConfidence, EvidenceId,
    EvidenceRecord, GenerationContext, GenerationContextId, ProvenanceClass,
""",
    "evidence imports",
)
marker = "\npub fn envelope(records: Vec<EvidenceRecord>)"
if evidence.count(marker) != 1:
    raise SystemExit("evidence envelope helper not found")
write("crates/wow-core/tests/evidence/support.rs", evidence.split(marker, 1)[0].rstrip() + "\n")

finding = read("crates/wow-core/tests/finding/support.rs")
finding = replace_once(
    finding,
    """    CanonicalResult, ClaimScope, ContentDigest, CoreErrorCode, CoreResult, E0CheckResultDraft,
    E0CheckResultEnvelope, EvidenceConfidence, EvidenceId, EvidenceRecord, Finding, FindingDraft,
    GenerationContext, GenerationContextId, ProvenanceClass, Remediation, SourceHandle, WarningId,
    WarningRecord, canonical_json_bytes, domain_separated_digest,
""",
    """    ClaimScope, CoreErrorCode, CoreResult, EvidenceConfidence, EvidenceId, EvidenceRecord,
    Finding, FindingDraft, GenerationContext, GenerationContextId, ProvenanceClass, Remediation,
    SourceHandle, WarningId, WarningRecord,
""",
    "finding imports",
)
marker = "\npub fn envelope_draft(value: &Value) -> TestResult<E0CheckResultDraft> {"
if finding.count(marker) != 1:
    raise SystemExit("finding envelope helpers not found")
write("crates/wow-core/tests/finding/support.rs", finding.split(marker, 1)[0].rstrip() + "\n")

coverage = read("crates/wow-core/tests/coverage_join/support.rs")
coverage = replace_once(
    coverage,
    """    CanonicalResult, CapabilitySummary, ContentDigest, CoreErrorCode, CoreResult, CoverageId,
    CoverageRecord, CoverageStatus, E0CheckResultDraft, E0CheckResultEnvelope, GenerationContext,
    NegativeAuthorityDecision, NotEvaluatedId, NotEvaluatedRecord, TruncationState,
    canonical_json_bytes, canonical_result_order, combine_coverage, domain_separated_digest,
    evaluate_negative_authority,
""",
    """    CapabilitySummary, CoreErrorCode, CoreResult, CoverageId, CoverageRecord, CoverageStatus,
    GenerationContext, NegativeAuthorityDecision, NotEvaluatedId, NotEvaluatedRecord,
    TruncationState, combine_coverage, evaluate_negative_authority,
""",
    "coverage imports",
)
start = "\npub fn draft(value: &Value) -> TestResult<E0CheckResultDraft> {"
end = "\npub fn negative("
if coverage.count(start) != 1 or coverage.count(end) != 1:
    raise SystemExit("coverage envelope helpers not found")
coverage = coverage.split(start, 1)[0].rstrip() + "\n\n" + end.lstrip("\n") + coverage.split(end, 1)[1]
write("crates/wow-core/tests/coverage_join/support.rs", coverage)

derivation = read("crates/wow-core/tests/evidence/derivation.rs")
derivation = replace_once(
    derivation,
    """    ClaimScope, CoreError, CoreErrorCode, E0CheckResultEnvelope, EvidenceConfidence as Confidence,
    EvidenceRecord, GenerationContextId, ProvenanceClass as Provenance, canonical_json_bytes,
""",
    """    ClaimScope, CoreError, CoreErrorCode, EvidenceConfidence as Confidence, EvidenceRecord,
    GenerationContextId, ProvenanceClass as Provenance, canonical_json_bytes,
""",
    "derivation imports",
)
derivation = replace_once(derivation, "    envelope(records)??.validate()?;\n", "", "derivation envelope check")
derivation = replace_once(
    derivation,
    """    assert_error(
        envelope(invalid.clone())?,
        CoreErrorCode::EvidenceAuthorityViolation,
        "evidence_records.claim_scope",
    )?;
    // Exercise the decoded-envelope consumer, not merely the direct validator.
    let mut wire = serde_json::to_value(envelope(valid)??)?;
    wire["evidence_records"] = serde_json::to_value(invalid)?;
    let decoded: E0CheckResultEnvelope = serde_json::from_value(wire)?;
    assert_error(
        decoded.validate(),
        CoreErrorCode::EvidenceAuthorityViolation,
        "evidence_records.claim_scope",
    )?;
""",
    "",
    "derivation envelope laundering checks",
)
derivation = replace_once(
    derivation,
    "    let golden = envelope(original.clone())??.canonical_bytes()?;\n",
    "",
    "derivation golden",
)
derivation = replace_once(
    derivation,
    """        validate_evidence_derivation_graph(&order)?;
        assert_eq!(envelope(order)??.canonical_bytes()?, golden, "seed {seed}");
""",
    """        validate_evidence_derivation_graph(&order)?;
""",
    "derivation permutation envelope",
)
write("crates/wow-core/tests/evidence/derivation.rs", derivation)

canonical = read("crates/wow-core/tests/canonical/admission.rs")
canonical = replace_once(
    canonical,
    """    CoreErrorCode, CoreResult, E0CheckResultEnvelope, E0OperationErrorEnvelope, StableHandleId,
    canonical_json_bytes, canonical_json_string, domain_separated_digest,
""",
    """    CoreErrorCode, CoreResult, StableHandleId, canonical_json_bytes, canonical_json_string,
    domain_separated_digest,
""",
    "canonical imports",
)
start = "\n#[test]\nfn canonical_016_all_committed_envelopes_match_independent_raw_json_encoding()"
end = "\n#[test]\nfn canonical_017_feature_unified_number_values_keep_scalar_semantics()"
if canonical.count(start) != 1 or canonical.count(end) != 1:
    raise SystemExit("canonical envelope test not found")
canonical = canonical.split(start, 1)[0].rstrip() + "\n\n" + end.lstrip("\n") + canonical.split(end, 1)[1]
write("crates/wow-core/tests/canonical/admission.rs", canonical)

handle = read("crates/wow-core/tests/e0_handle_conformance.rs")
handle = replace_once(
    handle,
    """use wow_core::{
    ContentDigest, CoreErrorCode, E0CheckResultEnvelope, E0DecodeLimits, SourceContent,
    SourceHandle, SourceSpan, canonical_json_bytes, verify_source_handle_content,
};
""",
    """use wow_core::{
    ContentDigest, CoreErrorCode, SourceContent, SourceHandle, SourceSpan, canonical_json_bytes,
    verify_source_handle_content,
};
""",
    "handle imports",
)
start = "\n#[test]\nfn handle_019_duplicate_registry_is_rejected_before_digest_validation()"
end = "\n#[test]\nfn handle_golden_consumers_keep_the_full_original_bytes_and_digests()"
if handle.count(start) != 1 or handle.count(end) != 1:
    raise SystemExit("handle envelope-only test not found")
handle = handle.split(start, 1)[0] + end + handle.split(end, 1)[1]
handle = replace_once(
    handle,
    """fn handle_golden_consumers_keep_the_full_original_bytes_and_digests() -> TestResult {
    let limits = E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)?;
""",
    """fn handle_golden_consumers_keep_the_source_handle_bytes_and_digests() -> TestResult {
""",
    "handle golden header",
)
handle = replace_once(
    handle,
    """        let envelope = E0CheckResultEnvelope::from_json_slice(golden.as_bytes(), limits)?;
        assert_eq!(
            envelope.canonical_bytes()?,
            canonical_json_bytes(&expected)?
        );
""",
    "",
    "handle envelope decode",
)
write("crates/wow-core/tests/e0_handle_conformance.rs", handle)

profile = read("crates/wow-core/tests/e0_profile_conformance.rs")
profile = replace_once(
    profile,
    """use wow_core::{
    CoreErrorCode, CoreResult, E0CheckResultEnvelope, E0DecodeLimits, GenerationContext,
    GenerationContextId, ProfileComparison, ProfileIdentity, ProfileIdentityBuilder,
    canonical_json_bytes, compare_profile_identity, require_profile_identity_match,
    require_same_generation, validate_profile_identity,
};
""",
    """use wow_core::{
    CoreErrorCode, CoreResult, GenerationContext, GenerationContextId, ProfileComparison,
    ProfileIdentity, ProfileIdentityBuilder, compare_profile_identity, require_profile_identity_match,
    require_same_generation, validate_profile_identity,
};
""",
    "profile imports",
)
profile = replace_once(
    profile,
    """fn profile_026_resealed_context_and_raw_envelope_recheck_nested_profile() -> TestResult {
    let original: Value = serde_json::from_str(GOLDEN)?;
    let limits = E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)?;
""",
    """fn profile_026_resealed_context_rechecks_nested_profile() -> TestResult {
    let original: Value = serde_json::from_str(GOLDEN)?;
""",
    "profile test header",
)
profile = replace_once(
    profile,
    """        let bytes = serde_json::to_vec(&mutated)?;
        assert_error(
            E0CheckResultEnvelope::from_json_slice(&bytes, limits),
            code,
            path,
        )?;
""",
    "",
    "profile envelope rejection",
)
profile = replace_once(
    profile,
    """    let accepted = E0CheckResultEnvelope::from_json_slice(GOLDEN.as_bytes(), limits)?;
    assert_eq!(
        accepted.canonical_bytes()?,
        canonical_json_bytes(&original)?
    );
""",
    "",
    "profile envelope acceptance",
)
write("crates/wow-core/tests/e0_profile_conformance.rs", profile)

subprocess.run(
    [
        "git",
        "rm",
        "crates/wow-core/CANONICALIZATION.md",
        "crates/wow-core/CANONICAL_ADMISSION_RECOVERY.md",
        "crates/wow-core/CONTRACT.json",
        "crates/wow-core/COVERAGE_CONSUMERS.md",
        "crates/wow-core/COVERAGE_JOIN_RECOVERY.md",
        "crates/wow-core/DATA_MODEL.md",
        "crates/wow-core/ERROR_MODEL.md",
        "crates/wow-core/JSON_ADMISSION.md",
        "crates/wow-core/OPERATIONS.md",
        "crates/wow-core/TEST_MATRIX.md",
        "crates/wow-core/examples/e0-generation-mismatch-error.json",
        "crates/wow-core/src/contract.rs",
        "crates/wow-core/src/envelope.rs",
        "crates/wow-core/src/envelope_decode.rs",
        "crates/wow-core/tests/budget/envelope.rs",
        "crates/wow-core/tests/contract_registry.rs",
        "crates/wow-core/tests/coverage_join/envelopes.rs",
        "crates/wow-core/tests/e0_authority_security.rs",
        "crates/wow-core/tests/e0_decode_conformance.rs",
        "crates/wow-core/tests/e0_error_conformance.rs",
        "crates/wow-core/tests/e0_examples.rs",
        "crates/wow-core/tests/e0_result_semantics.rs",
        "crates/wow-core/tests/e0_schema_conformance.rs",
        "crates/wow-core/tests/finding/envelope.rs",
    ],
    check=True,
)
