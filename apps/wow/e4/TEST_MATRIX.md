# `apps/wow` E4-C CLI acceptance and mutation matrix

**Status:** normative. All IDs are unique within the application E4-C package.

## Dependency and command routing

| ID | Case | Expected |
|---|---|---|
| `C4-DEP-001` | App depends only on `wow-service` | pass |
| `C4-DEP-002` | Import lower framework crate | architecture failure |
| `C4-CMD-001` | Every documented command maps to one service operation | pass |
| `C4-CMD-002` | Unknown command/option | exit 64; no service call |
| `C4-CMD-003` | Missing required option | exit 64; no service call |
| `C4-CMD-004` | Command performs two service calls | mutation fails |
| `C4-CMD-005` | Command retries service automatically | mutation fails |
| `C4-CMD-006` | Command changes semantic request from terminal/locale | mutation fails |

## Config and explicit input

| ID | Case | Expected |
|---|---|---|
| `C4-IN-001` | Valid explicit strict config | pass |
| `C4-IN-002` | No `--config` | no implicit config discovery |
| `C4-IN-003` | Unknown config field/version | exit 64 |
| `C4-IN-004` | Include/import/template/environment interpolation | reject |
| `C4-IN-005` | Oversized/deep JSON | bounded exit 64 |
| `C4-IN-006` | Directory/glob/recursive input | reject |
| `C4-IN-007` | Symlink/reparse/path escape | reject by platform profile |
| `C4-IN-008` | Archive/polyglot/media sniffing | no extraction/sniffing; reject mismatch |
| `C4-IN-009` | More than one stdin consumer | reject |
| `C4-IN-010` | Invalid UTF-8 where strict JSON required | reject |
| `C4-IN-011` | cwd/home/env/editor/Git/WoW discovery | mutation fails |
| `C4-IN-012` | Network fetch URL in config | reject/no network |

## Selectors

| ID | Case | Expected |
|---|---|---|
| `C4-SEL-001` | Valid exact publication selector token | typed service field |
| `C4-SEL-002` | Allowed `current` token | passed unchanged to service |
| `C4-SEL-003` | App resolves current/catalog locally | mutation fails |
| `C4-SEL-004` | Invalid selector prefix/size | exit 64 |
| `C4-SEL-005` | Name/path used as entity selector | reject |
| `C4-SEL-006` | Exact shard/snapshot token | passed mechanically |
| `C4-SEL-007` | App picks among multiple catalog matches | mutation fails |
| `C4-SEL-008` | Unavailable exact selector falls back to current | mutation fails |

## Search query and selection

| ID | Case | Expected |
|---|---|---|
| `C4-SEARCH-001` | Valid bounded CLI query subset | one service request |
| `C4-SEARCH-002` | Valid strict query JSON | one service request |
| `C4-SEARCH-003` | CLI constructs raw SQL/FTS/regex | mutation fails |
| `C4-SEARCH-004` | Query text interpreted as command/profile | mutation fails |
| `C4-SEARCH-005` | Search continue with selector/profile override | exit 64 |
| `C4-SEARCH-006` | Explain exact result/candidate | one service request |
| `C4-SEARCH-007` | Select exact result/result-set/candidate | one service request |
| `C4-SEARCH-008` | Select by rank/first/top/best/name | reject |
| `C4-SEARCH-009` | Sole candidate auto-selected | mutation fails |
| `C4-SEARCH-010` | Search context without explicit candidate | reject |
| `C4-SEARCH-011` | Query text reused as context root | mutation fails |
| `C4-SEARCH-012` | Candidate guard mismatch returned by service | faithful exit 3 |

## Lineage and review

| ID | Case | Expected |
|---|---|---|
| `C4-LIN-001` | Lineage build exact before/after/profile/operation | one service request |
| `C4-LIN-002` | Lineage compare by exact entities/assertion | pass |
| `C4-LIN-003` | Compare by display names only | reject |
| `C4-LIN-004` | Trace/explain exact IDs | pass |
| `C4-REV-001` | Valid review envelope file | transported to service |
| `C4-REV-002` | Plain prose approve/reject | reject |
| `C4-REV-003` | App infers reviewer from OS/GitHub/file owner | mutation fails |
| `C4-REV-004` | Raw private key/token in config/argv | reject/redact |
| `C4-REV-005` | App validates/promotes proof itself | mutation fails |
| `C4-REV-006` | Review apply without exact base snapshot/operation ID | reject |
| `C4-REV-007` | Review note printed in normal diagnostics | privacy failure |
| `C4-REV-008` | Service returns checked Unauthorized/Invalid | exact exit mapping |

## Migration and impact

| ID | Case | Expected |
|---|---|---|
| `C4-MIG-001` | Migration candidates exact snapshot/root | one service request |
| `C4-MIG-002` | Migration validate explicit artifact | one service request |
| `C4-MIG-003` | Apply/edit flag or command | unavailable/reject |
| `C4-MIG-004` | Artifact filename auto-determines schema | mutation fails |
| `C4-IMP-001` | Impact plan exact roots/snapshot/profile | one service request |
| `C4-IMP-002` | Impact run exact plan fields/artifact | one service request |
| `C4-IMP-003` | Impact continue with fresh budget/profile | reject |
| `C4-IMP-004` | Impact root inferred from query text | reject |
| `C4-IMP-005` | App labels path runtime breakage/severity | mutation fails |
| `C4-IMP-006` | Impact explain exact path/result | pass |

## Output and exit codes

| ID | Case | Expected |
|---|---|---|
| `C4-OUT-001` | Envelope JSON stdout | exact service bytes + LF |
| `C4-OUT-002` | Text output preserves candidate/partial/conflict/nonclaims | pass |
| `C4-OUT-003` | Artifact output | exact eligible bytes, no wrapper/newline |
| `C4-OUT-004` | Multiple/no eligible artifacts in artifact mode | typed failure |
| `C4-OUT-005` | Progress/banner on stdout JSON | mutation fails |
| `C4-OUT-006` | Broken pipe | no retry/double output |
| `C4-OUT-007` | Explicit file atomic write | pass |
| `C4-OUT-008` | Invalid destination before service | exit 64; no service call |
| `C4-OUT-009` | Output failure after service | exit 4; no second call |
| `C4-OUT-010` | Private review/source data on stderr | fail |
| `C4-EXIT-001` | Complete/NoChange/Valid | 0 |
| `C4-EXIT-002` | Invalid validation | 1 |
| `C4-EXIT-003` | Partial/CandidateOnly/ConflictBlocked/Truncated/NotEvaluated | 2 |
| `C4-EXIT-004` | Structured selector/artifact/retention/domain failure | 3 |
| `C4-EXIT-005` | Internal/closure/serialization/output failure | 4 |
| `C4-EXIT-006` | Pre-service CLI/config/input failure | 64 |
| `C4-EXIT-007` | Cancelled | 130 |
| `C4-EXIT-008` | Exact authoritative search miss with Complete status | 0 |

## Cancellation and lifecycle

| ID | Case | Expected |
|---|---|---|
| `C4-CAN-001` | Cancel before service call | no service call; typed exit |
| `C4-CAN-002` | Cancel during service | propagate once; exit 130 |
| `C4-CAN-003` | Cancel during output | stop; no reinvocation |
| `C4-CAN-004` | Background input/output/service work after return | mutation fails |
| `C4-CAN-005` | Broken pipe treated as new request | mutation fails |

## Security and determinism

| ID | Case | Expected |
|---|---|---|
| `C4-SEC-001` | Shell/process/network/editor/client/model/plugin call | absent |
| `C4-SEC-002` | Raw SQL/FTS/regex execution | absent |
| `C4-SEC-003` | Source/context text treated as CLI instruction | mutation fails |
| `C4-SEC-004` | Context artifact treated as edit/tool permission | mutation fails |
| `C4-SEC-005` | Private path/credential/signature leak | fail |
| `C4-SEC-006` | Oversized argv/input/output | bounded failure |
| `C4-DET-001` | Same service bytes, different terminal/locale | same stdout/exit |
| `C4-DET-002` | Reordered equivalent strict config | same service request |
| `C4-DET-003` | Host/path/timing difference | no semantic output change |
| `C4-FIX-001` | Null pins while not-started | allowed |
| `C4-FIX-002` | First app Rust commit with required nulls | fail |
| `C4-FIX-003` | Lower dependency/Cargo/workflow introduced in docs phase | fail |

## Acceptance

The app is not implementation-ready-to-merge until all command grammar, exact selector, one-call, candidate-selection, review transport, migration nonexecution, impact nonclaim, output/exit, cancellation, security and deterministic byte vectors are frozen and pass against the implemented E4-C service.
