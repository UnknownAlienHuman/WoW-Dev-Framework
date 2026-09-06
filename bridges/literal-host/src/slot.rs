use crate::{BridgeError, LiteralBridge, LiteralError, ModuleHandle, Receipt, Request, Result};
use std::sync::RwLock;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    epoch: u64,
    module_sha256: String,
}
impl Selection {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn module_sha256(&self) -> &str {
        &self.module_sha256
    }
}
struct Current {
    selection: Selection,
    module: ModuleHandle,
}
/// In-memory CAS selection. Persistence/distribution are separate owner ports.
pub struct ModuleSlot(RwLock<Current>);
#[derive(Clone)]
pub struct Snapshot {
    selection: Selection,
    module: ModuleHandle,
}
impl ModuleSlot {
    pub fn new(module: ModuleHandle) -> Self {
        Self(RwLock::new(Current {
            selection: Selection {
                epoch: 0,
                module_sha256: module.digest().into(),
            },
            module,
        }))
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let state = self.0.read().map_err(|_| BridgeError::Poisoned)?;
        Ok(Snapshot {
            selection: state.selection.clone(),
            module: state.module.clone(),
        })
    }
    /// Caller preflights a module without holding this lock. Only new snapshots
    /// see the replacement; existing snapshots retain their exact generation.
    /// Re-selecting a retained old module is rollback, with a fresh epoch (no ABA).
    pub fn replace(&self, expected: &Selection, module: ModuleHandle) -> Result<Selection> {
        let mut state = self.0.write().map_err(|_| BridgeError::Poisoned)?;
        if &state.selection != expected {
            return Err(BridgeError::StaleSelection);
        }
        let selection = Selection {
            epoch: state
                .selection
                .epoch
                .checked_add(1)
                .ok_or(BridgeError::EpochExhausted)?,
            module_sha256: module.digest().into(),
        };
        *state = Current {
            selection: selection.clone(),
            module,
        };
        Ok(selection)
    }
}
impl Snapshot {
    pub fn limits(&self) -> crate::Limits {
        self.module.limits()
    }
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    pub fn render(&self, request: &Request) -> Result<Receipt> {
        self.module.render(request)
    }
}
impl LiteralBridge for Snapshot {
    fn selected_module(&self) -> Option<crate::SelectedModule> {
        Some(crate::SelectedModule {
            sha256: self.selection.module_sha256.clone(),
            epoch: self.selection.epoch,
        })
    }
    fn render(&self, request: &Request) -> std::result::Result<String, LiteralError> {
        Snapshot::render(self, request)
            .map(|receipt| receipt.text)
            .map_err(|error| match error {
                BridgeError::Input(error) | BridgeError::Render(error) => error,
                _ => LiteralError::BridgeFailure,
            })
    }
}
