# Repository automation

Three maintained workflows run actual native commands:

- `ci.yml`: Rust on Linux/Windows; `cargo xtask check`, fmt, check, strict Clippy,
  debug/release tests and rustdoc. Updated dependency and rolling Emmy parser
  lanes exercise both the reference and annotation consumers.
- `current-source-bundle.yml`: current Gethe checkout, Rust source manifest build
  and verification, native Ketho generation, final artifact/hash/map validation.
  Source admission must be complete; projection omissions remain explicit partial
  reports. This is not a language-server semantic compatibility certificate.
- `branch-hygiene.yml`: remove only branches already represented by main.

Build/source jobs are read-only. Only the branch-hygiene job has write permission.
No interpreter setup, embedded interpreter script, self-modifying source payload,
finalizer or hidden publication step is permitted. `cargo xtask check` rejects
interpreter assets and ordinary reintroduction through scripts or CI.

The old v1 API/topology producer workflow has been retired. Native importers
remain compatibility readers tested with synthetic JSON fixtures; they are not
claimed to regenerate full UI topology from current source.
