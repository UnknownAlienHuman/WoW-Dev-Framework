from pathlib import Path

path = Path("crates/wow-recognizers/src/facts.rs")
text = path.read_text(encoding="utf-8")
old = "        let mut canonical_coverage = Vec::with_capacity(coverage.len());\n"
new = (
    "        let mut canonical_coverage: Vec<RecognizerFactCoverage> =\n"
    "            Vec::with_capacity(coverage.len());\n"
)
if text.count(old) != 1:
    raise SystemExit("expected exactly one canonical coverage accumulator")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
