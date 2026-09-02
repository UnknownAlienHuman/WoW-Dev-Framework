# E6-A implementation plan

**Status:** normative order; implementation has not started.

0. Freeze `wow-core`, reviewed provider descriptors, transport adapter contracts, external-state/query/normalization/score/loss/privacy/security profiles, synthetic response corpus, benchmarks, and checksums.
1. Implement provider descriptor and capability-set validation.
2. Implement stable/observed-mutable/opaque external-state records and compatibility checks.
3. Implement closed query grammar and allow-listed transport port traits without process/session/credential ownership.
4. Implement bounded raw-response validation and unknown/loss records.
5. Implement Candidate-only normalization, provider-local score/rank handling, and deterministic ordering.
6. Implement unverified locators and the E6-B mapping-request handoff record.
7. Implement zero-result, coverage, partial/truncation/conflict, and degradation semantics.
8. Implement continuation and cache-entry validation without physical storage.
9. Implement explanation, artifact subset with explicit candidate IDs, and descriptive comparison.
10. Implement privacy/license/redaction, cancellation, late-response, resource, and injection boundaries.
11. Run synthetic provider, malformed/adversarial, 1/2/N worker, shuffled-order, cancellation, and optional-degradation evaluations.
12. Freeze canonical bytes, implementation/profile/vector IDs, benchmark thresholds, and all SHA-256 values before marking implemented.

E6-B service/session/credential/mapping/selection/context/retention/CLI work remains deferred. No provider database/index effects, generic MCP surface, model call, or source-owner mapping is implemented in E6-A.