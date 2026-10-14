//! Finite implemented service selections, not full-contract or runtime acceptance.
use super::super::*;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct Capability {
    operation: &'static str,
    availability: &'static str,
    scope: &'static str,
}
pub(super) const CAPABILITIES: &[Capability] = &[
    Capability {
        operation: "context_status",
        availability: "partial_functional",
        scope: "primary_exact_metadata",
    },
    Capability {
        operation: "context_map",
        availability: "partial_functional",
        scope: "selected_primary_profiles; platform_and_combined_require_platform_selection",
    },
    Capability {
        operation: "context_inspect",
        availability: "partial_functional",
        scope: "selected_primary_l0_l1_frontier_and_progressive_profiles",
    },
    Capability {
        operation: "context_build",
        availability: "partial_functional",
        scope: "selected_primary_packs_and_renderers; combined_requires_platform_selection",
    },
    Capability {
        operation: "context_continue",
        availability: "partial_functional",
        scope: "primary_progressive_exact_generation_store_issued_replay",
    },
    Capability {
        operation: "context_validate",
        availability: "partial_functional",
        scope: "primary_structural_pack_validation",
    },
    Capability {
        operation: "context_render",
        availability: "partial_functional",
        scope: "external_primary_default_l0_and_entity_v1_l1_structural_json_input; other_recipes_not_evaluated",
    },
];

#[derive(Serialize)]
pub(super) struct Profile {
    profile_id: &'static str,
    role: &'static str,
    availability: &'static str,
}
const fn profile(profile_id: &'static str, role: &'static str) -> Profile {
    Profile {
        profile_id,
        role,
        availability: "partial_functional",
    }
}
const fn unselected_platform(profile_id: &'static str, role: &'static str) -> Profile {
    Profile {
        profile_id,
        role,
        availability: "requires_platform_selection",
    }
}
pub(super) const PROFILES: &[Profile] = &[
    profile(PRIMARY_NAVIGATION_MAP_PROFILE_ID, "primary_map"),
    profile(
        PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        "primary_project_map",
    ),
    profile(PRIMARY_L0_PROFILE_ID, "primary_l0"),
    profile(PRIMARY_L1_PROFILE_ID, "primary_l1"),
    profile(PRIMARY_STATE_ROOT_L0_PROFILE_ID, "primary_state_root_l0"),
    profile(
        PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        "primary_state_detail_l1",
    ),
    profile(
        PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
        "primary_reference_facts_l1",
    ),
    Profile {
        profile_id: wow_context::PRIMARY_REFERENCE_ROOT_L0_PROFILE,
        role: "combined_reference_facts_both_internal",
        availability: "not_requestable",
    },
    profile(PRIMARY_FRONTIER_EXPANSION_PROFILE_ID, "primary_frontier"),
    profile(
        PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
        "primary_progressive",
    ),
    profile(PRIMARY_CONTEXT_PACK_PROFILE_ID, "primary_pack"),
    profile(
        PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID,
        "primary_frontier_pack",
    ),
    profile(
        PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID,
        "primary_progressive_pack",
    ),
    profile(
        PRIMARY_SOURCE_CONTEXT_PACK_PROFILE_ID,
        "primary_source_excerpt_pack",
    ),
    profile(PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID, "primary_json"),
    profile(
        PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "primary_markdown",
    ),
    profile(
        PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
        "primary_frontier_json",
    ),
    profile(
        PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "primary_frontier_markdown",
    ),
    profile(
        PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
        "primary_progressive_json",
    ),
    profile(
        PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "primary_progressive_markdown",
    ),
    profile(
        PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID,
        "primary_source_json",
    ),
    profile(
        PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "primary_source_markdown",
    ),
    unselected_platform(PLATFORM_NATIVE_ONE_HOP_PROFILE_ID, "platform_map_inspect"),
    unselected_platform(COMBINED_NATIVE_MAP_PROFILE_ID, "combined_map"),
    unselected_platform(COMBINED_CONTEXT_PACK_PROFILE_ID, "combined_pack"),
    unselected_platform(COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID, "combined_json"),
    unselected_platform(
        COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "combined_markdown",
    ),
];
