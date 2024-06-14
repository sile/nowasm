#[derive(Debug, Clone, Copy)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagicNumber,
    UnsupportedVersion,
    InvalidSectionId { value: u8 },
    TooLargeSectionSize { section_id: SectionId, size: usize },
    TooLargeSize { size: usize },
    MalformedSectiondata,
    MalformedData,
    InvalidMemorySectionSize { size: usize },
    InvalidU32,
    InvalidValueType { value: u8 },
}

#[derive(Debug)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub fn block_reader(&mut self) -> Result<Self, DecodeError> {
        let size = self.read_u32()? as usize;
        if size > self.len() {
            // TODO: Change reason
            return Err(DecodeError::TooLargeSize { size });
        }
        let reader = Self::new(&self.data[self.position..][..size]);
        self.position += size;
        Ok(reader)
    }

    pub fn assert_empty(&self) -> Result<(), DecodeError> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::MalformedData)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if let Some(b) = self.data.get(self.position).copied() {
            self.position += 1;
            Ok(b)
        } else {
            Err(DecodeError::EndOfBytes)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let mut n = 0u32;
        let mut bits = 0;
        loop {
            let b = self.read_u8()?;
            n = n
                .checked_add((b as u32 & 0b0111_1111) << bits)
                .ok_or(DecodeError::InvalidU32)?;
            if b & 0b1000_0000 == 0 {
                break;
            }
            bits += 7;
            if bits >= 32 {
                return Err(DecodeError::InvalidU32);
            }
        }
        Ok(n)
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.len() {
            return Err(DecodeError::EndOfBytes);
        }

        let start = self.position;
        self.position += n;
        Ok(&self.data[start..][..n])
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        let n = buf.len();
        if n > self.len() {
            return Err(DecodeError::EndOfBytes);
        }

        buf.copy_from_slice(&self.data[self.position..][..n]);
        self.position += n;
        Ok(())
    }

    pub fn validate_preamble(&mut self) -> Result<(), DecodeError> {
        let magic = self.read_bytes(4)?;
        if magic != b"\0asm" {
            return Err(DecodeError::InvalidMagicNumber);
        }

        let version = self.read_bytes(4)?;
        if version != b"\x01\0\0\0" {
            return Err(DecodeError::UnsupportedVersion);
        }

        Ok(())
    }

    pub fn read_section_reader(&mut self) -> Result<(SectionId, Self), DecodeError> {
        let section_id = SectionId::new(self.read_u8()?)?;
        let size = self.read_u32()? as usize;
        if size > self.len() {
            return Err(DecodeError::TooLargeSectionSize { section_id, size });
        }

        let reader = Self::new(&self.data[self.position..][..size]);
        self.position += size;

        Ok((section_id, reader))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionId {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl SectionId {
    pub fn new(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(Self::Custom),
            1 => Ok(Self::Type),
            2 => Ok(Self::Import),
            3 => Ok(Self::Function),
            4 => Ok(Self::Table),
            5 => Ok(Self::Memory),
            6 => Ok(Self::Global),
            7 => Ok(Self::Export),
            8 => Ok(Self::Start),
            9 => Ok(Self::Element),
            10 => Ok(Self::Code),
            11 => Ok(Self::Data),
            _ => Err(DecodeError::InvalidSectionId { value }),
        }
    }
}

#[derive(Debug)]
pub struct TypeSection {}

impl TypeSection {
    pub fn decode(reader: &mut ByteReader) -> Result<Self, DecodeError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Type {}

#[derive(Debug)]
pub struct FuncType {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleSpec {
    pub func_type_len: usize,
    pub idx_len: usize,
    pub export_len: usize,
}

impl ModuleSpec {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = ByteReader::new(wasm_bytes);
        reader.validate_preamble()?;

        let mut this = Self {
            func_type_len: 0,
            idx_len: 0,
            export_len: 0,
        };
        while !reader.is_empty() {
            let (section_id, mut section_reader) = reader.read_section_reader()?;
            match section_id {
                SectionId::Custom => {}
                SectionId::Type => {
                    this.func_type_len = section_reader.read_u32()? as usize;
                }
                SectionId::Import => todo!(),
                SectionId::Function => {
                    this.idx_len += section_reader.read_u32()? as usize;
                }
                SectionId::Table => todo!(),
                SectionId::Memory => {
                    let size = section_reader.read_u32()? as usize;
                    if size != 1 {
                        return Err(DecodeError::InvalidMemorySectionSize { size });
                    }
                }
                SectionId::Global => todo!(),
                SectionId::Export => {
                    this.export_len = section_reader.read_u32()? as usize;
                }
                SectionId::Start => {}
                SectionId::Element => todo!(),
                SectionId::Code => {
                    this.handle_code_section(section_reader)?;
                }
                SectionId::Data => todo!(),
            }
        }

        Ok(this)
    }

    fn handle_code_section(&mut self, mut reader: ByteReader) -> Result<(), DecodeError> {
        let code_count = reader.read_u32()?;
        let mut code_reader = reader.block_reader()?;
        for _ in 0..code_count {
            let locals_size = code_reader.read_u32()?;
            for _ in 0..locals_size {
                let n = code_reader.read_u32()?;
                for _ in 0..n {
                    let v = code_reader.read_u8()?;
                    let vt = ValueType::new(v)?;
                }
            }
            // TODO
        }
        code_reader.assert_empty()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    pub fn new(v: u8) -> Result<Self, DecodeError> {
        match v {
            0x7f => Ok(Self::I32),
            0x7e => Ok(Self::I64),
            0x7d => Ok(Self::F32),
            0x7c => Ok(Self::F64),
            _ => Err(DecodeError::InvalidValueType { value: v }),
        }
    }
}

#[derive(Debug)]
pub struct ModuleDecoder<'a> {
    pub type_section: &'a mut [FuncType],
}

impl<'a> ModuleDecoder<'a> {
    pub fn decode(wasm: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = ByteReader::new(wasm);
        reader.validate_preamble()?;

        let mut ty = None;
        while !reader.is_empty() {
            let (section_id, mut section_reader) = reader.read_section_reader()?;
            match section_id {
                SectionId::Custom => todo!(),
                SectionId::Type => ty = Some(TypeSection::decode(&mut section_reader)?),
                SectionId::Import => todo!(),
                SectionId::Function => todo!(),
                SectionId::Table => todo!(),
                SectionId::Memory => todo!(),
                SectionId::Global => todo!(),
                SectionId::Export => todo!(),
                SectionId::Start => todo!(),
                SectionId::Element => todo!(),
                SectionId::Code => todo!(),
                SectionId::Data => todo!(),
            }
            if !section_reader.is_empty() {
                return Err(DecodeError::MalformedSectiondata);
            }
        }
        todo!()
    }
}
