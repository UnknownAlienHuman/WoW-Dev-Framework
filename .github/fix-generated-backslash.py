from pathlib import Path

path = Path("crates/wow-core/src/error.rs")
source = path.read_text(encoding="utf-8")
slash = chr(92)
old = (
    f"            !value.starts_with('/')\n"
    f"                && !value.starts_with('{slash}')\n"
    "                && !value.as_bytes().get(1).is_some_and(|byte| *byte == b':')\n"
    f"                && !value.replace('{slash}', \"/\").split('/').any(|part| part == \"..\")"
)
new = (
    "            !value.starts_with('/')\n"
    "                && value.as_bytes().first().is_none_or(|byte| *byte != 92)\n"
    "                && !value.as_bytes().get(1).is_some_and(|byte| *byte == b':')\n"
    "                && !value\n"
    "                    .split(|character| character == '/' || character == char::from(92_u8))\n"
    "                    .any(|part| part == \"..\")"
)
if source.count(old) != 1:
    raise SystemExit(
        f"invalid generated path guard: expected one match, found {source.count(old)}"
    )
path.write_text(source.replace(old, new, 1), encoding="utf-8")
