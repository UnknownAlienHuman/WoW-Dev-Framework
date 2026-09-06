# Repository automation

Three maintained workflows run actual native commands:

- `ci.yml`: Rust on Linux/Windows; repository/skill checks, fmt, check, strict
  Clippy, debug/release tests, rustdoc, rolling dependencies/parser and the
  separate real-Wasm replacement/rollback matrix.
- `current-source-bundle.yml`: current Gethe checkout, Rust source manifest
  verification, native Ketho generation and final artifact/hash/map validation.
  Source admission must be complete; projection omissions remain partial.
- `branch-hygiene.yml`: after successful CI for the exact current main, back up
  all refs and remove only unprotected ancestor branches without open PRs.
  Every deletion uses its expected tip SHA and read-back. A changed main stops
  cleanup; a changed branch fails its lease. Equal trees with independent
  commits are retained until their history is explicitly reconciled.

Build/source jobs have read-only GitHub permissions. Only branch hygiene has contents-write access.
It never merges code, chooses between competing implementations or overwrites
main. A successful CI run does not authorize discarding unreconciled branches.
The history bundle is retained before deletion, while merged histories remain
reachable through main regardless of artifact expiry.

No interpreter setup, embedded interpreter script, self-modifying source payload,
finalizer or hidden publication step is permitted. `cargo xtask check` rejects
interpreter assets and ordinary reintroduction through scripts or CI.

The old v1 API/topology producer workflow has been retired. Native importers
remain compatibility readers tested with synthetic JSON fixtures; they do not
claim to regenerate full UI topology from current source.

The Wasm matrix also runs complete synthetic source-library replacement/failure
probes. Current-source automation runs the same generator with two actual Rust
guests and compares every file and source metadata with its native baseline;
module-bound artifact verification is mandatory. Only the selected literal
algorithm is replaceable; a parity pass is not full analyzer certification.

Current-source CI also invokes the guarded source updater on each explicitly
cloned public HTTPS checkout before resolving build inputs. Its receipts are
retained. This checks the real transport/guard path (often an idempotent current
result); actual fast-forward and rejection cases run on Rust-owned Git fixtures.
It does not grant write permission to the GitHub job or any filesystem/network
capability to WASM guests.

The CI consumer matrix resolves and hashes the two approved upstream checker
packages, then explicitly runs `wow-annotations --test consumers -- --ignored`.
The Rust test owns assertions; acquisition shell only selects/downloads test
inputs. It does not install extensions, alter user settings or run addon Lua.
Results identify a bounded synthetic semantic slice, not full product support.
