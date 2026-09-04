# Blizzard generated API ReferenceDraft

## Purpose

`scripts/build-blizzard-api-reference.py` converts generated Blizzard API documentation from one exact source-manifest revision into a deterministic producer-owned `ReferenceDraft`.

This is a rolling update boundary, not a permanent client pin:

1. resolve the configured moving branch at operation start;
2. build or verify the source manifest for that revision;
3. read every generated documentation file from that same revision;
4. record the revision and source-file SHA-256 values in the result;
5. resolve the moving branch again on the next operation.

The recorded revision makes completed evidence reproducible. It does not prevent updates.

## Source transport

Local Git is preferred. The producer uses `git show <revision>:<path>` and never trusts the working-tree copy. Dirty working-tree edits cannot alter output.

When the exact revision is unavailable locally, `--github-repository owner/repository` enables exact-revision raw GitHub reads. One transport is selected for the operation. Moving-branch reads after revision resolution are forbidden.

Credentials are accepted only through the environment variable selected by `--github-token-env`; they are never written to output.

## Non-executing parser

The producer parses generated documentation as data. It does not invoke Lua or execute repository hooks, installers, build scripts, package managers or generated code.

Supported values include keyed/array tables, strings and long strings, numbers, booleans, `nil`, dotted symbolic values, and Lua comments. A new unsupported construct fails that source file.

Default mode aborts on any failed file. `--allow-partial` is diagnostic only: it emits explicit failed-file coverage and sets `negative_authority` to `false`.

## Normalized records

The draft preserves:

- system name, namespace, environment and documentation;
- functions, arguments and returns;
- events and payloads;
- structures, enumerations and fields;
- declared restriction/secrecy attributes;
- unrecognized generated fields under deterministic `attributes` objects;
- source path and content SHA-256 for every system;
- source-manifest digest, source revision and observed source version;
- per-file coverage and aggregate record counts.

No undocumented semantic interpretation is added. Restriction fields remain declared metadata for later owner logic.

## Identity

Canonical JSON uses UTF-8, sorted object keys, compact separators and no non-finite numbers. `draft_digest` is SHA-256 over the complete draft without the digest field itself.

Transport, checkout path and wall-clock time do not affect identity. Source revision, source bytes, parser version and producer configuration do.

## Commands

```bash
python scripts/build-blizzard-api-reference.py \
  --manifest .wow-dev/source-manifest.json \
  --source "$WOW_UI_SOURCE_DIR" \
  --github-repository "${WOW_UI_SOURCE_REPOSITORY:-Gethe/wow-ui-source}" \
  --output .wow-dev/blizzard-api-reference.json \
  --json
```

```bash
python scripts/verify-blizzard-api-reference.py \
  .wow-dev/blizzard-api-reference.json \
  --require-complete \
  --json
```

Omit `--github-repository` for strictly local/offline operation.

## Update behavior

- A moving-branch advance before an operation produces a new source manifest and draft.
- A branch advance during an operation does not change the already resolved revision.
- A clean behind clone may fast-forward according to `auto`, `prompt` or `never` policy.
- Dirty, ahead, diverged or mismatched clones are never mutated automatically.
- A generated grammar change fails coverage visibly; update the parser and tests before declaring completeness.
- Network unavailability does not invalidate an exact revision already present locally; currentness is reported separately.

## Authority boundary

The draft proves only what generated documentation declares at its exact revision. It does not independently prove implementation call paths, hotfix data, runtime restrictions, secret-value accessibility, combat state or protected-action legality. Those require Blizzard implementation/XML/TOC inspection and, where applicable, an exact runtime probe.
