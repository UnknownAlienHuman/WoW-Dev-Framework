from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"unexpected documentation block in {path}")
    file.write_text(text.replace(old, new))


replace(
    "docs/IMPLEMENTATION.md",
    """#194 starts at `wow-core/src/{envelope,canonical}.rs` and actual Service callers:
plain typed values internally, one serialization at CLI/protocol output, bounds
outside payload identity. Delete self-hashes, size fixed-point loops and durable
read-operation journals with their callers.
""",
    """Local status/check values in `wow-service/src/service.rs` are plain typed results:
no generic envelope, request/result self-digest, schema echo or service-version
echo. The application serializes once with an external byte ceiling. #194 now
continues only through separate Context/Search/graph result families; simplify
each from its actual caller rather than introducing another universal envelope.
""",
)
replace(
    "docs/REFACTOR_QUEUE.md",
    "| 2 | [#194 — result envelopes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194) | Core E0 envelopes and one-shot registries are removed; finish plain typed output and delete remaining self-hash/size/read journals. |\n",
    "| 2 | [#194 — result envelopes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194) | Core and local Service envelopes/registries are removed; simplify remaining Context/Search/graph self-identities only from real callers. |\n",
)
