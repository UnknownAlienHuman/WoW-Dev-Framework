//! Source registry's reviewed named-axis selection. Graph owns direction and
//! assertion association; this owner decides which source definitions participate.
use super::invalid;
use crate::ProjectResult;
use wow_graph::{GraphAxis, GraphAxisProfile, GraphRegistryBundle, GraphRelationKind};

/// Reconstructed by the service, never supplied by a graph-read request. An
/// unknown definition in a selected family rejects rather than widening a recipe.
pub fn source_graph_axis_profile(
    registry: &GraphRegistryBundle,
    axis: GraphAxis,
) -> ProjectResult<GraphAxisProfile> {
    let groups: &[(GraphRelationKind, &[&str])] = match axis {
        GraphAxis::Ownership => &[(
            GraphRelationKind::Owns,
            &[
                "source_package_owns",
                "source_declaration_owns",
                "toc_owns_state",
                "xml_owns_template",
                "xml_owns_object",
                "xml_owns_script",
                "xml_occurrence_owns_script_site",
                "xml_occurrence_owns_load_site",
                "xml_occurrence_owns_parent_reference",
                "xml_script_site_owns_virtual_lua",
            ],
        )],
        GraphAxis::Load => &[
            (
                GraphRelationKind::Loads,
                &[
                    "source_loads",
                    "source_package_loads",
                    "toc_loads",
                    "source_toc_occurrence_loads",
                    "xml_include_target",
                    "xml_external_script_target",
                ],
            ),
            (
                GraphRelationKind::DependsOn,
                &[
                    "source_depends_on",
                    "source_package_depends_on",
                    "toc_depends_on",
                ],
            ),
            (
                GraphRelationKind::LoadsBefore,
                &["toc_loads_before", "source_toc_occurs_before"],
            ),
            (
                GraphRelationKind::OptionalDependsOn,
                &["toc_optional_depends_on"],
            ),
        ],
        GraphAxis::Inheritance => &[
            (
                GraphRelationKind::Inherits,
                &["source_xml_inherits", "xml_inherits"],
            ),
            (GraphRelationKind::MixesIn, &["source_mixes_in"]),
        ],
        GraphAxis::Registration => &[
            (
                GraphRelationKind::RegistersNativeEvent,
                &["lua_registers_native_event"],
            ),
            (
                GraphRelationKind::HandlesNativeEvent,
                &["lua_handles_native_event"],
            ),
            (
                GraphRelationKind::BridgesNativeEvent,
                &["lua_bridges_native_event"],
            ),
            (
                GraphRelationKind::EmitsCustomSignal,
                &["lua_emits_custom_signal", "source_emits_custom_signal"],
            ),
            (
                GraphRelationKind::HandlesCustomSignal,
                &[
                    "lua_handles_custom_signal",
                    "source_subscribes_custom_signal",
                ],
            ),
            (
                GraphRelationKind::RegistersCvarCallback,
                &["lua_registers_cvar_callback"],
            ),
            (
                GraphRelationKind::SetsScript,
                &[
                    "source_xml_sets_script",
                    "lua_sets_script",
                    "xml_sets_script",
                ],
            ),
            (GraphRelationKind::HooksScript, &["lua_hooks_script"]),
            (
                GraphRelationKind::SecureHooksFunction,
                &["lua_secure_hooks_function"],
            ),
        ],
        // These source recipes currently have one definition per family. Keep
        // their existing profile bytes; native unique-family validation applies.
        _ => return GraphAxisProfile::bind(registry, axis).map_err(|_| invalid()),
    };
    // Other graph families/registries still use their original single-definition
    // route. Do not interpret a foreign registry as a source registry by enum.
    if registry.bundle_id() != super::SOURCE_GRAPH_REGISTRY_ID {
        return GraphAxisProfile::bind(registry, axis).map_err(|_| invalid());
    }
    let mut ids = Vec::new();
    for (kind, reviewed) in groups {
        for id in *reviewed {
            if let Some(definition) = registry.relation_kind(id)
                && definition.relation() != *kind
            {
                return Err(invalid());
            }
        }
    }
    for definition in registry.relation_kinds() {
        let Some((_, reviewed)) = groups
            .iter()
            .find(|(kind, _)| *kind == definition.relation())
        else {
            continue;
        };
        if axis == GraphAxis::Ownership
            && matches!(
                definition.relation_id(),
                "xml_occurrence_source_span" | "xml_map_piece_source_span"
            )
        {
            continue;
        }
        if !reviewed.contains(&definition.relation_id()) {
            return Err(invalid());
        }
        ids.push(definition.relation_id());
    }
    GraphAxisProfile::bind_definitions(registry, axis, &ids).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectKind;
    use crate::graph::{DirectRecipe, registry_for_recipe, registry_with_native_api};
    use wow_graph::{GraphConfidence, GraphDirection, GraphRelationKindDefinition};
    type ResultOf<T = ()> = Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn real_capability_registries_bind_exact_reviewed_definitions() -> ResultOf {
        use crate::{PlatformGraphCapabilities, PlatformGraphCapability::*};
        for (capabilities, ownership_count, custom_signals) in [
            (
                PlatformGraphCapabilities::new(&[InventorySpansV1, StructuralRolesV1])?,
                9,
                false,
            ),
            (
                PlatformGraphCapabilities::new(&[
                    InventorySpansV1,
                    StructuralRolesV1,
                    XmlSourceMapsV1,
                ])?,
                10,
                false,
            ),
            (
                PlatformGraphCapabilities::new(&[
                    InventorySpansV1,
                    StructuralRolesV1,
                    XmlSourceMapsV1,
                    CustomSignalRelationsV1,
                ])?,
                10,
                true,
            ),
        ] {
            let recipe = DirectRecipe::Capabilities(capabilities);
            let registry =
                registry_for_recipe(ProjectKind::BlizzardUiPlatformSource, true, recipe)?;
            assert_eq!(registry.version(), capabilities.version_id());
            let entities: Vec<wow_graph::GraphEntityKindDefinition> =
                serde_json::from_value(serde_json::to_value(&registry)?["entity_kinds"].clone())?;
            for axis in [
                GraphAxis::Ownership,
                GraphAxis::Load,
                GraphAxis::Inheritance,
                GraphAxis::Registration,
            ] {
                let profile = source_graph_axis_profile(&registry, axis)?;
                profile.validate(&registry)?;
                assert!(profile.definition_selected());
                let restored: GraphAxisProfile =
                    serde_json::from_slice(&serde_json::to_vec(&profile)?)?;
                assert_eq!(profile, restored);
                restored.validate(&registry)?;
                for step in profile.relations() {
                    let definition = registry
                        .relation_kind(step.relation_id())
                        .ok_or("definition")?;
                    assert_eq!(definition.relation(), step.relation());
                    let direction = if matches!(
                        step.relation(),
                        GraphRelationKind::DependsOn
                            | GraphRelationKind::OptionalDependsOn
                            | GraphRelationKind::Inherits
                            | GraphRelationKind::MixesIn
                    ) {
                        GraphDirection::Incoming
                    } else {
                        GraphDirection::Outgoing
                    };
                    assert_eq!(step.forward_direction(), direction);
                }
                if axis == GraphAxis::Ownership {
                    assert_eq!(profile.relations().len(), ownership_count);
                    assert!(profile.relations().iter().all(|step| !matches!(
                        step.relation_id(),
                        "xml_occurrence_source_span" | "xml_map_piece_source_span"
                    )));
                }
                if axis == GraphAxis::Load {
                    assert!(
                        GraphAxisProfile::bind_definitions(
                            &registry,
                            axis,
                            &["source_loads", "source_depends_on"]
                        )
                        .is_err()
                    );
                    assert_eq!(profile.relations().len(), 12);
                    let loads = registry
                        .relation_kind("xml_include_target")
                        .ok_or("include definition")?;
                    assert!(loads.allows_source_kind("xml_source_load_site"));
                    assert!(loads.allows_target_kind("source_file"));
                    assert!(loads.allows_confidence(GraphConfidence::Proven));
                    assert!(!loads.allows_confidence(GraphConfidence::Candidate));
                }
                if axis == GraphAxis::Registration {
                    assert!(registry.relation_kind("lua_emits_custom_signal").is_some());
                    assert!(
                        registry
                            .relation_kind("lua_handles_custom_signal")
                            .is_some()
                    );
                    if custom_signals {
                        let emitter = registry
                            .relation_kind("source_emits_custom_signal")
                            .ok_or("recognizer custom-signal emitter definition")?;
                        assert_eq!(emitter.relation(), GraphRelationKind::EmitsCustomSignal);
                        assert!(emitter.allows_source_kind("lua_source_function"));
                        assert!(emitter.allows_target_kind("custom_signal"));
                        assert!(emitter.allows_confidence(GraphConfidence::Derived));
                        assert!(emitter.allows_confidence(GraphConfidence::Possible));
                        let subscription = registry
                            .relation_kind("source_subscribes_custom_signal")
                            .ok_or("recognizer custom-signal subscription definition")?;
                        assert_eq!(
                            subscription.relation(),
                            GraphRelationKind::HandlesCustomSignal
                        );
                        assert!(subscription.allows_source_kind("lua_source_function"));
                        assert!(subscription.allows_target_kind("custom_signal"));
                        assert!(subscription.allows_confidence(GraphConfidence::Derived));
                        assert!(subscription.allows_confidence(GraphConfidence::Possible));
                        assert!(
                            profile
                                .relations()
                                .iter()
                                .any(|step| { step.relation_id() == "source_emits_custom_signal" })
                        );
                        assert!(profile.relations().iter().any(|step| {
                            step.relation_id() == "source_subscribes_custom_signal"
                        }));
                    } else {
                        assert!(
                            registry
                                .relation_kind("source_emits_custom_signal")
                                .is_none()
                        );
                        assert!(
                            registry
                                .relation_kind("source_subscribes_custom_signal")
                                .is_none()
                        );
                        assert!(profile.relations().iter().all(|step| {
                            !matches!(
                                step.relation_id(),
                                "source_emits_custom_signal" | "source_subscribes_custom_signal"
                            )
                        }));
                    }
                }
            }
            let mut definitions = registry.relation_kinds().to_vec();
            definitions.push(GraphRelationKindDefinition::new(
                "unreviewed_owns",
                GraphRelationKind::Owns,
                vec!["source_package".into()],
                vec!["source_file".into()],
                vec![GraphConfidence::Proven],
            )?);
            let unknown = GraphRegistryBundle::build(
                registry.bundle_id(),
                registry.version(),
                entities.clone(),
                definitions,
            )?;
            assert!(source_graph_axis_profile(&unknown, GraphAxis::Ownership).is_err());
            let mut definitions = registry.relation_kinds().to_vec();
            let position = definitions
                .iter()
                .position(|item| item.relation_id() == "source_loads")
                .ok_or("load definition")?;
            definitions[position] = GraphRelationKindDefinition::new(
                "source_loads",
                GraphRelationKind::Contains,
                vec!["source_file".into()],
                vec!["source_file".into()],
                vec![GraphConfidence::Proven],
            )?;
            let substituted = GraphRegistryBundle::build(
                registry.bundle_id(),
                registry.version(),
                entities,
                definitions,
            )?;
            assert!(source_graph_axis_profile(&substituted, GraphAxis::Load).is_err());
        }
        for (raw_inventory, version) in [(false, "15"), (true, "16")] {
            let legacy = registry_for_recipe(
                ProjectKind::BlizzardUiPlatformSource,
                raw_inventory,
                DirectRecipe::Original,
            )?;
            assert_eq!(legacy.version(), version);
            assert!(legacy.relation_kind("lua_emits_custom_signal").is_some());
            assert!(legacy.relation_kind("lua_handles_custom_signal").is_some());
            assert!(legacy.relation_kind("source_emits_custom_signal").is_none());
            assert!(
                legacy
                    .relation_kind("source_subscribes_custom_signal")
                    .is_none()
            );
        }

        let legacy_native_api =
            registry_with_native_api(ProjectKind::Fixture, false, DirectRecipe::Original, true)?;
        assert_eq!(legacy_native_api.version(), "native-api-uses-1");
        let selected = super::super::registry_with_components(
            ProjectKind::Fixture,
            false,
            DirectRecipe::Original,
            false,
            true,
        )?;
        assert_eq!(selected.version(), "source-signals-1");
        let emitter = selected
            .relation_kind("source_emits_custom_signal")
            .ok_or("selected source emitter")?;
        assert!(emitter.allows_source_kind("lua_source_function"));
        assert!(emitter.allows_target_kind("custom_signal"));
        assert!(emitter.allows_confidence(GraphConfidence::Possible));
        source_graph_axis_profile(&selected, GraphAxis::Registration)?;
        assert!(
            legacy_native_api
                .relation_kind("source_emits_custom_signal")
                .is_none()
        );
        Ok(())
    }
}
