//! Explicit-fixture qualification, never a test-only wire operation.
use super::*;
use wow_service::frontend::lsp::{LspDocumentOpenRequest, LspSessionOpenRequest};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture() -> TestResult<(Server, Arc<LspForegroundControl>, String)> {
    let host = std::env::var_os("WOW_LSP_CONFORMANCE_HOST")
        .ok_or("explicit WOW_LSP_CONFORMANCE_HOST required")?;
    let uri = std::env::var("WOW_LSP_CONFORMANCE_URI")?;
    let stop = AtomicBool::new(false);
    let host = LspHost::load(std::path::Path::new(&host), &stop)?;
    let (mut service, control) = LspFrontend::new_foreground(host)?;
    service.frontend_session_open(
        LspSessionOpenRequest {
            client_name: "owned-actor-conformance".into(),
            client_version: "1".into(),
            position_encodings: vec!["utf-16".into()],
            pull_diagnostics: true,
            workspace_uris: vec![],
        },
        &stop,
    )?;
    service.frontend_document_open(
        LspDocumentOpenRequest {
            uri: uri.clone(),
            language_id: "lua".into(),
            version: 1,
            content: "local value=1\nprint(value)".into(),
        },
        &stop,
    )?;
    Ok((
        Server {
            service,
            initialized: true,
            ready: true,
            shutdown: false,
        },
        Arc::new(control),
        uri,
    ))
}

fn exercise(panic_worker: bool) -> TestResult {
    let (server, control, uri) = fixture()?;
    std::thread::scope(|scope| -> TestResult {
        let (tx, commands) = mpsc::sync_channel(0);
        let (events, rx) = mpsc::sync_channel(1);
        let child_control = Arc::clone(&control);
        let worker = scope.spawn(move || {
            worker::run(
                server,
                child_control,
                commands,
                events,
                Arc::new(AtomicBool::new(false)),
            )
        });
        let prepared = control.prepare(
            LspRequestId::Number(909),
            LspReadRequest::Diagnostics {
                uri,
                previous_result_id: None,
            },
            None,
        )?;
        tx.send(Command::Read(Box::new(prepared)))
            .map_err(|_| "worker read send failed")?;
        let result = match rx.recv_timeout(Duration::from_secs(20))? {
            Event::ReadResult(result) => result,
            _ => return Err("real native read result required".into()),
        };
        assert!(result.result.is_ok());
        let key = result.ticket.ticket_id.clone();
        let wire_id = result.ticket.wire_id.clone();
        let bytes = worker::response(*result).map_err(|_| "native result projection refused")?;
        assert!(bytes.len() > 1);
        let mut driver =
            Driver::new(Arc::clone(&control)).map_err(|_| "driver construction failed")?;
        let now = Instant::now();
        driver.pending.insert(
            key.clone(),
            Pending {
                wire_id,
                prepared: now,
                progressed: now,
                timed_out: false,
            },
        );
        driver.output_bytes = bytes.len();
        // The real pipe test proves partial writes; this component state pins
        // the controller's observed offset before the shutdown interleaving.
        driver.current = Some(Outgoing {
            bytes,
            position: 1,
            chunk: WRITE_CHUNK_BYTES,
            ticket: Some(key.clone()),
            prepared: now,
            progressed: now,
            is_progress: false,
        });
        if panic_worker {
            tx.send(Command::PanicForConformance)
                .map_err(|_| "worker panic command send failed")?;
            let closing = rx.recv_timeout(Duration::from_secs(20))?;
            assert!(matches!(&closing, Event::Closing { code: 4, .. }));
            driver
                .event(closing, &control)
                .map_err(|_| "closing event refused")?;
            drop(driver);
            drop(tx);
            let exit = worker
                .join()
                .map_err(|_| "worker escaped its panic boundary")?;
            assert_eq!(exit.code, 4);
            assert!(exit.session_closed);
        } else {
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                // Reverse local-drop order matches the production controller:
                // classify Driver first, release event send, then close commands.
                let _sender = tx;
                let _receiver = rx;
                let _driver = driver;
                std::panic::resume_unwind(Box::new("controller conformance panic"));
            }));
            assert!(panic.is_err());
            let exit = worker
                .join()
                .map_err(|_| "worker join failed after controller unwind")?;
            assert!(exit.session_closed);
        }
        let receipt = control
            .ticket_metadata(&key)
            .ok_or("delivery receipt not retained")?;
        assert_eq!(
            receipt.delivery,
            LspForegroundDeliveryState::LostAfterPossible
        );
        assert_eq!(
            receipt.cancellation,
            wow_service::frontend::lsp::LspForegroundCancellationState::NotRequested
        );
        Ok(())
    })
}

#[test]
#[ignore = "requires an exact native host and document URI via WOW_LSP_CONFORMANCE_*; run explicitly"]
fn worker_panic_joins_and_closes_real_session_after_partial_final() -> TestResult {
    exercise(true)
}

#[test]
#[ignore = "requires an exact native host and document URI via WOW_LSP_CONFORMANCE_*; run explicitly"]
fn controller_unwind_classifies_partial_final_before_joined_close() -> TestResult {
    exercise(false)
}

#[test]
#[ignore = "requires an exact native host and document URI via WOW_LSP_CONFORMANCE_*; run explicitly"]
fn progress_pressure_preserves_real_final_results_and_records_coalescing() -> TestResult {
    let (mut server, control, uri) = fixture()?;
    let mut driver = Driver::new(Arc::clone(&control)).map_err(|_| "driver construction failed")?;
    let mut last = None;
    for id in 1..=12 {
        let prepared = control.prepare(
            LspRequestId::Number(id),
            LspReadRequest::Diagnostics {
                uri: uri.clone(),
                previous_result_id: None,
            },
            Some(LspProgressToken::String(format!("progress-{id}"))),
        )?;
        let dispatched = match server.service.frontend_dispatch_read(prepared)? {
            wow_service::frontend::lsp::LspReadDispatch::Ready(dispatched) => dispatched,
            _ => return Err("actual owner dispatch required".into()),
        };
        let ticket = dispatched.ticket().clone();
        driver
            .event(
                Event::Progress {
                    ticket: Box::new(ticket.clone()),
                    begin: true,
                },
                &control,
            )
            .map_err(|_| "progress pressure changed admission")?;
        let result = server.service.frontend_finish_read(*dispatched)?;
        assert!(result.result.is_ok());
        driver
            .event(Event::ReadResult(Box::new(result)), &control)
            .map_err(|_| "progress pressure lost a final response")?;
        driver
            .event(
                Event::Progress {
                    ticket: Box::new(ticket.clone()),
                    begin: false,
                },
                &control,
            )
            .map_err(|_| "progress end pressure failed the connection")?;
        last = Some(ticket.ticket_id);
    }
    assert_eq!(driver.output.len(), 12);
    assert!(driver.progress.len() <= LSP_FOREGROUND_EXECUTION.max_progress_events);
    let key = last.ok_or("final ticket required")?;
    let ticket = control
        .ticket_metadata(&key)
        .ok_or("ticket metadata required")?;
    assert_eq!(ticket.progress_events_coalesced, 2);
    assert_eq!(
        ticket.state,
        wow_service::frontend::lsp::LspForegroundTicketState::Completed
    );
    drop(driver);
    server.service.frontend_session_close()?;
    Ok(())
}

#[test]
#[ignore = "requires an exact native host and document URI via WOW_LSP_CONFORMANCE_*; run explicitly"]
fn request_deadline_is_intent_until_dispatch_observes_it() -> TestResult {
    let (mut server, control, uri) = fixture()?;
    let prepared = control.prepare(
        LspRequestId::Number(600),
        LspReadRequest::Diagnostics {
            uri,
            previous_result_id: None,
        },
        None,
    )?;
    let key: Box<str> = prepared.ticket_id().into();
    let mut driver = Driver::new(Arc::clone(&control)).map_err(|_| "driver construction failed")?;
    let old = Instant::now()
        .checked_sub(Duration::from_secs(31))
        .ok_or("monotonic clock range")?;
    driver.pending.insert(
        key.clone(),
        Pending {
            wire_id: LspRequestId::Number(600),
            prepared: old,
            progressed: old,
            timed_out: false,
        },
    );
    driver.expire(&control);
    let requested = control
        .ticket_metadata(&key)
        .ok_or("timeout receipt required")?;
    assert_eq!(
        requested.cancellation,
        wow_service::frontend::lsp::LspForegroundCancellationState::Requested
    );
    assert_eq!(
        requested.cancel_origin,
        Some(wow_service::frontend::lsp::LspForegroundCancelOrigin::RequestDeadline)
    );
    assert!(requested.result_digest.is_none());
    let stopped = match server.service.frontend_dispatch_read(prepared)? {
        wow_service::frontend::lsp::LspReadDispatch::Finished(result) => result,
        _ => return Err("expired queued read invoked its owner".into()),
    };
    assert_eq!(
        stopped.ticket.cancellation,
        wow_service::frontend::lsp::LspForegroundCancellationState::CancelledBeforeDispatch
    );
    assert!(stopped.ticket.overlay.is_none());
    control.record_delivery(&key, LspForegroundDeliveryState::Sent)?;
    driver.pending.clear();
    drop(driver);
    server.service.frontend_session_close()?;
    Ok(())
}
