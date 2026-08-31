# Reference Pack build operation

**Status:** normative E1-D orchestration state machine.

## Operation

```text
reference_pack_build(request) -> ReferencePackBuildOutcome
```

## State machine

```text
Requested
-> RequestValidated
-> ComponentsValidated
-> SourceSnapshotPreflighted
-> StagingSessionPrepared
-> ReferenceDataBuildInvoked
-> ReferenceDataValidated
-> AnnotationBuildInvoked
-> AnnotationValidated
-> PackAssemblyPlanned
-> MembersMaterialized
-> CandidateManifestFinalized
-> CandidateValidated
-> DeterminismGateEvaluated
-> Completed
```

Terminal states:

```text
Blocked
Failed
Cancelled
Quarantined
```

No stage is skipped. A stage may be `NotApplicable` only through the exact pack profile.

## Preflight

Validate before expensive work:

- source snapshot root/manifest/digests/path/license policy;
- exact profile, Interface, build, revision, content digest;
- all component contract/implementation/schema/profile identities;
- compatibility reports and prerequisite freeze state;
- pack layout and eligibility profiles;
- budgets/cancellation;
- output/staging root policy and destination nonoverlap;
- no floating source/oracle/consumer/current token;
- no missing mandatory member family.

## ReferenceData stage

Invoke `wow-reference` with its exact E1 request and `wow-store` profile. Preserve the full outcome and require:

- immutable ReferenceStore publication result;
- read-only reopen and ReferenceView validation;
- exact ReferenceGeneration/Profile closure;
- raw/normalized/correction/conflict/coverage manifests;
- declared ReferenceData eligibility;
- store logical integrity and security reports.

E1-D does not inspect SQL or reconstruct ReferenceData rows.

## Annotation stage

Invoke `wow-annotations` using the exact validated ReferenceView. Require:

- semantic model and rendered file manifests;
- exact profile/reference generation closure;
- source map and projection coverage/loss reports;
- Ketho semantic parity report under pinned baseline;
- EmmyLua/LuaLS consumer profile/probe results under declared artifact profile;
- annotation artifact eligibility;
- deterministic canonical files/sidecars.

E1-D does not render or patch annotations.

## Assembly planning

Build a complete member set from component artifacts and pack profile. Validate:

- every required member present exactly once;
- no undeclared extra member;
- exact path/kind/content identity;
- no profile/generation mismatch;
- no path collision/traversal/device/reserved name;
- license/provenance and redistribution policy for every member;
- byte/object/store identity available before materialization;
- noncyclic manifest/pack identity construction.

## Materialization

The application executes the typed plan in a unique staging root:

1. create fixed directories;
2. write/copy/link only declared members using safe primitives;
3. verify each written member bytes/size/digest;
4. fsync/durability according to profile where supported;
5. return a typed materialization report;
6. never run files or external commands.

Service validates the report. It does not accept “success” without member-by-member evidence.

## Manifest finalization

After materialization closure, finalize:

- checksum manifest;
- provenance/license manifests;
- pack capability/gate summary;
- pack manifest;
- build report;
- candidate identity.

Write/finalize order is profile-defined to avoid circular IDs. A temporary internal assembly descriptor may exist, but only the final manifest is public.

## Candidate validation

Invoke the independent validation operation against staged bytes. Build cannot self-certify by reusing only its in-memory assumptions.

## Destination finalization

For `validated-local` target only:

- candidate validation passes all mandatory gates;
- destination policy permits creation/replacement;
- prior destination remains untouched until atomic finalization;
- final rename/swap is attempted through application adapter;
- resulting destination is reopened and manifest/checksums revalidated;
- no global release channel or active production pointer is changed.

## Partial and blocked builds

A candidate may be retained/quarantined for review when policy allows. It must carry:

- exact incomplete gates;
- no validated-local claim;
- no final destination activation;
- no missing loss/conflict information;
- explicit cleanup retention policy.

## Cancellation

Cancellation checkpoints exist before/after every component and materialization stage. On cancel:

- no new final destination;
- no complete/validated manifest claim;
- no background continuation;
- component immutable generations already published retain their identities but are not relabeled as a pack;
- staging is cleaned/quarantined according to policy;
- prior destination remains unchanged.

## Required tests

- complete fixture/candidate/validated-local paths;
- every state transition and illegal skip;
- component mismatch/failure/partial/loss/parity/probe blocker;
- materialization member failure and post-write digest mismatch;
- cancellation at every boundary;
- prior destination preservation;
- manifest cycle and extra/missing member rejection;
- no publication/network/source execution/editor mutation.
