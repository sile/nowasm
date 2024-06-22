#![no_std]

mod decode_error;
mod module_spec;

pub(crate) mod instructions;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub mod symbols; // TODO: pub (crate) components
pub(crate) mod vectors;

pub mod decode;
pub mod module; // TODO: pub (crate)

pub use decode_error::DecodeError;
pub use module_spec::ModuleSpec;
pub use vectors::{VectorSlice, Vectors};
