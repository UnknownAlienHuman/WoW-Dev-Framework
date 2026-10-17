# EmmyLua upstream compatibility

The root workspace dependencies and Cargo.lock identify one exact compiled Emmy
revision shared by the adapter, Reference and Annotations. This pin is not a
claim of upstream freshness. Normal builds use the lockfile. The existing manual
CI rolling-parser lane first isolates Reference/Annotations, then changes only
that temporary workspace's parser dependency to upstream `main` before
`cargo update` and its consumer checks. The job retains its resolved inputs;
source freshness or a manifest change alone is not compile/probe evidence.

For an explicit local checkout:

```sh
cargo xtask check-source /path/to/emmylua-analyzer-rust main
```

This is a read-only public HTTPS remote-head check. A difference offers a reviewed
update; network failure is unverified-current, never success. Automatic checkout
management, the old source-surface report commands and their former interpreter
implementation have been retired, not silently replaced by equivalent claims.

The maintained workflows are listed in [.github](../.github/README.md). The
rolling parser lane compiles and tests the consumers present in the workspace.
`wow-emmy` has executable physical/virtual syntax, semantic and incremental
session adapters. Compilation alone does not establish all of their acceptance
gates. Current source checks are not a release/install or client-runtime gate.

## Compiled backend report

Build the native owner executable and explicitly supply a checkout containing
the exact compiled upstream commit (including a Cargo dependency checkout):

```text
cargo build --locked -p wow-emmy --bin wow-emmy-probe
wow-emmy-probe produce-backend <upstream-checkout> > backend-report.json
wow-emmy-probe verify-backend backend-report.json
```

`produce-backend` reads immutable Git objects with replacement refs disabled.
It verifies the compiled revision/tree, eight exact regular source members and
their measured compiled-pin surface fingerprint; it ignores working-tree files.
Each member is bounded to 1 MiB. This finite source selection is probe evidence,
not a complete upstream source inventory. A dependency pin update must remeasure
the selected fingerprint together with the compiled revision/tree and probe.

The existing native session owner executes a separate small inert probe corpus:
Main/Library registration, accepted generic diagnostics, resolved and unresolved
members, calls, local bindings/operations/guards/control flow, and a coherent
file update/removal. The report retains actual normalized observations, source
member hashes, the nonempty exercised public API set and the canonical self-digest.
Only successful observations produce output. The original compatibility-report
wire and importer remain supported; `verify-backend` validates supplied data and
does not independently repeat those observations or authenticate arbitrary reports.

Source freshness is deliberately `unverified_current` for this exact compiled
pin. The producer does not select/activate a new pin, certify P1-P12, freeze E0-C
fixtures, establish rollback/license acceptance, or qualify the full coordinate,
diagnostic, performance or client-runtime matrices. Those obligations stay open
under [PIN_AND_PROBE](../crates/wow-emmy/PIN_AND_PROBE.md). A generated report can
be pinned through the existing local input `analyzer.compatibility_report` port.
