// TODO
//#![no_std]

mod decode;

pub(crate) mod instructions;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub mod symbols; // TODO: pub (crate) components
pub(crate) mod vectors; // TODO: rename

pub mod execution; // TODO
pub mod module; // TODO: pub (crate)
pub mod sections;
pub mod validation; // TODO: pub (crate)

pub use decode::DecodeError;
pub use instructions::Instr;
pub use module::Module;
pub use symbols::FuncType;
pub use vectors::{Allocator, DecodeVector, Vector};

pub const PAGE_SIZE: u32 = 65536;
