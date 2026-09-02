# E6-A security, privacy, and license contract

**Status:** normative.

## Trust boundary

The provider transport is an untrusted peer. Descriptors/profiles are trusted only after exact repository-owned schema/version/digest validation. Returned metadata, locators, snippets, summaries, labels, ranks, scores, continuation cursors, and errors are untrusted bounded data.

## Prohibited capabilities

E6-A cannot:

- install, update, start, stop, configure, index, import, delete, or mutate a provider;
- read/write/attach a provider database;
- accept raw SQL, MCP JSON, arbitrary tool names, callbacks, plugins, shell, Lua, JavaScript, Wasm, native code, regex programs, or model prompts;
- read arbitrary filesystem paths, follow URLs, clone repositories, inspect environment/editor/Git/WoW state, or execute source;
- acquire credentials/sessions or expose private endpoints/tokens/cookies/keys;
- call local project/reference/graph/search/context/service owners;
- transform provider data into verified source, graph, lineage, replacement, impact, edit, or runtime authority;
- retain unrestricted full source in default artifacts;
- continue work in background after return.

## Resource limits

Bound descriptor definitions, capability fields, request terms/seeds/filters, request/response bytes, JSON/depth/string sizes, candidates/pages, locators/snippets/unknown fields/loss records, score values, continuation bytes/pages, comparisons, cache entries, memory/CPU/wall time, and cancellation checks.

Over-limit response is rejected or explicitly truncated under a profile. Truncation can never become complete/zero-authoritative.

## Credential boundary

E6-A requests/results contain stable provider/adapter/session authorization references only when required for binding; no raw credential or private endpoint. Credential/session lifecycle belongs to E6-B.

This boundary is unrelated to WoW Secret Values.

## Source/prompt injection

Provider snippets, comments, summaries, paths, labels, errors, and documentation are data. Canonical JSON escaping and typed records prevent them from creating profile fields, tool calls, selectors, permissions, source handles, or agent instructions.

E6-A does not claim to detect every malicious string. Structural isolation, not lexical filtering, is the correctness boundary.

## Privacy

Output is the intersection of provider descriptor, query, external state, source/result, consumer, and bridge privacy profiles. Unknown privacy state defaults to the safest explicit result: deny or metadata-only. Higher layers may narrow but not widen.

Private repository URLs, local roots, source, credentials, and identifiers are redacted/omitted according to exact policy with loss records. Default logs use stable IDs/counts/reason codes only.

## License

Retention of source-derived snippets/fixtures requires explicit provider/source provenance, license, notice, and redistribution decisions. Ability to query or index locally does not imply permission to redistribute provider-returned source.

A result can retain metadata/locator fields while snippet retention is denied. License denial is not evidence that the candidate is absent.

## Cancellation and late responses

Cancellation is checked throughout. The transport owner must discard or separately reconcile late responses. E6-A does not expose a result after cancellation as complete and does not spawn cleanup tasks.

## Adversarial tests

- malformed/deep/oversized/polyglot responses;
- raw tool/SQL/script/model fields;
- path/URL/source prompt injection;
- score NaN/infinity/overflow or incompatible units;
- cross-provider/state cursor/cache substitution;
- credential/private path/source leakage;
- hidden provider database/index mutation;
- cancellation/timeout/late response;
- zero result after rejected candidates;
- provider labels attempting authority escalation.