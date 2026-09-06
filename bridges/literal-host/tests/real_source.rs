//! Explicit real-guest source integration. CI must run these ignored tests.
#[path = "support/compare.rs"]
mod comparison;
#[path = "../../../crates/wow-annotations/examples/support/mod.rs"]
mod driver;
#[path = "support/source.rs"]
mod source;
use comparison::compare;
use source::{Fixture, TestResult};
use std::{cell::Cell, fs};
use wow_literal_host::*;

struct SwapDuringBuild<'a> {
    slot: &'a ModuleSlot,
    retained: Snapshot,
    next: ModuleHandle,
    swapped: Cell<bool>,
}
impl LiteralBridge for SwapDuringBuild<'_> {
    fn selected_module(&self) -> Option<SelectedModule> {
        self.retained.selected_module()
    }
    fn render(&self, request: &Request) -> std::result::Result<String, LiteralError> {
        let result = LiteralBridge::render(&self.retained, request);
        if !self.swapped.replace(true) {
            self.slot
                .replace(self.retained.selection(), self.next.clone())
                .map_err(|_| LiteralError::BridgeFailure)?;
        }
        result
    }
}
fn guests() -> TestResult<(ModuleHandle, ModuleHandle)> {
    let a = fs::read(std::env::var("WDF_WASM_A")?)?;
    let b = fs::read(std::env::var("WDF_WASM_B")?)?;
    assert_ne!(module_digest(&a), module_digest(&b));
    Ok((
        ModuleHandle::load(&a, &module_digest(&a), Limits::default())?,
        ModuleHandle::load(&b, &module_digest(&b), Limits::default())?,
    ))
}
#[test]
#[ignore = "requires two real compiled Rust guests; mandatory in Wasm CI"]
fn full_source_build_retains_module_during_swap_and_rollback() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.corrections()?;
    assert!(!driver::run(fixture.args("native"), None)?);
    let native = fixture.report("native")?;
    assert_eq!(
        native["library"]["corrections"]["applications"][0]["status"],
        "applied"
    );
    let (a, b) = guests()?;
    let slot = ModuleSlot::new(a.clone());
    let old = slot.snapshot()?;
    let during = SwapDuringBuild {
        slot: &slot,
        retained: old.clone(),
        next: b.clone(),
        swapped: Cell::new(false),
    };
    assert!(!driver::run(fixture.args("during-swap"), Some(&during))?);
    assert!(during.swapped.get());
    assert_eq!(slot.snapshot()?.selection().module_sha256(), b.digest());
    let current = slot.snapshot()?;
    let rolled = slot.replace(current.selection(), a.clone())?;
    for (output, snapshot, expected, epoch) in [
        ("retained-a", old, a.digest(), 0),
        ("retained-b", current, b.digest(), 1),
        ("rollback", slot.snapshot()?, a.digest(), 2),
    ] {
        assert_eq!(snapshot.selection().epoch(), epoch);
        assert!(!driver::run(fixture.args(output), Some(&snapshot))?);
        let report = fixture.report(output)?;
        assert_eq!(
            report["library"]["literal_execution"]["module"]["sha256"],
            expected
        );
        assert_eq!(
            report["library"]["literal_execution"]["module"]["epoch"],
            epoch
        );
        compare(&native, &report)?;
        assert!(driver::run(fixture.args(output), Some(&snapshot)).is_err());
    }
    assert_eq!(rolled.epoch(), 2);
    let report = fixture.report("during-swap")?;
    compare(&native, &report)?;
    assert_eq!(
        report["library"]["literal_execution"]["module"]["sha256"],
        a.digest()
    );
    assert!(
        report["library"]["literal_execution"]["calls"]
            .as_array()
            .ok_or("missing calls")?
            .len()
            > 2
    );
    println!(
        "source, aliases, final bytes and maps retained across A->B->A; a={} b={}",
        a.digest(),
        b.digest()
    );
    Ok(())
}
#[test]
#[ignore = "requires real Rust guest; mandatory in Wasm CI"]
fn guest_execution_failure_creates_no_output_and_has_no_native_fallback() -> TestResult {
    let fixture = Fixture::new()?;
    let bytes = fs::read(std::env::var("WDF_WASM_A")?)?;
    let module = ModuleHandle::load(
        &bytes,
        &module_digest(&bytes),
        Limits {
            fuel: 1000,
            ..Limits::default()
        },
    )?;
    let slot = ModuleSlot::new(module);
    assert!(driver::run(fixture.args("failed"), Some(&slot.snapshot()?)).is_err());
    assert!(!fixture.0.join("failed").exists());
    assert!(!driver::run(fixture.args("native"), None)?);
    Ok(())
}
