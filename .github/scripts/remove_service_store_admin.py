from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_exact(path: str, old: str, new: str, count: int = 1) -> None:
    text = read(path)
    found = text.count(old)
    if found != count:
        raise SystemExit(f"{path}: expected {count}, found {found}: {old[:100]!r}")
    write(path, text.replace(old, new))


replace_exact(
    "crates/wow-service/src/live_project.rs",
    '''mod quarantine;
pub use quarantine::{LiveProjectQuarantineInspection, QuarantinedLiveProject};
''',
    "",
)
replace_exact(
    "crates/wow-service/src/live_project.rs",
    '''pub use recovery::{
    CurrentState, RecoveryReport, ScopeState, recover_live_project, restore_live_project_to_new,
};
''',
    '''pub use recovery::{CurrentState, RecoveryReport, ScopeState, recover_live_project};
''',
)

write(
    "crates/wow-service/src/live_project/recovery.rs",
    '''use super::{LiveProjectStore, store_error};
use crate::ServiceResult;
use std::{path::Path, sync::atomic::AtomicBool};
pub use wow_store::project::{CurrentState, RecoveryReport, ScopeState};

impl LiveProjectStore {
    /// Observe physical seals, membership and receipts on one held snapshot.
    /// This does not authorize repair, replacement or another storage lifecycle.
    pub fn recovery_report(&self, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
        self.store.recovery_report(stop).map_err(store_error)
    }
}

/// Open only an exactly registered live-project profile, then observe it once.
pub fn recover_live_project(root: &Path, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
    LiveProjectStore::open(root)?.recovery_report(stop)
}
''',
)

for declaration in [
    '#[path = "quarantine_tests.rs"]\nmod quarantine;\n',
    '#[path = "recovery_tests.rs"]\nmod recovery_tests;\n',
    '#[path = "replacement_tests.rs"]\nmod replacement_tests;\n',
]:
    replace_exact("crates/wow-service/src/live_project/tests.rs", declaration, "")

live_doc = "apps/wow/LIVE_PROJECT.md"
replace_exact(
    live_doc,
    '''service APIs; this CLI does not dispatch restore. Older readers and semantic IDs
survive; guarded restore from quarantine, schema migration and full acceptance remain open. See
[PROJECT_RECOVERY.md](../../crates/wow-store/PROJECT_RECOVERY.md).
''',
    '''service APIs. The CLI does not dispatch backup, replacement or quarantine
administration; `recover` only observes the current registered store. Older readers
and semantic IDs survive. See [PROJECT_RECOVERY.md](../../crates/wow-store/PROJECT_RECOVERY.md).
''',
)
