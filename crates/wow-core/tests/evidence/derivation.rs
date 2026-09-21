use serde_json::json;
use wow_core::{
    ClaimScope, CoreErrorCode, E0CheckResultEnvelope, EvidenceConfidence as Confidence,
    EvidenceRecord, GenerationContextId, ProvenanceClass as Provenance, canonical_json_bytes,
    validate_evidence_derivation_graph,
};

use super::support::*;

const INPUTS: &str = "evidence_records.derivation_input_ids";

#[test]
fn evidence_graph_001_002_accept_empty_leaf_and_diamond_without_mutation() -> TestResult {
    validate_evidence_derivation_graph(&[])?; // Structural success is not absence authority.
    let root = leaf("root")?;
    let left = derived("left", &[&root])?;
    let right = derived("right", &[&root])?;
    let join = derived("join", &[&left, &right])?;
    let records = vec![join, right, root, left];
    let before = canonical_json_bytes(&records)?;
    validate_evidence_derivation_graph(&records)?;
    assert_eq!(canonical_json_bytes(&records)?, before);
    envelope(records)??.validate()?;
    Ok(())
}

#[test]
fn evidence_graph_003_rejects_mixed_contexts_in_connected_and_disconnected_records() -> TestResult {
    let root = leaf("root")?;
    let foreign = record_in(
        GenerationContextId::derive(&"another-context")?,
        "foreign",
        Confidence::Proven,
        Provenance::ProjectSource,
        ClaimScope::ProjectFact,
        Vec::new(),
    )?;
    let linked = derived("linked", &[&foreign])?;
    for mut records in [vec![root, foreign.clone()], vec![foreign, linked]] {
        for _ in 0..2 {
            assert_error(
                validate_evidence_derivation_graph(&records),
                CoreErrorCode::EvidenceContextMismatch,
                "evidence_records.context_id",
            )?;
            records.reverse();
        }
    }
    Ok(())
}

#[test]
fn evidence_graph_004_checks_every_confidence_edge_not_only_candidate_to_derived() -> TestResult {
    let confidences = [
        Confidence::Proven,
        Confidence::Derived,
        Confidence::Possible,
        Confidence::Candidate,
    ];
    let base = leaf("base")?;
    for parent_confidence in confidences {
        let parent = record(
            "parent",
            parent_confidence,
            Provenance::ProjectSource,
            ClaimScope::ProjectFact,
            if parent_confidence == Confidence::Derived {
                vec![base.evidence_id()]
            } else {
                Vec::new()
            },
        )?;
        // Proven may never carry derivation inputs (covered by record admission).
        for child_confidence in [
            Confidence::Derived,
            Confidence::Possible,
            Confidence::Candidate,
        ] {
            let child = record(
                "child",
                child_confidence,
                Provenance::ProjectSource,
                ClaimScope::ProjectFact,
                vec![parent.evidence_id()],
            )?;
            let records = [base.clone(), parent.clone(), child];
            let result = validate_evidence_derivation_graph(&records);
            if child_confidence < parent_confidence {
                assert_error(result, CoreErrorCode::EvidenceAuthorityViolation, INPUTS)?;
            } else {
                result?;
            }
        }
    }
    Ok(())
}

#[test]
fn evidence_graph_005_rejects_transitive_runtime_laundering() -> TestResult {
    let runtime = record(
        "runtime",
        Confidence::Proven,
        Provenance::RuntimeProbe,
        ClaimScope::RuntimeScenario,
        Vec::new(),
    )?;
    let middle = derived("middle", &[&runtime])?;
    let bridge = derived("bridge", &[&middle])?;
    let platform = record(
        "platform",
        Confidence::Derived,
        Provenance::PlatformSource,
        ClaimScope::PlatformContract,
        vec![bridge.evidence_id()],
    )?;
    let valid = vec![runtime, middle, bridge];
    validate_evidence_derivation_graph(&valid)?;
    let mut invalid = valid.clone();
    invalid.push(platform.clone());
    assert_error(
        validate_evidence_derivation_graph(&invalid),
        CoreErrorCode::EvidenceAuthorityViolation,
        "evidence_records.claim_scope",
    )?;
    assert_error(
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
    Ok(())
}

#[test]
fn evidence_graph_006_checks_all_branches_and_explicit_scenario_scopes() -> TestResult {
    // Provenance changes do not erase an explicit runtime-scenario claim scope.
    let scenario = record(
        "scenario",
        Confidence::Proven,
        Provenance::ProjectSource,
        ClaimScope::RuntimeScenario,
        Vec::new(),
    )?;
    let safe = leaf("safe")?;
    let left = derived("left", &[&safe])?;
    let right = derived("right", &[&scenario])?;
    let join = record(
        "join",
        Confidence::Derived,
        Provenance::ProjectSource,
        ClaimScope::PlatformContract,
        vec![left.evidence_id(), right.evidence_id()],
    )?;
    assert_error(
        validate_evidence_derivation_graph(&[join, safe, scenario, left, right]),
        CoreErrorCode::EvidenceAuthorityViolation,
        "evidence_records.claim_scope",
    )
}

#[test]
fn evidence_graph_007_allows_safe_platform_derivation_and_disconnected_runtime() -> TestResult {
    let platform = record(
        "source",
        Confidence::Proven,
        Provenance::PlatformSource,
        ClaimScope::PlatformContract,
        Vec::new(),
    )?;
    let projection = record(
        "projection",
        Confidence::Derived,
        Provenance::ProjectSource,
        ClaimScope::PlatformContract,
        vec![platform.evidence_id()],
    )?;
    let runtime = record(
        "runtime",
        Confidence::Proven,
        Provenance::RuntimeProbe,
        ClaimScope::RuntimeScenario,
        Vec::new(),
    )?;
    validate_evidence_derivation_graph(&[projection, runtime, platform])?;
    Ok(())
}

#[test]
fn evidence_graph_008_reports_missing_inputs_and_duplicate_ids() -> TestResult {
    let root = leaf("root")?;
    let child = derived("child", &[&root])?;
    assert_error(
        validate_evidence_derivation_graph(&[child]),
        CoreErrorCode::MissingEvidenceReference,
        INPUTS,
    )?;
    assert_error(
        validate_evidence_derivation_graph(&[root.clone(), root.clone()]),
        CoreErrorCode::DuplicateEvidenceReference,
        "evidence_records.evidence_id",
    )?;
    let mut changed = serde_json::to_value(&root)?;
    changed["producer_version"] = "0.2.0".into();
    let changed: EvidenceRecord = serde_json::from_value(changed)?;
    assert_error(
        validate_evidence_derivation_graph(&[root, changed]),
        CoreErrorCode::DuplicateEvidenceReference,
        "evidence_records.evidence_id",
    )
}

#[test]
fn evidence_graph_009_010_reports_direct_and_multirecord_cycles_before_hashes() -> TestResult {
    // No valid hash fixed point is manufactured: these are hostile wire IDs.
    // Their cycle must be diagnosed, and they must never reach the success path.
    let root = leaf("root")?;
    let a = derived("a", &[&root])?;
    let b = derived("b", &[&a])?;
    for size in [1, 2] {
        let mut wa = serde_json::to_value(&a)?;
        wa["derivation_input_ids"] = json!([if size == 1 {
            a.evidence_id()
        } else {
            b.evidence_id()
        }]);
        let mut records = vec![serde_json::from_value::<EvidenceRecord>(wa)?];
        if size == 2 {
            records.push(b.clone());
        }
        for _ in 0..2 {
            assert_error(
                validate_evidence_derivation_graph(&records),
                CoreErrorCode::EvidenceDerivationCycle,
                INPUTS,
            )?;
            records.reverse();
        }
    }
    Ok(())
}

#[test]
fn evidence_graph_011_checks_every_hash_after_successful_structural_walk() -> TestResult {
    let base = leaf("base")?;
    let mut wire = serde_json::to_value(&base)?;
    wire["producer_version"] = "0.2.0".into();
    let forged: EvidenceRecord = serde_json::from_value(wire)?;
    let child = derived("child", &[&forged])?;
    assert_error(
        validate_evidence_derivation_graph(&[forged, child]),
        CoreErrorCode::CanonicalDigestMismatch,
        "evidence_id",
    )
}

#[test]
fn evidence_graph_012_preserves_results_across_64_seeded_orders() -> TestResult {
    let root = leaf("root")?;
    let left = derived("left", &[&root])?;
    let right = derived("right", &[&root])?;
    let top = derived("top", &[&left, &right])?;
    let original = vec![root, left, right, top];
    let golden = envelope(original.clone())??.canonical_bytes()?;
    for seed in 0_u64..64 {
        let mut order = original.clone();
        let mut state = seed;
        for index in (1..order.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let other = usize::try_from(state % u64::try_from(index + 1)?)?;
            order.swap(index, other);
        }
        validate_evidence_derivation_graph(&order)?;
        assert_eq!(envelope(order)??.canonical_bytes()?, golden, "seed {seed}");
    }
    Ok(())
}

#[test]
fn evidence_graph_013_deep_chain_uses_heap_not_recursive_process_stack() -> TestResult {
    let ctx = context()?.context_id();
    let mut records = vec![leaf("base")?];
    for index in 1..16_384 {
        let input = records.last().ok_or("last evidence")?.evidence_id();
        records.push(record_in(
            ctx,
            &format!("chain{index}"),
            Confidence::Derived,
            Provenance::ProjectSource,
            ClaimScope::ProjectFact,
            vec![input],
        )?);
    }
    records.reverse();
    let expected = records.len();
    // This is an implementation-resource probe only; threads never enter core output.
    let checked = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || validate_evidence_derivation_graph(&records).map(|()| records.len()))?
        .join()
        .map_err(|_| "evidence validation thread panicked")??;
    assert_eq!(checked, expected);
    Ok(())
}

#[test]
fn evidence_graph_014_resealed_single_context_is_valid_not_a_source_certificate() -> TestResult {
    let ctx = GenerationContextId::derive(&"explicit synthetic context")?;
    let mut wire = serde_json::to_value(leaf("base")?)?;
    wire["context_id"] = ctx.to_string().into();
    let root = reseal_evidence(&mut wire)?;
    validate_evidence_derivation_graph(&[root])?;
    // Source/profile acquisition and provenance admission remain owner responsibilities.
    Ok(())
}

#[test]
fn evidence_graph_015_shared_inputs_do_not_enumerate_exponential_paths() -> TestResult {
    let mut records = vec![leaf("base")?];
    let mut previous = vec![records[0].evidence_id()];
    for layer in 0..64 {
        let mut next = Vec::new();
        for side in ["left", "right"] {
            let record = record(
                &format!("{side}{layer}"),
                Confidence::Derived,
                Provenance::ProjectSource,
                ClaimScope::ProjectFact,
                previous.clone(),
            )?;
            next.push(record.evidence_id());
            records.push(record);
        }
        previous = next;
    }
    validate_evidence_derivation_graph(&records)?;
    Ok(())
}
