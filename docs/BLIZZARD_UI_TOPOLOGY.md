# Blizzard UI topology import compatibility

`wow-reference::ui_topology` and `wow-reference-topology` retain validation and
lookup for existing v1 topology drafts. The old XML/TOC source producer is retired.
A full native replacement is **not implemented** by the annotation generator or
source-manifest inventory. Do not advertise a source-to-topology build command.

```sh
cargo run -p wow-reference --bin wow-reference-topology -- verify /path/to/ui-topology.json
cargo run -p wow-reference --bin wow-reference-source -- verify /path/to/api-reference.json /path/to/ui-topology.json
```

The retained importer validates exact source/manifest identities, canonical hashes,
coverage, ordering, references, issue multiplicity and cycles. `declared` remains
source text, not a safe path; invalid declarations must have no navigation target.
Partial/missing/conflicted references cannot become authoritative absence by
removing diagnostics and recomputing a digest. The Rust unit and native CLI
fixtures cover these cases; they do not prove current Gethe topology generation.

The future native producer belongs to the source/reference owner, must parse XML
without entities or script execution, and must retain ordered TOC declarations,
exact file identities, case mismatches, missing/invalid targets and bounded
partial evidence. It must not become a second Lua semantic parser or infer
runtime LoadOnDemand, template ancestry or protected behavior.
