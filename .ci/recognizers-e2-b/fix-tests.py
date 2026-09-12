import re
from pathlib import Path

PATH = Path("crates/wow-recognizers/tests/engine_e2.rs")
text = PATH.read_text(encoding="utf-8")

old_helper = """fn observation(
    snapshot: &GraphSnapshot,
    nodes: &[GraphNode],
    family: ObservationFamily,
    target: usize,
    origin: ObservationOrigin,
    confidence: GraphConfidence,
    evidence: &str,
    limits: RecognizerLimits,
) -> TestResult<StructuredObservation> {
    Ok(StructuredObservation::new(
        StructuredObservationInput {
            source_snapshot_id: snapshot.snapshot_id().clone(),
            family,
            from: nodes[0].node_id().clone(),
            to: nodes[target].node_id().clone(),
            origin,
            confidence,
            evidence_ids: vec![evidence.into()],
        },
        limits,
    )?)
}
"""
new_helper = """struct ObservationSpec<'a> {
    family: ObservationFamily,
    target: usize,
    origin: ObservationOrigin,
    confidence: GraphConfidence,
    evidence: &'a str,
}

fn observation(
    snapshot: &GraphSnapshot,
    nodes: &[GraphNode],
    spec: ObservationSpec<'_>,
    limits: RecognizerLimits,
) -> TestResult<StructuredObservation> {
    Ok(StructuredObservation::new(
        StructuredObservationInput {
            source_snapshot_id: snapshot.snapshot_id().clone(),
            family: spec.family,
            from: nodes[0].node_id().clone(),
            to: nodes[spec.target].node_id().clone(),
            origin: spec.origin,
            confidence: spec.confidence,
            evidence_ids: vec![spec.evidence.into()],
        },
        limits,
    )?)
}
"""
if text.count(old_helper) != 1:
    raise SystemExit("expected exactly one observation helper")
text = text.replace(old_helper, new_helper, 1)

pattern = re.compile(
    r"observation\(\n"
    r"(?P<indent>[ \t]+)&snapshot,\n"
    r"(?P=indent)&nodes,\n"
    r"(?P=indent)(?P<family>ObservationFamily::[A-Za-z]+),\n"
    r"(?P=indent)(?P<target>[0-9]+),\n"
    r"(?P=indent)(?P<origin>ObservationOrigin::[A-Za-z]+),\n"
    r"(?P=indent)(?P<confidence>GraphConfidence::[A-Za-z]+),\n"
    r"(?P=indent)(?P<evidence>\"[^\"\n]+\"),\n"
    r"(?P=indent)limits,\n"
    r"(?P<close>[ \t]*)\)"
)


def observation_call(match: re.Match[str]) -> str:
    indent = match.group("indent")
    close = match.group("close")
    return (
        "observation(\n"
        f"{indent}&snapshot,\n"
        f"{indent}&nodes,\n"
        f"{indent}ObservationSpec {{\n"
        f"{indent}    family: {match.group('family')},\n"
        f"{indent}    target: {match.group('target')},\n"
        f"{indent}    origin: {match.group('origin')},\n"
        f"{indent}    confidence: {match.group('confidence')},\n"
        f"{indent}    evidence: {match.group('evidence')},\n"
        f"{indent}}},\n"
        f"{indent}limits,\n"
        f"{close})"
    )

text, count = pattern.subn(observation_call, text)
if count != 6:
    raise SystemExit(f"expected six observation calls, found {count}")

for message in (
    "duplicate observation must fail",
    "stale observation must fail",
    "failed coverage cannot carry observations",
    "cancelled operation must fail",
):
    old = f'    .expect_err("{message}");'
    new = f'    .err()\n    .ok_or("{message}")?;'
    if text.count(old) != 1:
        raise SystemExit(f"expected one expect_err target: {message}")
    text = text.replace(old, new, 1)

PATH.write_text(text, encoding="utf-8")
