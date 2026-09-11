# `wow-store` implementation status

## Implemented

- Explicit-root initialization with real-directory and symlink checks.
- Bounded SHA-256 blob storage with no-clobber atomic persistence and read-time verification.
- Canonical immutable snapshot manifests with sorted paths, exact lengths, aggregate budgets, and order-invariant identity.
- Persistent load/reopen validation of every referenced blob.
- Bounded append-only ref journals with checksummed sequence/predecessor chains.
- Inter-process shared/exclusive locking and compare-and-swap publication.
- Last complete record recovery after a torn trailing append; complete malformed/corrupt records reject.
- Concurrent-writer acceptance test proving only one writer advances one expected ref.

## Remaining E1 integrations

- Reference-view snapshot adapter and higher-level schema migration registry.
- Retention roots, reachability inventory, and explicit garbage-collection plan.
- Backup/restore verification and release-grade durability probes on Windows filesystems.
- Project/service persistence adapters; no higher-level crate may bypass snapshot identities or CAS refs.
