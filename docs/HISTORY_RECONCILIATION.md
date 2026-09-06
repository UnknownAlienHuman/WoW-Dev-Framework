# Historical branch reconciliation

This is a historical disposition, not a second implementation ledger.
The active contracts and `IMPLEMENTATION_STATUS.md` continue to control.

The reconciliation starts from `6a769ec9fabe4027c2a5d7a3a829f5067eea6d8a`
(tree `6b9ae15b869c4de961283f277260c5beb60f5a63`). All 39 outstanding branch
heads are retained as ancestors of the reconciliation commit before branch refs
are removed. Current production Rust, donor goldens, data contracts and Wasm
bridges are not replaced with historical drafts. This is explicit supersession
of alternate history, not a claim that every old API was equivalent or ported.

## Disposition of alternate work

| Historical scope | Current owner / disposition |
|---|---|
| E0/E1 contracts and the accepted E2/E3/E4 packages | Exact old blobs already occur in main history; keep the subsequent current contracts. |
| Alternate E2-D project publication and store layouts | `wow-store/e2`, with project/graph ownership and the accepted single-epoch WAL partition model. Do not restore competing `wow-project/e2d` authority or obsolete physical layouts. |
| Alternate E3 source and context packages | `wow-project/e3` and `wow-context/e3`; retain source-universe separation, exact input views, budgets, continuations and privacy under their current schemas. |
| Alternate E4 search/lineage packages | `wow-search/e4`, `wow-graph/e4` and current named owner seams. Earlier type/operation IDs are not aliases for the accepted schemas. |
| Alternate E6 orchestration/handoff layout | `wow-service/e6`, `apps/wow/e6` and current E6_B owner seam files; Candidate/selection/mapping remain separate. |
| Alternate E7 standalone LSP/MCP apps and session registry | Superseded by ADR-032 and `apps/wow/e7` / `wow-service/e7`. Do not restore a second public host. |
| Two E0 implementation prototypes | Replaced by the current typed `ids`, `digest`, `envelope`, generation, source and evidence APIs. Useful unrepresented test paths are carried into `wow-core/tests/history_regressions.rs`. |
| Push/schema probes, bootstrap markers, temporary exporters and recovery manifests | Historical only; excluded from the current tree and all product bundles. |

Historical code and proposals remain inspectable by original commit/path with
`git show`, without a quarantine branch or a duplicate source archive in the
repository. A merge's retained parents establish reachability, not renewed
normative or runtime authority for their files.

## Recovered test intent

The new Rust regressions retain the old prototype's useful generation-merge,
negative-authority, evidence/remediation, finding-context and source-integrity
checks through the current contract. They cover explicit optional extension,
conflicting project generations in every merge mode, partial coverage and
conflicting duplicate partitions, Candidate/ExactEdit rejection at construction
and revalidation, cross-context findings, and content/handle identity checks.

Existing tests already cover identifier canonicalization, profile validity,
path escape, canonical result round-trips and ordered message arguments. The
old permissive IDs, implicit merge mode, map-shaped message arguments and
whole-result API are not reinstated. In particular, owner-specific remediation
eligibility is not reassigned to generic core merely to copy an old test.

No feature-completion gate advances because of history cleanup. Managed source
updates, remaining Ketho types/widget inheritance, full Wasm driver routing and
real dual-consumer semantic probes retain their existing incomplete status.
