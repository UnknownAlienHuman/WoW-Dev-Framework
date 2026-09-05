# Restricted APIDocumentation evaluator

**Status:** normative E1-B parse-without-execution semantics and security boundary.

The evaluator converts a frozen declarative subset of Blizzard APIDocumentation source into canonical raw values and registration observations. It is not a Lua runtime, partial interpreter for general code, or heuristic extractor.

## 1. Pipeline

```text
verified source bytes
-> pinned parser syntax tree/spans
-> evaluator preflight and allow-listed environment
-> bounded expression/statement evaluation
-> known registration call observations
-> raw canonical value tree
-> unsupported/malformed/security records
```

No source code executes on the host or in a WoW client.

## 2. Parser contract

Before code, freeze:

```text
parser crate/revision/version/features
accepted WoW/Lua dialect
source encoding/BOM/line-ending behavior
syntax node/span semantics
numeric/string literal behavior
parse error recovery behavior
large/malformed input behavior
compatibility probe/report ID
```

One correctness-path parser only. Regex/text matching may aid triage but cannot emit canonical facts.

Parser diagnostics become source evidence/coverage; recovery nodes are never treated as equivalent valid syntax without an explicit evaluator rule.

## 3. Evaluation environment

The environment is closed and immutable per file/partition/build:

```text
allow-listed local bindings created by supported source statements
allow-listed known constants/tables from exact registered environment bundle
allow-listed registration functions
allow-listed pure helper constructors/functions with frozen semantics
step/depth/size budgets
source/profile/partition/parser/evaluator identity
```

It excludes:

```text
_G and arbitrary globals
filesystem/network/process/environment
package/require/load/dofile
metatables/debug/coroutines
WoW runtime APIs
clock/randomness
user/editor config
source-supplied executable callbacks
```

Unknown global/call/access is unsupported, not nil/default.

## 4. Supported statements

Initial subset, activated only through corpus tests:

```text
local variable declarations with supported expressions
local assignment to existing allow-listed bindings
known registration-call expression statements
return only if a selected helper/module form requires it and exact semantics are frozen
```

Potentially unsupported until proven:

```text
nonlocal/global assignment
multiple assignment with complex arity
if/for/while/repeat
do blocks
function definitions/closures
method definitions
labels/goto
break
```

Do not implement them “generically” for future use.

## 5. Supported expressions

### Literals

```text
nil
boolean
string
integer/number under canonical numeric policy
```

### Tables

```text
bounded table constructors
explicit keyed fields
identifier-name fields
array-sequence fields
nested supported values
```

Table semantics record source field order and canonical key/value observations. Duplicate/computed/nil key/value behavior is explicit.

### References

```text
allow-listed local binding reference
field/index access on a canonical supported table/binding
allow-listed known constant/enum/reference path
```

No arbitrary metatable/index invocation.

### Constant expressions

Only frozen pure bounded forms required by the corpus, such as selected:

```text
unary sign on numeric literal
string concatenation of known canonical strings
basic arithmetic/bit-like enum composition if exact source requires and overflow/type semantics freeze
parentheses
```

Every operator is individually allow-listed. No generalized Lua expression evaluator by default.

### Helper calls

A helper can be supported only when:

- exact function identity and source semantics are reviewed;
- pure/deterministic/no side effects;
- inputs/outputs are canonical values;
- recursion/work bounded;
- implementation fixture and mutation tests exist;
- helper version enters evaluator/profile identity.

Unknown helper call remains unsupported.

## 6. Known registration calls

The evaluator recognizes exact registered forms, conceptually:

```text
APIDocumentation:AddDocumentationTable(value)
selected equivalent generated documentation registration surface proven by source
```

Each registration descriptor defines:

```text
exact callee identity/receiver form
argument arity and value requirements
registration kind/system/partition mapping
source order semantics
duplicate/error behavior
output RegistrationObservation shape
```

No call is accepted because its name looks similar.

## 7. Local bindings

```text
EvaluatorBinding
    binding ID/name/scope/source span
    canonical value ID
    assignment ordinal
    mutability policy
```

Rules:

- lexical scope exact;
- use-before-binding unsupported/error;
- unsupported assignment invalidates binding/dependent registrations, not guessed previous/default value;
- alias/reference cycles rejected;
- table mutation supported only through an exact frozen pattern if required; otherwise unsupported;
- no environment leakage across files unless an explicit partition environment bundle declares it.

## 8. Tables, keys, duplicates

Raw table observation preserves:

```text
source field ordinal
key form/value/type
value ID
source span
computed/explicit/array form
```

Canonical semantic lowering declares how to treat:

- array fields and Lua indexing;
- duplicate exact keys;
- explicit nil values;
- numeric key canonicalization;
- map ordering;
- mixed arrays/maps;
- unsupported keys.

Never silently collapse duplicate source observations before conflict/evidence policy. A last-write semantic can be emitted only if exact Lua/source contract requires it, while preserving all raw fields.

## 9. Numbers

Freeze canonical number policy before code:

```text
integer vs floating representation
accepted literal syntax/base/exponent
negative zero
NaN/infinity rejection or representation
precision/rounding
overflow/underflow
integer range
canonical text/binary encoding
```

If parser/source semantics cannot represent a value losslessly under the contract, preserve exact literal/raw syntax digest and mark dependent normalized field unsupported/partial rather than inventing a rounded value.

## 10. Strings and encoding

- verify source encoding policy;
- decode Lua string literal escapes exactly under pinned parser/contract;
- preserve logical string bytes/text and source literal evidence;
- bound literal/logical string length;
- distinguish invalid encoding/escape/recovery;
- no Unicode normalization unless field-specific canonical contract explicitly requires it;
- raw values preserve exact semantic string, while source handle retains source bytes/digest.

## 11. References and constants

Known constant bundle is versioned/profile-bound:

```text
EvaluatorEnvironmentBundle
    environment ID/version
    known binding/constant/table descriptors
    source/evidence
    canonical digest
```

Reference resolution records the reference path and resolved canonical value. Unknown/missing/contradictory constant makes dependent evaluation unsupported/conflicted; no lookup from local WoW/editor/global runtime.

## 12. Unsupported constructs

Produce `UnsupportedConstructRecord` with:

```text
source file/node/span
construct kind and evaluator code
bounded syntax summary/digest
registration/entity/field context if known
dependent bindings/observations/capabilities/partitions
parser/evaluator/profile IDs
```

Classification examples:

```text
unsupported_statement
unsupported_expression
unknown_global
unknown_call
unsupported_helper
unsupported_operator
unsupported_table_key
binding_cycle
budget_exceeded
parse_recovery_node
security_forbidden_construct
```

No raw full source in default error.

## 13. Failure propagation

Dependency graph:

```text
source node -> binding/value -> registration -> raw field -> normalized fact -> capability partition
```

An unsupported binding invalidates only exact dependents. If a mandatory registration/file/system completeness criterion depends on it, the owning partition becomes Partial/Failed.

Do not mark the entire profile failed unless identity/root/manifest/parser/evaluator state makes all results unreliable.

## 14. Parse diagnostics

- fatal file parse failure: file/partition failure; no recovered registrations unless an explicit safe recovery contract exists;
- bounded recoverable diagnostic outside observed registration: record diagnostic/coverage impact per policy;
- recovery node inside value/registration: unsupported, no fact guess;
- malformed duplicate/unterminated source: no best-effort regex extraction.

## 15. Budgets

Per build/partition/file/registration/value:

```text
source bytes/files
syntax nodes/depth
evaluation steps/call depth
bindings
registration calls
raw values/nodes/table entries/depth
string bytes
numeric literal size
unsupported/diagnostic records
output bytes
```

Budget exceed returns structured state and downgrades affected partitions. No partial table/fact treated complete.

## 16. Cancellation

Check cancellation:

- before/after file parse;
- between top-level statements/registrations;
- at bounded evaluator step/table-entry intervals;
- before raw/normalized/store plan publication.

Cancellation produces no ReferenceData publication and no background continuation. Intermediate records remain candidate-local and are discarded/quarantined according to build policy.

## 17. Determinism

Equivalent source/parser/environment/policy/budget produces equivalent:

```text
EvaluationRecord IDs/digests
RegistrationObservation sequence
RawCanonicalValue IDs/digests
UnsupportedConstructRecord IDs/digests
partition status
```

Independent of thread scheduling, hash-map order, temp path, diagnostic prose, wall clock.

## 18. Security mutation corpus

Reject/record without execution:

```text
load/loadstring/dofile/require/package
os/io/debug/coroutine
metatable/__index/__call tricks
function/closure passed to registration
infinite/huge loop or recursion
huge/deep table/string/number
filesystem/network/process/client/editor global
source comment/prompt instruction
unknown call named like registration/helper
computed key with side effect
malformed parser-recovery payload
```

Tests assert no marker file/network/process/global side effect occurred.

## 19. Required operations

```text
validate_parser_compatibility_probe
build_evaluator_environment_bundle
validate_evaluator_environment_bundle
parse_reference_source_file
evaluate_supported_statement
evaluate_supported_expression
evaluate_table_constructor
resolve_evaluator_binding_or_constant
recognize_known_registration_call
emit_registration_observation
emit_raw_canonical_value
emit_unsupported_construct_record
propagate_evaluation_failure_to_partitions
canonicalize_evaluation_output
```

## 20. Required tests

- every supported literal/table/binding/access/operator/registration form;
- duplicate/mixed/nil/computed table keys;
- number/string edge cases;
- unknown global/call/helper/operator;
- binding use-before-def/cycle/unsupported assignment;
- parse fatal/recovery node cases;
- no-execution security corpus;
- budget and cancellation at every level;
- dependent-only versus partition-wide failure propagation;
- deterministic outputs under statement-independent parallelism/input order where allowed;
- parser upgrade span/value/diagnostic compatibility and last-known-good pin.

## 21. Hard stops

- no general interpreter/runtime;
- no arbitrary call/global/control-flow execution;
- no regex fact extraction from invalid source;
- no unknown-as-nil/default;
- no unsupported value partial fact marked complete;
- no unbounded tables/strings/steps;
- no cross-file environment leakage without exact bundle;
- no source side effect;
- no parser/evaluator version drift outside generation identity.


## Implemented native subset: declarative profile v2

`../src/native.rs` uses EmmyLua's lexer/AST and captures immutable literal tables,
local bindings, exact registration calls, Enum/Constants paths and binary `+`/`-`.
It does not execute Lua. Bare unknown names inside data become `UnresolvedName`;
unknown registration roots, calls, mutation, helper execution and other binary
operators remain rejected. Original expression nodes and UTF-8 spans are retained.

`../src/native_constants.rs` resolves only the caller-selected normalized corpus.
Enumeration members use `Fields/EnumValue`; constant members use `Values/Value`
and retain their descriptor `Type`. A string naming a present enum member resolves
through that enum; a string with an unknown named descriptor type stays unresolved,
not an invented runtime string. Function defaults are already Lua values and do
not inherit descriptor-string interpretation. No global-name allowlist is added.

Scalar results include the source hashes/spans of every participating value.
Duplicate group/member definitions, including equal duplicates, block resolution.
Cycles, unresolved names/paths, mixed revisions, unsupported values, budget
exhaustion and cancellation remain distinct errors. Arithmetic is deliberately
integer-only within ±(2^53−1), including intermediates; no floating-point rounding,
string coercion or signed-zero normalization. Direct numeric lexemes stay raw.
Each resolution has a 48-depth/4096-step budget; the catalog admits at most 65536
value definitions. The annotation consumer also bounds accumulated evidence bytes.

The consumer reports a failed resolution against that value/declaration and keeps
unrelated supported declarations. This slice does not implement the full E1
persistent ReferenceView/correction contract, host runtime values, or a complete
Lua constant evaluator. Successful scalar resolution cannot upgrade coverage or
runtime safety authority. Profile versions identify behavior; they do not pin a
WoW build or permanently bind a source/dependency revision.
