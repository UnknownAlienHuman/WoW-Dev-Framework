# Integrity and security model

**Status:** normative E1-A storage trust-boundary, validation, and hardening contract.

SQLite files, schema bundles, object payloads, roots, manifests, and operation parameters are untrusted until validated under an exact registered contract. A reputable source/provider does not bypass checks.

## 1. Trust boundaries

Potential inputs:

```text
repository-owned compiled schema/operation bundles
SQLite runtime/binding/platform adapter
staging/published SQLite files
store/pointer/object manifests
logical object byte streams
encoded/compressed object payloads
consumer-provided typed operation parameters
retention/lease/reference records
old/last-known-good generations
untrusted external SQLite/object files (import only, not owned writable)
```

Trusted by default only after identity/contract validation:

- repository-owned exact compiled bundle/catalog IDs/digests;
- configured root/platform adapter;
- accepted SQLite runtime profile;
- sealed/published store/object manifests and verified payloads.

## 2. Path confinement

Every private runtime path is constructed from:

```text
configured trusted root
registered store namespace/kind
validated fixed-format generation/object/payload ID
private generated temp/quarantine component
```

Requirements:

- reject absolute, parent traversal, device, UNC/network when unsupported, reserved/special, NUL/invalid encoding components;
- normalize without following attacker-controlled links;
- verify resolved/opened object remains under root according to platform adapter;
- reject unsafe symlink/reparse/hardlink transitions/preplacement;
- use no-follow/handle-based APIs where practical and tested;
- no source/user filename in final path;
- private absolute path redacted in public manifests/errors.

## 3. SQLite execution boundary

- loadable extensions disabled; no enable operation exposed;
- no arbitrary ATTACH/DETACH; attached database limit zero/strict internal minimum;
- no SQL from analyzed source/user/external/transport;
- no raw SQL/connection/statement handle through service/app/MCP/LSP;
- repository-owned static schema/migration/prepared-operation catalogs only;
- parameters bound, not interpolated;
- result/cardinality/size limits enforced;
- trusted-schema/defensive policy selected/probed;
- foreign keys enabled/verified;
- no source-controlled SQLite triggers/functions outside registered schema bundle;
- authorizer/defensive controls used where supported and tested, but not sole protection.

## 4. Untrusted SQLite files

Never open an arbitrary downloaded/external DB as a writable owned Reference/ProjectStore.

Allowed patterns after policy/need:

```text
open read-only in isolated import validator with strict profile/limits
inspect/stream known data through explicit importer
rebuild into a new owned staging store/schema
validate/seal/publish new generation
```

E1-A need not implement generic DB import. Direct mutation/activation is prohibited.

Risks covered:

- malicious schema/triggers/views/virtual tables;
- corrupt b-tree/page data;
- huge database/row/blob/SQL/schema objects;
- incompatible encoding/page size/application ID;
- extension/function assumptions;
- path/URI confusion;
- crafted metadata/digests.

## 5. Schema integrity

Validate:

```text
registered schema registry/bundle IDs/digests
exact normalized schema object set/types/names/digests
migration ledger IDs/order/digests
no unexpected temp/attached objects
required constraints/indexes/triggers/virtual-table capabilities
standard metadata fields/store/generation/runtime profile
```

`user_version`/`application_id` are supplemental markers only.

Unexpected object is failure; do not ignore because queries still work.

## 6. Relational integrity

- effective foreign keys ON on relevant connections;
- FK check before seal/open policy;
- unique/not-null/check constraint validation through schema/application checks;
- cardinality/count/digest expectations from registered domain validation catalog;
- no orphan object reference/generation/metadata rows;
- no generation/profile cross-link.

Store validates declared checks without interpreting domain meaning.

## 7. SQLite database integrity

Defined policy uses a combination of:

```text
successful open/schema inspection
quick_check
integrity_check
foreign_key_check
registered application checks
file checksum and length
```

Exact stages/frequency:

- staging before seal: mandatory selected full checks;
- final-path reopen before active pointer: mandatory open/schema/manifest and configured check set;
- runtime open: fast checks plus retained previous full validation evidence, with periodic/full policy explicit;
- release verification: full required set.

Missing/skipped command is `skipped/unavailable`, never pass; if mandatory, activation fails.

## 8. File and manifest integrity

- StoreManifest canonical digest verifies semantic metadata;
- SQLite file SHA-256/length recorded after final write/sidecar closure;
- final-path file rechecked per activation policy;
- pointer contains generation path/manifest digest and has own digest;
- no pointer to absent/mismatched generation;
- same-generation path mismatch/collision fails;
- no mutation after seal; file digest change rejects open.

## 9. Object integrity

For every referenced ObjectId/payload:

- manifest ID/digest/type/length valid;
- payload digest/length/codec/profile valid;
- decode under expansion/memory limits;
- decoded logical bytes hash to ObjectId;
- reference-set count/digest exact;
- final root path valid/no unsafe link;
- missing/corrupt object blocks seal/activation/read as required;
- existing mismatch quarantined, not overwritten.

## 10. Compression and decompression safety

- registered codecs/version/parameters only;
- no source/user plugin/dictionary path;
- bound logical/encoded sizes and ratio;
- stream where practical;
- reject malformed/truncated/trailing-data cases according to codec contract;
- no allocation based solely on attacker-declared size;
- cancellation checkpoints;
- payload/logical digests verified.

## 11. Root permissions and sharing

- configured root should not be writable by untrusted principals;
- platform adapter records/validates expected permissions/ACL behavior where supported;
- shared/network filesystems require an explicit tested adapter/profile; not assumed supported;
- object/store temp files use restrictive creation policy;
- secrets/tokens never stored in SQLite/object/manifests;
- local absolute root stays private operational config.

## 12. Resource limits

Bound:

```text
SQLite file/page/schema object/migration/statement sizes
row/blob/result counts and bytes
transaction operation count/bytes
object count/logical/encoded sizes/ratio
staging/quarantine/temp disk consumption
integrity/validation/GC work units
open connection/read transaction counts
busy/retry duration/count
```

Limit breach fails/cancels safely without publication/pointer advance.

## 13. Concurrency and TOCTOU

- one writer for candidate/mutable store;
- path target validated/opened through adapter minimizing check-use race;
- atomic no-replace/replace operations;
- existing target revalidated after race/rename result;
- readers acquire exact immutable generation; no active pointer reread switch;
- GC uses complete reference/lease snapshot and revalidates before delete;
- no deletion of open/leased/uncertain generation/object;
- no temp filename trust as identity.

## 14. Corruption handling

On corruption/mismatch:

```text
stop trusting affected store/object
record structured failure/quarantine
leave prior active/last-known-good untouched
prevent pointer activation/read result
retain minimal evidence according to bounded policy
rebuild/reacquire through explicit higher-layer process
```

Do not:

- VACUUM/REINDEX/update metadata as silent repair;
- delete old active to force new store;
- overwrite same ObjectId/generation;
- treat partial readable data as complete/authoritative;
- hide corruption behind a clean empty result.

## 15. Error/privacy discipline

Public errors include IDs/codes/counts/digests/relative logical identifiers necessary for remediation. Exclude:

```text
raw SQL/parameters when sensitive or large
source/object raw bytes
Secret-capable runtime data
absolute local paths/usernames/home directories
tokens/credentials/private URLs
memory addresses/internal handles
```

Detailed private diagnostics can be opt-in/local and remain bounded/redacted.

## 16. Prompt/source instruction isolation

SQL comments, schema names, source docs, object text, and external manifest prose are data. They cannot change storage policy, root, SQL catalog, validation, retention, or publication behavior.

## 17. Security fixtures

Include:

```text
path traversal/absolute/device/UNC/reserved components
symlink/reparse/hardlink preplacement
malformed digest/object path
malicious SQLite schema/trigger/view/virtual table
extension-loading/attach attempt
oversized SQL/schema/row/blob/object/bomb
corrupt b-tree/file/object/payload/manifest/pointer
foreign-key/constraint violation
schema/ledger tampering
same-ID mismatch/collision
TOCTOU replacement between validation/publication/delete
raw SQL/source instruction injection
private-path/token payload leak
cancel/budget at every mutation/publication boundary
```

Tests do not execute external/source code.

## 18. Required operations

```text
validate_store_root_and_path_policy
resolve_confined_store_path
validate_sqlite_execution_policy
validate_store_schema_integrity
validate_store_relational_integrity
run_store_database_integrity_policy
validate_store_file_and_manifest
validate_store_object_integrity
validate_store_reference_closure
classify_store_corruption_or_quarantine
validate_store_resource_budgets
validate_store_public_error_redaction
```

## 19. Required tests

- all security fixtures above;
- no extension/attach/raw SQL surface;
- unsafe links/path race rejected;
- mandatory integrity skipped -> activation fails;
- corrupt DB/object/pointer/manifest rejects and old active remains;
- huge/malformed inputs bounded;
- GC revalidation prevents race deletion;
- no corruption auto-repair/overwrite;
- errors/manifests contain no absolute private path/token/raw value;
- source comments cannot alter policy;
- security result deterministic under temp path/message/order variations.

## 20. Hard stops

- no untrusted writable SQLite activation;
- no extension loading/attach/dynamic SQL;
- no path/symlink escape;
- no mandatory integrity skip-as-pass;
- no auto-repair into trusted truth;
- no overwrite of same-ID mismatch;
- no unbounded decode/query/file;
- no referenced/leased/uncertain delete;
- no private/source/Secret payload leak;
- no source instruction as policy.
