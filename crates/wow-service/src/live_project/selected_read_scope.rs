//! Existing selected namespace acquisition with reverse checked read/store close.
use super::{
    LiveProjectRead, LiveProjectStore, PlatformStoreSelection, ReadFailureClosure, fail,
    publication_checkpoint,
};
use crate::{ServiceErrorCode, ServiceResult};
use std::{cell::Cell, path::Path, sync::atomic::AtomicBool};
use wow_store::project::ReadSelector;

/// One selected read, one private draft, no owner or lease escaping delivery.
pub(crate) fn with_checked_selected_read<T>(
    root: &Path,
    selection: &PlatformStoreSelection,
    selector: &ReadSelector,
    stop: &AtomicBool,
    build: impl FnOnce(&LiveProjectRead) -> ServiceResult<T>,
) -> (ServiceResult<T>, &'static str) {
    let closure = Cell::new("not_acquired");
    let outcome = (|| {
        publication_checkpoint(stop)?;
        let store = match LiveProjectStore::open_in_namespace_checked(root, selection) {
            Ok(store) => store,
            Err(error) => {
                closure.set(closure_name(error.closure));
                return Err(error.error);
            }
        };
        closure.set("pending");
        let draft = match store.read_in_namespace_checked(selection, selector, stop) {
            Ok(read) => {
                let draft = build(&read);
                match read.close() {
                    Ok(()) => draft,
                    Err(_) => {
                        closure.set("failed");
                        Err(fail(ServiceErrorCode::InternalContractViolation))
                    }
                }
            }
            Err(error) => {
                closure.set(closure_name(error.closure));
                Err(error.error)
            }
        };
        finish_store(store, draft, &closure, stop)
    })();
    (outcome, closure.get())
}

// This private finish is also exercised with a real native live read lease;
// there is no alternate owner, injected close result or production fault hook.
pub(super) fn finish_store<T>(
    store: LiveProjectStore,
    draft: ServiceResult<T>,
    closure: &Cell<&'static str>,
    stop: &AtomicBool,
) -> ServiceResult<T> {
    if store.close().is_err() {
        closure.set("failed");
        return Err(fail(ServiceErrorCode::InternalContractViolation));
    }
    // A prior read close failure outranks a later cancellation too. Closing the
    // store must still be attempted before returning that failure.
    if closure.get() == "failed" {
        return Err(fail(ServiceErrorCode::InternalContractViolation));
    }
    if closure.get() == "pending" {
        closure.set("closed");
    }
    publication_checkpoint(stop)?;
    draft
}

fn closure_name(closure: ReadFailureClosure) -> &'static str {
    match closure {
        ReadFailureClosure::Closed => "closed",
        ReadFailureClosure::Failed => "failed",
        ReadFailureClosure::NotEvaluated => "not_evaluated",
    }
}
