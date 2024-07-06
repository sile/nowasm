// TODO
//#![no_std]

mod decode_error;

pub(crate) mod instructions;
#[cfg(feature = "sign_extension")]
pub(crate) mod instructions_sign_extension;
pub(crate) mod reader;
pub mod symbols; // TODO: pub (crate) components
pub(crate) mod vectors;

pub mod execution; // TODO
pub mod module; // TODO: pub (crate)
pub mod sections;
pub mod validation; // TODO: pub (crate)

pub use decode_error::DecodeError;
pub use instructions::Instr;
pub use module::Module;
pub use symbols::{FuncType, Locals};
pub use vectors::{
    AllocateVector, Counters, FixedSizeMutVector, FixedSizeMutVectors, Vector, VectorSlice, Vectors,
};

pub const PAGE_SIZE: u32 = 65536;
