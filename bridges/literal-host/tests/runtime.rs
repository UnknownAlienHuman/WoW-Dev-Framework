type TestResult<T> = std::result::Result<T, Box<dyn Error>>;
use std::error::Error;
use wow_literal_host::*;
fn request() -> Request {
    Request {
        schema: SCHEMA,
        max_output_bytes: 1024,
        input: LiteralInput::CVars(vec!["test".into()]),
    }
}
fn module(abi: i32, render: &str, pointer: i32, length: usize, extra: &str) -> TestResult<Vec<u8>> {
    Ok(wat::parse_str(format!(
        r#"(module
      {extra}
      (memory (export "memory") 1 2)
      (func (export "wow_abi_version") (result i32) i32.const {abi})
      (func (export "wow_request_buffer") (param i32) (result i32) i32.const 4096)
      (func (export "wow_render") (result i32) {render})
      (func (export "wow_response_ptr") (result i32) i32.const {pointer})
      (func (export "wow_response_len") (result i32) i32.const {length})
      (data (i32.const 0) "{{\22schema\22:1,\22result\22:{{\22Ok\22:\22ok\22}}}}")
    )"#
    ))?)
}
fn load(bytes: &[u8]) -> wow_literal_host::Result<ModuleHandle> {
    ModuleHandle::load(bytes, &module_digest(bytes), Limits::default())
}
#[test]
fn malformed_digest_imports_start_and_abi_are_rejected() -> TestResult<()> {
    let valid = module(1, "i32.const 0", 0, 33, "")?;
    assert!(matches!(
        ModuleHandle::load(&valid, "sha256:bad", Limits::default()),
        Err(BridgeError::DigestMismatch)
    ));
    assert!(load(b"text").is_err());
    assert!(load(&module(2, "i32.const 0", 0, 33, "")?).is_err());
    assert!(
        load(&module(
            1,
            "i32.const 0",
            0,
            33,
            r#"(import "wasi_snapshot_preview1" "random_get" (func))"#
        )?)
        .is_err()
    );
    assert!(
        load(&module(
            1,
            "i32.const 0",
            0,
            33,
            "(func $start) (start $start)"
        )?)
        .is_err()
    );
    assert!(
        ModuleHandle::load(
            &valid,
            &module_digest(&valid),
            Limits {
                fuel: 0,
                ..Limits::default()
            }
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn fuel_is_bounded() -> TestResult<()> {
    let infinite = module(1, "(loop $forever br $forever) i32.const 0", 0, 33, "")?;
    let limited = ModuleHandle::load(
        &infinite,
        &module_digest(&infinite),
        Limits {
            fuel: 1000,
            ..Limits::default()
        },
    )?;
    assert!(matches!(
        limited.render(&request()),
        Err(BridgeError::ExecutionFailed)
    ));
    Ok(())
}
#[test]
fn output_ranges_wire_and_requested_limits_are_verified() -> TestResult<()> {
    let response_len = r#"{"schema":1,"result":{"Ok":"ok"}}"#.len();
    let valid = load(&module(1, "i32.const 0", 0, response_len, "")?)?;
    assert_eq!(valid.render(&request())?.text, "ok");
    for (pointer, length) in [(-1, 1), (65535, 100), (0, 32 * 1024 * 1024), (0, 2)] {
        assert!(
            load(&module(1, "i32.const 0", pointer, length, "")?)?
                .render(&request())
                .is_err()
        );
    }
    let mut tiny = request();
    tiny.max_output_bytes = 1;
    assert!(valid.render(&tiny).is_err());
    Ok(())
}
#[test]
fn snapshots_rollback_and_stale_cas_do_not_change_old_operations() -> TestResult<()> {
    let length = r#"{"schema":1,"result":{"Ok":"ok"}}"#.len();
    let a = load(&module(1, "i32.const 0", 0, length, "")?)?;
    let b = load(&module(1, "i32.const 0 nop", 0, length, "")?)?;
    assert_ne!(a.digest(), b.digest());
    let slot = ModuleSlot::new(a.clone());
    let old = slot.snapshot()?;
    let second = slot.replace(old.selection(), b.clone())?;
    assert_eq!(old.render(&request())?.module_sha256, a.digest());
    assert_eq!(
        slot.snapshot()?.render(&request())?.module_sha256,
        b.digest()
    );
    assert_eq!(
        slot.replace(old.selection(), a.clone()),
        Err(BridgeError::StaleSelection)
    );
    slot.replace(&second, a.clone())?;
    assert_eq!(
        slot.snapshot()?.render(&request())?.module_sha256,
        a.digest()
    );
    assert_eq!(
        slot.replace(old.selection(), b),
        Err(BridgeError::StaleSelection)
    );
    Ok(())
}

#[test]
fn host_memory_limit_traps_growth_permitted_by_the_module() -> TestResult<()> {
    // Module: one initial page, two maximum. Grow to two pages so the guest's
    // own maximum does not short-circuit the host's one-page resource limiter.
    let grow = module(1, "i32.const 1 memory.grow drop i32.const 0", 0, 33, "")?;
    let handle = ModuleHandle::load(
        &grow,
        &module_digest(&grow),
        Limits {
            memory_bytes: 65_536,
            ..Limits::default()
        },
    )?;
    assert_eq!(handle.render(&request()), Err(BridgeError::ExecutionFailed));
    // Rejection must not poison the immutable compiled handle or remove limits.
    assert_eq!(handle.render(&request()), Err(BridgeError::ExecutionFailed));
    Ok(())
}

#[test]
fn growth_at_host_limit_succeeds_and_each_request_has_fresh_memory() -> TestResult<()> {
    let grow = module(
        1,
        "i32.const 1 memory.grow i32.const 1 i32.ne if unreachable end
         memory.size i32.const 2 i32.ne if unreachable end i32.const 0",
        0,
        33,
        "",
    )?;
    let handle = ModuleHandle::load(
        &grow,
        &module_digest(&grow),
        Limits {
            memory_bytes: 2 * 65_536,
            ..Limits::default()
        },
    )?;
    // The guest checks both memory.grow's previous-page result and final size.
    // A shared instance would make the second invocation fail these checks.
    let first = handle.render(&request())?;
    assert_eq!(first.text, "ok");
    assert_eq!(handle.render(&request())?, first);
    Ok(())
}

#[test]
fn module_maximum_failure_does_not_allocate_or_require_a_host_trap() -> TestResult<()> {
    // Exceed the module's two-page maximum, not the host's 128 MiB limit.
    // Wasmi can reject this before consulting the host limiter. Assert the
    // Wasm failure value and unchanged memory, rather than mistaking -1 for a
    // successful allocation or requiring a limiter trap that was never reached.
    let grow = module(
        1,
        "i32.const 100 memory.grow i32.const -1 i32.ne if unreachable end
         memory.size i32.const 1 i32.ne if unreachable end i32.const 0",
        0,
        33,
        "",
    )?;
    assert_eq!(load(&grow)?.render(&request())?.text, "ok");
    Ok(())
}

#[test]
fn initial_memory_cannot_exceed_the_host_limit() -> TestResult<()> {
    let bytes = module(1, "i32.const 0", 0, 33, "")?;
    assert!(matches!(
        ModuleHandle::load(
            &bytes,
            &module_digest(&bytes),
            Limits {
                memory_bytes: 65_535,
                ..Limits::default()
            }
        ),
        Err(BridgeError::ExecutionFailed)
    ));
    Ok(())
}
