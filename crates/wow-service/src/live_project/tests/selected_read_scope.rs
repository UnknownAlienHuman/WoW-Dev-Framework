use super::*;
use crate::live_project::selected_read_scope::finish_store;
use std::cell::Cell;

#[test]
fn selected_scope_native_close_failure_suppresses_drafts() -> TestResult {
    let running = AtomicBool::new(false);
    let (publisher, graph) = owners("return External()")?;
    for case in ["success", "semantic", "cancelled"] {
        let path = root(&format!("selected-close-{case}"))?;
        let mut store = LiveProjectStore::create(&path, graph.snapshot().universe().as_str())?;
        store.publish(&publisher, &graph, "fixture:selected-close", None, &running)?;
        // A genuine native read keeps the actual Store close from succeeding.
        // No synthetic error result or alternate owner is supplied to the scope.
        let held = store.read(&ReadSelector::Current, &running)?;
        let stop = AtomicBool::new(case == "cancelled");
        let draft = if case == "semantic" {
            Err(fail(ServiceErrorCode::InvalidRequest))
        } else {
            Ok(vec![1, 2, 3])
        };
        let closure = Cell::new("pending");
        let error = finish_store(store, draft, &closure, &stop)
            .err()
            .ok_or("native close failure delivered a draft")?;
        assert_eq!(error.code(), ServiceErrorCode::InternalContractViolation);
        assert_eq!(closure.get(), "failed");
        assert!(!held.publication_set_id().is_empty());
        held.close()?;
        LiveProjectStore::open(&path)?.close()?;
        let resolved = path.canonicalize()?;
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize()?));
        std::fs::remove_dir_all(resolved)?;
    }
    Ok(())
}

#[test]
fn selected_scope_closed_draft_and_cancellation_keep_their_outcomes() -> TestResult {
    let running = AtomicBool::new(false);
    let (publisher, graph) = owners("return External()")?;
    for case in ["success", "semantic", "cancelled"] {
        let path = root(&format!("selected-closed-{case}"))?;
        let mut store = LiveProjectStore::create(&path, graph.snapshot().universe().as_str())?;
        store.publish(
            &publisher,
            &graph,
            "fixture:selected-closed",
            None,
            &running,
        )?;
        store.read(&ReadSelector::Current, &running)?.close()?;
        let closure = Cell::new("pending");
        let draft = if case == "semantic" {
            Err(fail(ServiceErrorCode::InvalidRequest))
        } else {
            Ok(vec![1, 2, 3])
        };
        let result = finish_store(
            store,
            draft,
            &closure,
            &AtomicBool::new(case == "cancelled"),
        );
        assert_eq!(closure.get(), "closed");
        match case {
            "success" => assert_eq!(result?, vec![1, 2, 3]),
            "semantic" => assert_eq!(
                result.err().ok_or("missing refusal")?.code(),
                ServiceErrorCode::InvalidRequest
            ),
            _ => assert_eq!(
                result.err().ok_or("missing cancellation")?.code(),
                ServiceErrorCode::Cancelled
            ),
        }
        LiveProjectStore::open(&path)?.close()?;
        let resolved = path.canonicalize()?;
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize()?));
        std::fs::remove_dir_all(resolved)?;
    }
    Ok(())
}
