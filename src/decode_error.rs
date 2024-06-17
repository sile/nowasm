use crate::symbols::SectionId;
use core::str::Utf8Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic { value: [u8; 4] },
    InvalidVersion { value: [u8; 4] },
    InvalidSectionId { value: u8 },
    InvalidImportDescTag { value: u8 },
    InvalidLimitsTag { value: u8 },
    InvalidValType { value: u8 },
    InvalidMutabilityFlag { value: u8 },
    InvalidElemType { value: u8 },
    InvalidFuncTypeTag { value: u8 },
    InvalidMemoryCount { value: usize },
    InvalidUtf8(Utf8Error),
    MalformedInteger,
    OutOfOrderSectionId { last: SectionId, current: SectionId },
    FullBytes,
}
