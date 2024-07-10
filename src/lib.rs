#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod decode;
pub(crate) mod execute;
pub(crate) mod instance;
pub(crate) mod module;
pub(crate) mod reader;
#[cfg(feature = "sign_extension")]
pub(crate) mod sign_extension;
pub(crate) mod vector;

pub mod components;
pub mod instructions;

pub use decode::DecodeError;
pub use execute::{ExecuteError, GlobalVal, Val};
pub use instance::{HostFunc, ModuleInstance, Resolve};
pub use module::Module;
#[cfg(feature = "std")]
pub use vector::{StdVector, StdVectorFactory};
pub use vector::{Vector, VectorFactory};

pub const PAGE_SIZE: usize = 65536;
