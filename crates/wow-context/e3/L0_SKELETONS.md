# L0 structural skeletons

**Status:** normative E3-A compact structural projection.

## Purpose

L0 lets an agent or human identify an entity, its exact scope and location, and the shape of its immediate structural neighborhood without loading implementation bodies or speculative prose.

## Required fields

Every L0 skeleton includes:

```text
skeleton/schema/profile IDs
exact EntityKey and universe/generation
entity kind
canonical display label
exact declaration/source handles where available
stable owner/package/load-unit handles
role tags with confidence/assertion/evidence refs
direct axis parent/root handles allowed by profile
direct relation counts by declared relation kind/direction
conflict and coverage summaries with exact refs
available L1/detail/source capabilities
field-level derivation/source trace
canonical digest
```

## Allowed role labels

A role label is included only when it is:

- an exact source/reference field;
- an accepted graph kind or typed attribute;
- an accepted recognizer assertion preserving rule/version/evidence/confidence;
- a deterministic ContextProfile label derived from exact graph kinds/relations.

Do not infer `manager`, `controller`, `engine`, `core`, `service`, `module`, `singleton`, `secure`, `safe`, `hot path`, or similar meaning from a file/name/path convention.

## Direct structural summaries

L0 may summarize exact one-hop counts and selected handles for:

```text
lexical
ownership
load
object
inheritance
registration
lifecycle
state
call
API use
dependency/library
```

Every count records:

- graph query ID and snapshot;
- relation/axis/direction/confidence policy;
- visited and returned records;
- relevant producer coverage;
- conflict/truncation state.

A zero under partial coverage is not an absence claim.

## Parent handling

There is no generic parent field. The skeleton may expose separately:

```text
lexical enclosure
ownership parent(s)
load parent(s)
object/XML parent
inheritance base(s)
registration owner
state root
```

Each comes from its exact axis/relation definition. Multi-parent axes stay multi-parent.

## Source location

Prefer stable source handles and semantic symbol IDs. Human-readable path/line/column is presentation data and does not become entity identity.

Virtual XML-inline Lua units retain both virtual and mapped physical XML source coordinates.

## Size profile

Default L0 is strictly bounded per entity:

```text
maximum scalar/string bytes
maximum labels/aliases
maximum source handles
maximum displayed direct handles per axis
maximum relation-kind summaries
maximum conflict/coverage refs
maximum serialized bytes
```

If a direct neighborhood exceeds the displayed-handle limit, L0 records exact total/returned counts, deterministic omission, and an expansion recipe.

## Forbidden content

L0 must not contain:

- function/method implementation bodies;
- arbitrary comments or documentation paragraphs;
- full XML elements/documents;
- transitive call/load/inheritance closure;
- entire diagnostic sets;
- guessed intent or architecture;
- replacement/migration suggestions;
- model-generated prose;
- hidden `Possible`/`Candidate` facts;
- full local filesystem paths or secrets.

## Partitioning

One L0 partition is normally owned by one semantic entity key under one ContextInputSet/Profile. Project-level package/load-unit summary skeletons may use explicit project section owners.

Partition dependency closure includes the exact entity view, selected direct relation assertion/query records, source-handle metadata, role assertions, coverage/conflicts, and profile/rendering versions.

## Deterministic rendering

Machine representation is canonical. Compact text order:

```text
identity
kind/scope/location
roles
axis parents/roots
relation summaries
coverage/conflicts
available expansion handles
```

Labels and relations use registry/profile order, then canonical semantic key. Never source discovery, database row, hash map, or worker completion order.

## Completion criteria

- no implementation body bytes except bounded identifier/signature fragments explicitly classified as metadata;
- all fields have exact source/derivation refs;
- all zero counts preserve negative-authority state;
- independent axes remain separate;
- same logical inputs produce identical bytes;
- rename/path mutation changes only exact source/identity presentation fields allowed by the profile;
- high-fanout entities remain bounded and expandable.
