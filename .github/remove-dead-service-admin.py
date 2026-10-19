from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"unexpected block count in {path}: {count}")
    file.write_text(text.replace(old, new))


replace(
    "crates/wow-service/src/lib.rs",
    """pub mod annotation_admin;
mod backend;
""",
    """mod backend;
""",
)
replace(
    "crates/wow-service/src/lib.rs",
    """pub mod reference_admin;
pub mod reference_pack;
pub mod reference_pack_admin;
mod reference_pack_license;
""",
    """pub mod reference_pack;
mod reference_pack_license;
""",
)

replace(
    "crates/wow-service/README.md",
    """The active crate includes a local project/check path, graph and search
orchestration, reference and annotation administration, and several legacy
workflow modules. The target is simple coordination: load the selected source
""",
    """The active crate includes a local project/check path plus graph and search
orchestration. Uncalled annotation/reference publication journals are removed.
The target is simple coordination: load the selected source
""",
)
replace(
    "crates/wow-service/README.md",
    """- Graph and search use cases: `src/graph/`, `src/search.rs`
- Older annotation/reference pack administration: `src/annotation_admin.rs`,
  `src/reference_admin.rs`, `src/reference_pack_admin.rs`,
  `src/reference_pack_materialization.rs`
""",
    """- Graph and search use cases: `src/graph/`, `src/search.rs`
- Builder-only pack materialization still pending replacement:
  `src/reference_pack.rs`, `src/reference_pack_materialization.rs`
""",
)
replace(
    "crates/wow-service/README.md",
    """ordinary reads. Pack administration remains in the tree; native annotation
generation to a directory and removal of that bureaucracy are tracked by
[#196](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196).
""",
    """ordinary reads. Uncalled Service publication/admin layers are gone; the live
builder pack/materialization path is replaced by native directory generation in
[#196](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196).
""",
)

replace(
    "docs/IMPLEMENTATION.md",
    """4. Remove displaced pack requests/administration in the builder and affected
   `wow-service/src/{reference_pack,reference_pack_admin,reference_admin,annotation_admin}.rs`
   callers. Native generation must not require a pack registry or signed report.
""",
    """4. Uncalled `annotation_admin`, `reference_admin` and `reference_pack_admin`
   layers are removed. Replace the still-live builder `reference_pack` and
   `reference_pack_materialization` path with ordinary generated files. Native
   generation must not require a pack registry or signed report.
""",
)
replace(
    "docs/REFACTOR_QUEUE.md",
    "| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported native generation; remove pack administration and its exclusive retained Lua copy. Coordinate #194; no index prerequisite. |\n",
    "| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Uncalled admin journals are removed; replace builder pack/materialization and its exclusive retained Lua copy with generated files. |\n",
)
replace(
    "docs/AUDIT_2026-10-17.md",
    "| Pack and operation ceremony | `reference_pack*`, `reference_admin`, `annotation_admin`, Store publication/journal callers. | #196 generated native libraries; #194 plain results. Do not just relocate the pack builder. |\n",
    "| Pack and operation ceremony | Uncalled `reference_pack_admin`, `reference_admin` and `annotation_admin` are removed; builder `reference_pack`/materialization remains. | #196 generated native libraries. Do not just relocate the pack builder. |\n",
)
