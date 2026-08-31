# E2-B recognizer security and resource budgets

**Status:** normative.

## Threat model

Untrusted or malformed pack/fact/corpus data may attempt to:

- execute code or load external resources;
- cause pathological joins, recursion, fanout, memory, or output;
- inject repository/path/model/prompt text into semantic conditions;
- smuggle raw source, private paths, tokens, or URLs into graph output;
- create undeclared graph kinds/attributes or cross-universe identities;
- promote Possible/Candidate facts to Derived/Proven;
- use partial coverage as a successful negative condition;
- create nondeterministic first/last match behavior;
- retain stale producer outputs after updates;
- exploit regex/backtracking or dynamic expression engines.

## Pack trust

- E2 core packs are repository-owned immutable reviewed artifacts.
- External/user/calibration packs are disabled unless a later explicit audit/activation policy applies.
- Pack parsing never follows includes, URLs, file paths, environment variables, or aliases.
- No signatures are claimed before a signing/publication milestone exists.

## Non-executable schema

Disallowed in pack and facts:

```text
Lua/JS/Wasm/native code
shell/process commands
SQL
regex/glob programs in E2
callback/function pointers
dynamic module/plugin loading
reflection/expression evaluation
network/file includes
templates that generate clauses/code
```

Only frozen typed operators from `PACK_SCHEMA.md` are accepted.

## Resource profiles

Bound at pack, rule, bundle, partition, and request levels:

```text
packs/rules/clauses/nesting depth
fact kinds/facts/fields/string bytes
indexes and join tables
join expansions/capture bindings
matches/ambiguity groups
entity/relation proposals
source/evidence/coverage refs
explanation/report bytes
partitions/total output bytes
CPU/wall/memory/checkpoint counts
```

Budgets are validated against system maxima. Unlimited/negative/overflowing values are invalid.

## Join and fanout safety

- compiler estimates worst-case cardinality using declared bounds;
- many-to-many joins require explicit caps;
- deterministic early termination uses canonical ordering;
- high-fanout outputs become Partial/truncated, not silently sampled complete;
- duplicate fact amplification is canonicalized before joins where semantics permit;
- no recursive graph/source traversal inside matcher.

## Strings and literals

- bounded UTF-8 and normalized identifiers/tags only;
- no NUL/control/private path/token data in public proposals/errors;
- literal comparison uses exact typed value, never source-code interpolation;
- source comments/docs and prompt-like text are data only and normally excluded from rule input;
- errors use IDs/counts/digests and bounded safe arguments, not raw payloads.

## Identity and scope safety

- reject host paths, SQL IDs, memory addresses, timestamps, or credentials in stable IDs;
- reject cross-universe/profile/generation joins not explicitly allowed;
- candidate/external facts cannot collide with project/reference identities;
- repository/addon metadata cannot enter semantic key or condition;
- graph registry validates endpoint/type/attribute closure independently.

## Negative-clause safety

`not_exists` fails closed unless complete closed-scope coverage is proven. Partial/conflicted/truncated input yields NotEvaluated/Partial. No match sampling can establish absence.

## Cancellation

Check during parse, validation, compilation, indexing, joins, capture expansion, proposal/explanation/report generation, and canonicalization. Cancellation:

- stops bounded work;
- publishes no complete output partition;
- starts no background continuation;
- does not relabel prior output as current;
- records exact used budget/stage.

## Privacy and source minimization

Recognizer outputs contain stable source handles and IDs, not full source bodies, SavedVariables contents, event payload values, runtime secrets, local roots, or logs. Runtime values are not recognizer inputs in E2-B.

## Forbidden side effects

Library crate has no filesystem, network, process, environment, editor, GitHub, database, or client API. It does not execute tests, source, analyzer, project, graph, or external tools.

## Security corpus

Required mutations:

- executable/include/expression/regex pack fields;
- huge/deep clause graphs and cyclic references;
- high-fanout Cartesian joins;
- oversized strings/lists/evidence refs;
- repository-name/path/prompt injection;
- private path/token/raw source leakage;
- cross-generation/universe identity collision;
- candidate/proven confidence promotion;
- partial-coverage negative clause;
- cancellation at every phase;
- nondeterministic duplicate/ordering attacks.

## Determinism as integrity

Equivalent logical inputs must produce the same plan/match/proposal/report bytes. Nondeterminism is a correctness/security failure because it can conceal rule-order, amplification, and first-match defects.
