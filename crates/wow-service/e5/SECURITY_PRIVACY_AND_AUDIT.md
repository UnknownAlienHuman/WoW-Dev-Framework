# E5-B security, privacy, credentials, and audit

**Status:** normative.

Treat source metadata/content handles, labels, expected results, pack names, review notes, authorization envelopes, vault responses, and cached/catalog artifacts as untrusted until exact schema/digest/profile validation.

E5-B cannot execute repository hooks, workflows, generators, tests, Lua, XML scripts, shell, JavaScript, Wasm, native plugins, callbacks, SQL, FTS, expressions, models, embeddings, or Codebase Memory. It cannot access arbitrary filesystem/network/process/editor/client state, mutate candidate bytes/labels/splits/metrics/graphs/source, or publish/activate a core pack.

Credentials are deployment configuration of narrow authorization/vault ports. Private keys, bearer tokens, vault secrets, KMS/HSM material, and signing secrets are forbidden in canonical requests, CLI flags, fixtures, logs, or public envelopes. GitHub login, repository ownership, OS user, file owner, terminal, and commit author are not authorization.

Output permission is the intersection of owner, corpus/label, candidate/run/report, review confidentiality, holdout disclosure, license/notice, consumer trust, and service output policies. Higher layers can only narrow. Unknown state follows the safest frozen profile.

Audit records are append-only, hash-linked, exact-artifact-bound records. Required trusted time/sequence evidence is explicit; local clock alone is insufficient where the profile demands stronger evidence.

Bound selectors, artifacts, corpus members, labels, rules, cases, mutations, proposals, metrics, runs, pages, audit events, reviews, grants, submissions, bytes, memory, time, owner calls, and retries. Cancellation retains partial effect/audit state and closes synchronously.

Default logs expose stable IDs, counts, stages, statuses, and reason codes only—not source bodies, hidden labels/membership, confidential notes/signatures, credentials, private paths, database/vault handles, or unbounded stack data.