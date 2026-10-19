from pathlib import Path

path = Path("crates/wow-project/src/replay/publication.rs")
text = path.read_text()
anchor = """fn validate_native_binding(
    project: &ProjectView,
"""
helper = """fn validate_read_native_parts(
    project: &ProjectView,
    graph: &ValidatedGraphPartitionSnapshot,
    native: &crate::graph::PlatformIndexSourceParts,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    validate_native_binding(project, graph.as_snapshot(), native, stop)
}

"""
if text.count(anchor) != 1:
    raise SystemExit(f"expected one insertion anchor, found {text.count(anchor)}")
if "fn validate_read_native_parts(" in text:
    raise SystemExit("helper already exists")
path.write_text(text.replace(anchor, helper + anchor))
