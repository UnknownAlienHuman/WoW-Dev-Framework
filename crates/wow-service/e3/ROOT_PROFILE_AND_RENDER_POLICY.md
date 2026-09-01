# E3-C root, profile, budget, and renderer policy

**Status:** normative outer-boundary resolution policy.

## Exact roots

E3-C accepts E3-B root selector kinds only:

```text
ProjectId / exact project root
PackageId
ProjectFileId
SourceHandleId plus exact span where required
Graph EntityKey or EntityId
Graph RelationKey or RelationAssertionId
ReferenceEntityKey or ReferenceEntityId
Existing FindingId or EvidenceId
ProjectMapNodeId
L0SkeletonId / L1SkeletonId
```

`context_map` may derive the exact acquired project root from the selected publication. Every other semantic root is explicit.

Forbidden:

```text
free-text symbol name
filesystem path or glob
source substring
natural-language query
regex
SQL/expression/callback
"best" or "relevant" selector
repository/addon/provider name as semantic lookup
```

E4 `wow-search` may later resolve user text to explicit candidates; E3-C will require an explicit selected exact root and preserve search evidence separately.

## Profile aliases

Service configuration may expose stable administrative aliases for exact immutable profile IDs, for example:

```text
default-map -> ProjectMapProfileId X
inspect-entity -> ContextProfileSetId Y
local-markdown -> RendererProfileId Z
```

Rules:

- aliases are service configuration, not project/source data;
- resolve once before constructing the canonical E3-B request;
- record alias as supplemental request metadata and exact resolved ID as semantic input;
- unknown/ambiguous aliases fail;
- alias target change changes configuration ID and future request identity;
- continuation stores exact profile IDs and does not re-resolve aliases;
- no environment/cwd/editor/model/provider-dependent alias.

## Defaults

Defaults are exact profile IDs in `ContextServiceConfiguration`. Omitting a CLI flag selects a documented configured default, then the service records the exact ID.

No hidden default may:

- add source excerpts;
- broaden confidence/provenance classes;
- include Blizzard UI when selector says omitted;
- choose a newer ReferenceProfile;
- enable external transmission;
- invoke a renderer/model/search lane;
- raise budgets beyond system maxima.

## Budget overrides

A request can choose an exact budget profile and a closed set of bounded numeric overrides permitted by that profile. Service validates:

- type/range/overflow;
- system maximum;
- profile compatibility;
- mandatory minimum feasibility where cheaply known;
- total continuation-chain budget semantics.

Service passes normalized values to E3-B. It does not prune items or recalculate semantic costs.

## Source/privacy policy

Service selects exact:

```text
SourceExcerptProfileId
PrivacyConsumerProfileId
SourceBoundaryProfileId
```

The effective policy is no broader than service configuration and request. A caller cannot upgrade from metadata-only/local-only to external-source permission without an explicitly permitted profile.

Unknown privacy/license state remains denied or metadata-only according to E3-B profile. Service cannot override an E3-B denial.

## Renderer selection

Renderer selection is explicit:

```text
None
CanonicalJson(RendererProfileId)
DeterministicMarkdown(RendererProfileId)
```

Multiple renderers are permitted only within configured count/byte/token limits and are ordered by exact profile ID/request order rules.

Service validates profile availability/compatibility and delegates all rendering to `wow-context`.

Forbidden:

- service-written Markdown/JSON semantic rendering;
- terminal formatting affecting renderer bytes;
- renderer chosen from `Accept` header, terminal type, locale, file extension, or model name without an explicit transport adapter policy;
- renderer changing selected facts;
- exact token claim without exact tokenizer/framing profile;
- silently falling back from Markdown to JSON or exact tokenizer to estimate.

Fallback requires a new explicit policy/result identity and is reported.

## Artifact validation levels

```text
StructuralOnly
    schema, canonicalization, digest, internal reference, boundary, and local budget checks

ExactOwnerClosure
    additionally reacquire exact project/graph/reference generations and validate every origin/evidence/source reference
```

The requested level is exact. If owner generations are unavailable, `ExactOwnerClosure` does not downgrade silently.

## Operation profile registry

Each public operation has a versioned profile defining:

- allowed selector/root kinds;
- mandatory/optional universes;
- required owner capabilities;
- permitted context profiles/renderers;
- acquisition and closure behavior;
- status derivation;
- partial/truncated/NotEvaluated policy;
- source/privacy limits;
- continuation retention requirement;
- failure classification;
- output envelope schema.

Project/source data cannot modify this registry.

## Configuration evolution

Changing alias targets, defaults, allowed profiles, status/failure policy, acquisition order, privacy ceiling, renderer behavior, or exit-code-relevant classification creates a new configuration/contract version and tests.

## Validation tests

- exact roots accepted; fuzzy/name/path forms rejected;
- aliases resolve deterministically and exact IDs appear in requests/results;
- continuation ignores changed alias registry and uses original exact IDs;
- defaults cannot broaden source/privacy/confidence;
- renderer/tokenizer mismatch/fallback behavior explicit;
- budget overflow/negative/unlimited rejected;
- same exact request independent of terminal/locale/environment;
- source/project cannot inject profiles or aliases.
