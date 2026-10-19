from pathlib import Path
import re


def replace(path: str, old: str, new: str, count: int = 1) -> None:
    file = Path(path)
    text = file.read_text()
    actual = text.count(old)
    if actual != count:
        raise SystemExit(f"{path}: expected {count}, found {actual}: {old[:100]!r}")
    file.write_text(text.replace(old, new, count))


service_path = Path("crates/wow-service/src/service.rs")
text = service_path.read_text()
text = text.replace(
    "use crate::configuration::{RESULT_SCHEMA, SERVICE_SCHEMA, SERVICE_VERSION};\n"
    "use crate::identity::canonical_digest;\n",
    "",
)
for field in [
    "    schema: &'static str,\n",
    "    result_id: Box<str>,\n",
    "    request_digest: Box<str>,\n",
    "    service_schema: &'static str,\n",
    "    service_version: &'static str,\n",
    "    configuration_id: Box<str>,\n",
]:
    if text.count(field) != 2:
        raise SystemExit(f"unexpected Service result field count for {field!r}: {text.count(field)}")
    text = text.replace(field, "")
for accessor in [
    "    #[must_use]\n    pub fn result_id(&self) -> &str {\n        &self.result_id\n    }\n",
]:
    if text.count(accessor) != 2:
        raise SystemExit(f"unexpected result_id accessor count: {text.count(accessor)}")
    text = text.replace(accessor, "")
envelope = """#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", content = "result", rename_all = "snake_case")]
pub enum ServiceResultEnvelope {
    Status(StatusResult),
    Check(CheckResult),
}

"""
if text.count(envelope) != 1:
    raise SystemExit("ServiceResultEnvelope block changed")
text = text.replace(envelope, "")
text = text.replace(
    "/// Synchronous E0 service. Every operation binds one canonical request to one\n"
    "/// retained result or fails without publishing a replacement generation.\n",
    "/// Synchronous service for status and checks over one configured backend.\n",
)
status_method = """        let request_digest = canonical_digest(
            "service-request:sha256:",
            &(self.configuration.configuration_id(), request),
        )?;
        self.build_status(request, &request_digest, cancelled)
"""
if text.count(status_method) != 1:
    raise SystemExit("status digest path changed")
text = text.replace(status_method, "        self.build_status(request, cancelled)\n")
check_method = """        let request_digest = canonical_digest(
            "service-request:sha256:",
            &(self.configuration.configuration_id(), request),
        )?;
        self.build_check(request, &request_digest, cancelled)
"""
if text.count(check_method) != 1:
    raise SystemExit("check digest path changed")
text = text.replace(check_method, "        self.build_check(request, cancelled)\n")
text = text.replace(
    "        request: &StatusRequest,\n        request_digest: &str,\n        cancelled: &AtomicBool,\n",
    "        request: &StatusRequest,\n        cancelled: &AtomicBool,\n",
    1,
)
text = text.replace(
    "        request: &CheckRequest,\n        request_digest: &str,\n        cancelled: &AtomicBool,\n",
    "        request: &CheckRequest,\n        cancelled: &AtomicBool,\n",
    1,
)
identity_pattern = re.compile(
    r"\n        #\[derive\(Serialize\)\]\n"
    r"        struct Identity<'a> \{.*?"
    r"        let result_id = canonical_digest\(\"service-result:sha256:\", &identity\)\?;\n",
    re.S,
)
text, count = identity_pattern.subn("\n", text)
if count != 2:
    raise SystemExit(f"expected two self-identity blocks, removed {count}")
for line in [
    "            schema: RESULT_SCHEMA,\n",
    "            result_id,\n",
    "            request_digest: request_digest.into(),\n",
    "            service_schema: SERVICE_SCHEMA,\n",
    "            service_version: SERVICE_VERSION,\n",
    "            configuration_id: self.configuration.configuration_id().into(),\n",
]:
    occurrences = text.count(line)
    if occurrences != 2:
        raise SystemExit(f"unexpected constructor field count for {line!r}: {occurrences}")
    text = text.replace(line, "")
service_path.write_text(text)

replace(
    "crates/wow-service/src/lib.rs",
    "//! identity, conservative status derivation, deterministic presentation, and\n"
    "//! canonical result envelopes. It does not parse source, execute Lua, select a\n",
    "//! identity, conservative status derivation, and deterministic presentation. It\n"
    "//! does not parse source, execute Lua, select a\n",
)
replace(
    "crates/wow-service/src/lib.rs",
    "pub use service::{CheckResult, Service, ServiceResultEnvelope, StatusResult};\n",
    "pub use service::{CheckResult, Service, StatusResult};\n",
)
replace(
    "crates/wow-service/src/configuration.rs",
    "pub(crate) const SERVICE_SCHEMA: &str = \"wow-service/e0-f/1\";\n"
    "pub(crate) const RESULT_SCHEMA: &str = \"wow-service/result-envelope/1\";\n"
    "pub(crate) const SERVICE_VERSION: &str = env!(\"CARGO_PKG_VERSION\");\n\n",
    "",
)
replace(
    "crates/wow-service/tests/e0_service.rs",
    "fn canonical_result_is_invariant_to_owner_input_order() -> TestResult {\n",
    "fn result_order_is_invariant_to_owner_input_order() -> TestResult {\n",
)
replace(
    "crates/wow-service/tests/e0_service.rs",
    "    assert_eq!(left_result.result_id(), right_result.result_id());\n",
    "",
)
replace(
    "crates/wow-service/README.md",
    "- Status/check result construction: `src/service.rs` (`derive_status` and\n"
    "  `CheckResult`); #194 removes the wrapper while keeping useful output.\n",
    "- Status/check result construction: `src/service.rs` (`derive_status`,\n"
    "  `StatusResult`, and `CheckResult`); these are plain typed values.\n",
)
replace(
    "crates/wow-service/README.md",
    "Current typed results still pass through operation registries and legacy result\n"
    "and request-digest machinery. [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194)\n"
    "tracks replacing that path with ordinary typed results and one serialization at\n"
    "the output boundary. Pack administration remains in the tree; native annotation\n",
    "Local status/check results are ordinary typed values serialized once at the\n"
    "application boundary. [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194)\n"
    "continues only through other result families that still self-identify or journal\n"
    "ordinary reads. Pack administration remains in the tree; native annotation\n",
)
