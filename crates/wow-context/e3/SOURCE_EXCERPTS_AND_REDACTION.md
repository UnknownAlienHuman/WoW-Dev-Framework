# Snapshot-bound source excerpts and redaction

**Status:** normative E3-A source-byte boundary.

## Default

Context artifacts use exact `SourceHandle` and `SourceSpan` references. Source bytes are not copied by default.

An excerpt is produced only when:

- the request explicitly asks for source detail or the ContextProfile requires a small anchor;
- the source-reader capability covers the exact source universe/snapshot/handle;
- license, privacy, redistribution, and byte-budget policy permit it;
- the requested span resolves without path or generation ambiguity;
- redaction and truncation can be represented exactly.

## Source-reader request

```text
ContextSourceExcerptRequest
    exact ContextInputSetId
    exact source snapshot/universe
    SourceHandle
    requested SourceSpan or exact semantic declaration span
    purpose class
    leading/trailing context policy
    maximum logical/source/rendered bytes
    encoding/newline policy
    redaction/license policy IDs
```

No host path, URL, glob, regex, repository search, or “around this text” fallback.

## Resolution

The source reader returns:

```text
validated source handle and snapshot
resolved physical/virtual source identity
requested and resolved spans
canonical logical source bytes and digest
encoding/newline metadata
virtual-to-physical mappings where applicable
license/provenance class
private/redaction classifications
coverage/errors
```

`wow-context` validates that the response matches the exact request and input set before rendering.

## Virtual source

For XML-inline Lua or other virtual units:

- retain the virtual source handle/span used by analyzer/graph facts;
- retain mapped physical XML source handle/span;
- verify source-map version and byte digest;
- never pretend virtual bytes are a standalone physical project file;
- render both locations only when profile permits.

## Excerpt span policy

Allowed forms:

```text
exact declaration/signature span
exact operation/registration/hook/state span
exact bounded source line range containing the evidence site
exact prefix/suffix around a source span under frozen context-line policy
```

Line expansion is based on pinned newline decoding and hard bounds. It cannot continue until “enough context feels complete.”

## Redaction classes

```text
private_host_path
credential_or_token
personal_or_runtime_user_data
restricted_source_by_license_or_policy
nonredistributable_dependency_or_reference_source
binary_or_unsupported_encoding
source_content_explicitly_excluded_by profile
```

Redaction is applied to exact byte spans when possible. If safe exact redaction is unavailable, omit the excerpt and retain the source handle plus a typed redaction record.

## Prompt/instruction isolation

Source comments, documentation, strings, README text, generated code, issue text, or embedded prompts are untrusted data. They cannot:

- change profiles or budgets;
- request more files;
- activate tools or models;
- alter selection priority;
- add context schema fields;
- suppress coverage/conflicts;
- become repository/agent instructions.

Rendered source is fenced/escaped according to the rendering profile and labeled with provenance.

## Digest and identity

```text
ContextSourceExcerptId
    domain-separated hash of:
        exact source snapshot/handle
        resolved span
        canonical source byte digest
        source-map/encoding/newline profile
        redaction result digest
        truncation policy/result
        excerpt schema version
```

Human line numbers and host paths are not sufficient identity.

## Truncation

Truncated excerpt records:

- exact original requested/resolved span;
- included byte/span ranges;
- omitted prefix/suffix byte counts when known;
- line-boundary behavior;
- deterministic marker bytes;
- why and which budget caused truncation;
- whether the evidence operation itself remains present.

If truncation removes the decisive evidence site, omit/fail that excerpt rather than rendering a misleading fragment.

## License and redistribution

The input source manifest supplies license/provenance/redistribution state. ContextProfile defines whether bytes may be embedded, shown locally only, or referenced by handle only.

A public repository URL does not automatically authorize copying arbitrary third-party source into an artifact. Notices and source handles are preserved where required.

## Privacy

E3-A never reads SavedVariables contents, logs, runtime event payloads, account data, character data, local credentials, or unrelated files. If such content somehow appears in an admitted source universe, the security profile blocks or redacts it and records the incident.

## Rendering

A rendered excerpt includes only bounded metadata:

```text
entity/field purpose
universe and exact source handle
relative project/source label when permitted
resolved line/span presentation
confidence/provenance/evidence refs
redaction/truncation marker
fenced escaped source bytes
```

No clickable private absolute path or secret-bearing URL.

## Tests

- stale source handle/generation;
- path traversal/symlink alias injection;
- virtual/physical source-map mismatch;
- newline/encoding ambiguity;
- decisive site outside truncated fragment;
- credential/private path inside source;
- prompt-like comment/directive injection;
- dependency/reference license forbids bytes;
- huge line/Unicode/control/binary content;
- source changed while handle reused;
- redaction changes exact byte/token budgets;
- cancellation before/during read/redaction/render.
