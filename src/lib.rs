#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod components;
pub(crate) mod decode;
pub(crate) mod instructions;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub(crate) mod vectors; // TODO: rename

pub mod execution; // TODO
pub(crate) mod module;
pub(crate) mod validate;

pub use components::FuncType;
pub use decode::DecodeError;
pub use instructions::Instr;
pub use module::Module;
pub use validate::ValidateError;
pub use vectors::{Allocator, Vector};

#[cfg(feature = "std")]
pub use vectors::{StdAllocator, StdVector};

pub const PAGE_SIZE: u32 = 65536;
