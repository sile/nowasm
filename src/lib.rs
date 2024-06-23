#![no_std]

mod decode_error;

pub(crate) mod instructions;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub mod symbols; // TODO: pub (crate) components
pub(crate) mod vectors;

pub mod decode; // TODO: delete
pub mod module; // TODO: pub (crate)
pub mod sections;

pub use decode_error::DecodeError;
pub use module::Module;
pub use vectors::{Counters, VectorSlice, Vectors};
