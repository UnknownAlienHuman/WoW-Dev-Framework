from __future__ import annotations

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


coverage = Path("crates/wow-core/src/coverage.rs")
text = coverage.read_text()
text = replace_once(
    text,
    "pub(crate) use admission::{validate_evaluation, validate_retained_inputs};\n",
    "",
    "obsolete admission re-export",
)
coverage.write_text(text)

support = Path("crates/wow-core/tests/coverage_join/support.rs")
text = support.read_text()
marker = "\npub fn negative(\n"
helper = """
pub fn assert_error<T>(result: CoreResult<T>, code: CoreErrorCode, path: &str) -> TestResult {
    let Err(error) = result else {
        return Err(format!("expected {code:?} at {path}").into());
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(path));
    error.validate()?;
    Ok(())
}

"""
text = replace_once(text, marker, "\n" + helper + "pub fn negative(\n", "direct error helper")
support.write_text(text)

evaluations = Path("crates/wow-core/tests/coverage_join/evaluations.rs")
text = evaluations.read_text()
first_envelope_check = "    draft(&value)?.finalize()?.validate()?;\n"
if first_envelope_check not in text:
    raise SystemExit("join 002 envelope check not found")
text = text.replace(first_envelope_check, "", 1)
for label, old in [
    (
        "join 007 envelope",
        """    value["not_evaluated"] = json!([evaluation]);
    value["status"] = json!("partial");
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )
""",
    ),
    (
        "join 003 envelope",
        """    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )
""",
    ),
    (
        "join 004 envelope",
        """    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.conflict_ids",
    )
""",
    ),
    (
        "join 005 envelope",
        """    reject_envelope(
        value,
        CoreErrorCode::DuplicateCoverageRecord,
        "blocking_partitions",
    )
""",
    ),
    (
        "join 006 envelope",
        """    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.capability_id",
    )
""",
    ),
    (
        "join 008 envelope",
        """    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.conflict_ids",
    )
""",
    ),
]:
    text = replace_once(text, old, "    Ok(())\n", label)
text = replace_once(
    text,
    """        reject_envelope(value, code, "evaluation.blocking_partitions")?;
""",
    "",
    "join 009 envelope",
)
text = replace_once(
    text,
    """    value["not_evaluated"] = json!([unknown]);
    value["status"] = json!("partial");
    draft(&value)?.finalize()?.validate()?;

""",
    "",
    "join 011 unknown envelope",
)
text = replace_once(
    text,
    """    value["not_evaluated"] = json!([evaluation]);
    draft(&value)?.finalize()?.validate()?;
""",
    """    assert_eq!(
        negative(&value, &evaluation)??.outcome(),
        NegativeAuthorityOutcome::NotAuthoritative
    );
""",
    "join 011 nonapplicable envelope",
)
text = replace_once(
    text,
    """    reject_envelope(
        value,
        CoreErrorCode::CanonicalDigestMismatch,
        "not_evaluated_id",
    )
""",
    """    let evaluation: NotEvaluatedRecord =
        serde_json::from_value(value["not_evaluated"][0].clone())?;
    assert_error(
        evaluation.validate(),
        CoreErrorCode::CanonicalDigestMismatch,
        "not_evaluated_id",
    )
""",
    "join 012 direct identity check",
)
if "draft(" in text or "reject_envelope" in text:
    raise SystemExit("coverage evaluations still depend on the removed envelope")
evaluations.write_text(text)
