use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    canonical_json_digest, require_same_generation, CoreError, CoreErrorCode, CoreResult,
    EvidenceId, GenerationContext, ProducerId, SourceHandle, SourceOwner, ToolVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    Candidate,
    Possible,
    Derived,
    Proven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceClass {
    PlatformSource,
    ProjectSource,
    RuntimeProbe,
    CuratedCorrection,
    DifferentialOracle,
    ExternalImplementation,
    SemanticCandidate,
    HistoricalRecord,
    ModelInference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceRecord {
    provenance: ProvenanceClass,
    confidence: EvidenceLevel,
    source: Option<SourceHandle>,
    producer: ProducerId,
    producer_version: ToolVersion,
    generation: GenerationContext,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EvidenceRecordWire {
    provenance: ProvenanceClass,
    confidence: EvidenceLevel,
    source: Option<SourceHandle>,
    producer: ProducerId,
    producer_version: ToolVersion,
    generation: GenerationContext,
    note: Option<String>,
}

impl EvidenceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provenance: ProvenanceClass,
        confidence: EvidenceLevel,
        source: Option<SourceHandle>,
        producer: ProducerId,
        producer_version: ToolVersion,
        generation: GenerationContext,
        note: Option<String>,
    ) -> CoreResult<Self> {
        let record = Self {
            provenance,
            confidence,
            source,
            producer,
            producer_version,
            generation,
            note,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> CoreResult<()> {
        if matches!(
            self.provenance,
            ProvenanceClass::SemanticCandidate | ProvenanceClass::ModelInference
        ) && self.confidence != EvidenceLevel::Candidate
        {
            return Err(CoreError::new(
                CoreErrorCode::ResultContextViolation,
                "validate_evidence",
                "semantic candidates and model inference cannot exceed Candidate confidence",
            ));
        }

        if let Some(note) = &self.note {
            if note.len() > 2_048 || note.chars().any(char::is_control) {
                return Err(CoreError::new(
                    CoreErrorCode::ResultContextViolation,
                    "validate_evidence",
                    "evidence note is oversized or contains control characters",
                ));
            }
        }

        if let Some(source) = &self.source {
            source.validate()?;
            if let SourceOwner::ReferencePack {
                profile,
                reference_generation,
            } = source.owner()
            {
                if profile != self.generation.profile().id()
                    || reference_generation != self.generation.reference()
                {
                    return Err(CoreError::new(
                        CoreErrorCode::ResultContextViolation,
                        "validate_evidence",
                        "reference source handle does not match evidence generation context",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn require_context(&self, expected: &GenerationContext) -> CoreResult<()> {
        require_same_generation(expected, &self.generation)
    }

    #[must_use]
    pub fn provenance(&self) -> ProvenanceClass {
        self.provenance
    }

    #[must_use]
    pub fn confidence(&self) -> EvidenceLevel {
        self.confidence
    }

    #[must_use]
    pub fn source(&self) -> Option<&SourceHandle> {
        self.source.as_ref()
    }

    #[must_use]
    pub fn producer(&self) -> &ProducerId {
        &self.producer
    }

    #[must_use]
    pub fn producer_version(&self) -> &ToolVersion {
        &self.producer_version
    }

    #[must_use]
    pub fn generation(&self) -> &GenerationContext {
        &self.generation
    }

    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

pub fn combine_evidence_levels<I>(levels: I) -> Option<EvidenceLevel>
where
    I: IntoIterator<Item = EvidenceLevel>,
{
    levels.into_iter().min()
}

pub fn canonical_evidence_key(record: &EvidenceRecord) -> CoreResult<EvidenceId> {
    record.validate()?;
    let digest = canonical_json_digest(record)?;
    EvidenceId::parse(format!(
        "evidence:{}",
        digest.canonical_string().trim_start_matches("sha256:")
    ))
}

impl<'de> Deserialize<'de> for EvidenceRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EvidenceRecordWire::deserialize(deserializer)?;
        Self::new(
            wire.provenance,
            wire.confidence,
            wire.source,
            wire.producer,
            wire.producer_version,
            wire.generation,
            wire.note,
        )
        .map_err(D::Error::custom)
    }
}
