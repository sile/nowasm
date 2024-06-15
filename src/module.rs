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

    pub fn unread(&mut self) {
        self.position = self.position.saturating_sub(1);
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
            0x02 => Ok(Some(Instr::Block(BlockInstr::new(self)?))),
            0x0b => Ok(None),
            0x0c => Ok(Some(Instr::Br(LabelIdx(self.read_u32()?)))),
            0x0d => Ok(Some(Instr::BrIf(LabelIdx(self.read_u32()?)))),
            0x0e => Ok(Some(Instr::BrTable(BrTable::new(self)?))),
            0x0f => Ok(Some(Instr::Return)),

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
            0x45 => Ok(Some(Instr::I32Eqz)),
            0x46 => Ok(Some(Instr::I32Eq)),
            0x47 => Ok(Some(Instr::I32Ne)),
            0x48 => Ok(Some(Instr::I32LtS)),
            0x49 => Ok(Some(Instr::I32LtU)),
            0x4A => Ok(Some(Instr::I32GtS)),
            0x4B => Ok(Some(Instr::I32GtU)),
            0x4C => Ok(Some(Instr::I32LeS)),
            0x4D => Ok(Some(Instr::I32LeU)),
            0x4E => Ok(Some(Instr::I32GeS)),
            0x4F => Ok(Some(Instr::I32GeU)),
            0x50 => Ok(Some(Instr::I64Eqz)),
            0x51 => Ok(Some(Instr::I64Eq)),
            0x52 => Ok(Some(Instr::I64Ne)),
            0x53 => Ok(Some(Instr::I64LtS)),
            0x54 => Ok(Some(Instr::I64LtU)),
            0x55 => Ok(Some(Instr::I64GtS)),
            0x56 => Ok(Some(Instr::I64GtU)),
            0x57 => Ok(Some(Instr::I64LeS)),
            0x58 => Ok(Some(Instr::I64LeU)),
            0x59 => Ok(Some(Instr::I64GeS)),
            0x5A => Ok(Some(Instr::I64GeU)),
            0x5B => Ok(Some(Instr::F32Eq)),
            0x5C => Ok(Some(Instr::F32Ne)),
            0x5D => Ok(Some(Instr::F32Lt)),
            0x5E => Ok(Some(Instr::F32Gt)),
            0x5F => Ok(Some(Instr::F32Le)),
            0x60 => Ok(Some(Instr::F32Ge)),
            0x61 => Ok(Some(Instr::F64Eq)),
            0x62 => Ok(Some(Instr::F64Ne)),
            0x63 => Ok(Some(Instr::F64Lt)),
            0x64 => Ok(Some(Instr::F64Gt)),
            0x65 => Ok(Some(Instr::F64Le)),
            0x66 => Ok(Some(Instr::F64Ge)),
            0x67 => Ok(Some(Instr::I32Clz)),
            0x68 => Ok(Some(Instr::I32Ctz)),
            0x69 => Ok(Some(Instr::I32Popcnt)),
            0x6A => Ok(Some(Instr::I32Add)),
            0x6B => Ok(Some(Instr::I32Sub)),
            0x6C => Ok(Some(Instr::I32Mul)),
            0x6D => Ok(Some(Instr::I32DivS)),
            0x6E => Ok(Some(Instr::I32DivU)),
            0x6F => Ok(Some(Instr::I32RemS)),
            0x70 => Ok(Some(Instr::I32RemU)),
            0x71 => Ok(Some(Instr::I32And)),
            0x72 => Ok(Some(Instr::I32Or)),
            0x73 => Ok(Some(Instr::I32Xor)),
            0x74 => Ok(Some(Instr::I32Shl)),
            0x75 => Ok(Some(Instr::I32ShrS)),
            0x76 => Ok(Some(Instr::I32ShrU)),
            0x77 => Ok(Some(Instr::I32Rotl)),
            0x78 => Ok(Some(Instr::I32Rotr)),
            0x79 => Ok(Some(Instr::I64Clz)),
            0x7A => Ok(Some(Instr::I64Ctz)),
            0x7B => Ok(Some(Instr::I64Popcnt)),
            0x7C => Ok(Some(Instr::I64Add)),
            0x7D => Ok(Some(Instr::I64Sub)),
            0x7E => Ok(Some(Instr::I64Mul)),
            0x7F => Ok(Some(Instr::I64DivS)),
            0x80 => Ok(Some(Instr::I64DivU)),
            0x81 => Ok(Some(Instr::I64RemS)),
            0x82 => Ok(Some(Instr::I64RemU)),
            0x83 => Ok(Some(Instr::I64And)),
            0x84 => Ok(Some(Instr::I64Or)),
            0x85 => Ok(Some(Instr::I64Xor)),
            0x86 => Ok(Some(Instr::I64Shl)),
            0x87 => Ok(Some(Instr::I64ShrS)),
            0x88 => Ok(Some(Instr::I64ShrU)),
            0x89 => Ok(Some(Instr::I64Rotl)),
            0x8A => Ok(Some(Instr::I64Rotr)),
            0x8B => Ok(Some(Instr::F32Abs)),
            0x8C => Ok(Some(Instr::F32Neg)),
            0x8D => Ok(Some(Instr::F32Ceil)),
            0x8E => Ok(Some(Instr::F32Floor)),
            0x8F => Ok(Some(Instr::F32Trunc)),
            0x90 => Ok(Some(Instr::F32Nearest)),
            0x91 => Ok(Some(Instr::F32Sqrt)),
            0x92 => Ok(Some(Instr::F32Add)),
            0x93 => Ok(Some(Instr::F32Sub)),
            0x94 => Ok(Some(Instr::F32Mul)),
            0x95 => Ok(Some(Instr::F32Div)),
            0x96 => Ok(Some(Instr::F32Min)),
            0x97 => Ok(Some(Instr::F32Max)),
            0x98 => Ok(Some(Instr::F32Copysign)),
            0x99 => Ok(Some(Instr::F64Abs)),
            0x9A => Ok(Some(Instr::F64Neg)),
            0x9B => Ok(Some(Instr::F64Ceil)),
            0x9C => Ok(Some(Instr::F64Floor)),
            0x9D => Ok(Some(Instr::F64Trunc)),
            0x9E => Ok(Some(Instr::F64Nearest)),
            0x9F => Ok(Some(Instr::F64Sqrt)),
            0xA0 => Ok(Some(Instr::F64Add)),
            0xA1 => Ok(Some(Instr::F64Sub)),
            0xA2 => Ok(Some(Instr::F64Mul)),
            0xA3 => Ok(Some(Instr::F64Div)),
            0xA4 => Ok(Some(Instr::F64Min)),
            0xA5 => Ok(Some(Instr::F64Max)),
            0xA6 => Ok(Some(Instr::F64Copysign)),
            0xA7 => Ok(Some(Instr::I32WrapI64)),
            0xA8 => Ok(Some(Instr::I32TruncF32S)),
            0xA9 => Ok(Some(Instr::I32TruncF32U)),
            0xAA => Ok(Some(Instr::I32TruncF64S)),
            0xAB => Ok(Some(Instr::I32TruncF64U)),
            0xAC => Ok(Some(Instr::I64ExtendI32S)),
            0xAD => Ok(Some(Instr::I64ExtendI32U)),
            0xAE => Ok(Some(Instr::I64TruncF32S)),
            0xAF => Ok(Some(Instr::I64TruncF32U)),
            0xB0 => Ok(Some(Instr::I64TruncF64S)),
            0xB1 => Ok(Some(Instr::I64TruncF64U)),
            0xB2 => Ok(Some(Instr::F32ConvertI32S)),
            0xB3 => Ok(Some(Instr::F32ConvertI32U)),
            0xB4 => Ok(Some(Instr::F32ConvertI64S)),
            0xB5 => Ok(Some(Instr::F32ConvertI64U)),
            0xB6 => Ok(Some(Instr::F32DemoteF64)),
            0xB7 => Ok(Some(Instr::F64ConvertI32S)),
            0xB8 => Ok(Some(Instr::F64ConvertI32U)),
            0xB9 => Ok(Some(Instr::F64ConvertI64S)),
            0xBA => Ok(Some(Instr::F64ConvertI64U)),
            0xBB => Ok(Some(Instr::F64PromoteF32)),
            0xBC => Ok(Some(Instr::I32ReinterpretF32)),
            0xBD => Ok(Some(Instr::I64ReinterpretF64)),
            0xBE => Ok(Some(Instr::F32ReinterpretI32)),
            0xBF => Ok(Some(Instr::F64ReinterpretI64)),

            _ => Err(DecodeError::InvalidInstrOpcode { opcode }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockInstr {
    pub block_type: BlockType,
    pub instr_start: usize,
    pub instr_end: usize,
}

impl BlockInstr {
    pub fn new(reader: &mut ByteReader) -> Result<Self, DecodeError> {
        let block_type = BlockType::new(reader)?;
        let mut n = 0;
        while let Some(i) = reader.read_instr()? {
            n += i.instr_len();
        }
        Ok(Self {
            block_type,
            instr_start: 0,
            instr_end: n,
        })
    }

    pub fn len(self) -> usize {
        1 + self.instr_end - self.instr_start
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrTable {
    pub label_idx_start: usize,
    pub label_idx_end: usize,

    // TODO: rename
    pub last_label_idx: LabelIdx,
}

impl BrTable {
    pub fn new(reader: &mut ByteReader) -> Result<Self, DecodeError> {
        let n = reader.read_u32()? as usize;
        for _ in 0..n {
            let _ = LabelIdx(reader.read_u32()?);
        }
        Ok(Self {
            label_idx_start: 0,
            label_idx_end: n,
            last_label_idx: LabelIdx(reader.read_u32()?),
        })
    }

    pub fn idx_len(self) -> usize {
        self.label_idx_end - self.label_idx_start
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instr {
    // Control Instructions
    Unreachable,
    Nop,
    Block(BlockInstr),
    // loop
    // if
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(BrTable),
    Return,

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
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

impl Instr {
    pub fn instr_len(self) -> usize {
        match self {
            Self::Block(x) => x.len(),
            _ => 1,
        }
    }

    pub fn idx_len(self) -> usize {
        match self {
            Self::BrTable(x) => x.idx_len(),
            _ => 0,
        }
    }
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
pub struct LabelIdx(pub u32);

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

impl BlockType {
    pub fn new(reader: &mut ByteReader) -> Result<Self, DecodeError> {
        let v = reader.read_u8()?;
        if v == 0x40 {
            return Ok(Self::Empty);
        }
        if let Ok(t) = ValType::new(v) {
            return Ok(Self::Val(t));
        }

        reader.unread();
        Ok(Self::TypeIndex(S33(reader.read_integer(33)? as i64)))
    }
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
        while let Some(i) = reader.read_instr()? {
            self.instr_len += i.instr_len();
            self.idx_len += i.idx_len();
        }
        Ok(())
    }

    fn handle_code_section(&mut self, mut reader: ByteReader) -> Result<(), DecodeError> {
        let code_count = reader.read_u32()?;
        for _ in 0..code_count {
            let mut code_reader = reader.block_reader()?;
            let locals_size = code_reader.read_u32()?;

            for _ in 0..locals_size {
                let _n = code_reader.read_u32()?;
                let v = code_reader.read_u8()?;
                let _vt = ValType::new(v)?;
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
