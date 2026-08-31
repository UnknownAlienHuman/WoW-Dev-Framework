# Platform source corpus, roles, and package model

**Status:** normative E3-B inventory and package contract.

## Corpus scope

One corpus generation contains one exact product/flavor/build-oriented materialized snapshot under one profile. The initial mirror root is `Interface/`, with code packages under `Interface/AddOns/` and provider inventory/version files retained separately.

A corpus is not a user addon project and is not Reference Pack authority. It is an auxiliary source universe with its own project/graph/store identities.

## Corpus roots

```text
PlatformSourceRoot
    root ID
    snapshot/corpus generation
    normalized semantic root path
    allowed entry types/roles
    package discovery profile
    source-detail/license/security policy
    complete/partial/conflict state
```

No root is inferred from arbitrary checkout layout.

## Source-role classification order

1. exact profile-excluded metadata/automation roots;
2. exact generated API documentation package/file rules;
3. exact API documentation framework package rules;
4. actual selected TOC/XML/Lua membership and file kind;
5. provider inventory hints as corroboration only;
6. bounded unknown/unsupported classification.

Path names may select a documented provider role only when the profile owns that mapping. They cannot trigger universal addon architecture recognizers.

## Generated documentation segregation

```text
Interface/AddOns/Blizzard_APIDocumentation/**
    api_documentation_framework

Interface/AddOns/Blizzard_APIDocumentationGenerated/**
    generated_api_documentation
```

E3-B inventories and source-maps these entries but excludes them from the default implementation analyzer/recognizer workspace. `wow-reference` consumes them through its own evaluator.

## Package discovery and TOCs

A package candidate contains exact root, TOC candidates, provider-list observations, file/content manifest, role/flavor/build evidence, conflicts, and coverage. Directory presence alone is insufficient.

For each selected package:

- enumerate actual TOCs from the closed snapshot;
- parse via E2-C bounded rules;
- classify flavor/variant/interface/build compatibility;
- select exactly one variant or explicit none/conflict;
- never merge variants;
- preserve raw/unknown directives and source order;
- reconcile provider list observations separately.

## Package roles

```text
core_bootstrap
shared_framework
feature_package
load_on_demand_feature
generated_documentation
provider_tool_or_test
unknown
```

They describe packaging/load structure, not runtime execution or user-addon module/service semantics.

## Source membership

```text
selected direct TOC entry
selected XML include/external script
XML inline virtual Lua unit
present but unreachable
present outside selected package closure
provider inventory only
excluded/unsupported
```

Present is not loaded; statically reachable is not runtime executed.

A physical source entry may have multiple package/load membership assertions. Do not duplicate source identity or choose a first owner.

## Global namespace boundary

Platform packages may share globals, mixins, templates, registries, events, callbacks, and libraries. Package boundaries are deployment/load structure, not Lua namespace isolation. Package-local views cannot claim global absence without relevant corpus coverage.

## Unknown/unsupported entries

Retain exact entry/content/source handle, observed type/role evidence, unsupported reason, affected capabilities, source-detail availability, and coverage/loss. Never drop unfamiliar files or files missing from provider lists.

## Corpus manifest

```text
PlatformSourceCorpusManifest
    snapshot/profile/build observations
    selected roots/packages/TOCs
    source roles and inclusion states
    load/analyzer/recognizer/graph manifests
    unsupported/excluded/conflict/license/security records
    counts/digests by role/package/status
    canonical digest
```

## Required tests

Multiple variants, directories without TOCs, list/manifest disagreement, duplicate/case paths, generated-doc exclusion, multi-package file references, presence/reachability distinctions, namespace mutation, unknown files, repository/package/path rename invariance, and shuffled provider order.
