# Current domain recovery

**Status:** implemented at the service and CLI seam, with the workspace gates below
passing. Passing gates bound verification; they do not by themselves establish wider
acceptance.

The physical-only contract is [PROJECT_RECOVERY](../wow-store/PROJECT_RECOVERY.md).
This document covers the separate typed Current domain observation produced beside
it. Recovery still observes: it never initializes, repairs, or activates. Supplying
a native domain replay verdict is not the same as granting repair or activation
authority, so the observation never grants either.

## What is observed

`LiveProjectStore::current_domain_observation`
([current_recovery.rs](src/live_project/current_recovery.rs)) returns
`LiveProjectRecoveryObservation`, which carries the unchanged physical
`RecoveryReport` plus exactly one `CurrentDomainObservation`. The physical report
keeps its own epoch, profile, Current state, coverage, incidents and operations; the
domain outcome never rewrites them.

The domain observation classifies only the Current that the physical scan already
established. It reports no raw source fact and makes no claim about any other
generation. Blizzard source and Gethe/Ketho facts stay outside this observation
entirely; a domain verdict is never a source-freshness claim.

Eligibility follows the physical Current state. Only `CurrentState::Absent` yields
`CurrentDomainObservation::Absent`. `CurrentState::Corrupt` and
`CurrentState::Unverified` both yield `Unverified`, and the physical state and
incidents keep those two distinct. Neither is treated as absence, and neither
becomes a clean negative.

For `CurrentState::Validated`, the observation takes the exact
`RecoveryReport.current()` and acquires
`ReadSelector::Publication(current.record_id)`, so it replays the exact retained
activation-history record rather than re-resolving current. The acquired manifest
must match the reported epoch and generation. `current_at_acquisition` is a later,
independent observation and is not required to equal the reported Current and never
replaces the selected publication.

Replay uses native owners. The held `ReadSnapshot` stays alive through
`AcquiredProjectPair::read`, so the pair is really replayed and its usual
equivalence checks apply. IDs come only from that acquired pair through its own
getters; they are never parsed or predicted. `Validated` attests successful exact
retained-pair replay for the one reported publication, nothing more.

## Cancellation and failure ordering

An initial physical scan failure returns an ordinary `ServiceResult::Err`: no
physical report exists yet, so nothing is retained. Cancellation before the scan
completes behaves the same way, and `recover_current_live_project` checks the stop
flag before opening the store at all.

Once physical evidence exists, later failures are recorded in the observation while
the report is retained. Replay cancellation returns `Cancelled` with the physical
evidence intact rather than discarding it through `?`. Domain failures are typed
with a `CurrentDomainPhase` of `AcquirePublication` or `Replay` and a
`ServiceErrorCode`; cancellation is always `Cancelled`, bounded resource,
environment and missing-target conditions are `Incomplete`, and genuine contract
violations are `Failed`.

## Usage

```text
wow project recover --store-root <directory> [--domain-current] [--format json|text]
```

Without `--domain-current` the command keeps its physical JSON and exit behavior
unchanged. With it, the report contains the physical observation plus
`current_domain`, and the exit ladder is:

| Outcome | Exit |
| --- | --- |
| `Failed` | 4 |
| invalid physical state or scope, which still overrides | 4 |
| `Incomplete` or `Unverified` | 2 |
| `Cancelled` | 130 |
| `Validated` or `Absent` | the physical exit |

A completed physical observation is written even when replay is later cancelled, so
the physical evidence survives the 130 exit.

## Verification

Verified workspace gates, all passing over 107 targets: `cargo fmt --all --check`,
`cargo check --workspace --all-targets --all-features`, strict `cargo clippy
--workspace --all-targets --all-features -- -D warnings`, `cargo doc --workspace
--all-features --no-deps`, `cargo test --workspace --all-targets --all-features`
with 909 passed, 0 failed, 1 ignored, and `cargo build --workspace --all-targets
--all-features`. The native-only policy check passed over 1578 distributable files.

Two items remain unevaluated and are recorded as such: a positive CLI published-root
smoke against a published root is `NotEvaluated`, and injected availability and
cancellation acceptance is `NotEvaluated`.

## Not claimed

Passing gates do not widen this observation. It does not assert complete
capabilities, validate other generations, rewrite physical coverage, issue a
`ValidatedRead`, or authorize repair or activation, and it does not establish
migration, W16/E2, or domain quarantine acceptance.
