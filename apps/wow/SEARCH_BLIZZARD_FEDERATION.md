# Exact Blizzard UI source universe

The existing `search federated` command also accepts the closed input schema
`wow-service/exact-platform-search-federation-input/1`. It contains the original
`primary`, `reference` and `query` records plus required `blizzard`. The Blizzard
record uses the same exact source-hold/epoch/generation/shard/profile/bounds/query
shape as `primary`. Its ProjectStore must be a genuine E3-A platform-source
publication, separately built from primary; the slot label grants no authority.

The corresponding federation query is:

```json
{
  "schema":"wow-search/exact-primary-platform-reference-query/1",
  "blizzard_ui":"required",
  "universe_priority":["primary_user_project","blizzard_ui_source","reference_platform_contract"],
  "max_component_result_bytes":16777216,
  "max_input_candidates":256,
  "max_candidates":128,
  "max_result_bytes":16777216
}
```

Each priority occurs exactly once. Missing/null Blizzard input, unknown policy,
ordinary project in its slot, wrong original source hold/epoch or incompatible
Reference profile/generation refuses. There is no Current, missing-shard build or
nearest profile fallback. Excluded Blizzard keeps the original two-universe
schemas, bytes and profiles; an unused supplied owner is not silently dropped.

Search obtains native `ProjectKind`, original `BlizzardUiSourceProfile` and
`PlatformPackageBinding` from the genuine Project configuration while validating
its acquired Project/Graph pair. The original source profile, content/admission/
snapshot/source-universe and package load/Main binding identities are retained in
the selected universe binding. All three complete Reference profiles/generations
must agree exactly, and primary/platform graph universes remain distinct.

Source grade stays explicit. `SyntheticFixture` requires a Fixture profile;
`VendorUiSourceMirror` requires Release. A fixture never becomes vendor/current
evidence. Generic functional tests construct real fixture owners through native
parser, ReferenceStore/Search ports and platform application publication; that
does not qualify a real vendor corpus, authenticity, client runtime or Tier X.

Both source universes use the existing immutable entity projection and native
graph query. No second parser or graph implementation is introduced. Opaque
owner keys remain metadata; their exact-match cap applies to both source
universes. Original candidate/rank/coverage/conflict/graph/support records remain
separate. Cross-universe equal names do not merge; only explicit priority and
local ordinal features combine within authority bands.

Optional [captured pages](SEARCH_FEDERATED_PAGINATION.md) retain FIVE independent
roots: result, primary shard, Blizzard shard, Reference shard and ReferenceView.
Continuation validates each exact shard and captured binding without reopening
either SourceStore or executing Graph against current. Original two-universe
four-root sessions still reopen exactly. Source leases use distinct purpose IDs;
Reference closes before Blizzard before primary, including later-open failures,
and all ambiguous closure identities are preserved.

The initial three-universe paged transaction fences four original roots and adds
five session roots, so select a Store catalog-mutation limit admitting at least
nine mutations. Limits belong to the original Store configuration; existing
instances are not silently migrated or widened. Ordinary retire removes original
source/index holds, while independent sessions retain exact objects until their
last explicit release. Native GC and historical response replay retain their
existing contract.

`wow-search/exact-primary-platform-reference-federation/1` binds the new selected
interpretation; the captured Service session has its named platform schema.
Existing generic page math and physical Store remain unchanged. Additional typed
names/aliases/similarity, search-to-context and full E4/vendor/platform/runtime/
launch acceptance remain separate in #169/#106/#113.
