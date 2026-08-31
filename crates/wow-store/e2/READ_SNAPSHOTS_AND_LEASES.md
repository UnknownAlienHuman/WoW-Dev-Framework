# Read snapshots and generation leases

**Status:** normative reader-isolation contract.

## Open sequence

```text
resolve exact ProjectPublicationHead once
-> validate head schema and referenced generation/artifact
-> acquire GenerationLease
-> open sealed generation read-only
-> validate requested head/generation closure
-> construct registered project/graph read handles
```

A caller may also request an exact historical generation and acquire a lease without using current head policy.

## Lease invariants

- binds one exact generation/artifact;
- is a retention root;
- does not follow head changes;
- cannot be transferred across process/owner classes without explicit protocol;
- release is explicit and idempotent;
- expired/abandoned reclamation follows a frozen owner-death/heartbeat policy;
- wall-clock expiry alone cannot collect a generation still open under supported platform behavior.

## Reader consistency

A reader holding head N continues to see:

```text
ProjectStoreGeneration N
ProjectGeneration N
GraphGeneration N
AnalyzerSnapshot N
Recognizer manifests N
Profile/ReferenceGeneration N
```

Publishing head N+1 does not alter any N view or row.

## Registered reads

- prepared/reviewed operations only;
- exact generation-bound handle required;
- explicit parameters and budgets;
- deterministic ordering;
- truncation/continuation reported;
- no mutable cursor surviving handle close;
- no query causes writes, schema changes, checkpoints, or head resolution.

## Concurrent behavior

- one writer may build/seal N+1 while readers hold N;
- validation reader may open sealed inactive N+1;
- head CAS is independent from existing read handles;
- GC excludes all leased generations;
- platform-specific delete/open behavior is not relied on for correctness.

## Lease registry

Lease records may be in process memory plus durable registry depending on owner/crash profile. The chosen profile must prove:

- no live generation collected;
- abandoned lease eventually classifiable;
- registry crash does not create false nonreachability;
- conservative retention on uncertainty;
- bounded lease count/metadata;
- privacy-safe owner IDs.

## Errors

Distinguish:

```text
head_not_found
head_changed_before_lease
generation_not_found
artifact_manifest_mismatch
lease_conflict_or_limit
read_handle_generation_mismatch
read_operation_not_registered
read_budget_exceeded
read_cancelled
```

No fallback to current/latest after exact failure.
