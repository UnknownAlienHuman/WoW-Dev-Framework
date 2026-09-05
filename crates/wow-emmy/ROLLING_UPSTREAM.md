# Rolling EmmyLua upstream

The upstream is updateable; exact dependency identities belong to tested builds.
See [the current upstream check](../../docs/EMMYLUA_UPSTREAM.md).

`cargo xtask check-source <checkout> <branch>` reports local/public HTTPS remote
revisions without checkout mutation. The CI dependency-update lane compiles and
tests the active parser consumers. Managed auto-update and the former standalone
surface-probe scripts are not implemented in the new command family.

The real semantic analyzer adapter remains separate unfinished work: coherent
workspace creation, diagnostics, symbol/reference/type queries, result/evidence
mapping, cancellation and resource bounds. A current parser or passing source
inventory never substitutes for those semantic acceptance tests. Do not expose
upstream database/session handles through the framework API.
