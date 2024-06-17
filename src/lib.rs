#![no_std]

mod decode_error;
mod module_spec;

pub(crate) mod reader;
pub(crate) mod symbols;
pub(crate) mod vectors;

pub mod module;

pub use decode_error::DecodeError;
pub use module_spec::ModuleSpec;
