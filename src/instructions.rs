use crate::vector::Vector;
use crate::{
    components::{Blocktype, Funcidx, Globalidx, Labelidx, Localidx, Memarg, Typeidx},
    decode::Decode,
    reader::Reader,
    DecodeError, VectorFactory,
};
use core::fmt::{Debug, Formatter};

#[cfg(feature = "sign_extension")]
pub use crate::sign_extension::SignExtensionInstr;

pub enum Instr<V: VectorFactory> {
    // Control Instructions
    Unreachable,
    Nop,
    Block(BlockInstr<V>),
    Loop(LoopInstr<V>),
    If(IfInstr<V>),
    Br(Labelidx),
    BrIf(Labelidx),
    BrTable(BrTableInstr<V>),
    Return,
    Call(Funcidx),
    CallIndirect(Typeidx),

    // Parametric Instructions
    Drop,
    Select,

    // Variable Instructions
    LocalGet(Localidx),
    LocalSet(Localidx),
    LocalTee(Localidx),
    GlobalGet(Globalidx),
    GlobalSet(Globalidx),

    // Memory Instructions
    I32Load(Memarg),
    I64Load(Memarg),
    F32Load(Memarg),
    F64Load(Memarg),
    I32Load8S(Memarg),
    I32Load8U(Memarg),
    I32Load16S(Memarg),
    I32Load16U(Memarg),
    I64Load8S(Memarg),
    I64Load8U(Memarg),
    I64Load16S(Memarg),
    I64Load16U(Memarg),
    I64Load32S(Memarg),
    I64Load32U(Memarg),
    I32Store(Memarg),
    I64Store(Memarg),
    F32Store(Memarg),
    F64Store(Memarg),
    I32Store8(Memarg),
    I32Store16(Memarg),
    I64Store8(Memarg),
    I64Store16(Memarg),
    I64Store32(Memarg),
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

impl<V: VectorFactory> Decode for Instr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let opcode = reader.read_u8()?;
        match opcode {
            // Control Instructions
            0x00 => Ok(Self::Unreachable),
            0x01 => Ok(Self::Nop),
            0x02 => Ok(Self::Block(BlockInstr::decode(reader)?)),
            0x03 => Ok(Self::Loop(LoopInstr::decode(reader)?)),
            0x04 => Ok(Self::If(IfInstr::decode(reader)?)),
            0x0c => Ok(Self::Br(Labelidx::decode(reader)?)),
            0x0d => Ok(Self::BrIf(Labelidx::decode(reader)?)),
            0x0e => Ok(Self::BrTable(BrTableInstr::decode(reader)?)),
            0x0f => Ok(Self::Return),
            0x10 => Ok(Self::Call(Funcidx::decode(reader)?)),
            0x11 => {
                let idx = Typeidx::decode(reader)?;
                let table = reader.read_u8()?;
                if table != 0 {
                    return Err(DecodeError::InvalidTableIdx {
                        value: table as u32,
                    });
                }
                Ok(Self::CallIndirect(idx))
            }

            // Parametric Instructions
            0x1a => Ok(Self::Drop),
            0x1b => Ok(Self::Select),

            // Variable Instructions
            0x20 => Ok(Self::LocalGet(Localidx::decode(reader)?)),
            0x21 => Ok(Self::LocalSet(Localidx::decode(reader)?)),
            0x22 => Ok(Self::LocalTee(Localidx::decode(reader)?)),
            0x23 => Ok(Self::GlobalGet(Globalidx::decode(reader)?)),
            0x24 => Ok(Self::GlobalSet(Globalidx::decode(reader)?)),

            // Memory Instructions
            0x28 => Ok(Self::I32Load(Memarg::decode(reader)?)),
            0x29 => Ok(Self::I64Load(Memarg::decode(reader)?)),
            0x2a => Ok(Self::F32Load(Memarg::decode(reader)?)),
            0x2b => Ok(Self::F64Load(Memarg::decode(reader)?)),
            0x2c => Ok(Self::I32Load8S(Memarg::decode(reader)?)),
            0x2d => Ok(Self::I32Load8U(Memarg::decode(reader)?)),
            0x2e => Ok(Self::I32Load16S(Memarg::decode(reader)?)),
            0x2f => Ok(Self::I32Load16U(Memarg::decode(reader)?)),
            0x30 => Ok(Self::I64Load8S(Memarg::decode(reader)?)),
            0x31 => Ok(Self::I64Load8U(Memarg::decode(reader)?)),
            0x32 => Ok(Self::I64Load16S(Memarg::decode(reader)?)),
            0x33 => Ok(Self::I64Load16U(Memarg::decode(reader)?)),
            0x34 => Ok(Self::I64Load32S(Memarg::decode(reader)?)),
            0x35 => Ok(Self::I64Load32U(Memarg::decode(reader)?)),
            0x36 => Ok(Self::I32Store(Memarg::decode(reader)?)),
            0x37 => Ok(Self::I64Store(Memarg::decode(reader)?)),
            0x38 => Ok(Self::F32Store(Memarg::decode(reader)?)),
            0x39 => Ok(Self::F64Store(Memarg::decode(reader)?)),
            0x3a => Ok(Self::I32Store8(Memarg::decode(reader)?)),
            0x3b => Ok(Self::I32Store16(Memarg::decode(reader)?)),
            0x3c => Ok(Self::I64Store8(Memarg::decode(reader)?)),
            0x3d => Ok(Self::I64Store16(Memarg::decode(reader)?)),
            0x3e => Ok(Self::I64Store32(Memarg::decode(reader)?)),
            0x3f => {
                let value = reader.read_u8()? as u32;
                if value != 0 {
                    return Err(DecodeError::InvalidMemIdx { value });
                }
                Ok(Self::MemorySize)
            }
            0x40 => {
                let value = reader.read_u8()? as u32;
                if value != 0 {
                    return Err(DecodeError::InvalidMemIdx { value });
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

impl<V: VectorFactory> Debug for Instr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unreachable => write!(f, "Unreachable"),
            Self::Nop => write!(f, "Nop"),
            Self::Block(v) => write!(f, "Block({v:?})"),
            Self::Loop(v) => write!(f, "Loop({v:?})"),
            Self::If(v) => write!(f, "If({v:?})"),
            Self::Br(v) => write!(f, "Br({v:?})"),
            Self::BrIf(v) => write!(f, "BrIf({v:?})"),
            Self::BrTable(v) => write!(f, "BrTable({v:?})"),
            Self::Return => write!(f, "Return"),
            Self::Call(v) => write!(f, "Call({v:?})"),
            Self::CallIndirect(v) => write!(f, "CallIndirect({v:?})"),
            Self::Drop => write!(f, "Drop"),
            Self::Select => write!(f, "Select"),
            Self::LocalGet(v) => write!(f, "LocalGet({v:?})"),
            Self::LocalSet(v) => write!(f, "LocalSet({v:?})"),
            Self::LocalTee(v) => write!(f, "LocalTee({v:?})"),
            Self::GlobalGet(v) => write!(f, "GlobalGet({v:?})"),
            Self::GlobalSet(v) => write!(f, "GlobalSet({v:?})"),
            Self::I32Load(v) => write!(f, "I32Load({v:?})"),
            Self::I64Load(v) => write!(f, "I64Load({v:?})"),
            Self::F32Load(v) => write!(f, "F32Load({v:?})"),
            Self::F64Load(v) => write!(f, "F64Load({v:?})"),
            Self::I32Load8S(v) => write!(f, "I32Load8S({v:?})"),
            Self::I32Load8U(v) => write!(f, "I32Load8U({v:?})"),
            Self::I32Load16S(v) => write!(f, "I32Load16S({v:?})"),
            Self::I32Load16U(v) => write!(f, "I32Load16U({v:?})"),
            Self::I64Load8S(v) => write!(f, "I64Load8S({v:?})"),
            Self::I64Load8U(v) => write!(f, "I64Load8U({v:?})"),
            Self::I64Load16S(v) => write!(f, "I64Load16S({v:?})"),
            Self::I64Load16U(v) => write!(f, "I64Load16U({v:?})"),
            Self::I64Load32S(v) => write!(f, "I64Load32S({v:?})"),
            Self::I64Load32U(v) => write!(f, "I64Load32U({v:?})"),
            Self::I32Store(v) => write!(f, "I32Store({v:?})"),
            Self::I64Store(v) => write!(f, "I64Store({v:?})"),
            Self::F32Store(v) => write!(f, "F32Store({v:?})"),
            Self::F64Store(v) => write!(f, "F64Store({v:?})"),
            Self::I32Store8(v) => write!(f, "I32Store8({v:?})"),
            Self::I32Store16(v) => write!(f, "I32Store16({v:?})"),
            Self::I64Store8(v) => write!(f, "I64Store8({v:?})"),
            Self::I64Store16(v) => write!(f, "I64Store16({v:?})"),
            Self::I64Store32(v) => write!(f, "I64Store32({v:?})"),
            Self::MemorySize => write!(f, "MemorySize"),
            Self::MemoryGrow => write!(f, "MemoryGrow"),
            Self::I32Const(v) => write!(f, "I32Const({v:?})"),
            Self::I64Const(v) => write!(f, "I64Const({v:?})"),
            Self::F32Const(v) => write!(f, "F32Const({v:?})"),
            Self::F64Const(v) => write!(f, "F64Const({v:?})"),
            Self::I32Eqz => write!(f, "I32Eqz"),
            Self::I32Eq => write!(f, "I32Eq"),
            Self::I32Ne => write!(f, "I32Ne"),
            Self::I32LtS => write!(f, "I32LtS"),
            Self::I32LtU => write!(f, "I32LtU"),
            Self::I32GtS => write!(f, "I32GtS"),
            Self::I32GtU => write!(f, "I32GtU"),
            Self::I32LeS => write!(f, "I32LeS"),
            Self::I32LeU => write!(f, "I32LeU"),
            Self::I32GeS => write!(f, "I32GeS"),
            Self::I32GeU => write!(f, "I32GeU"),
            Self::I64Eqz => write!(f, "I64Eqz"),
            Self::I64Eq => write!(f, "I64Eq"),
            Self::I64Ne => write!(f, "I64Ne"),
            Self::I64LtS => write!(f, "I64LtS"),
            Self::I64LtU => write!(f, "I64LtU"),
            Self::I64GtS => write!(f, "I64GtS"),
            Self::I64GtU => write!(f, "I64GtU"),
            Self::I64LeS => write!(f, "I64LeS"),
            Self::I64LeU => write!(f, "I64LeU"),
            Self::I64GeS => write!(f, "I64GeS"),
            Self::I64GeU => write!(f, "I64GeU"),
            Self::F32Eq => write!(f, "F32Eq"),
            Self::F32Ne => write!(f, "F32Ne"),
            Self::F32Lt => write!(f, "F32Lt"),
            Self::F32Gt => write!(f, "F32Gt"),
            Self::F32Le => write!(f, "F32Le"),
            Self::F32Ge => write!(f, "F32Ge"),
            Self::F64Eq => write!(f, "F64Eq"),
            Self::F64Ne => write!(f, "F64Ne"),
            Self::F64Lt => write!(f, "F64Lt"),
            Self::F64Gt => write!(f, "F64Gt"),
            Self::F64Le => write!(f, "F64Le"),
            Self::F64Ge => write!(f, "F64Ge"),
            Self::I32Clz => write!(f, "I32Clz"),
            Self::I32Ctz => write!(f, "I32Ctz"),
            Self::I32Popcnt => write!(f, "I32Popcnt"),
            Self::I32Add => write!(f, "I32Add"),
            Self::I32Sub => write!(f, "I32Sub"),
            Self::I32Mul => write!(f, "I32Mul"),
            Self::I32DivS => write!(f, "I32DivS"),
            Self::I32DivU => write!(f, "I32DivU"),
            Self::I32RemS => write!(f, "I32RemS"),
            Self::I32RemU => write!(f, "I32RemU"),
            Self::I32And => write!(f, "I32And"),
            Self::I32Or => write!(f, "I32Or"),
            Self::I32Xor => write!(f, "I32Xor"),
            Self::I32Shl => write!(f, "I32Shl"),
            Self::I32ShrS => write!(f, "I32ShrS"),
            Self::I32ShrU => write!(f, "I32ShrU"),
            Self::I32Rotl => write!(f, "I32Rotl"),
            Self::I32Rotr => write!(f, "I32Rotr"),
            Self::I64Clz => write!(f, "I64Clz"),
            Self::I64Ctz => write!(f, "I64Ctz"),
            Self::I64Popcnt => write!(f, "I64Popcnt"),
            Self::I64Add => write!(f, "I64Add"),
            Self::I64Sub => write!(f, "I64Sub"),
            Self::I64Mul => write!(f, "I64Mul"),
            Self::I64DivS => write!(f, "I64DivS"),
            Self::I64DivU => write!(f, "I64DivU"),
            Self::I64RemS => write!(f, "I64RemS"),
            Self::I64RemU => write!(f, "I64RemU"),
            Self::I64And => write!(f, "I64And"),
            Self::I64Or => write!(f, "I64Or"),
            Self::I64Xor => write!(f, "I64Xor"),
            Self::I64Shl => write!(f, "I64Shl"),
            Self::I64ShrS => write!(f, "I64ShrS"),
            Self::I64ShrU => write!(f, "I64ShrU"),
            Self::I64Rotl => write!(f, "I64Rotl"),
            Self::I64Rotr => write!(f, "I64Rotr"),
            Self::F32Abs => write!(f, "F32Abs"),
            Self::F32Neg => write!(f, "F32Neg"),
            Self::F32Ceil => write!(f, "F32Ceil"),
            Self::F32Floor => write!(f, "F32Floor"),
            Self::F32Trunc => write!(f, "F32Trunc"),
            Self::F32Nearest => write!(f, "F32Nearest"),
            Self::F32Sqrt => write!(f, "F32Sqrt"),
            Self::F32Add => write!(f, "F32Add"),
            Self::F32Sub => write!(f, "F32Sub"),
            Self::F32Mul => write!(f, "F32Mul"),
            Self::F32Div => write!(f, "F32Div"),
            Self::F32Min => write!(f, "F32Min"),
            Self::F32Max => write!(f, "F32Max"),
            Self::F32Copysign => write!(f, "F32Copysign"),
            Self::F64Abs => write!(f, "F64Abs"),
            Self::F64Neg => write!(f, "F64Neg"),
            Self::F64Ceil => write!(f, "F64Ceil"),
            Self::F64Floor => write!(f, "F64Floor"),
            Self::F64Trunc => write!(f, "F64Trunc"),
            Self::F64Nearest => write!(f, "F64Nearest"),
            Self::F64Sqrt => write!(f, "F64Sqrt"),
            Self::F64Add => write!(f, "F64Add"),
            Self::F64Sub => write!(f, "F64Sub"),
            Self::F64Mul => write!(f, "F64Mul"),
            Self::F64Div => write!(f, "F64Div"),
            Self::F64Min => write!(f, "F64Min"),
            Self::F64Max => write!(f, "F64Max"),
            Self::F64Copysign => write!(f, "F64Copysign"),
            Self::I32WrapI64 => write!(f, "I32WrapI64"),
            Self::I32TruncF32S => write!(f, "I32TruncF32S"),
            Self::I32TruncF32U => write!(f, "I32TruncF32U"),
            Self::I32TruncF64S => write!(f, "I32TruncF64S"),
            Self::I32TruncF64U => write!(f, "I32TruncF64U"),
            Self::I64ExtendI32S => write!(f, "I64ExtendI32S"),
            Self::I64ExtendI32U => write!(f, "I64ExtendI32U"),
            Self::I64TruncF32S => write!(f, "I64TruncF32S"),
            Self::I64TruncF32U => write!(f, "I64TruncF32U"),
            Self::I64TruncF64S => write!(f, "I64TruncF64S"),
            Self::I64TruncF64U => write!(f, "I64TruncF64U"),
            Self::F32ConvertI32S => write!(f, "F32ConvertI32S"),
            Self::F32ConvertI32U => write!(f, "F32ConvertI32U"),
            Self::F32ConvertI64S => write!(f, "F32ConvertI64S"),
            Self::F32ConvertI64U => write!(f, "F32ConvertI64U"),
            Self::F32DemoteF64 => write!(f, "F32DemoteF64"),
            Self::F64ConvertI32S => write!(f, "F64ConvertI32S"),
            Self::F64ConvertI32U => write!(f, "F64ConvertI32U"),
            Self::F64ConvertI64S => write!(f, "F64ConvertI64S"),
            Self::F64ConvertI64U => write!(f, "F64ConvertI64U"),
            Self::F64PromoteF32 => write!(f, "F64PromoteF32"),
            Self::I32ReinterpretF32 => write!(f, "I32ReinterpretF32"),
            Self::I64ReinterpretF64 => write!(f, "I64ReinterpretF64"),
            Self::F32ReinterpretI32 => write!(f, "F32ReinterpretI32"),
            Self::F64ReinterpretI64 => write!(f, "F64ReinterpretI64"),
            Self::SignExtension(v) => write!(f, "SignExtension({v:?})"),
        }
    }
}

impl<V: VectorFactory> Clone for Instr<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Unreachable => Self::Unreachable,
            Self::Nop => Self::Nop,
            Self::Block(v) => Self::Block(v.clone()),
            Self::Loop(v) => Self::Loop(v.clone()),
            Self::If(v) => Self::If(v.clone()),
            Self::Br(v) => Self::Br(v.clone()),
            Self::BrIf(v) => Self::BrIf(v.clone()),
            Self::BrTable(v) => Self::BrTable(v.clone()),
            Self::Return => Self::Return,
            Self::Call(v) => Self::Call(v.clone()),
            Self::CallIndirect(v) => Self::CallIndirect(v.clone()),
            Self::Drop => Self::Drop,
            Self::Select => Self::Select,
            Self::LocalGet(v) => Self::LocalGet(v.clone()),
            Self::LocalSet(v) => Self::LocalSet(v.clone()),
            Self::LocalTee(v) => Self::LocalTee(v.clone()),
            Self::GlobalGet(v) => Self::GlobalGet(v.clone()),
            Self::GlobalSet(v) => Self::GlobalSet(v.clone()),
            Self::I32Load(v) => Self::I32Load(v.clone()),
            Self::I64Load(v) => Self::I64Load(v.clone()),
            Self::F32Load(v) => Self::F32Load(v.clone()),
            Self::F64Load(v) => Self::F64Load(v.clone()),
            Self::I32Load8S(v) => Self::I32Load8S(v.clone()),
            Self::I32Load8U(v) => Self::I32Load8U(v.clone()),
            Self::I32Load16S(v) => Self::I32Load16S(v.clone()),
            Self::I32Load16U(v) => Self::I32Load16U(v.clone()),
            Self::I64Load8S(v) => Self::I64Load8S(v.clone()),
            Self::I64Load8U(v) => Self::I64Load8U(v.clone()),
            Self::I64Load16S(v) => Self::I64Load16S(v.clone()),
            Self::I64Load16U(v) => Self::I64Load16U(v.clone()),
            Self::I64Load32S(v) => Self::I64Load32S(v.clone()),
            Self::I64Load32U(v) => Self::I64Load32U(v.clone()),
            Self::I32Store(v) => Self::I32Store(v.clone()),
            Self::I64Store(v) => Self::I64Store(v.clone()),
            Self::F32Store(v) => Self::F32Store(v.clone()),
            Self::F64Store(v) => Self::F64Store(v.clone()),
            Self::I32Store8(v) => Self::I32Store8(v.clone()),
            Self::I32Store16(v) => Self::I32Store16(v.clone()),
            Self::I64Store8(v) => Self::I64Store8(v.clone()),
            Self::I64Store16(v) => Self::I64Store16(v.clone()),
            Self::I64Store32(v) => Self::I64Store32(v.clone()),
            Self::MemorySize => Self::MemorySize,
            Self::MemoryGrow => Self::MemoryGrow,
            Self::I32Const(v) => Self::I32Const(v.clone()),
            Self::I64Const(v) => Self::I64Const(v.clone()),
            Self::F32Const(v) => Self::F32Const(v.clone()),
            Self::F64Const(v) => Self::F64Const(v.clone()),
            Self::I32Eqz => Self::I32Eqz,
            Self::I32Eq => Self::I32Eq,
            Self::I32Ne => Self::I32Ne,
            Self::I32LtS => Self::I32LtS,
            Self::I32LtU => Self::I32LtU,
            Self::I32GtS => Self::I32GtS,
            Self::I32GtU => Self::I32GtU,
            Self::I32LeS => Self::I32LeS,
            Self::I32LeU => Self::I32LeU,
            Self::I32GeS => Self::I32GeS,
            Self::I32GeU => Self::I32GeU,
            Self::I64Eqz => Self::I64Eqz,
            Self::I64Eq => Self::I64Eq,
            Self::I64Ne => Self::I64Ne,
            Self::I64LtS => Self::I64LtS,
            Self::I64LtU => Self::I64LtU,
            Self::I64GtS => Self::I64GtS,
            Self::I64GtU => Self::I64GtU,
            Self::I64LeS => Self::I64LeS,
            Self::I64LeU => Self::I64LeU,
            Self::I64GeS => Self::I64GeS,
            Self::I64GeU => Self::I64GeU,
            Self::F32Eq => Self::F32Eq,
            Self::F32Ne => Self::F32Ne,
            Self::F32Lt => Self::F32Lt,
            Self::F32Gt => Self::F32Gt,
            Self::F32Le => Self::F32Le,
            Self::F32Ge => Self::F32Ge,
            Self::F64Eq => Self::F64Eq,
            Self::F64Ne => Self::F64Ne,
            Self::F64Lt => Self::F64Lt,
            Self::F64Gt => Self::F64Gt,
            Self::F64Le => Self::F64Le,
            Self::F64Ge => Self::F64Ge,
            Self::I32Clz => Self::I32Clz,
            Self::I32Ctz => Self::I32Ctz,
            Self::I32Popcnt => Self::I32Popcnt,
            Self::I32Add => Self::I32Add,
            Self::I32Sub => Self::I32Sub,
            Self::I32Mul => Self::I32Mul,
            Self::I32DivS => Self::I32DivS,
            Self::I32DivU => Self::I32DivU,
            Self::I32RemS => Self::I32RemS,
            Self::I32RemU => Self::I32RemU,
            Self::I32And => Self::I32And,
            Self::I32Or => Self::I32Or,
            Self::I32Xor => Self::I32Xor,
            Self::I32Shl => Self::I32Shl,
            Self::I32ShrS => Self::I32ShrS,
            Self::I32ShrU => Self::I32ShrU,
            Self::I32Rotl => Self::I32Rotl,
            Self::I32Rotr => Self::I32Rotr,
            Self::I64Clz => Self::I64Clz,
            Self::I64Ctz => Self::I64Ctz,
            Self::I64Popcnt => Self::I64Popcnt,
            Self::I64Add => Self::I64Add,
            Self::I64Sub => Self::I64Sub,
            Self::I64Mul => Self::I64Mul,
            Self::I64DivS => Self::I64DivS,
            Self::I64DivU => Self::I64DivU,
            Self::I64RemS => Self::I64RemS,
            Self::I64RemU => Self::I64RemU,
            Self::I64And => Self::I64And,
            Self::I64Or => Self::I64Or,
            Self::I64Xor => Self::I64Xor,
            Self::I64Shl => Self::I64Shl,
            Self::I64ShrS => Self::I64ShrS,
            Self::I64ShrU => Self::I64ShrU,
            Self::I64Rotl => Self::I64Rotl,
            Self::I64Rotr => Self::I64Rotr,
            Self::F32Abs => Self::F32Abs,
            Self::F32Neg => Self::F32Neg,
            Self::F32Ceil => Self::F32Ceil,
            Self::F32Floor => Self::F32Floor,
            Self::F32Trunc => Self::F32Trunc,
            Self::F32Nearest => Self::F32Nearest,
            Self::F32Sqrt => Self::F32Sqrt,
            Self::F32Add => Self::F32Add,
            Self::F32Sub => Self::F32Sub,
            Self::F32Mul => Self::F32Mul,
            Self::F32Div => Self::F32Div,
            Self::F32Min => Self::F32Min,
            Self::F32Max => Self::F32Max,
            Self::F32Copysign => Self::F32Copysign,
            Self::F64Abs => Self::F64Abs,
            Self::F64Neg => Self::F64Neg,
            Self::F64Ceil => Self::F64Ceil,
            Self::F64Floor => Self::F64Floor,
            Self::F64Trunc => Self::F64Trunc,
            Self::F64Nearest => Self::F64Nearest,
            Self::F64Sqrt => Self::F64Sqrt,
            Self::F64Add => Self::F64Add,
            Self::F64Sub => Self::F64Sub,
            Self::F64Mul => Self::F64Mul,
            Self::F64Div => Self::F64Div,
            Self::F64Min => Self::F64Min,
            Self::F64Max => Self::F64Max,
            Self::F64Copysign => Self::F64Copysign,
            Self::I32WrapI64 => Self::I32WrapI64,
            Self::I32TruncF32S => Self::I32TruncF32S,
            Self::I32TruncF32U => Self::I32TruncF32U,
            Self::I32TruncF64S => Self::I32TruncF64S,
            Self::I32TruncF64U => Self::I32TruncF64U,
            Self::I64ExtendI32S => Self::I64ExtendI32S,
            Self::I64ExtendI32U => Self::I64ExtendI32U,
            Self::I64TruncF32S => Self::I64TruncF32S,
            Self::I64TruncF32U => Self::I64TruncF32U,
            Self::I64TruncF64S => Self::I64TruncF64S,
            Self::I64TruncF64U => Self::I64TruncF64U,
            Self::F32ConvertI32S => Self::F32ConvertI32S,
            Self::F32ConvertI32U => Self::F32ConvertI32U,
            Self::F32ConvertI64S => Self::F32ConvertI64S,
            Self::F32ConvertI64U => Self::F32ConvertI64U,
            Self::F32DemoteF64 => Self::F32DemoteF64,
            Self::F64ConvertI32S => Self::F64ConvertI32S,
            Self::F64ConvertI32U => Self::F64ConvertI32U,
            Self::F64ConvertI64S => Self::F64ConvertI64S,
            Self::F64ConvertI64U => Self::F64ConvertI64U,
            Self::F64PromoteF32 => Self::F64PromoteF32,
            Self::I32ReinterpretF32 => Self::I32ReinterpretF32,
            Self::I64ReinterpretF64 => Self::I64ReinterpretF64,
            Self::F32ReinterpretI32 => Self::F32ReinterpretI32,
            Self::F64ReinterpretI64 => Self::F64ReinterpretI64,
            Self::SignExtension(v) => Self::SignExtension(v.clone()),
        }
    }
}

pub struct BlockInstr<V: VectorFactory> {
    pub blocktype: Blocktype,
    pub instrs: V::Vector<Instr<V>>,
}

impl<V: VectorFactory> Decode for BlockInstr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = Blocktype::decode(reader)?;
        let mut instrs = V::create_vector(None);
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self {
            blocktype: block_type,
            instrs,
        })
    }
}

impl<V: VectorFactory> Debug for BlockInstr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlockInstr")
            .field("blocktype", &self.blocktype)
            .field("instrs", &self.instrs.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for BlockInstr<V> {
    fn clone(&self) -> Self {
        Self {
            blocktype: self.blocktype.clone(),
            instrs: V::clone_vector(&self.instrs),
        }
    }
}

pub struct LoopInstr<V: VectorFactory> {
    pub blocktype: Blocktype,
    pub instrs: V::Vector<Instr<V>>,
}

impl<V: VectorFactory> Decode for LoopInstr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = Blocktype::decode(reader)?;
        let mut instrs = V::create_vector(None);
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self {
            blocktype: block_type,
            instrs,
        })
    }
}

impl<V: VectorFactory> Debug for LoopInstr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LoopInstr")
            .field("blocktype", &self.blocktype)
            .field("instrs", &self.instrs.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for LoopInstr<V> {
    fn clone(&self) -> Self {
        Self {
            blocktype: self.blocktype.clone(),
            instrs: V::clone_vector(&self.instrs),
        }
    }
}

pub struct IfInstr<V: VectorFactory> {
    pub blocktype: Blocktype,
    pub then_instrs: V::Vector<Instr<V>>,
    pub else_instrs: V::Vector<Instr<V>>,
}

impl<V: VectorFactory> Decode for IfInstr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let block_type = Blocktype::decode(reader)?;
        let mut then_instrs = V::create_vector(None);
        let mut else_instrs = V::create_vector(None);

        loop {
            let b = reader.peek_u8()?;
            if b == 0x0B {
                reader.read_u8()?;
                return Ok(Self {
                    blocktype: block_type,
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
            blocktype: block_type,
            then_instrs,
            else_instrs,
        })
    }
}

impl<V: VectorFactory> Debug for IfInstr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IfInstr")
            .field("blocktype", &self.blocktype)
            .field("then_instrs", &self.then_instrs.as_ref())
            .field("else_instrs", &self.else_instrs.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for IfInstr<V> {
    fn clone(&self) -> Self {
        Self {
            blocktype: self.blocktype.clone(),
            then_instrs: V::clone_vector(&self.then_instrs),
            else_instrs: V::clone_vector(&self.else_instrs),
        }
    }
}

pub struct BrTableInstr<V: VectorFactory> {
    pub labels: V::Vector<Labelidx>,
}

impl<V: VectorFactory> Decode for BrTableInstr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let n = reader.read_u32()? as usize + 1;
        let mut labels = V::create_vector(Some(n));
        for _ in 0..n {
            labels.push(Labelidx::decode(reader)?);
        }
        Ok(Self { labels })
    }
}

impl<V: VectorFactory> Debug for BrTableInstr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BrTableInstr")
            .field("labels", &self.labels.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for BrTableInstr<V> {
    fn clone(&self) -> Self {
        Self {
            labels: V::clone_vector(&self.labels),
        }
    }
}
