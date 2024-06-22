use crate::symbols::SectionId;
use core::str::Utf8Error;

// TODO: impl Display
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic { value: [u8; 4] },
    InvalidVersion { value: [u8; 4] },
    InvalidSectionId { value: u8 },
    InvalidImportDescTag { value: u8 },
    InvalidExportDescTag { value: u8 },
    InvalidLimitsFlag { value: u8 },
    InvalidValType { value: u8 },
    InvalidMutabilityFlag { value: u8 },
    InvalidElemType { value: u8 },
    InvalidFuncTypeTag { value: u8 },
    InvalidMemoryCount { value: usize },
    InvalidMemorySizeMemoryIndex { value: u8 },
    InvalidMemoryGrowMemoryIndex { value: u8 },
    InvalidCallIndirectTableIndex { value: u8 },
    InvalidOpcode { value: u8 },
    InvalidUtf8(Utf8Error),
    MalformedInteger,
    // TODO: InvalidSectionOrder
    OutOfOrderSectionId { last: SectionId, current: SectionId },
    FullBytes,
    FullValTypes,
    FullInstrs,
    FullIdxs,
    FullLocals, // TODO: rename
}
