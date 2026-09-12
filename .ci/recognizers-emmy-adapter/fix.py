from pathlib import Path

path = Path("crates/wow-recognizers/src/model.rs")
text = path.read_text(encoding="utf-8")
old = "fn normalize_ids(\n"
new = "pub(crate) fn normalize_ids(\n"
if text.count(old) != 1:
    raise SystemExit(f"expected one normalize_ids declaration, found {text.count(old)}")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
