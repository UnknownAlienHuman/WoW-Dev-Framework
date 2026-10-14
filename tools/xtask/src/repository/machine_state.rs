//! Bounded validation for reviewed machine-state contract owners.
use crate::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// Declared anchors only. This owner validates explicit contract records; it never
// greps Rust source for symbol spellings and never infers a state from a file list.
const GRAPH_CONTRACT: &str = "crates/wow-graph/e2/CONTRACT.json";
const GRAPH_MANIFEST: &str = "crates/wow-graph/e2/examples/CHECKSUMS.json";
const SERVICE_CONTRACT: &str = "crates/wow-service/e1/CONTRACT.json";
const SERVICE_MANIFEST: &str = "crates/wow-service/e1/examples/CHECKSUMS.json";
const STORE_E2_CONTRACT: &str = "crates/wow-store/e2/CONTRACT.json";
const STORE_E2_MANIFEST: &str = "crates/wow-store/e2/examples/CHECKSUMS.json";

const ANNOTATION_CONTRACT: &str = "crates/wow-annotations/e1/CONTRACT.json";
const ANNOTATION_MANIFEST: &str = "crates/wow-annotations/e1/examples/CHECKSUMS.json";
const PROJECT_CONTRACT: &str = "crates/wow-project/CONTRACT.json";
const PROJECT_MANIFEST: &str = "crates/wow-project/examples/CHECKSUMS.json";
const RECOGNIZER_CONTRACT: &str = "crates/wow-recognizers/e2/CONTRACT.json";
const RECOGNIZER_MANIFEST: &str = "crates/wow-recognizers/e2/examples/CHECKSUMS.json";

struct CapabilitySpec {
    name: &'static str,
    owners: &'static [(&'static str, &'static str)],
    application_routes: &'static [(&'static str, &'static str)],
    service_routes: Option<&'static [(&'static str, &'static str)]>,
    application_selection: Option<&'static str>,
}

struct OwnerSpec {
    id: &'static str,
    contract_path: &'static str,
    manifest_path: &'static str,
    manifest_id: &'static str,
    capabilities: &'static [CapabilitySpec],
    manifest_history_is_object: bool,
    application_selection: Option<&'static str>,
    operation_matrix_matches_capabilities: bool,
    capability_matrix_field: Option<&'static str>,
    canonical_issue: Option<u64>,
    required_blocking_issues: &'static [u64],
}

const PROJECT_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "native-project-publication",
        owners: &[
            (
                "crates/wow-project/src/publication.rs",
                "publish_initial_cancellable",
            ),
            (
                "crates/wow-project/src/publication.rs",
                "apply_update_cancellable",
            ),
            ("crates/wow-project/src/snapshot.rs", "open_view"),
        ],
        application_routes: &[
            ("crates/wow-service/src/local/backend.rs", "acquire_project"),
            ("apps/wow/src/project.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "native-library-byte-view",
        owners: &[
            ("crates/wow-project/src/library_view.rs", "qualify"),
            ("crates/wow-project/src/library_view.rs", "file_view"),
            ("crates/wow-project/src/library_view.rs", "slice"),
        ],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/native_api.rs",
                "reference_origins_prepared",
            ),
            ("apps/wow/src/context.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
];

const ANNOTATION_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "native-library-projection",
        owners: &[(
            "crates/wow-annotations/src/native.rs",
            "project_with_alias_catalogs",
        )],
        application_routes: &[
            (
                "crates/wow-service/src/local/native_library_input.rs",
                "rebuild_for_project",
            ),
            ("apps/wow/src/native.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "source-bound-callable-lowering",
        owners: &[(
            "crates/wow-annotations/src/reference_links.rs",
            "callable_lowering",
        )],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/native_api.rs",
                "reference_origins_prepared",
            ),
            ("apps/wow/src/context.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "native-artifact-and-projection-sidecars",
        owners: &[
            (
                "crates/wow-annotations/src/artifact.rs",
                "from_native_library",
            ),
            (
                "crates/wow-annotations/src/sidecars.rs",
                "from_library_cancellable",
            ),
        ],
        application_routes: &[
            (
                "crates/wow-service/src/local/native_input.rs",
                "from_native_manifest",
            ),
            ("apps/wow/src/native.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
];

const SERVICE_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "reference-pack-build",
        owners: &[(
            "crates/wow-service/src/reference_pack.rs",
            "reference_pack_build",
        )],
        application_routes: &[("apps/wow-reference-builder/src/lib.rs", "build")],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "reference-pack-validation",
        owners: &[(
            "crates/wow-service/src/reference_pack.rs",
            "reference_pack_validate",
        )],
        application_routes: &[("apps/wow-reference-builder/src/lib.rs", "validate")],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "reference-pack-rebuild-comparison",
        owners: &[(
            "crates/wow-service/src/reference_pack.rs",
            "reference_pack_rebuild_compare",
        )],
        application_routes: &[("apps/wow-reference-builder/src/lib.rs", "rebuild_compare")],
        service_routes: None,
        application_selection: None,
    },
];

const STORE_PUBLISH_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/persistence.rs",
        "publish_graph_bundle",
    ),
    (
        "crates/wow-service/src/live_project/operations.rs",
        "publish_local_project",
    ),
];

const STORE_PUBLISH_APPLICATION_ROUTES: &[(&str, &str)] = &[
    ("apps/wow/src/graph_store.rs", "run"),
    ("apps/wow/src/project.rs", "run"),
];

const STORE_READ_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/persistence.rs",
        "execute_graph_store_read",
    ),
    (
        "crates/wow-service/src/live_project/operations.rs",
        "read_live_project",
    ),
];

const STORE_READ_APPLICATION_ROUTES: &[(&str, &str)] = &[
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/project.rs", "run"),
];

const STORE_RECOVERY_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/persistence.rs",
        "reconcile_graph_publication",
    ),
    (
        "crates/wow-service/src/live_project/operations.rs",
        "reconcile_live_project",
    ),
    (
        "crates/wow-service/src/live_project/recovery.rs",
        "recover_live_project",
    ),
];

const STORE_RECOVERY_APPLICATION_ROUTES: &[(&str, &str)] = &[
    ("apps/wow/src/graph_store.rs", "run"),
    ("apps/wow/src/project.rs", "run"),
];

const STORE_BACKUP_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/live_project/recovery.rs",
        "backup_to_new",
    ),
    (
        "crates/wow-service/src/live_project/recovery.rs",
        "restore_live_project_to_new",
    ),
];

const STORE_RETENTION_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/live_project/retention.rs",
        "retention_roots",
    ),
    (
        "crates/wow-service/src/live_project/retention.rs",
        "put_retention_root",
    ),
    (
        "crates/wow-service/src/live_project/retention.rs",
        "remove_retention_root",
    ),
];

const STORE_GC_SERVICE_ROUTES: &[(&str, &str)] = &[
    ("crates/wow-service/src/live_project/gc.rs", "plan_gc"),
    ("crates/wow-service/src/live_project/gc.rs", "execute_gc"),
    ("crates/wow-service/src/live_project/gc.rs", "reconcile_gc"),
];

const STORE_MIGRATION_SERVICE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/live_project/migration.rs",
        "prepare_live_project_migration",
    ),
    (
        "crates/wow-service/src/live_project/migration.rs",
        "migrate_live_project_to_new",
    ),
    (
        "crates/wow-service/src/live_project/migration.rs",
        "activate_ready_selection",
    ),
];

const STORE_E2_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "validate_project_store_runtime_profile",
        owners: &[("crates/wow-store/src/project/mod.rs", "ProjectStore::open")],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "validate_project_store_schema_set",
        owners: &[
            (
                "crates/wow-store/src/project/model.rs",
                "RecordCatalog::new",
            ),
            ("crates/wow-store/src/project/mod.rs", "ProjectStore::open"),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "create_project_store_epoch",
        owners: &[
            (
                "crates/wow-store/src/project/mod.rs",
                "ProjectStore::create",
            ),
            (
                "crates/wow-store/src/project/mod.rs",
                "ProjectStore::open_or_create",
            ),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "open_project_store_epoch",
        owners: &[("crates/wow-store/src/project/mod.rs", "ProjectStore::open")],
        application_routes: STORE_READ_APPLICATION_ROUTES,
        service_routes: Some(STORE_READ_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "acquire_project_store_writer",
        owners: &[
            ("crates/wow-store/src/project/mod.rs", "ProjectStore::open"),
            (
                "crates/wow-store/src/project/publication.rs",
                "ProjectStore::prepare",
            ),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "plan_partition_version_materialization",
        owners: &[
            (
                "crates/wow-store/src/project/model.rs",
                "PartitionRecord::new",
            ),
            (
                "crates/wow-store/src/project/model.rs",
                "PublicationRequest::new",
            ),
            (
                "crates/wow-store/src/project/publication.rs",
                "ProjectStore::prepare",
            ),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "build_inactive_project_store_generation",
        owners: &[(
            "crates/wow-store/src/project/publication.rs",
            "ProjectStore::prepare",
        )],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "open_project_store_generation_snapshot",
        owners: &[
            ("crates/wow-store/src/project/mod.rs", "ProjectStore::read"),
            (
                "crates/wow-store/src/project/read.rs",
                "ReadSnapshot::acquire",
            ),
        ],
        application_routes: STORE_READ_APPLICATION_ROUTES,
        service_routes: Some(STORE_READ_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "validate_inactive_project_store_generation",
        owners: &[
            (
                "crates/wow-store/src/project/publication.rs",
                "ProjectStore::validate_inactive",
            ),
            (
                "crates/wow-store/src/project/read.rs",
                "ReadSnapshot::owner_validation",
            ),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "activate_project_store_generation",
        owners: &[
            (
                "crates/wow-store/src/project/publication.rs",
                "ProjectStore::activate",
            ),
            (
                "crates/wow-store/src/project/mod.rs",
                "ProjectStore::current",
            ),
        ],
        application_routes: STORE_PUBLISH_APPLICATION_ROUTES,
        service_routes: Some(STORE_PUBLISH_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "open_current_project_store_snapshot",
        owners: &[
            ("crates/wow-store/src/project/mod.rs", "ProjectStore::read"),
            (
                "crates/wow-store/src/project/read.rs",
                "ReadSnapshot::acquire",
            ),
        ],
        application_routes: STORE_READ_APPLICATION_ROUTES,
        service_routes: Some(STORE_READ_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "checkpoint_project_store",
        owners: &[(
            "crates/wow-store/src/project/mod.rs",
            "ProjectStore::checkpoint",
        )],
        application_routes: &[],
        service_routes: Some(&[]),
        application_selection: Some("library-only"),
    },
    CapabilitySpec {
        name: "backup_project_store",
        owners: &[
            (
                "crates/wow-store/src/project/backup.rs",
                "ProjectStore::backup_to_new",
            ),
            (
                "crates/wow-store/src/project/backup.rs",
                "VerifiedBackup::open",
            ),
            (
                "crates/wow-store/src/project/backup.rs",
                "VerifiedBackup::verify",
            ),
            (
                "crates/wow-store/src/project/backup.rs",
                "VerifiedBackup::restore_to_new",
            ),
            (
                "crates/wow-store/src/project/backup.rs",
                "VerifiedBackup::finish_restore",
            ),
        ],
        application_routes: &[],
        service_routes: Some(STORE_BACKUP_SERVICE_ROUTES),
        application_selection: Some("library-only"),
    },
    CapabilitySpec {
        name: "recover_project_store",
        owners: &[
            (
                "crates/wow-store/src/project/recovery.rs",
                "ProjectStore::recovery_report",
            ),
            (
                "crates/wow-store/src/project/publication.rs",
                "ProjectStore::reconcile",
            ),
        ],
        application_routes: STORE_RECOVERY_APPLICATION_ROUTES,
        service_routes: Some(STORE_RECOVERY_SERVICE_ROUTES),
        application_selection: Some("application-selected"),
    },
    CapabilitySpec {
        name: "retain_project_store_generations",
        owners: &[
            (
                "crates/wow-store/src/project/retention.rs",
                "ProjectStore::retention_roots",
            ),
            (
                "crates/wow-store/src/project/retention.rs",
                "ProjectStore::put_retention_root",
            ),
            (
                "crates/wow-store/src/project/retention.rs",
                "ProjectStore::remove_retention_root",
            ),
        ],
        application_routes: &[],
        service_routes: Some(STORE_RETENTION_SERVICE_ROUTES),
        application_selection: Some("library-only"),
    },
    CapabilitySpec {
        name: "garbage_collect_project_store",
        owners: &[
            (
                "crates/wow-store/src/project/gc.rs",
                "ProjectStore::plan_gc",
            ),
            (
                "crates/wow-store/src/project/gc.rs",
                "ProjectStore::execute_gc",
            ),
            (
                "crates/wow-store/src/project/gc.rs",
                "ProjectStore::reconcile_gc",
            ),
        ],
        application_routes: &[],
        service_routes: Some(STORE_GC_SERVICE_ROUTES),
        application_selection: Some("library-only"),
    },
    CapabilitySpec {
        name: "build_and_activate_project_store_epoch",
        owners: &[
            (
                "crates/wow-store/src/project/migration.rs",
                "MigrationCandidate::stage",
            ),
            (
                "crates/wow-store/src/project/migration.rs",
                "ValidatedMigration::finish",
            ),
            (
                "crates/wow-store/src/project/migration/selection.rs",
                "ProjectStore::activate_ready_selection",
            ),
        ],
        application_routes: &[],
        service_routes: Some(STORE_MIGRATION_SERVICE_ROUTES),
        application_selection: Some("library-only"),
    },
];

const RECOGNIZER_TOC_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    ("crates/wow-service/src/graph/build/toc.rs", "publish_bound"),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_STATE_ROOT_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    (
        "crates/wow-service/src/graph/build/toc.rs",
        "publish_state_root_bound",
    ),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_XML_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    ("crates/wow-service/src/graph/build/xml.rs", "publish_bound"),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_CONSTRUCTION_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    (
        "crates/wow-service/src/graph/build/construction.rs",
        "publish",
    ),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_MIXIN_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    (
        "crates/wow-service/src/graph/build/lua_mixins.rs",
        "publish",
    ),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_SIGNAL_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    (
        "crates/wow-service/src/graph/build/signals.rs",
        "publish_signals",
    ),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];
const RECOGNIZER_STATE_CORE_ROUTES: &[(&str, &str)] = &[
    (
        "crates/wow-service/src/graph/build.rs",
        "execute_graph_build_diagnosed",
    ),
    (
        "crates/wow-service/src/graph/build/state_core.rs",
        "publish_bound",
    ),
    ("apps/wow/src/graph.rs", "run"),
    ("apps/wow/src/graph_build.rs", "run"),
];

const RECOGNIZER_TOC_OWNERS: &[(&str, &str)] = &[
    (
        "crates/wow-recognizers/src/source_toc.rs",
        "recognize_source_toc",
    ),
    (
        "crates/wow-recognizers/src/source_toc.rs",
        "recognize_source_toc_assertions",
    ),
];
const RECOGNIZER_XML_OWNERS: &[(&str, &str)] = &[
    (
        "crates/wow-recognizers/src/source_xml.rs",
        "recognize_source_xml",
    ),
    (
        "crates/wow-recognizers/src/source_xml.rs",
        "recognize_source_xml_assertions",
    ),
];
const RECOGNIZER_CONSTRUCTION_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_construction.rs",
    "recognize_source_construction",
)];
const RECOGNIZER_MIXIN_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_mixins.rs",
    "recognize_source_mixins",
)];
const RECOGNIZER_MIXIN_ASSIGNMENT_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_mixins.rs",
    "recognize_source_mixin_assignments",
)];
const RECOGNIZER_FRAME_EVENT_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_signals.rs",
    "w1_recognize_native_frame_events",
)];
const RECOGNIZER_EVENT_BRIDGE_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_bridge.rs",
    "w2_recognize_native_event_bridges",
)];
const RECOGNIZER_CUSTOM_SIGNAL_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_signals.rs",
    "w3_recognize_signals",
)];
const RECOGNIZER_CVAR_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_signals.rs",
    "recognize_source_cvar_callbacks",
)];
const RECOGNIZER_HOOK_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_scripts.rs",
    "recognize_source_hooks",
)];
const RECOGNIZER_LIBRARY_OWNERS: &[(&str, &str)] = &[(
    "crates/wow-recognizers/src/source_state.rs",
    "recognize_source_library",
)];
const RECOGNIZER_STATE_CORE_OWNERS: &[(&str, &str)] = &[
    (
        "crates/wow-recognizers/src/source_state_core.rs",
        "recognize_source_state_core",
    ),
    (
        "crates/wow-recognizers/src/source_state_core.rs",
        "recognize_source_state_core_assertions",
    ),
];

const RECOGNIZER_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "core.toc.package@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_TOC_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.toc.file_order@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_TOC_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.toc.dependencies@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_TOC_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.toc.load_on_demand@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_TOC_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.toc.saved_variables@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_TOC_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.xml.template@1",
        owners: RECOGNIZER_XML_OWNERS,
        application_routes: RECOGNIZER_XML_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.xml.object@1",
        owners: RECOGNIZER_XML_OWNERS,
        application_routes: RECOGNIZER_XML_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.xml.inherits@1",
        owners: RECOGNIZER_XML_OWNERS,
        application_routes: RECOGNIZER_XML_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.xml.script@1",
        owners: RECOGNIZER_XML_OWNERS,
        application_routes: RECOGNIZER_XML_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.lua.create_frame@1",
        owners: RECOGNIZER_CONSTRUCTION_OWNERS,
        application_routes: RECOGNIZER_CONSTRUCTION_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.lua.create_from_mixins@1",
        owners: RECOGNIZER_MIXIN_OWNERS,
        application_routes: RECOGNIZER_MIXIN_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.lua.mixin_assignment@1",
        owners: RECOGNIZER_MIXIN_ASSIGNMENT_OWNERS,
        application_routes: RECOGNIZER_MIXIN_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.signal.native_frame_event@1",
        owners: RECOGNIZER_FRAME_EVENT_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.signal.native_event_registry_bridge@1",
        owners: RECOGNIZER_EVENT_BRIDGE_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.signal.custom_registry_producer@1",
        owners: RECOGNIZER_CUSTOM_SIGNAL_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.signal.custom_registry_subscription@1",
        owners: RECOGNIZER_CUSTOM_SIGNAL_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.signal.cvar_callback@1",
        owners: RECOGNIZER_CVAR_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.hook.set_script@1",
        owners: RECOGNIZER_HOOK_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.hook.hook_script@1",
        owners: RECOGNIZER_HOOK_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.hook.secure_posthook@1",
        owners: RECOGNIZER_HOOK_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.library.libstub_require@1",
        owners: RECOGNIZER_LIBRARY_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.library.libstub_new@1",
        owners: RECOGNIZER_LIBRARY_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.library.embed@1",
        owners: RECOGNIZER_LIBRARY_OWNERS,
        application_routes: RECOGNIZER_SIGNAL_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.state.saved_variable_root@1",
        owners: RECOGNIZER_TOC_OWNERS,
        application_routes: RECOGNIZER_STATE_ROOT_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.state.literal_path_read@1",
        owners: RECOGNIZER_STATE_CORE_OWNERS,
        application_routes: RECOGNIZER_STATE_CORE_ROUTES,
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "core.state.literal_path_write@1",
        owners: RECOGNIZER_STATE_CORE_OWNERS,
        application_routes: RECOGNIZER_STATE_CORE_ROUTES,
        service_routes: None,
        application_selection: None,
    },
];

const OWNERS: &[OwnerSpec] = &[
    OwnerSpec {
        id: "wow-annotations/e1-projection-and-parity",
        contract_path: ANNOTATION_CONTRACT,
        manifest_path: ANNOTATION_MANIFEST,
        manifest_id: "wow-annotations-e1-pending-acceptance-v2",
        capabilities: ANNOTATION_CAPABILITIES,
        manifest_history_is_object: false,
        application_selection: None,
        operation_matrix_matches_capabilities: false,
        capability_matrix_field: None,
        canonical_issue: None,
        required_blocking_issues: &[],
    },
    OwnerSpec {
        id: "wow-project/e0-d/1",
        contract_path: PROJECT_CONTRACT,
        manifest_path: PROJECT_MANIFEST,
        manifest_id: "wow-project-e0-d-pending-acceptance-v2",
        capabilities: PROJECT_CAPABILITIES,
        manifest_history_is_object: true,
        application_selection: None,
        operation_matrix_matches_capabilities: false,
        capability_matrix_field: None,
        canonical_issue: None,
        required_blocking_issues: &[],
    },
    OwnerSpec {
        id: "wow-graph/e2-a/typed-assertion-partition-query-core",
        contract_path: GRAPH_CONTRACT,
        manifest_path: GRAPH_MANIFEST,
        manifest_id: "wow-graph-e2-a-pending-acceptance-v2",
        capabilities: GRAPH_CAPABILITIES,
        manifest_history_is_object: false,
        application_selection: None,
        operation_matrix_matches_capabilities: false,
        capability_matrix_field: None,
        canonical_issue: None,
        required_blocking_issues: &[],
    },
    OwnerSpec {
        id: "wow-service/e1-d-reference-pack-build-validation",
        contract_path: SERVICE_CONTRACT,
        manifest_path: SERVICE_MANIFEST,
        manifest_id: "wow-service-e1-d-pending-acceptance-v2",
        capabilities: SERVICE_CAPABILITIES,
        manifest_history_is_object: false,
        application_selection: Some("application-selected"),
        operation_matrix_matches_capabilities: false,
        capability_matrix_field: None,
        canonical_issue: None,
        required_blocking_issues: &[],
    },
    OwnerSpec {
        id: "wow-store/e2-d/projectstore-coherent-publication",
        contract_path: STORE_E2_CONTRACT,
        manifest_path: STORE_E2_MANIFEST,
        manifest_id: "wow-store-e2-d-pending-acceptance-v3",
        capabilities: STORE_E2_CAPABILITIES,
        manifest_history_is_object: false,
        application_selection: None,
        operation_matrix_matches_capabilities: true,
        capability_matrix_field: None,
        canonical_issue: None,
        required_blocking_issues: &[],
    },
    OwnerSpec {
        id: "wow-recognizers/e2-b/core-structural-rules",
        contract_path: RECOGNIZER_CONTRACT,
        manifest_path: RECOGNIZER_MANIFEST,
        manifest_id: "wow-recognizers-e2-b-pending-acceptance-v2",
        capabilities: RECOGNIZER_CAPABILITIES,
        manifest_history_is_object: false,
        application_selection: Some("application-selected"),
        operation_matrix_matches_capabilities: false,
        capability_matrix_field: Some("active_rule_ids"),
        canonical_issue: Some(103),
        required_blocking_issues: &[103, 120],
    },
];

const GRAPH_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "versioned-kind-relation-axis-registry",
        owners: &[("crates/wow-graph/src/registry.rs", "GraphRegistryBundle")],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/state_core.rs",
                "publish_bound",
            ),
            ("apps/wow/src/graph.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "atomic-producer-partition-replacement",
        owners: &[("crates/wow-graph/src/partition.rs", "prepare_replacement")],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/state_core.rs",
                "publish_bound",
            ),
            ("apps/wow/src/graph.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "immutable-graph-snapshot-publication",
        owners: &[
            ("crates/wow-graph/src/persistent.rs", "publish_current"),
            ("crates/wow-graph/src/persistent.rs", "read_current"),
        ],
        application_routes: &[
            (
                "crates/wow-service/src/graph/persistence.rs",
                "publish_graph_bundle",
            ),
            (
                "crates/wow-service/src/graph/persistence.rs",
                "execute_graph_store_read",
            ),
            ("apps/wow/src/graph_store.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
    CapabilitySpec {
        name: "exact-neighbor-axis-path-subgraph-explain-queries",
        owners: &[
            ("crates/wow-graph/src/direct.rs", "GraphEntityQuery"),
            (
                "crates/wow-graph/src/direct/neighbors.rs",
                "GraphNeighborReadQuery",
            ),
            ("crates/wow-graph/src/axes.rs", "GraphAxisQuery"),
            ("crates/wow-graph/src/paths.rs", "GraphPathQuery"),
            ("crates/wow-graph/src/subgraph.rs", "GraphSubgraphQuery"),
            ("crates/wow-graph/src/explain.rs", "GraphExplainQuery"),
        ],
        application_routes: &[
            ("crates/wow-service/src/graph/mod.rs", "execute_graph_read"),
            ("apps/wow/src/graph.rs", "run"),
        ],
        service_routes: None,
        application_selection: None,
    },
];
#[derive(Default)]
pub(super) struct Scope {
    documents: BTreeMap<&'static str, Value>,
}

impl Scope {
    pub(super) fn observe(&mut self, path: &str, value: &Value) -> Result<()> {
        if let Some(owner) = OWNERS
            .iter()
            .find(|owner| path == owner.contract_path || path == owner.manifest_path)
        {
            let key = if path == owner.contract_path {
                owner.contract_path
            } else {
                owner.manifest_path
            };
            if self.documents.insert(key, value.clone()).is_some() {
                return Err("machine-state document appears twice in inventory".into());
            }
        }
        Ok(())
    }

    pub(super) fn finish(self, files: &BTreeSet<&str>) -> Result<()> {
        if !OWNERS.iter().any(|owner| {
            files.contains(owner.contract_path)
                || files.contains(owner.manifest_path)
                || owner.capabilities.iter().any(|capability| {
                    capability
                        .owners
                        .iter()
                        .any(|(path, _)| files.contains(path))
                })
        }) {
            return Ok(());
        }
        for owner in OWNERS {
            let contract = self
                .documents
                .get(owner.contract_path)
                .ok_or("machine-state contract is absent")?;
            let manifest = self
                .documents
                .get(owner.manifest_path)
                .ok_or("machine-state checksum manifest is absent")?;
            validate_contract(owner, contract, files)?;
            validate_manifest(owner, manifest)?;
        }
        Ok(())
    }
}

fn validate_contract(owner: &OwnerSpec, value: &Value, files: &BTreeSet<&str>) -> Result<()> {
    common_state(
        value,
        "partial-functional-freeze-open",
        owner.canonical_issue.unwrap_or(120),
    )?;
    if value["schema_version"].as_u64() != Some(2) || text(value, "contract_id")? != owner.id {
        return Err("machine-state contract identity/schema mismatch".into());
    }
    if owner
        .canonical_issue
        .is_some_and(|issue| value["canonical_issue"].as_u64() != Some(issue))
    {
        return Err("machine-state canonical Issue differs from the explicit owner map".into());
    }
    match owner.application_selection {
        Some(expected) if text(value, "application_selection")? != expected => {
            return Err(
                "machine-state application selection differs from the explicit owner map".into(),
            );
        }
        _ => {}
    }
    if !value
        .get("historical_preimplementation_requirements")
        .is_some_and(Value::is_array)
        || value.get("implementation_gate").is_some()
        || value.get("freeze_gate").is_some()
        || value.get("freeze_rules").is_some()
    {
        return Err(
            "contract must retain historical requirements without an active old gate".into(),
        );
    }
    let checksum = value
        .get("checksum_contract")
        .ok_or("checksum contract is absent")?;
    if text(checksum, "state")? != "pending-acceptance-freeze"
        || text(checksum, "manifest")? != owner.manifest_path
        || text(checksum, "rule")?.is_empty()
    {
        return Err("contract checksum rule must remain structurally pending".into());
    }
    validate_checkpoint(
        value
            .get("functional_checkpoint")
            .ok_or("functional checkpoint is absent")?,
    )?;
    validate_operation_matrix(owner, value)?;
    validate_capabilities(owner, value, files)
}

fn validate_manifest(owner: &OwnerSpec, value: &Value) -> Result<()> {
    common_state(
        value,
        "pending-acceptance-freeze",
        owner.canonical_issue.unwrap_or(120),
    )?;
    if value["schema_version"].as_u64() != Some(2)
        || text(value, "manifest_id")? != owner.manifest_id
        || text(value, "contract_id")? != owner.id
    {
        return Err("machine-state manifest identity/schema mismatch".into());
    }
    if owner
        .canonical_issue
        .is_some_and(|issue| value["canonical_issue"].as_u64() != Some(issue))
    {
        return Err(
            "machine-state manifest canonical Issue differs from the explicit owner map".into(),
        );
    }
    let historical = value
        .get("historical_preimplementation_requirements")
        .ok_or("historical manifest requirements are absent")?;
    if if owner.manifest_history_is_object {
        !historical.is_object()
    } else {
        !historical.is_array()
    } || value.get("freeze_gate").is_some()
        || value.get("freeze_rules").is_some()
    {
        return Err("manifest historical requirements have an invalid shape".into());
    }
    let rules = value
        .get("acceptance_rules")
        .and_then(Value::as_array)
        .ok_or("acceptance_rules must be an array")?;
    if rules.is_empty() || rules.iter().any(|rule| !rule.is_string()) {
        return Err("acceptance_rules must contain strings".into());
    }
    // A pending manifest cannot claim a frozen output. Other pending pins remain data.
    let digest = if owner.id == "wow-annotations/e1-projection-and-parity" {
        value
            .get("bundle")
            .and_then(|bundle| bundle.get("bundle_sha256"))
    } else {
        value.get("bundle_sha256")
    };
    if !digest.is_some_and(Value::is_null) {
        return Err("pending manifest cannot claim a frozen bundle digest".into());
    }
    Ok(())
}

fn common_state(value: &Value, expected_state: &str, canonical_issue: u64) -> Result<()> {
    if text(value, "state")? != expected_state
        || text(value, "implementation_status")? != "partial-functional"
        || text(value, "acceptance_status")? != "pending"
        || value["canonical_issue"].as_u64() != Some(canonical_issue)
        || value.get("implementation_state").is_some()
    {
        return Err(
            "machine state must remain partial-functional and pending on its canonical Issue"
                .into(),
        );
    }
    let freeze = value
        .get("preimplementation_freeze")
        .ok_or("preimplementation freeze record is absent")?;
    exact(freeze, &["status", "evaluation"])?;
    if text(freeze, "status")? != "missed" || text(freeze, "evaluation")? != "NotEvaluated" {
        return Err("preimplementation freeze must remain missed/NotEvaluated".into());
    }
    Ok(())
}

fn validate_checkpoint(value: &Value) -> Result<()> {
    exact(value, &["commit", "quick_run"])?;
    let commit = text(value, "commit")?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value["quick_run"].as_u64().is_none_or(|run| run == 0)
    {
        return Err(
            "functional checkpoint must contain lowercase 40-hex commit and positive run".into(),
        );
    }
    Ok(())
}

fn validate_operation_matrix(owner: &OwnerSpec, contract: &Value) -> Result<()> {
    if !owner.operation_matrix_matches_capabilities {
        return Ok(());
    }
    let operations = contract
        .get("operations")
        .and_then(Value::as_array)
        .ok_or("operations must be an array")?;
    if operations.len() != owner.capabilities.len()
        || operations
            .iter()
            .zip(owner.capabilities)
            .any(|(operation, capability)| operation.as_str() != Some(capability.name))
    {
        return Err("Store E2 operations differ from the explicit owner routing matrix".into());
    }
    Ok(())
}

fn validate_capabilities(
    owner: &OwnerSpec,
    contract: &Value,
    files: &BTreeSet<&str>,
) -> Result<()> {
    if let Some(field) = owner.capability_matrix_field {
        let matrix = contract
            .get(field)
            .and_then(Value::as_array)
            .ok_or("explicit capability identity matrix must be an array")?;
        if matrix.len() != owner.capabilities.len()
            || matrix
                .iter()
                .zip(owner.capabilities)
                .any(|(identity, capability)| identity.as_str() != Some(capability.name))
        {
            return Err(
                "explicit capability identity matrix differs from the owner mapping".into(),
            );
        }
    }
    let rows = contract
        .get("capability_routing")
        .and_then(Value::as_array)
        .ok_or("capability_routing must be an array")?;
    let mut names = BTreeSet::new();
    for row in rows {
        let name = text(row, "capability")?;
        let expected = owner
            .capabilities
            .iter()
            .find(|expected| expected.name == name)
            .ok_or("capability is outside the explicit owner mapping")?;
        let mut fields = vec![
            "capability",
            "status",
            "owners",
            "application_routes",
            "acceptance_status",
            "blocking_issues",
        ];
        if expected.service_routes.is_some() {
            fields.push("service_routes");
        }
        if expected.application_selection.is_some() {
            fields.push("application_selection");
        }
        exact(row, &fields)?;
        if !names.insert(name) {
            return Err("capability routing contains a duplicate".into());
        }
        match expected.application_selection {
            Some(selection) if text(row, "application_selection")? != selection => {
                return Err(
                    "capability application selection differs from the explicit owner map".into(),
                );
            }
            _ => {}
        }
        if let Some(selection) = expected.application_selection {
            let app_routes = row["application_routes"]
                .as_array()
                .ok_or("application_routes must be an array")?;
            match selection {
                "application-selected" if expected.application_routes.is_empty() => {
                    return Err(
                        "application-selected capability requires an explicit app route".into(),
                    );
                }
                "library-only" if !expected.application_routes.is_empty() => {
                    return Err("library-only capability cannot map an app route".into());
                }
                "library-only" if !app_routes.is_empty() => {
                    return Err("library-only capability must have an empty app route set".into());
                }
                "application-selected" | "library-only" => {}
                _ => return Err("capability has an unsupported application selection".into()),
            }
        }
        text(row, "status")?;
        text(row, "acceptance_status")?;
        let blockers = row["blocking_issues"]
            .as_array()
            .ok_or("blocking_issues must be an array")?;
        if owner.required_blocking_issues.iter().any(|required| {
            !blockers
                .iter()
                .any(|issue| issue.as_u64() == Some(*required))
        }) {
            return Err("capability is missing an explicit acceptance blocker".into());
        }
        routes(row.get("owners").ok_or("owners are absent")?, files)?;
        routes(
            row.get("application_routes")
                .ok_or("application_routes are absent")?,
            files,
        )?;
        if let Some(expected_routes) = expected.service_routes {
            let service_routes = row
                .get("service_routes")
                .ok_or("service_routes are absent")?;
            routes(service_routes, files)?;
            require_routes(service_routes, expected_routes)?;
        }
    }
    for expected in owner.capabilities {
        let row = rows
            .iter()
            .find(|row| row["capability"].as_str() == Some(expected.name))
            .ok_or("required selected capability is absent")?;
        if text(row, "status")? != "partial-functional"
            || text(row, "acceptance_status")? != "pending"
        {
            return Err("mapped capability must remain partial-functional/pending".into());
        }
        require_routes(&row["owners"], expected.owners)?;
        require_routes(&row["application_routes"], expected.application_routes)?;
        let blockers = row["blocking_issues"]
            .as_array()
            .ok_or("blocking_issues must be an array")?;
        if !blockers.iter().any(|issue| issue.as_u64() == Some(120)) {
            return Err("selected capability must retain Issue #120".into());
        }
    }
    Ok(())
}

fn routes(value: &Value, files: &BTreeSet<&str>) -> Result<()> {
    let rows = value.as_array().ok_or("routes must be an array")?;
    let mut seen = BTreeSet::new();
    for row in rows {
        exact(row, &["path", "symbol"])?;
        let path = text(row, "path")?;
        let symbol = text(row, "symbol")?;
        crate::manifest::validate_path(path)?;
        if !path.ends_with(".rs")
            || !files.contains(path)
            || symbol.trim().is_empty()
            || symbol.trim() != symbol
            || symbol.chars().any(char::is_control)
            || !seen.insert((path, symbol))
        {
            return Err("route must name a checked Rust path and unique explicit symbol".into());
        }
    }
    Ok(())
}

fn require_routes(value: &Value, expected: &[(&str, &str)]) -> Result<()> {
    let rows = value.as_array().ok_or("routes must be an array")?;
    if rows.len() != expected.len() {
        return Err("route set differs from the explicit owner mapping".into());
    }
    for (path, symbol) in expected {
        if !rows
            .iter()
            .any(|row| row["path"].as_str() == Some(path) && row["symbol"].as_str() == Some(symbol))
        {
            return Err("required explicit owner/application route is missing".into());
        }
    }
    Ok(())
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{field} must be a string").into())
}

fn exact(value: &Value, fields: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or("machine-state record must be an object")?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err("machine-state record has missing or unknown fields".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid(spec: &OwnerSpec) -> (Value, Value, BTreeSet<&'static str>) {
        let mut files = BTreeSet::from([spec.contract_path, spec.manifest_path]);
        let mut routing = Vec::new();
        for capability in spec.capabilities {
            let owners = capability
                .owners
                .iter()
                .map(|(path, symbol)| {
                    files.insert(*path);
                    json!({"path":path,"symbol":symbol})
                })
                .collect::<Vec<_>>();
            let routes = capability
                .application_routes
                .iter()
                .map(|(path, symbol)| {
                    files.insert(*path);
                    json!({"path":path,"symbol":symbol})
                })
                .collect::<Vec<_>>();
            let mut blockers = vec![120, 122];
            for required in spec.required_blocking_issues {
                if !blockers.contains(required) {
                    blockers.push(*required);
                }
            }
            let mut row = json!({"capability":capability.name,"status":"partial-functional",
                "owners":owners,"application_routes":routes,"acceptance_status":"pending","blocking_issues":blockers});
            if let Some(selection) = capability.application_selection {
                row["application_selection"] = json!(selection);
            }
            if let Some(service_routes) = capability.service_routes {
                row["service_routes"] = Value::Array(
                    service_routes
                        .iter()
                        .map(|(path, symbol)| {
                            files.insert(*path);
                            json!({"path":path,"symbol":symbol})
                        })
                        .collect(),
                );
            }
            routing.push(row);
        }
        let operations = spec
            .capabilities
            .iter()
            .map(|capability| capability.name)
            .collect::<Vec<_>>();
        let mut contract = json!({"schema_version":2,"contract_id":spec.id,"state":"partial-functional-freeze-open",
            "implementation_status":"partial-functional","acceptance_status":"pending","canonical_issue":spec.canonical_issue.unwrap_or(120),
            "preimplementation_freeze":{"status":"missed","evaluation":"NotEvaluated"},
            "functional_checkpoint":{"commit":"abcdefabcdefabcdefabcdefabcdefabcdefabcd","quick_run":7},
            "historical_preimplementation_requirements":[],
            "checksum_contract":{"state":"pending-acceptance-freeze","manifest":spec.manifest_path,"rule":"pending"},
            "operations":operations,"capability_routing":routing});
        if let Some(selection) = spec.application_selection {
            contract["application_selection"] = json!(selection);
        }
        if let Some(field) = spec.capability_matrix_field {
            contract[field] = json!(
                spec.capabilities
                    .iter()
                    .map(|capability| capability.name)
                    .collect::<Vec<_>>()
            );
        }
        let history = if spec.manifest_history_is_object {
            json!({})
        } else {
            json!([])
        };
        let mut manifest = json!({"schema_version":2,"manifest_id":spec.manifest_id,"contract_id":spec.id,
            "state":"pending-acceptance-freeze","implementation_status":"partial-functional",
            "acceptance_status":"pending","canonical_issue":spec.canonical_issue.unwrap_or(120),
            "preimplementation_freeze":{"status":"missed","evaluation":"NotEvaluated"},
            "historical_preimplementation_requirements":history,
            "acceptance_rules":["pending"]});
        if spec.id == "wow-annotations/e1-projection-and-parity" {
            manifest["bundle"] = json!({"bundle_sha256":null});
        } else {
            manifest["bundle_sha256"] = Value::Null;
        }
        (contract, manifest, files)
    }

    fn check(
        spec: &OwnerSpec,
        contract: Value,
        manifest: Value,
        files: &BTreeSet<&str>,
    ) -> Result<()> {
        validate_contract(spec, &contract, files)?;
        validate_manifest(spec, &manifest)
    }

    #[test]
    fn both_explicit_owners_accept_partial_pending_state() -> Result<()> {
        for spec in OWNERS {
            let (contract, manifest, files) = valid(spec);
            check(spec, contract, manifest, &files)?;
        }
        Ok(())
    }

    #[test]
    fn representative_stale_and_frozen_mutations_are_rejected() -> Result<()> {
        let spec = &OWNERS[0];
        let (contract, manifest, files) = valid(spec);
        let mut stale = contract.clone();
        stale["state"] = json!("implementation-ready-documentation-no-rust-code");
        assert!(check(spec, stale, manifest.clone(), &files).is_err());
        let mut legacy_state = contract.clone();
        legacy_state["implementation_state"] = json!("not-started");
        assert!(check(spec, legacy_state, manifest.clone(), &files).is_err());
        let mut unknown_capability = contract.clone();
        unknown_capability["capability_routing"][0]["capability"] = json!("future-owner");
        assert!(check(spec, unknown_capability, manifest.clone(), &files).is_err());
        let mut missing_owner = contract.clone();
        missing_owner["capability_routing"][0]["owners"] = json!([]);
        assert!(check(spec, missing_owner, manifest.clone(), &files).is_err());
        let mut missing_route = contract.clone();
        missing_route["capability_routing"][0]["application_routes"] = json!([]);
        assert!(check(spec, missing_route, manifest.clone(), &files).is_err());
        let mut extra_route = contract.clone();
        let mut owner_routes = extra_route["capability_routing"][0]["owners"]
            .as_array()
            .ok_or("fixture owners must be an array")?
            .clone();
        owner_routes
            .push(json!({"path": spec.capabilities[0].owners[0].0, "symbol": "fabricated_symbol"}));
        extra_route["capability_routing"][0]["owners"] = Value::Array(owner_routes);
        assert!(check(spec, extra_route, manifest.clone(), &files).is_err());
        let mut invalid_checkpoint = contract.clone();
        invalid_checkpoint["functional_checkpoint"]["commit"] = json!("UPPERCASE");
        assert!(check(spec, invalid_checkpoint, manifest.clone(), &files).is_err());
        let mut missing_file = files.clone();
        missing_file.remove("crates/wow-annotations/src/native.rs");
        assert!(check(spec, contract.clone(), manifest.clone(), &missing_file).is_err());
        let mut accepted = manifest.clone();
        accepted["acceptance_status"] = json!("accepted");
        assert!(check(spec, contract.clone(), accepted, &files).is_err());
        let mut frozen = manifest;
        frozen["bundle"]["bundle_sha256"] = json!("digest");
        assert!(check(spec, contract, frozen, &files).is_err());
        Ok(())
    }

    #[test]
    fn implemented_owner_without_machine_records_is_rejected() {
        let files = BTreeSet::from(["crates/wow-project/src/publication.rs"]);
        assert!(Scope::default().finish(&files).is_err());
    }

    #[test]
    fn unrelated_future_only_inventory_is_out_of_scope() -> Result<()> {
        Scope::default().finish(&BTreeSet::from(["docs/future-contract.json"]))
    }

    #[test]
    fn graph_e2_stale_state_is_rejected_while_real_routes_are_honored() -> Result<()> {
        let spec = &OWNERS[2];
        let (contract, manifest, files) = valid(spec);
        check(spec, contract.clone(), manifest.clone(), &files)?;
        let mut legacy_state = contract.clone();
        legacy_state["state"] = json!("implementation-ready-documentation-no-rust-code");
        assert!(check(spec, legacy_state, manifest.clone(), &files).is_err());
        let mut legacy_manifest = manifest.clone();
        legacy_manifest["manifest_id"] = json!("wow-graph-e2-a-checksums-v1");
        legacy_manifest["implementation_state"] = json!("not-started");
        assert!(check(spec, contract.clone(), legacy_manifest, &files).is_err());
        let mut fabricated_route = contract.clone();
        fabricated_route["capability_routing"][0]["owners"] = json!([{
            "path": "crates/wow-graph/src/registry.rs",
            "symbol": "fabricated_symbol"
        }]);
        assert!(check(spec, fabricated_route, manifest.clone(), &files).is_err());
        let mut no_route = contract.clone();
        no_route["capability_routing"][0]["application_routes"] = json!([]);
        assert!(check(spec, no_route, manifest.clone(), &files).is_err());
        let mut stale_prerequisite = contract.clone();
        stale_prerequisite["capability_routing"][0]["status"] = json!("accepted");
        assert!(check(spec, stale_prerequisite, manifest.clone(), &files).is_err());
        let mut frozen = manifest.clone();
        frozen["bundle_sha256"] = json!("not-an-acceptance-proof");
        assert!(check(spec, contract, frozen, &files).is_err());
        Ok(())
    }

    fn service_e1_records() -> Result<(&'static OwnerSpec, Value, Value, BTreeSet<&'static str>)> {
        let spec = OWNERS
            .iter()
            .find(|owner| owner.id == "wow-service/e1-d-reference-pack-build-validation")
            .ok_or("Service E1-D owner mapping is absent")?;
        let contract = serde_json::from_str(include_str!(
            "../../../../crates/wow-service/e1/CONTRACT.json"
        ))?;
        let manifest = serde_json::from_str(include_str!(
            "../../../../crates/wow-service/e1/examples/CHECKSUMS.json"
        ))?;
        let mut files = BTreeSet::from([spec.contract_path, spec.manifest_path]);
        for capability in spec.capabilities {
            for (path, _) in capability
                .owners
                .iter()
                .chain(capability.application_routes)
            {
                files.insert(*path);
            }
        }
        Ok((spec, contract, manifest, files))
    }

    fn store_e2_records() -> Result<(&'static OwnerSpec, Value, Value, BTreeSet<&'static str>)> {
        let spec = OWNERS
            .iter()
            .find(|owner| owner.id == "wow-store/e2-d/projectstore-coherent-publication")
            .ok_or("Store E2-D owner mapping is absent")?;
        let contract = serde_json::from_str(include_str!(
            "../../../../crates/wow-store/e2/CONTRACT.json"
        ))?;
        let manifest = serde_json::from_str(include_str!(
            "../../../../crates/wow-store/e2/examples/CHECKSUMS.json"
        ))?;
        let mut files = BTreeSet::from([spec.contract_path, spec.manifest_path]);
        for capability in spec.capabilities {
            for (path, _) in capability
                .owners
                .iter()
                .chain(capability.application_routes)
                .chain(capability.service_routes.into_iter().flatten())
            {
                files.insert(*path);
            }
        }
        Ok((spec, contract, manifest, files))
    }

    #[test]
    fn service_e1_rejects_stale_no_code_and_accepted_missing_evidence() -> Result<()> {
        let (spec, contract, manifest, files) = service_e1_records()?;
        check(spec, contract.clone(), manifest.clone(), &files)?;

        assert_eq!(
            text(&contract, "application_selection")?,
            "application-selected"
        );
        assert!(
            manifest["prerequisite_pins"]
                .as_object()
                .is_some_and(|pins| pins.values().all(Value::is_null))
        );
        assert!(
            manifest["profile_pins"]
                .as_object()
                .is_some_and(|pins| pins.values().all(Value::is_null))
        );
        assert!(
            manifest["member_sha256"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
        assert!(manifest["bundle_sha256"].is_null());

        let mut no_code_contract = contract.clone();
        no_code_contract["state"] = json!("implementation-ready-documentation-no-rust-code");
        no_code_contract["implementation_status"] = json!("not-started");
        no_code_contract["implementation_state"] = json!("not-started");
        assert!(validate_contract(spec, &no_code_contract, &files).is_err());

        let mut no_code_manifest = manifest.clone();
        no_code_manifest["state"] = json!("pending-byte-freeze-before-first-rust-implementation");
        no_code_manifest["implementation_status"] = json!("not-started");
        no_code_manifest["implementation_state"] = json!("not-started");
        assert!(validate_manifest(spec, &no_code_manifest).is_err());

        let mut accepted_contract = contract.clone();
        accepted_contract["state"] = json!("accepted");
        accepted_contract["acceptance_status"] = json!("accepted");
        assert!(validate_contract(spec, &accepted_contract, &files).is_err());

        let mut accepted_manifest = manifest.clone();
        accepted_manifest["state"] = json!("accepted");
        accepted_manifest["acceptance_status"] = json!("accepted");
        assert!(validate_manifest(spec, &accepted_manifest).is_err());

        let mut library_only = contract;
        library_only["application_selection"] = json!("library-only");
        assert!(validate_contract(spec, &library_only, &files).is_err());
        Ok(())
    }

    #[test]
    fn store_e2_rejects_stale_and_accepted_claims_without_evidence() -> Result<()> {
        let (spec, contract, manifest, files) = store_e2_records()?;
        check(spec, contract.clone(), manifest.clone(), &files)?;

        let operations = contract["operations"]
            .as_array()
            .ok_or("Store E2 operations must be an array")?;
        let routes = contract["capability_routing"]
            .as_array()
            .ok_or("Store E2 capability routing must be an array")?;
        assert_eq!(operations.len(), 17);
        assert_eq!(routes.len(), operations.len());
        assert_eq!(
            routes
                .iter()
                .filter(|row| row["application_selection"] == "application-selected")
                .count(),
            12
        );
        assert_eq!(
            routes
                .iter()
                .filter(|row| row["application_selection"] == "library-only")
                .count(),
            5
        );
        assert_eq!(
            routes
                .iter()
                .filter(|row| {
                    row["application_selection"] == "library-only"
                        && row["application_routes"]
                            .as_array()
                            .is_some_and(Vec::is_empty)
                })
                .count(),
            5
        );
        assert!(
            routes
                .iter()
                .filter(|row| row["application_selection"] == "application-selected")
                .all(|row| row["application_routes"]
                    .as_array()
                    .is_some_and(|routes| !routes.is_empty()))
        );

        let prerequisites = manifest["prerequisite_pins"]
            .as_object()
            .ok_or("prerequisite pins must be an object")?;
        assert_eq!(prerequisites.len(), 10);
        assert!(prerequisites.values().all(Value::is_null));
        let runtime = manifest["runtime_profile_pins"]
            .as_object()
            .ok_or("runtime profile pins must be an object")?;
        assert_eq!(runtime.len(), 7);
        assert_eq!(
            runtime.get("physical_profile_id").and_then(Value::as_str),
            Some("project-store-wal-manifested-partitions-v1")
        );
        assert!(
            runtime
                .iter()
                .filter(|(key, _)| key.as_str() != "physical_profile_id")
                .all(|(_, value)| value.is_null())
        );
        for group in ["schema_profile_pins", "vector_pins"] {
            let pins = manifest[group]
                .as_object()
                .ok_or("schema/vector pins must be objects")?;
            assert!(pins.values().all(Value::is_null));
        }
        assert!(
            manifest["member_sha256"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
        assert!(manifest["bundle_sha256"].is_null());

        let mut no_code_contract = contract.clone();
        no_code_contract["state"] = json!("implementation-ready-documentation-no-rust-code");
        no_code_contract["implementation_status"] = json!("not-started");
        no_code_contract["implementation_state"] = json!("not-started");
        assert!(validate_contract(spec, &no_code_contract, &files).is_err());

        let mut no_code_manifest = manifest.clone();
        no_code_manifest["state"] = json!("pending-byte-freeze-before-first-rust-implementation");
        no_code_manifest["implementation_status"] = json!("not-started");
        no_code_manifest["implementation_state"] = json!("not-started");
        assert!(validate_manifest(spec, &no_code_manifest).is_err());

        let mut accepted_contract = contract.clone();
        accepted_contract["state"] = json!("accepted");
        accepted_contract["acceptance_status"] = json!("accepted");
        assert!(validate_contract(spec, &accepted_contract, &files).is_err());

        let mut accepted_manifest = manifest.clone();
        accepted_manifest["state"] = json!("accepted");
        accepted_manifest["acceptance_status"] = json!("accepted");
        assert!(validate_manifest(spec, &accepted_manifest).is_err());

        let mut stale_matrix = contract.clone();
        stale_matrix["operations"][16] = json!("unsupported_store_operation");
        assert!(validate_contract(spec, &stale_matrix, &files).is_err());

        let mut journal_read_as_creator = contract.clone();
        journal_read_as_creator["capability_routing"][6]["owners"] = json!([
            {"path":"crates/wow-store/src/project/publication.rs","symbol":"ProjectStore::prepare"},
            {"path":"crates/wow-store/src/project/mod.rs","symbol":"ProjectStore::operation"}
        ]);
        assert!(validate_contract(spec, &journal_read_as_creator, &files).is_err());

        let mut state_type_as_creator = contract.clone();
        state_type_as_creator["capability_routing"][6]["owners"] = json!([
            {"path":"crates/wow-store/src/project/publication.rs","symbol":"ProjectStore::prepare"},
            {"path":"crates/wow-store/src/project/model.rs","symbol":"PublicationOperation"}
        ]);
        assert!(validate_contract(spec, &state_type_as_creator, &files).is_err());

        let mut routed_library_only = contract.clone();
        routed_library_only["capability_routing"][11]["application_routes"] =
            json!([{"path":"apps/wow/src/project.rs","symbol":"run"}]);
        assert!(validate_contract(spec, &routed_library_only, &files).is_err());

        let mut missing_service_route = contract.clone();
        missing_service_route["capability_routing"][16]["service_routes"] = json!([]);
        assert!(validate_contract(spec, &missing_service_route, &files).is_err());

        let mut stale_migration_owner = contract.clone();
        stale_migration_owner["capability_routing"][16]["owners"][2]["symbol"] =
            json!("LiveProjectStore::activate_ready_selection");
        assert!(validate_contract(spec, &stale_migration_owner, &files).is_err());

        let mut misclassified = contract;
        misclassified["capability_routing"][0]["application_selection"] = json!("library-only");
        assert!(validate_contract(spec, &misclassified, &files).is_err());
        Ok(())
    }

    fn recognizer_e2_records() -> Result<(&'static OwnerSpec, Value, Value, BTreeSet<&'static str>)>
    {
        let spec = OWNERS
            .iter()
            .find(|owner| owner.id == "wow-recognizers/e2-b/core-structural-rules")
            .ok_or("Recognizer E2-B owner mapping is absent")?;
        let contract = serde_json::from_str(include_str!(
            "../../../../crates/wow-recognizers/e2/CONTRACT.json"
        ))?;
        let manifest = serde_json::from_str(include_str!(
            "../../../../crates/wow-recognizers/e2/examples/CHECKSUMS.json"
        ))?;
        let mut files = BTreeSet::from([spec.contract_path, spec.manifest_path]);
        for capability in spec.capabilities {
            for (path, _) in capability
                .owners
                .iter()
                .chain(capability.application_routes)
            {
                files.insert(*path);
            }
        }
        Ok((spec, contract, manifest, files))
    }

    #[test]
    fn recognizer_e2_maps_all_26_rules_and_keeps_acceptance_pending() -> Result<()> {
        let (spec, contract, manifest, files) = recognizer_e2_records()?;
        check(spec, contract.clone(), manifest.clone(), &files)?;
        assert!(manifest.get("implementation_state").is_none());

        let active = contract["active_rule_ids"]
            .as_array()
            .ok_or("active_rule_ids must be an array")?;
        let routes = contract["capability_routing"]
            .as_array()
            .ok_or("capability_routing must be an array")?;
        assert_eq!(active.len(), 26);
        assert_eq!(routes.len(), active.len());
        assert_eq!(
            text(&contract, "application_selection")?,
            "application-selected"
        );
        assert_eq!(
            text(&contract, "implementation_status")?,
            "partial-functional"
        );
        assert_eq!(text(&contract, "acceptance_status")?, "pending");
        assert_eq!(contract["canonical_issue"].as_u64(), Some(103));
        assert_eq!(text(&manifest, "contract_id")?, spec.id);
        assert_eq!(manifest["canonical_issue"].as_u64(), Some(103));
        assert_eq!(
            text(&contract["preimplementation_freeze"], "status")?,
            "missed"
        );
        assert_eq!(
            text(&contract["preimplementation_freeze"], "evaluation")?,
            "NotEvaluated"
        );
        for (identity, route) in active.iter().zip(routes) {
            assert_eq!(identity.as_str(), route["capability"].as_str());
            assert_eq!(text(route, "status")?, "partial-functional");
            assert_eq!(text(route, "acceptance_status")?, "pending");
            let blockers = route["blocking_issues"]
                .as_array()
                .ok_or("blocking_issues must be an array")?;
            assert!(blockers.iter().any(|issue| issue.as_u64() == Some(103)));
            assert!(blockers.iter().any(|issue| issue.as_u64() == Some(120)));
        }

        let prerequisites = manifest["prerequisite_pins"]
            .as_object()
            .ok_or("prerequisite_pins must be an object")?;
        assert!(prerequisites.values().all(Value::is_null));
        let profiles = manifest["profile_pins"]
            .as_object()
            .ok_or("profile_pins must be an object")?;
        for (key, value) in profiles {
            if matches!(
                key.as_str(),
                "recognizer_contract_id" | "core_pack_id" | "core_pack_version"
            ) {
                assert!(value.is_string());
            } else {
                assert!(value.is_null(), "profile pin {key} must remain null");
            }
        }
        assert!(
            manifest["frozen_vectors"]
                .as_object()
                .is_some_and(|pins| pins.values().all(Value::is_null))
        );
        assert!(
            manifest["member_sha256"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
        assert!(manifest["bundle_sha256"].is_null());

        let mut stale_contract = contract.clone();
        stale_contract["state"] = json!("implementation-ready-documentation-no-rust-code");
        stale_contract["implementation_status"] = json!("not-started");
        assert!(validate_contract(spec, &stale_contract, &files).is_err());

        let mut stale_manifest = manifest.clone();
        stale_manifest["state"] = json!("pending-byte-freeze-before-first-rust-implementation");
        stale_manifest["implementation_status"] = json!("not-started");
        assert!(validate_manifest(spec, &stale_manifest).is_err());

        let mut legacy_manifest = manifest.clone();
        legacy_manifest["implementation_state"] = json!("partial-functional");
        assert!(validate_manifest(spec, &legacy_manifest).is_err());

        let mut accepted_contract = contract.clone();
        accepted_contract["acceptance_status"] = json!("accepted");
        assert!(validate_contract(spec, &accepted_contract, &files).is_err());

        let mut accepted_manifest = manifest.clone();
        accepted_manifest["acceptance_status"] = json!("accepted");
        assert!(validate_manifest(spec, &accepted_manifest).is_err());

        let mut stale_rule_matrix = contract.clone();
        stale_rule_matrix["active_rule_ids"][25] = json!("core.future.rule@1");
        assert!(validate_contract(spec, &stale_rule_matrix, &files).is_err());

        let mut type_only_owner = contract.clone();
        type_only_owner["capability_routing"][0]["owners"][0]["symbol"] = json!("SourceTocFamily");
        assert!(validate_contract(spec, &type_only_owner, &files).is_err());

        let mut missing_app_route = contract.clone();
        missing_app_route["capability_routing"][0]["application_routes"] = json!([]);
        assert!(validate_contract(spec, &missing_app_route, &files).is_err());

        let mut missing_acceptance_issue = contract;
        missing_acceptance_issue["capability_routing"][0]["blocking_issues"] = json!([120]);
        assert!(validate_contract(spec, &missing_acceptance_issue, &files).is_err());
        Ok(())
    }
}
