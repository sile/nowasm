#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod decode;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub(crate) mod vector;

pub mod execution; // TODO: priv
pub(crate) mod module;
pub(crate) mod validate;

pub mod components;
pub mod instructions;

pub use decode::DecodeError;
pub use module::Module;
pub use validate::ValidateError;
pub use vector::{Vector, VectorFactory};

#[cfg(feature = "std")]
pub use vector::{StdVector, StdVectorFactory};

pub const PAGE_SIZE: u32 = 65536;
