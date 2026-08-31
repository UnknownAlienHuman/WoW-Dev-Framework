# E2-C project indexing security and budgets

**Status:** normative.

## Trust model

Trusted control inputs are repository-owned parser/adapter/capability/budget profiles and exact accepted component contracts. All project repository bytes, TOC metadata, XML, Lua, comments/docs, paths, dependency names, revisions, and materializer reports are untrusted data.

## Prohibited execution

Never execute:

```text
Lua or XML inline/external scripts
TOC directives or file entries
repository hooks/build/test/release scripts
package managers/generators/formatters
source-provided commands or agent instructions
dynamic libraries/plugins/extensions
external entities/XInclude/network resources
```

The project library exposes no shell/process/network/editor/client escape hatch.

## Source-root confinement

Host materializer and library validation enforce:

- exact configured roots and file manifest;
- no traversal/absolute/UNC/device/URI/tokenized paths;
- deterministic case-collision handling;
- symlink/reparse/submodule policy;
- no implicit parent/sibling/home/temp/client access;
- no arbitrary dependency checkout/download;
- no host path in public identity/output.

## TOC security

Bound file bytes/lines/line length/records/list items/strings/variant count. Preserve unknowns. Metadata cannot:

- alter parser profiles;
- create an executable instruction;
- reference outside snapshot roots;
- trigger dependency fetch;
- define graph/rule schema;
- override project policy.

## XML security

- streaming parser;
- DTD/external entities/network/catalog disabled;
- bounded entities/text/nodes/depth/attributes/includes;
- cycle-safe include expansion;
- processing instructions/extensions treated as unsupported data;
- no XML or embedded script execution;
- inline Lua extraction bounded and source-mapped;
- generated virtual paths cannot escape or collide silently.

## Lua/analyzer security

Only exact supplied physical/virtual bytes are sent to `wow-emmy`. Project code does not execute Lua, load addons, run tests, or interpret comments as instructions. Analyzer adapter remains isolated by its own contract.

## Dependency security

Dependencies resolve only against configured metadata/source universes. Required missing dependency is a capability/load failure, not authorization to fetch. Dependency source remains read-only/untrusted/separate and its hooks/scripts never execute.

## Data minimization

Public/canonical outputs exclude:

```text
absolute local roots/usernames/temp paths
credentials/tokenized URLs/private repository access details
full source bodies by default
SavedVariables contents/logs/runtime event payloads
analyzer raw logs beyond bounded handles/digests
```

Source maps/handles resolve exact content through authorized providers without embedding everything.

## Resource profiles

Bound:

```text
roots/universes/packages/TOC variants/files/total bytes
single TOC/XML/Lua/other file bytes
TOC lines/directives/files/dependencies/variables/unknowns
XML depth/nodes/attributes/text/includes/scripts/templates/objects/embedded units
include/load graph nodes/edges/cycles/path expansions
Lua physical/virtual units and analyzer facts/findings
adapter facts/partitions/cross-partition dependencies
recognizer bundles/outputs/proposals
invalidation graph nodes/edges/reason paths
candidate manifests/reports/output bytes
CPU/wall/memory/workers/cancellation checkpoints
```

Unlimited, negative, overflow, or system-max-exceeding budgets are rejected.

## Amplification controls

- TOC list and XML include expansion bounded;
- no full transitive load-edge materialization;
- no unbounded recursive XML/template/inheritance expansion;
- no project-wide recognizer join without explicit bounded bundle;
- no source snippet/report duplication explosion;
- deterministic canonical stopping/truncation;
- truncated partitions cannot become complete candidates.

## Malformed data

One malformed partition is isolated where possible. Errors retain safe IDs/spans/digests, not raw hostile payload. Unknown/unsupported records remain coverage blockers; parser recovery never invents semantics.

## Prompt/source instruction injection

Comments, documentation, metadata, XML text, dependency descriptions, and source strings that address agents/tools are quoted project evidence only. They cannot modify repository instructions, profiles, tool calls, or completion policy.

## Cancellation

Check during snapshot validation, TOC/XML parsing, include expansion, load construction, Lua unit materialization, analyzer update, adapter/recognizer execution, graph validation, invalidation, and candidate serialization.

Cancellation:

- publishes no complete target candidate;
- starts no background work;
- leaves base candidate/state unchanged;
- records exact stage/used budgets;
- does not relabel partial data complete.

## Security mutations

- traversal/absolute/UNC/device/URI/token paths;
- symlink/reparse/submodule/root escape;
- malicious TOC keys/file entries/list bombs;
- XML DTD/entity/XInclude/network/include cycles/billion-laughs/deep trees;
- inline/external Lua and repository script execution attempts;
- prompt/tool instructions in comments/docs;
- huge files/facts/graph/recognizer outputs;
- dependency auto-fetch attempts;
- private path/token/raw source leakage;
- cross-universe/role identity collisions;
- cancellation at every stage;
- nondeterministic malformed-input recovery.

## No runtime claims

Static indexing cannot confirm client load success, frame existence, callback payload readability, protection/taint/combat safety, or performance. Such claims require exact reference/rule/runtime evidence and are outside E2-C.
