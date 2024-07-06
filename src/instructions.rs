#[cfg(feature = "sign_extension")]
use crate::instructions_sign_extension::SignExtensionInstr;
use crate::vectors::Vector;
use crate::{
    reader::Reader,
    symbols::{BlockType, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemArg, TypeIdx},
    Allocator, DecodeError,
};

#[derive(Debug, Clone)]
pub enum Instr<A: Allocator> {
    // Control Instructions
    Unreachable,
    Nop,
    Block(BlockInstr<A>),
    Loop(LoopInstr<A>),
    If(IfInstr<A>),
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(BrTableInstr<A>),
    Return,
    Call(FuncIdx),
    CallIndirect(TypeIdx),

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

    // Sign Extension
    #[cfg(feature = "sign_extension")]
    SignExtension(SignExtensionInstr),
}

impl<A: Allocator> Instr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let opcode = reader.read_u8()?;
        match opcode {
            // Control Instructions
            0x00 => Ok(Self::Unreachable),
            0x01 => Ok(Self::Nop),
            0x02 => Ok(Self::Block(BlockInstr::decode(reader)?)),
            0x03 => Ok(Self::Loop(LoopInstr::decode(reader)?)),
            0x04 => Ok(Self::If(IfInstr::decode(reader)?)),
            0x0c => Ok(Self::Br(LabelIdx::decode(reader)?)),
            0x0d => Ok(Self::BrIf(LabelIdx::decode(reader)?)),
            0x0e => Ok(Self::BrTable(BrTableInstr::decode(reader)?)),
            0x0f => Ok(Self::Return),
            0x10 => Ok(Self::Call(FuncIdx::decode(reader)?)),
            0x11 => {
                let idx = TypeIdx::decode(reader)?;
                let table = reader.read_u8()?;
                if table != 0 {
                    return Err(DecodeError::InvalidCallIndirectTableIndex { value: table });
                }
                Ok(Self::CallIndirect(idx))
            }

            // Parametric Instructions
            0x1a => Ok(Self::Drop),
            0x1b => Ok(Self::Select),

            // Variable Instructions
            0x20 => Ok(Self::LocalGet(LocalIdx::decode(reader)?)),
            0x21 => Ok(Self::LocalSet(LocalIdx::decode(reader)?)),
            0x22 => Ok(Self::LocalTee(LocalIdx::decode(reader)?)),
            0x23 => Ok(Self::GlobalGet(GlobalIdx::decode(reader)?)),
            0x24 => Ok(Self::GlobalSet(GlobalIdx::decode(reader)?)),

            // Memory Instructions
            0x28 => Ok(Self::I32Load(MemArg::decode(reader)?)),
            0x29 => Ok(Self::I64Load(MemArg::decode(reader)?)),
            0x2a => Ok(Self::F32Load(MemArg::decode(reader)?)),
            0x2b => Ok(Self::F64Load(MemArg::decode(reader)?)),
            0x2c => Ok(Self::I32Load8S(MemArg::decode(reader)?)),
            0x2d => Ok(Self::I32Load8U(MemArg::decode(reader)?)),
            0x2e => Ok(Self::I32Load16S(MemArg::decode(reader)?)),
            0x2f => Ok(Self::I32Load16U(MemArg::decode(reader)?)),
            0x30 => Ok(Self::I64Load8S(MemArg::decode(reader)?)),
            0x31 => Ok(Self::I64Load8U(MemArg::decode(reader)?)),
            0x32 => Ok(Self::I64Load16S(MemArg::decode(reader)?)),
            0x33 => Ok(Self::I64Load16U(MemArg::decode(reader)?)),
            0x34 => Ok(Self::I64Load32S(MemArg::decode(reader)?)),
            0x35 => Ok(Self::I64Load32U(MemArg::decode(reader)?)),
            0x36 => Ok(Self::I32Store(MemArg::decode(reader)?)),
            0x37 => Ok(Self::I64Store(MemArg::decode(reader)?)),
            0x38 => Ok(Self::F32Store(MemArg::decode(reader)?)),
            0x39 => Ok(Self::F64Store(MemArg::decode(reader)?)),
            0x3a => Ok(Self::I32Store8(MemArg::decode(reader)?)),
            0x3b => Ok(Self::I32Store16(MemArg::decode(reader)?)),
            0x3c => Ok(Self::I64Store8(MemArg::decode(reader)?)),
            0x3d => Ok(Self::I64Store16(MemArg::decode(reader)?)),
            0x3e => Ok(Self::I64Store32(MemArg::decode(reader)?)),
            0x3f => {
                let value = reader.read_u8()?;
                if value != 0 {
                    return Err(DecodeError::InvalidMemorySizeMemoryIndex { value });
                }
                Ok(Self::MemorySize)
            }
            0x40 => {
                let value = reader.read_u8()?;
                if value != 0 {
                    return Err(DecodeError::InvalidMemoryGrowMemoryIndex { value });
                }
                Ok(Self::MemoryGrow)
            }

            // Numeric Instructions
            0x41 => Ok(Self::I32Const(reader.read_i32()?)),
            0x42 => Ok(Self::I64Const(reader.read_i64()?)),
            0x43 => Ok(Self::F32Const(reader.read_f32()?)),
            0x44 => Ok(Self::F64Const(reader.read_f64()?)),
            0x45 => Ok(Self::I32Eqz),
            0x46 => Ok(Self::I32Eq),
            0x47 => Ok(Self::I32Ne),
            0x48 => Ok(Self::I32LtS),
            0x49 => Ok(Self::I32LtU),
            0x4A => Ok(Self::I32GtS),
            0x4B => Ok(Self::I32GtU),
            0x4C => Ok(Self::I32LeS),
            0x4D => Ok(Self::I32LeU),
            0x4E => Ok(Self::I32GeS),
            0x4F => Ok(Self::I32GeU),
            0x50 => Ok(Self::I64Eqz),
            0x51 => Ok(Self::I64Eq),
            0x52 => Ok(Self::I64Ne),
            0x53 => Ok(Self::I64LtS),
            0x54 => Ok(Self::I64LtU),
            0x55 => Ok(Self::I64GtS),
            0x56 => Ok(Self::I64GtU),
            0x57 => Ok(Self::I64LeS),
            0x58 => Ok(Self::I64LeU),
            0x59 => Ok(Self::I64GeS),
            0x5A => Ok(Self::I64GeU),
            0x5B => Ok(Self::F32Eq),
            0x5C => Ok(Self::F32Ne),
            0x5D => Ok(Self::F32Lt),
            0x5E => Ok(Self::F32Gt),
            0x5F => Ok(Self::F32Le),
            0x60 => Ok(Self::F32Ge),
            0x61 => Ok(Self::F64Eq),
            0x62 => Ok(Self::F64Ne),
            0x63 => Ok(Self::F64Lt),
            0x64 => Ok(Self::F64Gt),
            0x65 => Ok(Self::F64Le),
            0x66 => Ok(Self::F64Ge),
            0x67 => Ok(Self::I32Clz),
            0x68 => Ok(Self::I32Ctz),
            0x69 => Ok(Self::I32Popcnt),
            0x6A => Ok(Self::I32Add),
            0x6B => Ok(Self::I32Sub),
            0x6C => Ok(Self::I32Mul),
            0x6D => Ok(Self::I32DivS),
            0x6E => Ok(Self::I32DivU),
            0x6F => Ok(Self::I32RemS),
            0x70 => Ok(Self::I32RemU),
            0x71 => Ok(Self::I32And),
            0x72 => Ok(Self::I32Or),
            0x73 => Ok(Self::I32Xor),
            0x74 => Ok(Self::I32Shl),
            0x75 => Ok(Self::I32ShrS),
            0x76 => Ok(Self::I32ShrU),
            0x77 => Ok(Self::I32Rotl),
            0x78 => Ok(Self::I32Rotr),
            0x79 => Ok(Self::I64Clz),
            0x7A => Ok(Self::I64Ctz),
            0x7B => Ok(Self::I64Popcnt),
            0x7C => Ok(Self::I64Add),
            0x7D => Ok(Self::I64Sub),
            0x7E => Ok(Self::I64Mul),
            0x7F => Ok(Self::I64DivS),
            0x80 => Ok(Self::I64DivU),
            0x81 => Ok(Self::I64RemS),
            0x82 => Ok(Self::I64RemU),
            0x83 => Ok(Self::I64And),
            0x84 => Ok(Self::I64Or),
            0x85 => Ok(Self::I64Xor),
            0x86 => Ok(Self::I64Shl),
            0x87 => Ok(Self::I64ShrS),
            0x88 => Ok(Self::I64ShrU),
            0x89 => Ok(Self::I64Rotl),
            0x8A => Ok(Self::I64Rotr),
            0x8B => Ok(Self::F32Abs),
            0x8C => Ok(Self::F32Neg),
            0x8D => Ok(Self::F32Ceil),
            0x8E => Ok(Self::F32Floor),
            0x8F => Ok(Self::F32Trunc),
            0x90 => Ok(Self::F32Nearest),
            0x91 => Ok(Self::F32Sqrt),
            0x92 => Ok(Self::F32Add),
            0x93 => Ok(Self::F32Sub),
            0x94 => Ok(Self::F32Mul),
            0x95 => Ok(Self::F32Div),
            0x96 => Ok(Self::F32Min),
            0x97 => Ok(Self::F32Max),
            0x98 => Ok(Self::F32Copysign),
            0x99 => Ok(Self::F64Abs),
            0x9A => Ok(Self::F64Neg),
            0x9B => Ok(Self::F64Ceil),
            0x9C => Ok(Self::F64Floor),
            0x9D => Ok(Self::F64Trunc),
            0x9E => Ok(Self::F64Nearest),
            0x9F => Ok(Self::F64Sqrt),
            0xA0 => Ok(Self::F64Add),
            0xA1 => Ok(Self::F64Sub),
            0xA2 => Ok(Self::F64Mul),
            0xA3 => Ok(Self::F64Div),
            0xA4 => Ok(Self::F64Min),
            0xA5 => Ok(Self::F64Max),
            0xA6 => Ok(Self::F64Copysign),
            0xA7 => Ok(Self::I32WrapI64),
            0xA8 => Ok(Self::I32TruncF32S),
            0xA9 => Ok(Self::I32TruncF32U),
            0xAA => Ok(Self::I32TruncF64S),
            0xAB => Ok(Self::I32TruncF64U),
            0xAC => Ok(Self::I64ExtendI32S),
            0xAD => Ok(Self::I64ExtendI32U),
            0xAE => Ok(Self::I64TruncF32S),
            0xAF => Ok(Self::I64TruncF32U),
            0xB0 => Ok(Self::I64TruncF64S),
            0xB1 => Ok(Self::I64TruncF64U),
            0xB2 => Ok(Self::F32ConvertI32S),
            0xB3 => Ok(Self::F32ConvertI32U),
            0xB4 => Ok(Self::F32ConvertI64S),
            0xB5 => Ok(Self::F32ConvertI64U),
            0xB6 => Ok(Self::F32DemoteF64),
            0xB7 => Ok(Self::F64ConvertI32S),
            0xB8 => Ok(Self::F64ConvertI32U),
            0xB9 => Ok(Self::F64ConvertI64S),
            0xBA => Ok(Self::F64ConvertI64U),
            0xBB => Ok(Self::F64PromoteF32),
            0xBC => Ok(Self::I32ReinterpretF32),
            0xBD => Ok(Self::I64ReinterpretF64),
            0xBE => Ok(Self::F32ReinterpretI32),
            0xBF => Ok(Self::F64ReinterpretI64),

            // Sign Extension
            #[cfg(feature = "sign_extension")]
            0xC0..=0xC4 => {
                reader.unread_u8();
                Ok(Self::SignExtension(SignExtensionInstr::decode(reader)?))
            }

            _ => Err(DecodeError::InvalidOpcode { value: opcode }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockInstr<A: Allocator> {
    pub block_type: BlockType,
    pub instrs: A::Vector<Instr<A>>,
}

impl<A: Allocator> BlockInstr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = BlockType::decode(reader)?;
        let mut instrs = A::allocate_vector();
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self { block_type, instrs })
    }

    pub fn len(self) -> usize {
        self.instrs.as_ref().len()
    }
}

#[derive(Debug, Clone)]
pub struct LoopInstr<A: Allocator> {
    pub block_type: BlockType,
    pub instrs: A::Vector<Instr<A>>,
}

impl<A: Allocator> LoopInstr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = BlockType::decode(reader)?;
        let mut instrs = A::allocate_vector();
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self { block_type, instrs })
    }
}

#[derive(Debug, Clone)]
pub struct IfInstr<A: Allocator> {
    pub block_type: BlockType,
    pub then_instrs: A::Vector<Instr<A>>,
    pub else_instrs: A::Vector<Instr<A>>,
}

impl<A: Allocator> IfInstr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = BlockType::decode(reader)?;
        let mut then_instrs = A::allocate_vector();
        let mut else_instrs = A::allocate_vector();

        loop {
            let b = reader.peek_u8()?;
            if b == 0x0B {
                reader.read_u8()?;
                return Ok(Self {
                    block_type,
                    then_instrs,
                    else_instrs,
                });
            } else if b == 0x05 {
                reader.read_u8()?;
                break;
            }

            then_instrs.push(Instr::decode(reader)?);
        }

        while reader.peek_u8()? != 0x0B {
            else_instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;

        Ok(Self {
            block_type,
            then_instrs,
            else_instrs,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BrTableInstr<A: Allocator> {
    pub labels: A::Vector<LabelIdx>,
}

impl<A: Allocator> BrTableInstr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let n = reader.read_u32()? as usize + 1;
        let mut labels = A::allocate_vector();
        for _ in 0..n {
            labels.push(LabelIdx::decode(reader)?);
        }
        Ok(Self { labels })
    }

    pub fn idx_len(self) -> usize {
        self.labels.as_ref().len()
    }
}
