#![forbid(unsafe_code)]

//! Annotation generation implemented in Rust, using Ketho as implementation donor.
//!
//! The first executable slice is the pure [`ketho`] renderer. It consumes explicitly
//! selected, already resolved declaration data. It does not claim ReferenceView
//! coverage, acquire source, execute Lua, or configure a language server.

pub mod ketho;

pub mod literals;

pub mod native;
