#![forbid(unsafe_code)]

//! Explicit, deterministic workspace and analyzer-adapter boundaries for WoW Lua.
//!
//! `wow-emmy` never discovers a current directory, addon, client, Git checkout,
//! profile, or source generation implicitly. Lua semantic correctness is owned
//! by the single EmmyLua adapter; this crate does not contain a second parser.

pub mod workspace;

pub use workspace::{
    EmmyBackendIdentity, EmmyWorkspaceError, EmmyWorkspaceErrorCode, EmmyWorkspaceResult,
    LuaWorkspaceFile, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse,
};
