from pathlib import Path

path = Path("crates/wow-project/src/replay/publication.rs")
text = path.read_text()

names = [
    "STORAGE_SCHEMAS_V1",
    "STORAGE_SCHEMAS_V2",
    "STORAGE_SCHEMAS_V3",
    "STORAGE_SCHEMAS_V4",
    "STORAGE_SCHEMAS_V5",
    "STORAGE_SCHEMAS_V6",
    "STORAGE_SCHEMAS_V7",
    "STORAGE_SCHEMAS_V8",
    "STORAGE_SCHEMAS_V9",
    "STORAGE_SCHEMAS_V10",
    "STORAGE_SCHEMAS_V12",
    "STORAGE_SCHEMAS_V13",
    "STORAGE_SCHEMAS_V14",
    "STORAGE_SCHEMAS_V15",
    "STORAGE_SCHEMAS_V16",
]

for name in names:
    marker = f"pub const {name}:"
    if text.count(marker) != 1:
        raise SystemExit(f"unexpected declaration count for {name}")
    start = text.index(marker)
    line_start = text.rfind("\n", 0, start) + 1
    comment_start = line_start
    while comment_start > 0:
        previous_end = comment_start - 1
        previous_start = text.rfind("\n", 0, previous_end) + 1
        previous = text[previous_start:previous_end]
        if not previous.startswith("///"):
            break
        comment_start = previous_start
    end = text.index("];", start) + 2
    if end < len(text) and text[end] == "\n":
        end += 1
    text = text[:comment_start] + text[end:]

for name in names:
    line = f"    {name},\n"
    if text.count(line) != 1:
        raise SystemExit(f"unexpected catalog entry count for {name}")
    text = text.replace(line, "")

if "STORAGE_SCHEMAS_V" in text:
    raise SystemExit("legacy catalog identifier remains in publication.rs")

text = text.replace(
    "/// Exact catalogs admitted by open/recovery/migration. Never widen a retained epoch.\n",
    "/// Current catalogs admitted by ordinary open and selected component routes.\n",
)
path.write_text(text)
