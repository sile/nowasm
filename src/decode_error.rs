use crate::symbols::SectionId;
use core::str::Utf8Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic { value: [u8; 4] },
    InvalidVersion { value: [u8; 4] },
    InvalidSectionId { value: u8 },
    InvalidUtf8(Utf8Error),
    MalformedInteger,
    OutOfOrderSectionId { last: SectionId, current: SectionId },
    FullBytes,
}
