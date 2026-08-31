# Reference Pack validation operation

**Status:** normative read-only nonrepairing validation contract.

## Operation

```text
reference_pack_validate(request) -> ReferencePackValidationReport
```

## Principles

- Validation never repairs, rewrites, downloads, regenerates, or upgrades the candidate.
- Every check identifies exact member, profile, generation, and component inputs.
- An unavailable mandatory check is `NotEvaluated` and blocks the corresponding eligibility.
- One successful subsystem never overrides a failure in another.
- Validation can report multiple independent blockers within budgets.

## Validation sequence

```text
request/root policy
-> root and path inventory
-> pack manifest syntax/schema/identity
-> member-set closure
-> checksums and lengths
-> component manifest/profile/generation closure
-> ReferenceStore open/integrity/read-only checks
-> ReferenceView exact golden checks
-> annotation file/syntax/manifest checks
-> source-map and final-byte span checks
-> projection coverage/loss closure
-> parity and consumer result identity/gates
-> license/provenance/redistribution closure
-> pack capability and eligibility recomputation
-> canonical validation report
```

## Root and path checks

- explicit configured root only;
- no traversal, absolute member paths, device/reserved paths, NUL, unsafe symlink/reparse escape, case collision, duplicate normalized path, or undeclared file;
- member count, size, depth, and total budgets;
- no unexpected executables, scripts, archives, or source trees;
- no files outside root touched.

## Manifest checks

- supported version and exact pack ID derivation;
- one exact profile/reference generation;
- component IDs match member manifests;
- noncyclic identity fields;
- ordered member list deterministic;
- no self-referential checksum;
- exact eligibility/deferred capability declarations;
- no volatile host, time, or temporary-path data in canonical identity.

## Member checks

For every declared member:

- exists exactly once;
- correct kind, path, encoding, and mode policy;
- expected length and SHA-256, object, or store identity;
- profile/generation applicability;
- provenance and license references;
- required/optional policy;
- member-specific schema and integrity checks.

Extra members are rejected or classified only by an explicit extension policy; E1-D defaults to rejection.

## Reference checks

- open ReferenceStore read-only using the exact store profile;
- validate SQLite/store manifest, integrity, schema, and operation-catalog identities;
- validate ReferenceData manifest and raw, normalized, correction, conflict, and coverage closure;
- execute frozen exact ReferenceView positive, negative-authority, partial, conflict, raw, and restriction queries;
- reject cross-profile/generation or mutable store state;
- do not require physical SQLite byte equality unless the profile promises it.

## Annotation checks

- all rendered files match file-manifest bytes, digests, and path rules;
- generated syntax and inert shape validated;
- semantic, file, source-map, loss, parity, and consumer manifests close;
- every material fragment maps to exact reference or derivation evidence;
- every unsupported, approximated, sanitized, or partial input has a status/loss record;
- mandatory sidecar semantics are consumed by declared consumers;
- no editor/config mutation or diagnostic suppression in probe reports;
- no runtime or full source-body package content.

## License and provenance checks

- every source, component, and member has provenance;
- redistribution class is known for embedded bytes;
- required notices are included exactly;
- no private repository URL or local path is exposed;
- provider identity does not replace source-content identity;
- unknown mandatory license state blocks `validated-local`.

## Eligibility recomputation

The validator derives eligibility from gate records. It never trusts the manifest string alone.

`validated-local` requires:

- all mandatory structural, integrity, checksum, schema, and security gates pass;
- exact profile/generation/component closure;
- no unresolved mandatory ReferenceData conflict, coverage, or correction blocker;
- no unreviewed blocking annotation loss, parity, or consumer blocker;
- all declared mandatory checks evaluated;
- deterministic rebuild gate present and passing when the validation profile requires it.

## Validation output

Report exact records for:

```text
Passed
Failed
NotEvaluated
NotApplicable
```

Human text is secondary. Default errors contain no private root or raw source payload.

## Mutation corpus

- missing, extra, renamed, or case-colliding member;
- altered bytes, size, or checksum;
- manifest points to another profile, generation, or component;
- mutable, corrupt, or wrong-schema store;
- ReferenceView golden mismatch;
- annotation syntax, source-map span, or loss-manifest mutation;
- parity/probe report from another source or consumer revision;
- hidden partial/reference conflict under Complete projection;
- missing notice or forbidden redistribution;
- unsafe symlink/path/private metadata;
- validator attempts repair.
