# `wow` application

`wow-cli` is the offline frontend. Delivery state lives in
[current status](../../docs/CURRENT_STATUS.md); interface contracts are in
[FRONTENDS.md](../../docs/FRONTENDS.md).

Current check/status routes are transitional. Slice 1A is:

```text
strict explicit config
  -> one public Service ordinary-check operation
  -> Project target-bound physical plan + collision-free admitted input
  -> existing configured Emmy/check owners
  -> root-relative text/JSON result or typed pre-result error
```

The application owns argument/config precedence, display roots, rendering and
exit status. It rejects unknown config fields/unsupported options, but it does not
open source, parse XML/load facts, construct snapshots, attach limitations or
fabricate internal identities.

Complete/partial results go once to stdout. Pre-result errors after format
admission go to stderr; JSON mode emits one bounded typed error and leaves stdout
empty. Earlier CLI syntax errors are text. Channel, dependencies, full XML,
bindings, secure context and WoW rules are later slices.
