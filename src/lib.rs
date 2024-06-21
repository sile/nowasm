#![no_std]

mod decode_error;
mod module_spec;

pub(crate) mod instructions;
pub(crate) mod reader;
pub mod symbols; // TODO: pub (crate) components
pub(crate) mod vectors;

pub mod module;

pub use decode_error::DecodeError;
pub use module_spec::ModuleSpec;
