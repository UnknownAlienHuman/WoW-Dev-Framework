# Checked E0 coverage consumers

This slice continues R0-01 in `wow-core`, after the path/schema checkpoint
`ce1ae019c378077ea58f7eb473e14de83a8dabe1`. It implements the existing
CORE-010/CORE-022 requirement that summaries cannot supply negative authority
without retained raw coverage. Full E0-A/R0 acceptance is still incomplete.

## Admission before decisions

Both coverage consumer operations require a nonempty set of required summaries
and explicit raw records/conflicts from one expected context. They verify record
identity and status fields, unique `(capability, partition, producer)` statements,
conflict identity/context/scope, and summary equality with recomputation from all
supplied records for the selected capability. Removing a worse partition from a
summary does not remove that raw record from validation. A known affecting
conflict cannot disappear from the raw coverage or its summary.

Raw records for an optional, unselected capability are validated but do not block
the required subject. The caller therefore selects the complete required scope
before invoking the operation; unrelated partitions of that same capability must
not be passed as though they were part of a narrower required selection.
An empty selection is a contract error, not proof of completeness.

Duplicate detection uses the logical owner key, not adjacent entries after
sorting by cryptographic digest. Two different producers on one partition remain
independent, preserved records. Their statuses are combined conservatively; core
does not select a winning producer. Summary refs retain their existing canonical
order, so valid existing coverage IDs and envelope golden bytes are unchanged.

Availability validates even the complete/runnable path. Partial/unknown/failed/
required-nonapplicable/conflicted inputs produce `NotEvaluated`, retaining every
blocking partition and conflict. Shared blockers across capabilities or summary
producers are deduplicated explicitly, avoiding spurious duplicate-reference
errors during legitimate aggregation.

## Negative decisions

The caller supplies a reported exact miss, the retained expected context, and
admitted raw coverage. The operation returns a fallible, context-bound decision.
It never performs the lookup itself. A positive lookup must bypass this helper.

All applicable partial, unknown and failed partitions contribute denial reasons,
not only the worst combined status. Unknown scope, not-run lookup, candidate-only
input, blocked evaluation and truncation remain distinct reasons. Nonapplicable
partitions are neutral alongside applicable ones. All-nonapplicable input can
return `not_applicable` only after all independent blockers have been considered.
No missing registry or unresolved conflict reference is converted to absence.

`NotEvaluated` input is validated and context-bound; each supplied blocker must
resolve to the same raw coverage record and each conflict must resolve. Its
capability/coverage/conflict identifiers remain visible in the denial.

## Call migration

See [CONSUMER_GUIDE.md](CONSUMER_GUIDE.md#coverage-consumer-call-migration).
Availability adds the explicit conflict registry. Negative authority adds
`context_id` and raw records, returns `CoreResult`, and serializes `context_id`.
This is an intentional internal E0 API tightening, not a public release promise.
The only preexisting Rust callers were core tests; no sibling owner is changed.
The old unchecked call must not be reintroduced as a compatibility success path.

## Executable acceptance

```text
cargo test --locked -p wow-core --test e0_coverage_conformance
cargo test --release --locked -p wow-core --test e0_coverage_conformance
cargo test --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

The focused suite contains 34 tests across `tests/coverage/`: all 125 status
triples under six permutations, missing/contradictory/resealed/mixed-context
inputs, conflict closure/scope, optional lanes, shared blockers, deterministic
IDs and every implemented negative-decision branch. The existing history
regression is migrated without replacing any fixture or hash baseline.

A guarded Linux/Windows acceptance run records the exact formatted tree, toolchain,
locked debug/release results and native baseline regressions. The validation run
and publication commit are independent evidence; use their actual conclusions,
not this document, to claim a pass. No local Rust runtime is assumed.

## Remaining boundaries

These calls do not authenticate source completeness, source-registry eligibility,
profile materialization, actual lookup execution or WoW runtime behavior. The
caller owns scope selection, resource admission and exact input acquisition.
The complete envelope/evidence owner still resolves conflict evidence and source
handles; this helper only admits the supplied conflict relation and coverage
scope. Missing owner/runtime evidence remains unavailable, never a clean negative.

This is not a full audit of all deserializers, full evidence-DAG admission,
complete fixture closure for downstream E0 owners, or implementation of the
public `wow` executable. Continue the existing implementation handoff, not a new
architecture or a replacement source parser.
