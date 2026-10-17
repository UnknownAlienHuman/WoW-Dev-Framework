# Explicit client local-file URI projection

`OverlayUriPolicy::StrictCanonical` retains the existing exact owner URI rules.
`OverlayUriPolicy::LspLocalFileV1` selects the separate
`lsp-local-file-uri/1` projection. `resolve_document_uri` returns the same
registered canonical relative path and private canonical document URI; it never
opens, scans or chooses a root. Workspace URI validation uses the same policy.
Every call requires the actual live opaque session scope and held registration.

The new policy decodes one bounded ASCII local-file URI into valid UTF-8. It
accepts an empty authority only, normalizes the Windows drive letter to uppercase
and admits its percent-encoded colon. All other component bytes and case remain
exact. Equivalent valid percent encodings resolve to one canonical path and
therefore the same private document/snapshot identity. A literal encoded percent
is not decoded again: `%252e` names literal `%2e`, not a dot segment.

Traversal, raw/encoded dot segments, duplicate separators, encoded slash or
backslash, control/invalid UTF-8/percent, query/fragment, unsupported schemes,
authority/UNC/device/ADS and root-prefix escapes are refused. Components retain
the existing portable path checks. The root is compared at a component boundary;
one final slash is allowed only for workspace URI validation. Case aliases beyond
the drive are not merged, and existing document collision guards still apply.
URI normalization never changes the physical nofollow acquisition policy.

Wire URI length is capped at 65536 bytes and relative paths at the existing 4096
bytes. Percent decoding is shared with the old strict inverse; strict calls
still require exact re-encoding and retain their refusals. Returned canonical
URIs are private owner-consumer data, without automatic Serialize/Debug exposure.
Default workspace/diagnostic metadata does not include an absolute root.

The concrete consumer is Service host-v3 and the existing thin LSP application:
[`apps/wow/LSP.md`](../../apps/wow/LSP.md). Earlier host-v1/v2 remain strict.
The new session/transport profile schemas are versioned for the additional
URI-policy metadata; old profiles retain their previous bytes. No source bytes,
Project generation, storage layout or dependency is changed by this projection.

The observed SDK source example is VS Code 1.141.0's exact installed revision:
`URI.file('C:/win/path').toString()` expects `file:///c%3A/win/path`. That
source-format fixture failed against the prior product's strict initialize
path. The installed CLI version and exact SDK source/test blobs were verified;
the SDK code was not executed, and this is not live-editor LSP qualification.
Native workspace/codec and real product wire checks retain this distinction.
