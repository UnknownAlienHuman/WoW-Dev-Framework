# `wow-store` E6 supporting seam

E6-B uses generic registered persistence, catalog, retention, audit, idempotency, response-loss, and GC operations described in [`EXTERNAL_CANDIDATE_STORE_HANDOFF.md`](EXTERNAL_CANDIDATE_STORE_HANDOFF.md).

`wow-store` does not import provider, Candidate, mapping, selection, or context semantics and never reads or mutates provider databases.
