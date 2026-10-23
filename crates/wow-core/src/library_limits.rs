//! Shared acquisition ceilings for generated or existing Lua annotation libraries.
//! Main source and analysis-output budgets are separate.

pub const MAX_FILES: usize = 4096;
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;
