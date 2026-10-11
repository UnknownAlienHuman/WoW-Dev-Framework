//! Fixed-size, nonsemantic observations of the existing metadata counter.
//! Never persisted in a proposal, configuration, report, or replay recipe.
use super::{ProducerBudget, ProjectResult, ProjectView};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase};
use serde::Serialize;
use std::{sync::atomic::AtomicBool, time::Instant};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CountingMethod {
    ConservativeEstimate,
    SerializedBytes,
    Mixed,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Phase {
    SelectedSources,
    PackageXmlBindings,
    RawInventory,
    FilesAndLoads,
    Packages,
    Xml,
    Mixins,
    Functions,
    Calls,
    Scripts,
    State,
    TocFacts,
    XmlFacts,
    CollectorEncoding,
    DirectPreparation,
    InventoryStage,
    TocLoadStage,
    AnalyzerStructureStage,
    XmlStructureStage,
    DirectSupport,
}
impl Phase {
    const ALL: [Self; 20] = [
        Self::SelectedSources,
        Self::PackageXmlBindings,
        Self::RawInventory,
        Self::FilesAndLoads,
        Self::Packages,
        Self::Xml,
        Self::Mixins,
        Self::Functions,
        Self::Calls,
        Self::Scripts,
        Self::State,
        Self::TocFacts,
        Self::XmlFacts,
        Self::CollectorEncoding,
        Self::DirectPreparation,
        Self::InventoryStage,
        Self::TocLoadStage,
        Self::AnalyzerStructureStage,
        Self::XmlStructureStage,
        Self::DirectSupport,
    ];
    const fn name(self) -> &'static str {
        match self {
            Self::SelectedSources => "selected_sources",
            Self::PackageXmlBindings => "package_xml_bindings",
            Self::RawInventory => "raw_inventory",
            Self::FilesAndLoads => "files_and_loads",
            Self::Packages => "packages",
            Self::Xml => "xml",
            Self::Mixins => "mixins",
            Self::Functions => "functions",
            Self::Calls => "calls",
            Self::Scripts => "scripts",
            Self::State => "state",
            Self::TocFacts => "toc_facts",
            Self::XmlFacts => "xml_facts",
            Self::CollectorEncoding => "collector_encoding",
            Self::DirectPreparation => "direct_preparation",
            Self::InventoryStage => "inventory_stage",
            Self::TocLoadStage => "toc_load_stage",
            Self::AnalyzerStructureStage => "analyzer_structure_stage",
            Self::XmlStructureStage => "xml_structure_stage",
            Self::DirectSupport => "direct_support",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    NotEvaluated,
    Completed,
    Refused,
    Cancelled,
    Failed,
}

#[derive(Debug, Serialize)]
struct Observation {
    phase: &'static str,
    outcome: Outcome,
    elapsed_microseconds: Option<u64>,
    counter_before_bytes: Option<u64>,
    counter_after_bytes: Option<u64>,
    configured_limit_bytes: Option<u64>,
    counting_method: Option<CountingMethod>,
    /// Complete available input population, not processed or accepted records.
    available_input_records: Option<u64>,
    completed_records: Option<u64>,
    metadata_refusal: Option<super::producer_budget::MetadataRefusal>,
    failure_code: Option<String>,
}
impl Observation {
    const fn unreached(phase: Phase) -> Self {
        Self {
            phase: phase.name(),
            outcome: Outcome::NotEvaluated,
            elapsed_microseconds: None,
            counter_before_bytes: None,
            counter_after_bytes: None,
            configured_limit_bytes: None,
            counting_method: None,
            available_input_records: None,
            completed_records: None,
            metadata_refusal: None,
            failure_code: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct InputPopulations {
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    configuration_digest: String,
    main_files: u64,
    selected_packages: Option<u64>,
    functions: Option<u64>,
    calls: Option<u64>,
    global_accesses: Option<u64>,
    /// Complete available report, not completed or accepted collector records.
    function_total_metadata_estimate_bytes: Option<u64>,
    /// Excludes conditional Main declaration charges from call arguments.
    call_base_total_metadata_estimate_bytes: Option<u64>,
    callable_base_total_metadata_estimate_bytes: Option<u64>,
    callable_charge_counting_method: CountingMethod,
}

/// Optional bounded sidecar. Times and counters never enter semantic identities.
/// Output contains only fixed labels, exact digests/IDs and numeric observations.
#[derive(Debug, Serialize, Default)]
pub struct SourceGraphDiagnostics {
    #[serde(skip)]
    enabled: bool,
    input: Option<InputPopulations>,
    phases: Vec<Observation>,
}
impl SourceGraphDiagnostics {
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            input: None,
            phases: Phase::ALL.into_iter().map(Observation::unreached).collect(),
        }
    }

    /// Begin another observation without carrying measurements across operations.
    pub fn reset(&mut self) {
        self.input = None;
        for (observation, phase) in self.phases.iter_mut().zip(Phase::ALL) {
            *observation = Observation::unreached(phase);
        }
    }

    pub(super) fn bind(&mut self, project: &ProjectView, stop: &AtomicBool) -> ProjectResult<()> {
        if !self.enabled {
            return Ok(());
        }
        self.reset();
        let report = project.snapshot().analyzer_binding().function_call_report();
        self.input = Some(InputPopulations {
            project_snapshot_id: project.snapshot_id().into(),
            analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
            configuration_digest: project.configuration().configuration_digest().to_string(),
            main_files: project.file_manifest().len() as u64,
            selected_packages: project
                .configuration()
                .package_load_plan()
                .map(|plan| plan.packages().len() as u64),
            functions: report.map(|report| report.functions().len() as u64),
            calls: report.map(|report| report.calls().len() as u64),
            global_accesses: report.map(|report| report.global_accesses().len() as u64),
            function_total_metadata_estimate_bytes: None,
            call_base_total_metadata_estimate_bytes: None,
            callable_base_total_metadata_estimate_bytes: None,
            callable_charge_counting_method: CountingMethod::ConservativeEstimate,
        });
        let Some(report) = report else {
            return Ok(());
        };
        let input = self.input.as_mut().ok_or_else(super::invalid)?;
        let functions = total_charge(
            report.functions().iter().map(|fact| fact.path()),
            super::functions::function_metadata_charge_bytes,
            stop,
        )?;
        input.function_total_metadata_estimate_bytes = Some(functions);
        let calls = total_charge(
            report.calls().iter().map(|call| call.path()),
            super::functions::call_base_metadata_charge_bytes,
            stop,
        )?;
        input.call_base_total_metadata_estimate_bytes = Some(calls);
        input.callable_base_total_metadata_estimate_bytes =
            Some(functions.checked_add(calls).ok_or_else(estimate_overflow)?);
        crate::analyzer::checkpoint(stop)?;
        Ok(())
    }

    pub(super) fn measure<T>(
        &mut self,
        phase: Phase,
        budget: &mut ProducerBudget,
        records: Option<usize>,
        method: CountingMethod,
        operation: impl FnOnce(&mut ProducerBudget) -> ProjectResult<T>,
    ) -> ProjectResult<T> {
        if !self.enabled {
            return operation(budget);
        }
        let before = budget.used();
        let started = Instant::now();
        let result = operation(budget);
        self.phases[phase as usize] = Observation {
            phase: phase.name(),
            outcome: match &result {
                Ok(_) => Outcome::Completed,
                Err(error)
                    if matches!(
                        error.code(),
                        ProjectErrorCode::AnalysisCancelled | ProjectErrorCode::SourceReadCancelled
                    ) =>
                {
                    Outcome::Cancelled
                }
                Err(error) if error.code() == ProjectErrorCode::SourceBudgetExceeded => {
                    Outcome::Refused
                }
                Err(_) => Outcome::Failed,
            },
            elapsed_microseconds: Some(
                u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            ),
            counter_before_bytes: Some(before as u64),
            counter_after_bytes: Some(budget.used() as u64),
            configured_limit_bytes: Some(budget.limit() as u64),
            counting_method: Some(method),
            available_input_records: records.map(|count| count as u64),
            completed_records: None,
            metadata_refusal: budget.refusal(),
            failure_code: result
                .as_ref()
                .err()
                .map(|error| format!("{:?}", error.code())),
        };
        result
    }

    pub(super) fn completed_records(&mut self, phase: Phase, count: usize) {
        if self.enabled {
            self.phases[phase as usize].completed_records = Some(count as u64);
        }
    }
}

// Observe the same per-row charges without reserving, resetting or discounting
// the collector ledger. Publish only complete sums; a cancelled prefix stays null.
fn total_charge<'a>(
    paths: impl Iterator<Item = &'a str>,
    charge_bytes: fn(&str) -> usize,
    stop: &AtomicBool,
) -> ProjectResult<u64> {
    let mut total = 0_u64;
    for path in paths {
        crate::analyzer::checkpoint(stop)?;
        let bytes = u64::try_from(charge_bytes(path)).map_err(|_| estimate_overflow())?;
        total = total.checked_add(bytes).ok_or_else(estimate_overflow)?;
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(total)
}

fn estimate_overflow() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        "source graph diagnostic metadata estimate exceeds u64",
    )
}

/// Capture the actual local counter before the caller propagates either result.
/// The observation is a copy only; it never commits or discounts scratch charges.
pub(super) fn capture_scratch<T>(
    observation: &mut ProducerBudget,
    scratch: &mut ProducerBudget,
    operation: impl FnOnce(&mut ProducerBudget) -> ProjectResult<T>,
) -> ProjectResult<T> {
    let result = operation(scratch);
    *observation = *scratch;
    result
}
