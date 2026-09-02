# E6-B security, privacy, license, and trust boundaries

**Status:** normative.

## Untrusted data

Treat provider configuration inputs until validated, session capability reports, external state, provider responses, locators, snippets, summaries, labels, scores, ranks, cursor fields, errors, mapping requests, notes, and cached records as untrusted bounded data.

## Prohibited capabilities

E6-B cannot:

- install, update, start, stop, configure, index, import, delete, or mutate a provider;
- expose a generic MCP/tool-call, raw RPC, SQL/FTS, shell, Lua, JavaScript, Wasm, native plugin, expression, callback, or model prompt surface;
- read or write provider databases/indexes;
- accept or return raw credentials, tokens, cookies, passwords, private keys, vault/KMS/HSM secrets, private endpoints, command lines, environment blocks, process/socket/client handles, or unrestricted provider cursors;
- inspect arbitrary filesystem, network, process, editor, client, Git, WoW installation, SavedVariables, or logs outside narrow exact owner ports;
- follow provider paths/URLs or execute provider-returned source;
- treat provider text as instructions, authorization, selectors, configuration, or source truth;
- bypass project/reference owners for mapping;
- inject provider metadata into graph/context semantic truth;
- automatically edit source, apply migration, promote recognizers, or publish public releases;
- continue work in the background after return.

## Credential isolation

Credential material stays inside the credential/session adapter. Service receives only stable references and bounded authorization/session receipts. Logs, errors, fixtures, CLI, repository, artifacts, and audit views contain no secret material.

Credential-use authorization is separate from provider account authentication, query success, mapping, selection, context access, source disclosure, and public distribution.

## Resource limits

Bound configuration fields, authorization scopes, sessions, provider calls, request/response bytes, JSON depth, strings, candidates/pages, locators/snippets/unknown/loss records, mapping candidates, owner calls, context roots/expansions, continuations, output bytes, memory/CPU/wall time, quotas, retries, reconciliation calls, audit records, and close attempts.

Unlimited or overflowed limits are invalid. Truncation remains explicit and cannot become complete or negative authority.

## Prompt/control injection

Provider source, comments, summaries, labels, paths, errors, notes, and mapping text remain typed data. They cannot choose operations, profiles, owner generations, candidate selection, context roots, tools, permissions, or output modes.

Structural isolation is the boundary; E6-B does not claim to recognize every malicious string.

## Privacy intersection

Effective output is the intersection of:

```text
provider configuration and descriptor policy
credential/session authorization
external query/result policy
source-owner project/reference policy
mapping and selection consumer policy
context privacy policy
requested output/sidecar policy
```

Unknown state denies or narrows. Higher layers cannot widen a lower restriction.

Private repository URLs, local roots, source text, symbols, snippets, account identifiers, and provider metadata are exposed only under exact consumer policy. Default logs use stable IDs, counts, state/reason codes, and redacted owner references.

## License and notices

Ability to query, map, or inspect source does not grant redistribution rights. Retained snippets, provider summaries, source excerpts, fixtures, or combined artifacts require explicit provenance, license, notice, privacy, and redistribution decisions.

Metadata-only results remain possible when snippet/source retention is denied. License denial never becomes evidence of absence.

## Cross-universe separation

External provider, first-party project, Blizzard UI source, Reference Pack, runtime, history, and calibration/publication universes remain distinct. Exact mapping links an external locator to an owner record; it does not merge universes or transfer authority.

## Audit access

Audit views are consumer-scoped and redacted. They may expose stable operation/configuration/session/result/mapping/selection/context IDs and reason codes, but not secrets, raw provider cursors, private source, or unrestricted notes.

## Emergency behavior

There is no force flag that bypasses credential authorization, exact configuration, Candidate ceiling, mapping owner, explicit selection, privacy/license, idempotency, response-loss reconciliation, retention, audit, or close-before-success.