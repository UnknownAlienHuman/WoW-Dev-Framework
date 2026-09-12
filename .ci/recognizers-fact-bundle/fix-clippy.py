from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one target, found {count}")
    return text.replace(old, new, 1)


facts_path = Path("crates/wow-recognizers/src/facts.rs")
facts = facts_path.read_text(encoding="utf-8")

coverage_marker = """#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactCoverage {
"""
coverage_with_input = """#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognizerFactCoverageInput {
    pub context_id: GenerationContextId,
    pub partition_id: Box<str>,
    pub capability_id: Box<str>,
    pub producer_id: Box<str>,
    pub producer_version: Box<str>,
    pub state: RecognizerFactCoverageState,
    pub blocker_ids: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactCoverage {
"""
facts = replace_once(facts, coverage_marker, coverage_with_input, "coverage input insertion")

old_constructor = """impl RecognizerFactCoverage {
    pub fn new(
        context_id: GenerationContextId,
        partition_id: impl Into<Box<str>>,
        capability_id: impl Into<Box<str>>,
        producer_id: impl Into<Box<str>>,
        producer_version: impl Into<Box<str>>,
        state: RecognizerFactCoverageState,
        blocker_ids: Vec<Box<str>>,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let partition_id = partition_id.into();
        let capability_id = capability_id.into();
        let producer_id = producer_id.into();
        let producer_version = producer_version.into();
"""
new_constructor = """impl RecognizerFactCoverage {
    pub fn new(
        input: RecognizerFactCoverageInput,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let RecognizerFactCoverageInput {
            context_id,
            partition_id,
            capability_id,
            producer_id,
            producer_version,
            state,
            blocker_ids,
        } = input;
"""
facts = replace_once(facts, old_constructor, new_constructor, "coverage constructor")

old_validate = """        let rebuilt = Self::new(
            self.context_id,
            self.partition_id.clone(),
            self.capability_id.clone(),
            self.producer_id.clone(),
            self.producer_version.clone(),
            self.state,
            self.blocker_ids.clone(),
            limits,
        )?;
"""
new_validate = """        let rebuilt = Self::new(
            RecognizerFactCoverageInput {
                context_id: self.context_id,
                partition_id: self.partition_id.clone(),
                capability_id: self.capability_id.clone(),
                producer_id: self.producer_id.clone(),
                producer_version: self.producer_version.clone(),
                state: self.state,
                blocker_ids: self.blocker_ids.clone(),
            },
            limits,
        )?;
"""
facts = replace_once(facts, old_validate, new_validate, "coverage validation rebuild")

old_fact_id_call = """        let fact_id = derive_fact_id(
            context_id,
            &kind,
            &partition_id,
            &scope,
            &producer_id,
            &producer_version,
            confidence,
            &fields,
        )?;
"""
new_fact_id_call = """        let identity = FactIdentity {
            schema: "wow-recognizers/fact/e2-b/1",
            context_id,
            kind: &kind,
            partition_id: &partition_id,
            scope: &scope,
            producer_id: &producer_id,
            producer_version: &producer_version,
            confidence,
            fields: &fields,
        };
        let fact_id = derive_fact_id(&identity)?;
"""
facts = replace_once(facts, old_fact_id_call, new_fact_id_call, "fact identity call")

old_fact_id_function = """fn derive_fact_id(
    context_id: GenerationContextId,
    kind: &str,
    partition_id: &str,
    scope: &RecognizerFactScope,
    producer_id: &str,
    producer_version: &str,
    confidence: GraphConfidence,
    fields: &BTreeMap<Box<str>, RecognizerFactValue>,
) -> RecognizerResult<RecognizerFactId> {
    let bytes = canonical_json_bytes(&FactIdentity {
        schema: "wow-recognizers/fact/e2-b/1",
        context_id,
        kind,
        partition_id,
        scope,
        producer_id,
        producer_version,
        confidence,
        fields,
    })
"""
new_fact_id_function = """fn derive_fact_id(identity: &FactIdentity<'_>) -> RecognizerResult<RecognizerFactId> {
    let bytes = canonical_json_bytes(identity)
"""
facts = replace_once(facts, old_fact_id_function, new_fact_id_function, "fact identity helper")

facts = replace_once(
    facts,
    """    #[must_use]
    pub fn facts_by_kind<'a>(
""",
    """    pub fn facts_by_kind<'a>(
""",
    "facts_by_kind must_use",
)

facts_path.write_text(facts, encoding="utf-8")

lib_path = Path("crates/wow-recognizers/src/lib.rs")
lib = lib_path.read_text(encoding="utf-8")
lib = replace_once(
    lib,
    """    RecognizerFactCoverage, RecognizerFactCoverageState, RecognizerFactInput,
""",
    """    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput,
""",
    "fact coverage input export",
)
lib_path.write_text(lib, encoding="utf-8")

test_path = Path("crates/wow-recognizers/tests/fact_bundle.rs")
test = test_path.read_text(encoding="utf-8")
test = replace_once(
    test,
    """    RecognizerErrorCode, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageState, RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope,
""",
    """    RecognizerErrorCode, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope,
""",
    "fact bundle test import",
)
old_test_constructor = """    Ok(RecognizerFactCoverage::new(
        context.context_id(),
        partition,
        capability,
        "wow.emmy",
        "e0-c.1",
        state,
        blocker.into_iter().map(Into::into).collect(),
        RecognizerFactLimits::default(),
    )?)
"""
new_test_constructor = """    Ok(RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: context.context_id(),
            partition_id: partition.into(),
            capability_id: capability.into(),
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            state,
            blocker_ids: blocker.into_iter().map(Into::into).collect(),
        },
        RecognizerFactLimits::default(),
    )?)
"""
test = replace_once(test, old_test_constructor, new_test_constructor, "coverage test helper")
test_path.write_text(test, encoding="utf-8")
