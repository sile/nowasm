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
    MalformedInstr,
    InvalidMemorySectionSize { size: usize },
    InvalidU32,
    InvalidInteger,
    InvalidValueType { value: u8 },
    InvalidInstrOpcode { opcode: u8 },
}

impl DecodeError {
    fn end_of_bytes() -> Self {
        DecodeError::EndOfBytes
    }
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
            Err(DecodeError::end_of_bytes())
        }
    }

    pub fn read_integer(&mut self, bits: usize) -> Result<u64, DecodeError> {
        let mut n = 0u64;
        let mut offset = 0;
        loop {
            let b = self.read_u8()?;
            let v = (b as u64 & 0b0111_1111) << offset;

            if b & 0b1000_0000 == 0 {
                let remaining_bits = bits - offset;
                if remaining_bits < 7 && (b as u64 & 0b0111_1111) >= (1 << remaining_bits) {
                    return Err(DecodeError::InvalidInteger);
                }
                n += v;
                break;
            }

            n += v;
            offset += 7;
            if offset >= bits {
                return Err(DecodeError::InvalidInteger);
            }
        }
        Ok(n)
    }

    pub fn read_u64(&mut self) -> Result<u64, DecodeError> {
        self.read_integer(64)
    }

    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        self.read_u64().map(|n| n as i64)
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        self.read_integer(32).map(|n| n as u32)
    }

    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        self.read_u32().map(|n| n as i32)
    }

    pub fn read_f32(&mut self) -> Result<f32, DecodeError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64, DecodeError> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.len() {
            return Err(DecodeError::end_of_bytes());
        }

        let start = self.position;
        self.position += n;
        Ok(&self.data[start..][..n])
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        let n = buf.len();
        if n > self.len() {
            return Err(DecodeError::end_of_bytes());
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

    pub fn read_instr(&mut self) -> Result<Option<Instr>, DecodeError> {
        let opcode = self.read_u8()?;
        match opcode {
            // Control Instructions
            0x00 => Ok(Some(Instr::Unreachable)),
            0x01 => Ok(Some(Instr::Nop)),
            0x0b => Ok(None),

            // Parametric Instructions
            0x1a => Ok(Some(Instr::Drop)),
            0x1b => Ok(Some(Instr::Select)),

            // Variable Instructions
            0x20 => Ok(Some(Instr::LocalGet(LocalIdx(self.read_u32()?)))),
            0x21 => Ok(Some(Instr::LocalSet(LocalIdx(self.read_u32()?)))),
            0x22 => Ok(Some(Instr::LocalTee(LocalIdx(self.read_u32()?)))),
            0x23 => Ok(Some(Instr::GlobalGet(GlobalIdx(self.read_u32()?)))),
            0x24 => Ok(Some(Instr::GlobalSet(GlobalIdx(self.read_u32()?)))),

            // Memory Instructions
            0x28 => Ok(Some(Instr::I32Load(MemArg::new(self)?))),
            0x29 => Ok(Some(Instr::I64Load(MemArg::new(self)?))),
            0x2a => Ok(Some(Instr::F32Load(MemArg::new(self)?))),
            0x2b => Ok(Some(Instr::F64Load(MemArg::new(self)?))),
            0x2c => Ok(Some(Instr::I32Load8S(MemArg::new(self)?))),
            0x2d => Ok(Some(Instr::I32Load8U(MemArg::new(self)?))),
            0x2e => Ok(Some(Instr::I32Load16S(MemArg::new(self)?))),
            0x2f => Ok(Some(Instr::I32Load16U(MemArg::new(self)?))),
            0x30 => Ok(Some(Instr::I64Load8S(MemArg::new(self)?))),
            0x31 => Ok(Some(Instr::I64Load8U(MemArg::new(self)?))),
            0x32 => Ok(Some(Instr::I64Load16S(MemArg::new(self)?))),
            0x33 => Ok(Some(Instr::I64Load16U(MemArg::new(self)?))),
            0x34 => Ok(Some(Instr::I64Load32S(MemArg::new(self)?))),
            0x35 => Ok(Some(Instr::I64Load32U(MemArg::new(self)?))),
            0x36 => Ok(Some(Instr::I32Store(MemArg::new(self)?))),
            0x37 => Ok(Some(Instr::I64Store(MemArg::new(self)?))),
            0x38 => Ok(Some(Instr::F32Store(MemArg::new(self)?))),
            0x39 => Ok(Some(Instr::F64Store(MemArg::new(self)?))),
            0x3a => Ok(Some(Instr::I32Store8(MemArg::new(self)?))),
            0x3b => Ok(Some(Instr::I32Store16(MemArg::new(self)?))),
            0x3c => Ok(Some(Instr::I64Store8(MemArg::new(self)?))),
            0x3d => Ok(Some(Instr::I64Store16(MemArg::new(self)?))),
            0x3e => Ok(Some(Instr::I64Store32(MemArg::new(self)?))),
            0x3f => {
                if self.read_u8()? != 0 {
                    return Err(DecodeError::MalformedInstr);
                }
                Ok(Some(Instr::MemorySize))
            }
            0x40 => {
                if self.read_u8()? != 0 {
                    return Err(DecodeError::MalformedInstr);
                }
                Ok(Some(Instr::MemoryGrow))
            }

            // Numeric Instructions
            0x41 => Ok(Some(Instr::I32Const(self.read_i32()?))),
            0x42 => Ok(Some(Instr::I64Const(self.read_i64()?))),
            0x43 => Ok(Some(Instr::F32Const(self.read_f32()?))),
            0x44 => Ok(Some(Instr::F64Const(self.read_f64()?))),

            // End
            _ => Err(DecodeError::InvalidInstrOpcode { opcode }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instr {
    // Control Instructions
    Unreachable,
    Nop,

    // Parametric Instructions
    Drop,
    Select,

    // Variable Instructions
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),

    // Memory Instructions
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize,
    MemoryGrow,

    // Numeric Instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl MemArg {
    pub fn new(reader: &mut ByteReader) -> Result<Self, DecodeError> {
        let align = reader.read_u32()?;
        let offset = reader.read_u32()?;
        Ok(Self { align, offset })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Empty,
    Val(ValType),
    TypeIndex(S33),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S33(pub i64);

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
    pub fn decode(_reader: &mut ByteReader) -> Result<Self, DecodeError> {
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
    pub instr_len: usize,
    pub data_segments_len: usize,
    pub bytes_len: usize,
}

impl ModuleSpec {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = ByteReader::new(wasm_bytes);
        reader.validate_preamble()?;

        let mut this = Self {
            func_type_len: 0,
            idx_len: 0,
            export_len: 0,
            instr_len: 0,
            data_segments_len: 0,
            bytes_len: 0,
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
                SectionId::Data => {
                    this.handle_data_section(section_reader)?;
                }
            }
        }

        Ok(this)
    }

    fn handle_data_section(&mut self, mut reader: ByteReader) -> Result<(), DecodeError> {
        self.data_segments_len = reader.read_u32()? as usize;
        for _ in 0..self.data_segments_len {
            // data
            let _mem_idx = MemIdx(reader.read_u32()?);

            // offset
            self.handle_expr(&mut reader)?;

            // init
            let init_len = reader.read_u32()? as usize;
            reader.read_bytes(init_len)?;
            self.bytes_len += init_len;
        }
        reader.assert_empty()?;
        Ok(())
    }

    fn handle_expr(&mut self, reader: &mut ByteReader) -> Result<(), DecodeError> {
        while let Some(_) = reader.read_instr()? {
            self.instr_len += 1;
        }
        Ok(())
    }

    fn handle_code_section(&mut self, mut reader: ByteReader) -> Result<(), DecodeError> {
        let code_count = reader.read_u32()?;
        for _ in 0..code_count {
            let mut code_reader = reader.block_reader()?;
            let locals_size = code_reader.read_u32()?;
            for _ in 0..locals_size {
                let n = code_reader.read_u32()?;
                for _ in 0..n {
                    let v = code_reader.read_u8()?;
                    let _vt = ValType::new(v)?;
                }
            }

            self.handle_expr(&mut code_reader)?;
            code_reader.assert_empty()?;
        }
        reader.assert_empty()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl ValType {
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
        _ = ty;
        todo!()
    }
}
