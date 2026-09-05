# Blizzard UI TOC/XML topology producer

The topology producer converts one exact Blizzard UI source snapshot into a non-executing load and declaration graph. It is deliberately separate from Lua semantics: Lua symbol, type, reference, and diagnostic correctness belongs to the EmmyLua adapter.

No client build, Interface value, branch head, or source revision is compiled into the producer. A moving selector is resolved at operation start; all files are read from that exact Git revision and the next operation resolves the selector again.

## Scope

The candidate corpus is every `.toc` and `.xml` file under `Interface/` listed by the verified source manifest. Other files remain in the source inventory so TOC and XML references can be resolved exactly.

The producer records:

- each TOC descriptor, metadata declaration, ordered file entry, descriptor source identity, and load edge;
- each XML document, include/script edge, inline-script count, named/virtual element, and declared template inheritance name;
- exact path, Git object, SHA-256, and line for every source reference;
- missing, invalid, ambiguous-case, and case-mismatched targets;
- duplicate TOC metadata and XML include cycles;
- candidate, parsed, failed, and unresolved counts;
- explicit limitations and a canonical self-digest.

Template inheritance remains a name declaration. It is not collapsed into runtime object ancestry. `<Script file="…">` records a load edge; inline script text is never executed or treated as semantic truth.

## Security boundary

The implementation reads blobs with `git cat-file --batch` from the exact manifest revision. It does not read dirty worktree bytes. TOC is parsed line by line. XML uses Expat with parameter-entity parsing disabled, bounded bytes/elements/depth, and entity/external-entity declarations rejected.

Repository hooks, submodules, package managers, Lua, XML scripts, build tools, and generated code are never executed.

Relative `.` and `..` references are normalized only while they remain inside the repository `Interface/` tree. Absolute paths, URLs, drive paths, interior control characters, and root escapes are invalid.

`declared` retains original source text, including surrounding whitespace or an invalid
empty/control-bearing declaration. It is diagnostic data, never a validated navigation
path; structured output escapes control characters. Resolution trims surrounding
whitespace. Only a non-invalid reference may have a canonical `target`; an invalid
reference has no target. Empty XML `file` attributes are invalid references, not inline
scripts or silently absent includes. Rust checks that each unresolved edge has a matching
issue and no reference issue is orphaned, including multiplicity. Removing diagnostics
and recomputing hashes cannot promote an invalid edge to complete coverage.

## Build and verify

```bash
python scripts/build-blizzard-ui-topology.py \
  --source "$WOW_UI_SOURCE_DIR" \
  --manifest .wow-dev/source-manifest.json \
  --output .wow-dev/ui-topology.json \
  --json

python scripts/verify-blizzard-ui-topology.py \
  .wow-dev/ui-topology.json \
  --source "$WOW_UI_SOURCE_DIR" \
  --manifest .wow-dev/source-manifest.json \
  --current-ref origin/live \
  --json
```

Use `--require-complete` when the caller requires every candidate file to parse and every local file reference to resolve exactly. Use `--allow-partial` only for investigation after parse failures; partial output never has negative authority.

Exit codes:

- `0`: internally valid and, when requested, current;
- `2`: malformed, inconsistent, unreproducible, or incomplete when completeness is required;
- `3`: valid for its exact revision, but the checked moving ref has advanced.

An unresolved target does not disappear. The draft remains queryable, but coverage is `partial` and `negative_authority` is false. Updating the local source creates a new draft; it never rewrites the evidence identity of an earlier draft.

## Evidence role

This lane proves descriptor/XML declarations and exact load references only. It does not prove:

- Lua control flow, symbols, types, or call targets;
- actual LoadOnDemand activation order in a running client;
- protected/secure behavior;
- data- or hotfix-dependent runtime state;
- effective template ancestry after runtime mutation.

Those claims require the EmmyLua, implementation, project, or target-client runtime lanes as appropriate.
