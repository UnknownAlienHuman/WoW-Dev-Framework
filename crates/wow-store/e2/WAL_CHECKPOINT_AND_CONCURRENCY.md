# WAL, checkpoint, locking, and concurrency profile

**Status:** normative semantics; exact values freeze after executable probe and benchmark.

## Required effective profile

Record and verify effective values for:

```text
journal_mode=WAL
synchronous
foreign_keys
trusted_schema / defensive mode
extension loading disabled
busy timeout
wal_autocheckpoint or manual mode
cache/page/mmap/temp policy where used
SQLite limits
read/write open flags
```

Defaults are not a contract.

## One writer

One process owner serializes generation build, activation, checkpoint, epoch change, retention/GC, backup coordination, and recovery mutation. A second owner is rejected. SQLite's single-writer behavior is a backstop, not the ownership protocol.

## Busy policy

- finite configured timeout;
- no unbounded spin or retry;
- cancellation interrupts waiting where supported;
- errors name semantic IDs, not private paths;
- caller may retry only after reading exact current state;
- no retry against an implicit newer base.

## Transaction modes

- readers begin explicit read transactions;
- inactive build and activation use a tested explicit write mode, expected `BEGIN IMMEDIATE`;
- no autocommit-dependent correctness;
- no hidden nested transaction in owner operation catalogs.

## Checkpoint policy

Freeze:

```text
autocheckpoint enabled/disabled and threshold
passive checkpoint trigger
restart/truncate eligibility
maximum WAL frames/bytes before warning or admission block
reader-pressure behavior
post-activation budget
shutdown policy
failure/cancellation handling
```

Recommended v1:

- manual/passive checkpoint after activation when threshold exceeded;
- no blocking truncate while readers require old frames;
- restart/truncate only under explicit maintenance and compatible reader state;
- bounded busy result;
- WAL growth warning/admission control before unbounded expansion.

## Logical independence

Checkpoint cannot change semantic IDs, partition membership, current record, or domain query results. Any such change is corruption.

## Crash and power-loss classification

Test process crash, OS crash where available, WAL/SHM recovery, and platform flush behavior separately. Never claim power-loss durability from process-kill tests.

## Reader/checkpoint interaction

Old readers may pin WAL frames. Checkpoint reports frames/checkpointed/busy without switching readers. Retained DB generations and WAL frame retention are separate concerns.

## Backup interaction

Online backup reads one exact snapshot. Copying only the main DB while WAL contains committed frames is forbidden.

## Metrics

Operational metrics include writer lock wait, transaction duration, WAL frames/bytes, checkpoint frames/busy, reader count/class, and pages/bytes written. Timing and host data are noncanonical.
