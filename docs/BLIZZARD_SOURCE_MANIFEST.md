# Blizzard source manifest

The source manifest is the deterministic handoff between rolling source acquisition and reference producers.

## Boundary

A moving selector such as `live`, `ptr`, `beta`, or a Classic branch is resolved at the beginning of an operation. The resulting full commit identifier is then used for every file read in that operation. The selector may move later; the completed result continues to identify the bytes it actually consumed.

The manifest producer reads the selected commit through the local Git object database. It does not read uncommitted working-tree files, execute repository code, inspect a WoW installation, infer a current branch, or refresh the selector midway through the operation.

```bash
python scripts/build-blizzard-source-manifest.py \
  --source "$WOW_UI_SOURCE_DIR" \
  --revision <resolved-commit> \
  --selector live \
  --output .wow-dev/source-manifest.json
```

Use the source acquisition and version-check commands before this step. A local clone is preferred. Network fallback, update policy, and stale-checkout handling belong to the source acquisition layer, not to the deterministic producer.

## Included data

The default manifest includes:

- Lua implementation files;
- generated API documentation Lua files;
- XML layouts and templates;
- TOC load-order files;
- XSD schemas;
- `version.txt`.

Each file record contains its canonical repository-relative path, semantic class, byte count, Git blob identity, and SHA-256 content digest. Records are bytewise path-sorted. Asset files and unrelated text are excluded unless their extension is explicitly requested.

The manifest also records:

- an opaque source identifier;
- the optional moving selector used to locate the source;
- the exact resolved commit;
- the version reported by that exact commit;
- inclusion and exclusion counts;
- a digest over every other manifest field.

Local paths, remote URLs, credentials, wall-clock time, and working-tree state are absent from manifest identity.

## Update behavior

A manifest is immutable evidence for one source snapshot. It is never edited in place to pretend that it represents a newer client.

For the next operation:

1. check the configured source selector;
2. safely update or refresh the local object database according to policy;
3. resolve the selector once;
4. build a new manifest;
5. create a new reference generation when bytes, producer version, or configuration changed.

A source update does not silently invalidate historical results. It makes them non-current for operations that explicitly require the latest selected channel.

## Authority

The manifest proves source byte identity and acquisition coverage. It does not by itself prove runtime behavior, hotfix state, combat restrictions, user data, or that every possible source file was semantically interpreted. Reference producers must preserve unsupported, conflicted, partial, and not-evaluated coverage instead of manufacturing clean negative claims.
