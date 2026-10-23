# Repository automation

The maintained workflows are:

- `quick-check.yml`: repository policy, Rust formatting, workspace compilation
  and strict Clippy. Its source archive identifies the checked tree; it is not
  proof of the public workflows or a semantic-consumer test run.
- `current-source-bundle.yml`: selects current Gethe/Ketho input, generates Lua,
  warns on partial projection, rejects omitted source inputs and uploads both
  generated directories. It no longer runs the incompatible `/1` verifier.
- `branch-hygiene.yml`: guarded cleanup of merged ancestor branches; it does not
  merge code or discard unreconciled work.

Build/source jobs have read-only GitHub permissions. Branch hygiene alone has
contents-write access. Preserve dirty/diverged checkouts and user files.

There is no current `ci.yml`, Wasm parity matrix or automatic external-checker
consumer matrix. The explicit EmmyLua/LuaLS probe is now owned by
`cargo test -p wow-reference-builder --test consumers`; follow the
[consumer guide](../docs/ANNOTATION_CONSUMER_PROBES.md) for its inputs and limits.
Do not describe an unexecuted probe or a removed workflow as passing coverage.

No interpreter setup, self-modifying source payload or hidden publication step
belongs in product CI. `cargo xtask check` enforces the native repository policy.
