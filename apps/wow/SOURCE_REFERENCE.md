# Explicit native source reference

```text
wow source reference --config <source-reference.json> [--format json|text]
```

This read-only operation derives a Release `ProfileIdentity`, typed
`ReferenceGenerationId` and callable `ReferenceView` from an explicitly pinned
generated-API corpus. It shares the source-selection, profile, generation,
document ingestion and callable projection used by
[`local-project-native`](NATIVE_INPUT.md). Main, analyzer and annotation inputs
are unnecessary for this operation.

The strict input has exactly three fields:

| Field | Contract |
|---|---|
| `schema` | `wow-service/source-reference-input/1` |
| `profile` | Existing native profile declaration from [native input](NATIVE_INPUT.md#configuration) |
| `native_source` | Existing explicit pinned-file or [manifested TOC](NATIVE_INPUT.md#manifested-toc-input) source declaration |

The config parent is the explicit input directory. Paths, raw lengths/digests,
manifest/TOC/version closure and exact selector validation use the existing
confined project reader. The manifested route retains its local-Git acquisition
policy. No repository scan, download, Lua execution or source mutation occurs.
Flavor, Interface, version, build and environment remain explicit caller labels;
they are never inferred from an installed client.

`wow-service/source-reference-result/1` retains the configuration content digest,
exact profile/generation, source revision/environment, source member identities,
manifest receipt when present, ingestion failures, and the complete callable
projection with source evidence, candidates, conflicts and issues. The two new
contracts are transient unreleased contracts. Existing local-project profile,
generation and report contracts retain their bytes and interpretation.

The result's `profile` and `reference_generation` supply the corresponding
`PlatformTarget` fields for a separately declared
[source census](SOURCE_CENSUS.md). Copy the actual returned identities; never
manufacture them from a revision or fixture. The source-only operation uses the
existing no-corrections profile selection. Corrected/alias annotation production
remains on its existing native input route.

The existing reference owner determines callable authority. Explicit-file input,
incomplete selected closure, failed documents and in-domain projection loss
remain Partial. Conflicts and out-of-domain omissions remain explicit. A closed
callable partition grants absence authority only within that exact generated-API
selection/environment; it does not certify full WoW reference coverage.
Freshness remains `unverified-current`, and Git membership remains unattested.

Configuration reads retain the 32-MiB disk limit, and source/parser limits are
unchanged. Result serialization uses the existing cancellable 64-MiB native
report ceiling. JSON is the exact service result plus LF; text prefixes those
same bytes with a scope label. Exit codes: `0` has scoped callable authority,
`2` is Partial, `4` is acquisition/identity/budget/output failure, `64` is invalid
transport/config input, and `130` is cancellation. No annotation/Ketho parity,
Reference Pack, runtime, graph or whole-project acceptance is implied.
