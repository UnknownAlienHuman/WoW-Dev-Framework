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
fn fuel_and_memory_growth_are_bounded() -> TestResult<()> {
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
    let grow = module(1, "i32.const 100 memory.grow drop i32.const 0", 0, 33, "")?;
    assert!(load(&grow)?.render(&request()).is_err());
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
