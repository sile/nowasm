use crate::symbols::SectionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic { value: [u8; 4] },
    InvalidVersion { value: [u8; 4] },
    InvalidSectionId { value: u8 },
    MalformedInteger,
    OutOfOrderSectionId { last: SectionId, current: SectionId },
}
