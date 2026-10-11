//! Finite, canonical producer selection. Publication policy is a separate owner.
use super::*;

/// Independently versioned native graph capabilities; dependencies are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformGraphCapability {
    InventorySpansV1,
    StructuralRolesV1,
    XmlSourceMapsV1,
    XmlSchemaClassificationV1,
}
impl PlatformGraphCapability {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::InventorySpansV1 => "inventory_spans_v1",
            Self::StructuralRolesV1 => "structural_roles_v1",
            Self::XmlSourceMapsV1 => "xml_source_maps_v1",
            Self::XmlSchemaClassificationV1 => "xml_schema_classification_v1",
        }
    }
    #[must_use]
    pub const fn dependencies(self) -> &'static [Self] {
        match self {
            Self::InventorySpansV1 => &[],
            Self::StructuralRolesV1 => &[Self::InventorySpansV1],
            Self::XmlSourceMapsV1 | Self::XmlSchemaClassificationV1 => {
                &[Self::InventorySpansV1, Self::StructuralRolesV1]
            }
        }
    }
}

/// Construction and decoding refuse missing dependencies rather than adding them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformGraphCapabilities {
    inventory_spans: bool,
    structural_roles: bool,
    xml_source_maps: bool,
    xml_schema_classification: bool,
}
impl PlatformGraphCapabilities {
    pub fn new(capabilities: &[PlatformGraphCapability]) -> ProjectResult<Self> {
        let invalid = || {
            ProjectError::new(
                ProjectErrorCode::InvalidConfiguration,
                ProjectPhase::Configuration,
                "graph capability selection contains duplicate or missing dependencies",
            )
        };
        if capabilities.len() > 4 {
            return Err(invalid());
        }
        let mut selected = Self::default();
        for &capability in capabilities {
            if selected.contains(capability) {
                return Err(invalid());
            }
            match capability {
                PlatformGraphCapability::InventorySpansV1 => selected.inventory_spans = true,
                PlatformGraphCapability::StructuralRolesV1 => selected.structural_roles = true,
                PlatformGraphCapability::XmlSourceMapsV1 => selected.xml_source_maps = true,
                PlatformGraphCapability::XmlSchemaClassificationV1 => {
                    selected.xml_schema_classification = true;
                }
            }
        }
        for &capability in capabilities {
            if capability
                .dependencies()
                .iter()
                .any(|&dependency| !selected.contains(dependency))
            {
                return Err(invalid());
            }
        }
        Ok(selected)
    }
    #[must_use]
    pub const fn contains(self, capability: PlatformGraphCapability) -> bool {
        match capability {
            PlatformGraphCapability::InventorySpansV1 => self.inventory_spans,
            PlatformGraphCapability::StructuralRolesV1 => self.structural_roles,
            PlatformGraphCapability::XmlSourceMapsV1 => self.xml_source_maps,
            PlatformGraphCapability::XmlSchemaClassificationV1 => self.xml_schema_classification,
        }
    }
    pub fn iter(self) -> impl Iterator<Item = PlatformGraphCapability> {
        [
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
            PlatformGraphCapability::XmlSourceMapsV1,
            PlatformGraphCapability::XmlSchemaClassificationV1,
        ]
        .into_iter()
        .filter(move |&capability| self.contains(capability))
    }
    /// Exact canonical identity, used by registry and producer metadata.
    #[must_use]
    pub fn version_id(self) -> String {
        let mut version = String::from("capabilities_v1");
        for capability in self.iter() {
            version.push(':');
            version.push_str(capability.id());
        }
        version
    }
}
impl Serialize for PlatformGraphCapabilities {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
impl<'de> Deserialize<'de> for PlatformGraphCapabilities {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SelectionVisitor;
        impl<'de> serde::de::Visitor<'de> for SelectionVisitor {
            type Value = PlatformGraphCapabilities;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .write_str("at most four distinct graph capabilities with exact dependencies")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut capabilities = Vec::with_capacity(4);
                while let Some(capability) = sequence.next_element::<PlatformGraphCapability>()? {
                    if capabilities.len() == 4 {
                        return Err(serde::de::Error::custom("graph capability count exceeded"));
                    }
                    capabilities.push(capability);
                }
                PlatformGraphCapabilities::new(&capabilities).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_seq(SelectionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformGraphCapabilities, PlatformGraphCapability::*, PlatformGraphProfile};
    type ResultOf = Result<(), Box<dyn std::error::Error>>;
    #[test]
    fn selection_canonicalizes_order_and_refuses_dependency_substitution() -> ResultOf {
        let selected = PlatformGraphCapabilities::new(&[
            XmlSourceMapsV1,
            InventorySpansV1,
            StructuralRolesV1,
        ])?;
        assert_eq!(
            serde_json::to_string(&selected)?,
            r#"["inventory_spans_v1","structural_roles_v1","xml_source_maps_v1"]"#
        );
        assert_eq!(
            selected,
            serde_json::from_str(
                r#"["structural_roles_v1","xml_source_maps_v1","inventory_spans_v1"]"#
            )?
        );
        assert_ne!(
            selected.version_id(),
            PlatformGraphCapabilities::new(&[InventorySpansV1, StructuralRolesV1])?.version_id()
        );
        for invalid in [
            r#"["structural_roles_v1"]"#,
            r#"["xml_source_maps_v1","inventory_spans_v1"]"#,
            r#"["inventory_spans_v1","inventory_spans_v1"]"#,
            r#"["inventory_spans_v2"]"#,
            r#"["inventory_spans_v1","structural_roles_v1","xml_source_maps_v1","inventory_spans_v1"]"#,
            r#"{"inventory_spans_v1":true}"#,
        ] {
            assert!(
                serde_json::from_str::<PlatformGraphCapabilities>(invalid).is_err(),
                "admitted {invalid}"
            );
        }
        let profile = PlatformGraphProfile::DirectCapabilitiesV1(selected);
        assert_eq!(
            profile,
            serde_json::from_slice(&serde_json::to_vec(&profile)?)?
        );
        assert_eq!(
            serde_json::to_string(
                &PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1
            )?,
            r#""direct_platform_producers_with_raw_inventory_v1""#
        );
        Ok(())
    }
}
