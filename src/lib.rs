#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod decode;
pub(crate) mod execute;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod module;
pub(crate) mod reader;
pub(crate) mod vector;

pub mod components;
pub mod instructions;

pub use decode::DecodeError;
pub use execute::{ExecuteError, ModuleInstance, ModuleInstanceOptions, Value};
pub use module::Module;
#[cfg(feature = "std")]
pub use vector::{StdVector, StdVectorFactory};
pub use vector::{Vector, VectorFactory};

pub const PAGE_SIZE: usize = 65536;
