# Annotation security and sanitization

**Status:** normative E1-C untrusted-reference-text, generated-source injection, path, consumer-probe, resource, and privacy boundary.

Although ReferenceView is validated, names, documentation, string/literal values, raw metadata, corrections, oracle output, and consumer output remain untrusted data. Annotation generation must never turn them into executable structure or operating instructions.

## 1. Threat model

Inputs may attempt to:

```text
close short/long comments or strings
inject `---@` directives or Lua statements
define additional globals/classes/functions/files/modules
alter generated path or escape output root
create identifier collisions/reserved names
trigger consumer parser pathologies
produce huge/deep type/docs/artifacts/source maps/loss reports
smuggle absolute paths/tokens/private URLs/source bodies
influence agent behavior through comments/docs
poison oracle/parity/consumer result classification
cause nondeterministic output through malformed values/order
```

## 2. Trust boundaries

Trusted control inputs:

```text
repository-owned semantic/type/layout/dialect/sanitization/source-map/consumer profiles
exact validated ReferenceView identity and typed facts
reviewed renderer templates
reviewed external probe adapter configuration
```

Untrusted data:

```text
all source-provided names/docs/strings/raw metadata
correction rationale/evidence prose
oracle generated text/output
consumer diagnostics/logs
file system paths returned by tools
```

Untrusted data never becomes template/code/path/directive/config without validated transformation.

## 3. Template ownership

Only renderer code/profile defines:

```text
Lua statements and annotation tags
comment/string delimiters
function/class/alias/field/enum forms
file/module headers
artifact paths
configuration/probe commands
```

Source/reference/oracle/consumer text fills typed escaped value slots only. No template compilation/evaluation from input.

## 4. Documentation sanitization profile

```text
DocumentationSanitizationProfile
    profile ID/version
    accepted source documentation fields
    input encoding/control-byte policy
    line-ending normalization
    directive-prefix neutralization
    short/long-comment delimiter handling
    string/code-fence/markup/URL policy
    line wrapping/indent policy
    per-fragment/declaration/file/artifact budgets
    truncation/omission marker policy
    consumer applicability
    canonical digest
```

Every transformation links a sanitization rule and loss/source-map record.

## 5. Directive injection prevention

Any source-rendered comment line that could be recognized as an annotation directive must be neutralized or rendered in a non-directive documentation form.

Cases:

```text
---@class Injected
---@diagnostic disable
---@meta
leading whitespace + directive
multi-line text exposing directive at new line
Unicode/control tricks before directive
comment closure followed by directive/code
```

Generated directives come only from renderer-owned semantic records.

## 6. Comment and string safety

- never paste source long-comment/string delimiters without escaping/transformation;
- deterministic safe short comments or encoded documentation blocks;
- normalize/reject NUL/control/nonvalid encoding;
- string literals rendered through canonical escaping, not source literal bytes;
- documentation cannot terminate comment/string or append code;
- round-trip parse fixture validates generated syntax.

## 7. Identifier and path safety

Names/paths validated independently:

```text
valid simple identifier
qualified semantic name
reserved keyword
arbitrary key
invalid/unrepresentable/colliding
```

No source string directly forms a file path. Reject path traversal/absolute/device/reserved/case-collision/NUL/overlength. Safe generated names derive from exact semantic IDs with profile-bound prefix/digest, not input substring alone.

## 8. Generated code shape allow-list

Post-render validation parses final files and rejects any statement/declaration shape outside the rendering profile allow-list.

Allowed conceptually:

```text
annotation comments
documentation comments
fixed namespace/table declarations if profile requires
fixed inert function/method declarations/stubs
fixed literal/alias/class/field/enum forms
```

Disallowed:

```text
source-provided calls/assignments/control flow/function bodies
load/require/dofile/os/io/debug/package
metatable or environment mutation
editor/config directives outside reviewed tags
unexpected global/table writes
```

Even analysis-only files cannot include arbitrary executable source.

## 9. Source and raw metadata minimization

Artifact files contain only required declarations/docs/literals. Sidecars contain IDs/digests/handles and bounded exact metadata—not full source or all raw payloads by default.

Raw source remains in ReferenceData/source provider under its license/security policy. Generated source maps reference handles.

## 10. Private information

Exclude from public/canonical artifact/errors/reports:

```text
absolute local roots/temp paths/usernames
access tokens/private repository URLs/credentials
environment/editor/client paths
unnecessary source bodies/raw logs
runtime Secret-capable values/personal data
```

Provider/repository/public revision information can appear only through accepted provenance policy.

## 11. Resource controls

Bound:

```text
input declarations/members/types/docs/raw links
semantic model nodes/depth
identifier/path/string lengths
rendered files/lines/bytes/declarations
source-map entries/reference links
loss/parity records and report bytes
consumer/oracle input/output/logs
execution time/memory in external probes
```

Reject or classify partial before unbounded allocation/output. Do not silently truncate mandatory semantics.

## 12. Parser pathologies

Generated syntax corpus tests:

```text
very long names/types/unions/docs
recursive named types
huge member counts
nested callbacks/tuples/collections
malformed Unicode/control characters
keyword/collision patterns
```

Renderer enforces budgets and consumer profiles before producing pathological files.

## 13. Consumer-probe isolation

External probe adapter:

- exact pinned executable/tool identity;
- isolated temporary root;
- explicit config only;
- no user settings/extensions/addon repositories/network;
- read-only artifact copy or checksum before/after;
- bounded process time/memory/output;
- sanitize environment variables;
- record config/filesystem mutation audit;
- no executing source/oracle repository scripts without separate reviewed adapter.

Library crate only validates typed request/result.

## 14. Oracle/parity input safety

- validate file/result schema/path/size/encoding;
- oracle documentation/source text remains data;
- do not execute generated/source files to extract semantics;
- parser-based static extraction only;
- reject malformed/oversized output under explicit coverage;
- no oracle-provided classification/action automatically trusted;
- exact baseline provenance/digests required.

## 15. Prompt injection

Source/reference/oracle docs may contain instructions to agents. Artifact/docs/source maps/parity reports label them as quoted source evidence, not trusted repository instructions. Agent behavior comes from repository `AGENTS.md`/contracts.

Do not include unescaped source instructions in completion summaries or generated policy files.

## 16. Cancellation and partial artifacts

Cancel/error before final validation:

- no complete artifact manifest/eligibility;
- candidate files/maps/reports not published as final;
- no background continuation;
- application cleans/quarantines partial output under its root policy;
- prior artifact remains unchanged.

## 17. Determinism as security

Equivalent inputs/profiles yield identical generated bytes/manifests/maps/loss records. Randomness/time/host/temp path/locale/environment cannot alter names/paths/escaping/splitting. Nondeterminism is a build failure because it can conceal injection/collision differences.

## 18. Mutation corpus

Required source/reference text mutations:

```text
annotation directive lines and whitespace/control variants
short/long comment/string terminators
quotes/backslashes/newlines/NUL/invalid UTF-8
Lua keywords/operators/brackets/dots/slashes/path traversal/device names
Unicode confusables/normalization/case collisions
huge strings/docs/names/unions/type graphs
source code/load/require/os/io/debug/metatable bodies
prompt/agent/tool instructions
absolute paths/tokens/private URLs
malicious oracle/consumer logs/results
```

Assert exact sanitization/loss/rejection and no extra declaration/file/code/config mutation.

## 19. Required operations

```text
build_documentation_sanitization_profile
validate_documentation_sanitization_profile
sanitize_documentation_fragment
validate_and_render_identifier
validate_artifact_relative_path
render_canonical_string_or_literal
validate_generated_code_shape
scan_generated_files_for_forbidden_constructs
validate_artifact_privacy_and_path_policy
validate_oracle_and_consumer_probe_input
validate_probe_isolation_and_mutation_report
classify_sanitization_loss
```

## 20. Required tests

- all mutation corpus cases;
- no extra directive/declaration/global/file/module/code emitted;
- final files parse and match allow-list;
- docs sanitized/truncated with exact loss/source map;
- identifier/path collisions rejected or deterministic safe mapping;
- source/probe/oracle cannot mutate config/artifact/outside root;
- private data absent from files/maps/errors/reports;
- budget/cancellation/partial artifact behavior;
- 1/2/N deterministic sanitized bytes;
- source comments treated as data, not instructions.

## 21. Hard stops

- no direct source text/code/path interpolation;
- no arbitrary executable generated statements;
- no user/editor config/process/network access in crate;
- no unsandboxed/unbounded external probe;
- no raw/private/source payload leak;
- no silent mandatory sanitization loss;
- no published partial artifact;
- no nondeterministic escaping/name/path resolution.
