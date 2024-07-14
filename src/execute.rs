use crate::{
    components::{Blocktype, Funcidx, Functype, Importdesc, Localidx},
    instance::FuncInst,
    instructions::Instr,
    Env, GlobalVal, HostFunc, Module, Val, Vector, VectorFactory, PAGE_SIZE,
};
use core::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum ExecuteError {
    NotExportedFunction,
    UnresolvedImport { index: usize },
    InvalidImportedMem,
    InvalidImportedTable,
    InvalidData { index: usize },
    InvalidElem { index: usize },
    InvalidGlobal { index: usize },
    InvalidMemidx,
    InvalidFuncidx,
    InvalidTypeidx,
    InvalidFuncArgs,
    Trapped,
}

impl Display for ExecuteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotExportedFunction => write!(f, "Not exported function"),
            Self::UnresolvedImport { index } => write!(f, "Unresolved import: {}", index),
            Self::InvalidImportedMem => write!(f, "Invalid imported memory"),
            Self::InvalidImportedTable => write!(f, "Invalid imported table"),
            Self::InvalidData { index } => write!(f, "Invalid data: {}", index),
            Self::InvalidElem { index } => write!(f, "Invalid elem: {}", index),
            Self::InvalidGlobal { index } => write!(f, "Invalid global: {}", index),
            Self::InvalidMemidx => write!(f, "Invalid memidx"),
            Self::InvalidFuncidx => write!(f, "Invalid funcidx"),
            Self::InvalidTypeidx => write!(f, "Invalid typeidx"),
            Self::InvalidFuncArgs => write!(f, "Invalid function arguments"),
            Self::Trapped => write!(f, "Trapped"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecuteError {}

pub struct Executor<V: VectorFactory> {
    pub mem: V::Vector<u8>,
    pub table: V::Vector<Option<Funcidx>>,
    pub globals: V::Vector<GlobalVal>,
    pub locals: V::Vector<Val>,
    pub values: V::Vector<Val>,
    pub current_frame: Frame,
    pub current_block: Block,
}

impl<V: VectorFactory> Executor<V> {
    pub fn new(
        mem: V::Vector<u8>,
        table: V::Vector<Option<Funcidx>>,
        globals: V::Vector<GlobalVal>,
    ) -> Self {
        Self {
            mem,
            table,
            globals,
            locals: V::create_vector(None),
            values: V::create_vector(None),
            current_frame: Frame::default(),
            current_block: Block::default(),
        }
    }

    pub fn enter_frame(&mut self, ty: &Functype<V>) -> Frame {
        let locals_start = self.locals.len();
        for _ in 0..ty.params.len() {
            let v = self.pop_value();
            self.locals.push(v);
        }
        self.locals[locals_start..].reverse();
        let values_start = self.values.len();

        let prev = self.current_frame;
        self.current_frame = Frame {
            arity: ty.result.len(),
            locals_start,
            values_start,
        };
        prev
    }

    // TODO: delete unused parameter
    pub fn exit_frame(&mut self, _ty: &Functype<V>, prev: Frame) {
        let frame = self.current_frame;

        assert!(frame.locals_start <= self.locals.len());
        self.locals.truncate(frame.locals_start);

        self.values
            .remove_range(frame.values_start..self.values.len() - frame.arity);

        self.current_frame = prev;
    }

    pub fn enter_block(&mut self, ty: Blocktype) -> Block {
        assert!(matches!(ty, Blocktype::Empty)); // TODO

        let prev = self.current_block;
        self.current_block = Block {
            arity: ty.arity(),
            values_start: self.values.len(),
        };
        prev
    }

    // TODO: rename skipped
    pub fn exit_block(&mut self, ty: Blocktype, skipped: bool, prev: Block) {
        assert!(matches!(ty, Blocktype::Empty)); // TODO

        let block = self.current_block;

        if !skipped {
            self.values
                .remove_range(block.values_start..self.values.len() - block.arity);
        }

        self.current_block = prev;
    }

    pub fn set_local(&mut self, i: Localidx, v: Val) {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i] = v;
    }

    pub fn get_local(&self, i: Localidx) -> Val {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i]
    }

    pub fn push_value(&mut self, v: Val) {
        self.values.push(v);
    }

    pub fn pop_value(&mut self) -> Val {
        self.values.pop().expect("unreachable")
    }

    pub fn pop_value_i32(&mut self) -> i32 {
        let Some(Val::I32(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn pop_value_i64(&mut self) -> i64 {
        let Some(Val::I64(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn pop_value_u64(&mut self) -> u64 {
        let Some(Val::I64(v)) = self.values.pop() else {
            unreachable!();
        };
        v as u64
    }

    pub fn pop_value_u32(&mut self) -> u32 {
        let Some(Val::I32(v)) = self.values.pop() else {
            unreachable!();
        };
        v as u32
    }

    pub fn pop_value_f32(&mut self) -> f32 {
        let Some(Val::F32(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn pop_value_f64(&mut self) -> f64 {
        let Some(Val::F64(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn call_function<H: HostFunc>(
        &mut self,
        func_idx: Funcidx,
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<(), ExecuteError> {
        // TODO: Add validation phase
        let func = funcs
            .get_mut(func_idx.get())
            .ok_or(ExecuteError::InvalidFuncidx)?;
        let func_type = func.get_type(module).ok_or(ExecuteError::InvalidFuncidx)?; // TODO: change reason

        let prev_frame = self.enter_frame(func_type);
        match func {
            FuncInst::Imported {
                imports_index,
                host_func,
            } => {
                let Importdesc::Func(typeidx) = module.imports()[*imports_index].desc else {
                    unreachable!()
                };
                let func_type = &module.types()[typeidx.get()];
                let args_end = self.locals.len();
                let args_start = args_end - func_type.params.len();
                let args = &self.locals[args_start..args_end];

                let mut env = Env {
                    mem: &mut self.mem,
                    globals: &mut self.globals,
                };
                let value = host_func.invoke(args, &mut env);

                // TODO: check return value type
                if let Some(v) = value {
                    self.values.push(v);
                }
            }
            FuncInst::Module { funcs_index } => {
                let func = module
                    .funcs()
                    .get(*funcs_index)
                    .ok_or(ExecuteError::InvalidFuncidx)?;
                for v in func.locals.iter().copied().map(Val::zero) {
                    self.locals.push(v);
                }
                self.execute_instrs(func.body.instrs(), 0, funcs, module)?;
            }
        };
        self.exit_frame(func_type, prev_frame);
        Ok(())
    }

    pub fn execute_instrs<H: HostFunc>(
        &mut self,
        instrs: &[Instr<V>],
        level: usize, // TODO: label
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<Option<usize>, ExecuteError> {
        for instr in instrs {
            match instr {
                // Control Instructions
                Instr::Unreachable => return Err(ExecuteError::Trapped),
                Instr::Nop => {}
                Instr::Block(block) => {
                    let prev_block = self.enter_block(block.blocktype);
                    let return_level =
                        self.execute_instrs(&block.instrs, level + 1, funcs, module)?;
                    let skipped = return_level.map_or(false, |return_level| return_level <= level);
                    self.exit_block(block.blocktype, skipped, prev_block);
                    if skipped {
                        return Ok(return_level);
                    }
                }
                Instr::Loop(block) => {
                    let current_level = level + 1;
                    let blocktype = Blocktype::Empty;
                    let prev_block = self.enter_block(blocktype);
                    loop {
                        let return_level =
                            self.execute_instrs(&block.instrs, current_level, funcs, module)?;
                        if return_level == Some(current_level) {
                            continue;
                        }
                        let skipped =
                            return_level.map_or(false, |return_level| return_level <= level);
                        self.exit_block(blocktype, skipped, prev_block);
                        if skipped {
                            return Ok(return_level);
                        }
                        break;
                    }
                }
                Instr::If(block) => {
                    let c = self.pop_value_i32();
                    let prev_block = self.enter_block(block.blocktype);
                    let return_level = if c != 0 {
                        self.execute_instrs(&block.then_instrs, level + 1, funcs, module)?
                    } else {
                        self.execute_instrs(&block.else_instrs, level + 1, funcs, module)?
                    };
                    let skipped = return_level.map_or(false, |return_level| return_level <= level);
                    self.exit_block(block.blocktype, skipped, prev_block);
                    if skipped {
                        return Ok(return_level);
                    }
                }
                Instr::Br(label) => {
                    return Ok(Some(level - label.get()));
                }
                Instr::BrIf(label) => {
                    let c = self.pop_value_i32();
                    if c != 0 {
                        return Ok(Some(level - label.get()));
                    }
                }
                Instr::BrTable(table) => {
                    let i = self.pop_value_i32() as usize;
                    let label = table
                        .labels
                        .get(i)
                        .unwrap_or_else(|| table.labels.last().expect("unreachable"));
                    return Ok(Some(level - label.get()));
                }
                Instr::Return => {
                    return Ok(Some(0));
                }
                Instr::Call(funcidx) => {
                    self.call_function(*funcidx, funcs, module)?;
                }
                Instr::CallIndirect(typeidx) => {
                    todo!("CallIndirect: {typeidx:?}");
                }

                // Parametric Instructions
                Instr::Drop => {
                    self.pop_value();
                }
                Instr::Select => {
                    let c = self.pop_value_i32();
                    let v0 = self.pop_value();
                    let v1 = self.pop_value();
                    self.push_value(if c != 0 { v0 } else { v1 });
                }

                // Variable Instructions
                Instr::LocalTee(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                    self.push_value(v);
                }
                Instr::LocalGet(idx) => {
                    let v = self.get_local(*idx);
                    self.push_value(v);
                }
                Instr::LocalSet(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                }
                Instr::GlobalGet(idx) => {
                    let v = self.globals[idx.get()].get();
                    self.push_value(v);
                }
                Instr::GlobalSet(idx) => {
                    let v = self.pop_value();
                    self.globals[idx.get()].set(v);
                }

                // Memory Instructions
                Instr::I32Load(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i32::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I32(v));
                }
                Instr::I64Load(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 8;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i64::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I64(v));
                }
                Instr::F32Load(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = f32::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::F32(v));
                }
                Instr::F64Load(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 8;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = f64::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::F64(v));
                }
                Instr::I32Load8S(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = self.mem[i] as i8 as i32;
                    self.values.push(Val::I32(v));
                }
                Instr::I32Load8U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = self.mem[i] as i32;
                    self.values.push(Val::I32(v));
                }
                Instr::I32Load16S(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i16::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I32(v as i32));
                }
                Instr::I32Load16U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = u16::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I32(v as i32));
                }
                Instr::I64Load8S(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = self.mem[i] as i8 as i64;
                    self.values.push(Val::I64(v));
                }
                Instr::I64Load8U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = self.mem[i] as i64;
                    self.values.push(Val::I64(v));
                }
                Instr::I64Load16S(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i16::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I64(v as i64));
                }
                Instr::I64Load16U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = u16::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I64(v as i64));
                }
                Instr::I64Load32S(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i32::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I64(v as i64));
                }
                Instr::I64Load32U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = u32::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I64(v as i64));
                }
                Instr::I32Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::I64Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::F32Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::F64Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::I32Store8(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = v.as_i32().ok_or(ExecuteError::Trapped)? as u8; // TODO:
                    self.mem[i] = v;
                }
                Instr::I32Store16(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }

                    let v = v.as_i32().ok_or(ExecuteError::Trapped)? as i16; // TODO:
                    (&mut self.mem[start..end]).copy_from_slice(&v.to_le_bytes());
                }
                Instr::I64Store8(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = v.as_i64().ok_or(ExecuteError::Trapped)? as u8; // TODO:
                    self.mem[i] = v;
                }
                Instr::I64Store16(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }

                    let v = v.as_i64().ok_or(ExecuteError::Trapped)? as i16; // TODO:
                    (&mut self.mem[start..end]).copy_from_slice(&v.to_le_bytes());
                }
                Instr::I64Store32(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }

                    let v = v.as_i64().ok_or(ExecuteError::Trapped)? as i32; // TODO:
                    (&mut self.mem[start..end]).copy_from_slice(&v.to_le_bytes());
                }
                Instr::MemorySize => {
                    let size = self.mem.len() / PAGE_SIZE;
                    self.push_value(Val::I32(size as i32));
                }
                Instr::MemoryGrow => {
                    let delta = self.pop_value_i32();
                    let max = module.mem().and_then(|m| m.limits.max).unwrap_or_default();
                    let current = self.mem.len() / PAGE_SIZE;
                    let new = current + delta as usize;
                    if new <= max as usize {
                        // TODO: use resize()
                        for _ in 0..delta as usize * PAGE_SIZE {
                            self.mem.push(0);
                        }
                        self.push_value(Val::I32(current as i32));
                    } else {
                        self.push_value(Val::I32(-1));
                    };
                }

                // Numeric Instructions
                Instr::I32Const(v) => self.push_value(Val::I32(*v)),
                Instr::I64Const(v) => self.push_value(Val::I64(*v)),
                Instr::F32Const(v) => self.push_value(Val::F32(*v)),
                Instr::F64Const(v) => self.push_value(Val::F64(*v)),
                Instr::I32Eqz => self.apply_unop_cmp_i32(|v| v == 0),
                Instr::I32Eq => self.apply_binop_cmp_i32(|v0, v1| v0 == v1),
                Instr::I32Ne => self.apply_binop_cmp_i32(|v0, v1| v0 != v1),
                Instr::I32LtS => self.apply_binop_cmp_i32(|v0, v1| v0 < v1),
                Instr::I32LtU => self.apply_binop_cmp_u32(|v0, v1| v0 < v1),
                Instr::I32GtS => self.apply_binop_cmp_i32(|v0, v1| v0 > v1),
                Instr::I32GtU => self.apply_binop_cmp_u32(|v0, v1| v0 > v1),
                Instr::I32LeS => self.apply_binop_cmp_i32(|v0, v1| v0 <= v1),
                Instr::I32LeU => self.apply_binop_cmp_u32(|v0, v1| v0 <= v1),
                Instr::I32GeS => self.apply_binop_cmp_i32(|v0, v1| v0 >= v1),
                Instr::I32GeU => self.apply_binop_cmp_u32(|v0, v1| v0 >= v1),
                Instr::I64Eqz => self.apply_unop_cmp_i64(|v| v == 0),
                Instr::I64Eq => self.apply_binop_cmp_i64(|v0, v1| v0 == v1),
                Instr::I64Ne => self.apply_binop_cmp_i64(|v0, v1| v0 != v1),
                Instr::I64LtS => self.apply_binop_cmp_i64(|v0, v1| v0 < v1),
                Instr::I64LtU => self.apply_binop_cmp_u64(|v0, v1| v0 < v1),
                Instr::I64GtS => self.apply_binop_cmp_i64(|v0, v1| v0 > v1),
                Instr::I64GtU => self.apply_binop_cmp_u64(|v0, v1| v0 > v1),
                Instr::I64LeS => self.apply_binop_cmp_i64(|v0, v1| v0 <= v1),
                Instr::I64LeU => self.apply_binop_cmp_u64(|v0, v1| v0 <= v1),
                Instr::I64GeS => self.apply_binop_cmp_i64(|v0, v1| v0 >= v1),
                Instr::I64GeU => self.apply_binop_cmp_u64(|v0, v1| v0 >= v1),
                Instr::F32Eq => self.apply_binop_cmp_f32(|v0, v1| v0 == v1),
                Instr::F32Ne => self.apply_binop_cmp_f32(|v0, v1| v0 != v1),
                Instr::F32Lt => self.apply_binop_cmp_f32(|v0, v1| v0 < v1),
                Instr::F32Gt => self.apply_binop_cmp_f32(|v0, v1| v0 > v1),
                Instr::F32Le => self.apply_binop_cmp_f32(|v0, v1| v0 <= v1),
                Instr::F32Ge => self.apply_binop_cmp_f32(|v0, v1| v0 >= v1),
                Instr::F64Eq => self.apply_binop_cmp_f64(|v0, v1| v0 == v1),
                Instr::F64Ne => self.apply_binop_cmp_f64(|v0, v1| v0 != v1),
                Instr::F64Lt => self.apply_binop_cmp_f64(|v0, v1| v0 < v1),
                Instr::F64Gt => self.apply_binop_cmp_f64(|v0, v1| v0 > v1),
                Instr::F64Le => self.apply_binop_cmp_f64(|v0, v1| v0 <= v1),
                Instr::F64Ge => self.apply_binop_cmp_f64(|v0, v1| v0 >= v1),
                Instr::I32Clz => self.apply_unop_i32(|v| v.leading_zeros() as i32),
                Instr::I32Ctz => self.apply_unop_i32(|v| v.trailing_zeros() as i32),
                Instr::I32Popcnt => self.apply_unop_i32(|v| v.count_ones() as i32),
                Instr::I32Add => self.apply_binop_i32(|v0, v1| v0 + v1),
                Instr::I32Sub => self.apply_binop_i32(|v0, v1| v0 - v1),
                Instr::I32Mul => self.apply_binop_i32(|v0, v1| v0 * v1),
                Instr::I32DivS => self.apply_binop_i32(|v0, v1| v0.wrapping_div(v1)), // TODO: wrapping?
                Instr::I32DivU => self.apply_binop_u32(|v0, v1| v0.wrapping_div(v1)), // TODO: wrapping?
                Instr::I32RemS => self.apply_binop_i32(|v0, v1| v0.wrapping_rem(v1)), // TODO: wrapping?
                Instr::I32RemU => self.apply_binop_u32(|v0, v1| v0.wrapping_rem(v1)), // TODO: wrapping?
                Instr::I32And => self.apply_binop_i32(|v0, v1| v0 & v1),
                Instr::I32Or => self.apply_binop_i32(|v0, v1| v0 | v1),
                Instr::I32Xor => self.apply_binop_i32(|v0, v1| v0 ^ v1),
                Instr::I32Shl => self.apply_binop_i32(|v0, v1| v0.wrapping_shl(v1 as u32)), // TODO: wrapping?
                Instr::I32ShrS => self.apply_binop_i32(|v0, v1| v0.wrapping_shr(v1 as u32)), // TODO: wrapping?
                Instr::I32ShrU => self.apply_binop_u32(|v0, v1| v0.wrapping_shr(v1 as u32)), // TODO: wrapping?
                Instr::I32Rotl => self.apply_binop_i32(|v0, v1| v0.rotate_left(v1 as u32)),
                Instr::I32Rotr => self.apply_binop_i32(|v0, v1| v0.rotate_right(v1 as u32)),
                Instr::I64Clz => self.apply_unop_i64(|v| v.leading_zeros() as i64),
                Instr::I64Ctz => self.apply_unop_i64(|v| v.trailing_zeros() as i64),
                Instr::I64Popcnt => self.apply_unop_i64(|v| v.count_ones() as i64),
                Instr::I64Add => self.apply_binop_i64(|v0, v1| v0 + v1),
                Instr::I64Sub => self.apply_binop_i64(|v0, v1| v0 - v1),
                Instr::I64Mul => self.apply_binop_i64(|v0, v1| v0 * v1),
                Instr::I64DivS => self.apply_binop_i64(|v0, v1| v0.wrapping_div(v1)), // TODO: wrapping?
                Instr::I64DivU => self.apply_binop_u64(|v0, v1| v0.wrapping_div(v1)), // TODO: wrapping?
                Instr::I64RemS => self.apply_binop_i64(|v0, v1| v0.wrapping_rem(v1)), // TODO: wrapping?
                Instr::I64RemU => self.apply_binop_u64(|v0, v1| v0.wrapping_rem(v1)), // TODO: wrapping?
                Instr::I64And => self.apply_binop_i64(|v0, v1| v0 & v1),
                Instr::I64Or => self.apply_binop_i64(|v0, v1| v0 | v1),
                Instr::I64Xor => self.apply_binop_i64(|v0, v1| v0 ^ v1),
                Instr::I64Shl => self.apply_binop_i64(|v0, v1| v0.wrapping_shl(v1 as u32)), // TODO: wrapping?
                Instr::I64ShrS => self.apply_binop_i64(|v0, v1| v0.wrapping_shr(v1 as u32)), // TODO: wrapping?
                Instr::I64ShrU => self.apply_binop_u64(|v0, v1| v0.wrapping_shr(v1 as u32)), // TODO: wrapping?
                Instr::I64Rotl => self.apply_binop_i64(|v0, v1| v0.rotate_left(v1 as u32)),
                Instr::I64Rotr => self.apply_binop_i64(|v0, v1| v0.rotate_right(v1 as u32)),
                Instr::F32Abs => self.apply_unop_f32(|v| v.abs()),
                Instr::F32Neg => self.apply_unop_f32(|v| -v),
                Instr::F32Ceil => self.apply_unop_f32(|v| v.ceil()),
                Instr::F32Floor => self.apply_unop_f32(|v| v.floor()),
                Instr::F32Trunc => self.apply_unop_f32(|v| v.trunc()),
                Instr::F32Nearest => self.apply_unop_f32(|v| v.round()), // TODO: round?
                Instr::F32Sqrt => self.apply_unop_f32(|v| v.sqrt()),
                Instr::F32Add => self.apply_binop_f32(|v0, v1| v0 + v1),
                Instr::F32Sub => self.apply_binop_f32(|v0, v1| v0 - v1),
                Instr::F32Mul => self.apply_binop_f32(|v0, v1| v0 * v1),
                Instr::F32Div => self.apply_binop_f32(|v0, v1| v0 / v1),
                Instr::F32Min => self.apply_binop_f32(|v0, v1| v0.min(v1)),
                Instr::F32Max => self.apply_binop_f32(|v0, v1| v0.max(v1)),
                Instr::F32Copysign => self.apply_binop_f32(|v0, v1| v0.copysign(v1)),
                Instr::F64Abs => self.apply_unop_f64(|v| v.abs()),
                Instr::F64Neg => self.apply_unop_f64(|v| -v),
                Instr::F64Ceil => self.apply_unop_f64(|v| v.ceil()),
                Instr::F64Floor => self.apply_unop_f64(|v| v.floor()),
                Instr::F64Trunc => self.apply_unop_f64(|v| v.trunc()),
                Instr::F64Nearest => self.apply_unop_f64(|v| v.round()), // TODO: round?
                Instr::F64Sqrt => self.apply_unop_f64(|v| v.sqrt()),
                Instr::F64Add => self.apply_binop_f64(|v0, v1| v0 + v1),
                Instr::F64Sub => self.apply_binop_f64(|v0, v1| v0 - v1),
                Instr::F64Mul => self.apply_binop_f64(|v0, v1| v0 * v1),
                Instr::F64Div => self.apply_binop_f64(|v0, v1| v0 / v1),
                Instr::F64Min => self.apply_binop_f64(|v0, v1| v0.min(v1)),
                Instr::F64Max => self.apply_binop_f64(|v0, v1| v0.max(v1)),
                Instr::F64Copysign => self.apply_binop_f64(|v0, v1| v0.copysign(v1)),
                Instr::I32WrapI64 => self.convert_from_i64(|v| Val::I32(v as i32)),
                Instr::I32TruncF32S => self.convert_from_f32(|v| Val::I32(v.trunc() as i32)), // TODO: NaN, etc
                Instr::I32TruncF32U => self.convert_from_f32(|v| Val::I32(v.trunc() as i32)), // TODO: NaN, etc
                Instr::I32TruncF64S => self.convert_from_f64(|v| Val::I32(v.trunc() as i32)), // TODO: NaN, etc
                Instr::I32TruncF64U => self.convert_from_f64(|v| Val::I32(v.trunc() as i32)), // TODO: NaN, etc
                Instr::I64ExtendI32S => self.convert_from_i32(|v| Val::I64(v as i64)),
                Instr::I64ExtendI32U => self.convert_from_i32(|v| Val::I64(v as u32 as i64)),
                Instr::I64TruncF32S => self.convert_from_f32(|v| Val::I64(v.trunc() as i64)), // TODO: NaN, etc
                Instr::I64TruncF32U => self.convert_from_f32(|v| Val::I64(v.trunc() as i64)), // TODO: NaN, etc
                Instr::I64TruncF64S => self.convert_from_f64(|v| Val::I64(v.trunc() as i64)), // TODO: NaN, etc
                Instr::I64TruncF64U => self.convert_from_f64(|v| Val::I64(v.trunc() as i64)), // TODO: NaN, etc
                Instr::F32ConvertI32S => self.convert_from_i32(|v| Val::F32(v as f32)), // TODO
                Instr::F32ConvertI32U => self.convert_from_i32(|v| Val::F32(v as u32 as f32)), // TODO
                Instr::F32ConvertI64S => self.convert_from_i64(|v| Val::F32(v as f32)), // TODO
                Instr::F32ConvertI64U => self.convert_from_i64(|v| Val::F32(v as u64 as f32)), // TODO
                Instr::F32DemoteF64 => self.convert_from_f64(|v| Val::F32(v as f32)), // TODO
                Instr::F64ConvertI32S => self.convert_from_i32(|v| Val::F64(v as f64)), // TODO
                Instr::F64ConvertI32U => self.convert_from_i32(|v| Val::F64(v as u32 as f64)), // TODO
                Instr::F64ConvertI64S => self.convert_from_i64(|v| Val::F64(v as f64)), // TODO
                Instr::F64ConvertI64U => self.convert_from_i64(|v| Val::F64(v as u64 as f64)), // TODO
                Instr::F64PromoteF32 => self.convert_from_f32(|v| Val::F64(v as f64)),
                Instr::I32ReinterpretF32 => self.convert_from_f32(|v| Val::I32(v.to_bits() as i32)),
                Instr::I64ReinterpretF64 => self.convert_from_f64(|v| Val::I64(v.to_bits() as i64)),
                Instr::F32ReinterpretI32 => {
                    self.convert_from_i32(|v| Val::F32(f32::from_bits(v as u32)))
                }
                Instr::F64ReinterpretI64 => {
                    self.convert_from_i64(|v| Val::F64(f64::from_bits(v as u64)))
                }

                // Sign Extension
                #[cfg(feature = "sign_extension")]
                Instr::SignExtension(instr) => match instr {
                    crate::sign_extension::SignExtensionInstr::I32Extend8S => {
                        self.convert_from_i32(|v| Val::I32(v as i8 as i32))
                    }
                    crate::sign_extension::SignExtensionInstr::I32Extend16S => {
                        self.convert_from_i32(|v| Val::I32(v as i16 as i32))
                    }
                    crate::sign_extension::SignExtensionInstr::I64Extend8S => {
                        self.convert_from_i64(|v| Val::I64(v as i8 as i64))
                    }
                    crate::sign_extension::SignExtensionInstr::I64Extend16S => {
                        self.convert_from_i64(|v| Val::I64(v as i16 as i64))
                    }
                    crate::sign_extension::SignExtensionInstr::I64Extend32S => {
                        self.convert_from_i64(|v| Val::I64(v as i32 as i64))
                    }
                },
            }
        }
        Ok(None)
    }

    fn convert_from_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32) -> Val,
    {
        let v = self.pop_value_i32();
        self.push_value(f(v));
    }

    fn convert_from_i64<F>(&mut self, f: F)
    where
        F: FnOnce(i64) -> Val,
    {
        let v = self.pop_value_i64();
        self.push_value(f(v));
    }

    fn convert_from_f32<F>(&mut self, f: F)
    where
        F: FnOnce(f32) -> Val,
    {
        let v = self.pop_value_f32();
        self.push_value(f(v));
    }

    fn convert_from_f64<F>(&mut self, f: F)
    where
        F: FnOnce(f64) -> Val,
    {
        let v = self.pop_value_f64();
        self.push_value(f(v));
    }

    fn apply_unop_f32<F>(&mut self, f: F)
    where
        F: FnOnce(f32) -> f32,
    {
        let v = self.pop_value_f32();
        self.push_value(Val::F32(f(v)));
    }

    fn apply_binop_f32<F>(&mut self, f: F)
    where
        F: FnOnce(f32, f32) -> f32,
    {
        let v0 = self.pop_value_f32();
        let v1 = self.pop_value_f32();
        self.push_value(Val::F32(f(v1, v0)));
    }

    fn apply_unop_f64<F>(&mut self, f: F)
    where
        F: FnOnce(f64) -> f64,
    {
        let v = self.pop_value_f64();
        self.push_value(Val::F64(f(v)));
    }

    fn apply_binop_f64<F>(&mut self, f: F)
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let v0 = self.pop_value_f64();
        let v1 = self.pop_value_f64();
        self.push_value(Val::F64(f(v1, v0)));
    }

    fn apply_unop_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32) -> i32,
    {
        let v = self.pop_value_i32();
        self.push_value(Val::I32(f(v)));
    }

    fn apply_binop_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let v0 = self.pop_value_i32();
        let v1 = self.pop_value_i32();
        self.push_value(Val::I32(f(v1, v0)));
    }

    fn apply_binop_u32<F>(&mut self, f: F)
    where
        F: FnOnce(u32, u32) -> u32,
    {
        let v0 = self.pop_value_u32();
        let v1 = self.pop_value_u32();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_unop_i64<F>(&mut self, f: F)
    where
        F: FnOnce(i64) -> i64,
    {
        let v = self.pop_value_i64();
        self.push_value(Val::I64(f(v)));
    }

    fn apply_binop_i64<F>(&mut self, f: F)
    where
        F: FnOnce(i64, i64) -> i64,
    {
        let v0 = self.pop_value_i64();
        let v1 = self.pop_value_i64();
        self.push_value(Val::I64(f(v1, v0)));
    }

    fn apply_binop_u64<F>(&mut self, f: F)
    where
        F: FnOnce(u64, u64) -> u64,
    {
        let v0 = self.pop_value_u64();
        let v1 = self.pop_value_u64();
        self.push_value(Val::I64(f(v1, v0) as i64));
    }

    fn apply_unop_cmp_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32) -> bool,
    {
        let v = self.pop_value_i32();
        self.push_value(Val::I32(f(v) as i32));
    }

    fn apply_binop_cmp_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32, i32) -> bool,
    {
        let v0 = self.pop_value_i32();
        let v1 = self.pop_value_i32();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_binop_cmp_u32<F>(&mut self, f: F)
    where
        F: FnOnce(u32, u32) -> bool,
    {
        let v0 = self.pop_value_u32();
        let v1 = self.pop_value_u32();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_unop_cmp_i64<F>(&mut self, f: F)
    where
        F: FnOnce(i64) -> bool,
    {
        let v = self.pop_value_i64();
        self.push_value(Val::I32(f(v) as i32));
    }

    fn apply_binop_cmp_i64<F>(&mut self, f: F)
    where
        F: FnOnce(i64, i64) -> bool,
    {
        let v0 = self.pop_value_i64();
        let v1 = self.pop_value_i64();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_binop_cmp_u64<F>(&mut self, f: F)
    where
        F: FnOnce(u64, u64) -> bool,
    {
        let v0 = self.pop_value_u64();
        let v1 = self.pop_value_u64();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_binop_cmp_f32<F>(&mut self, f: F)
    where
        F: FnOnce(f32, f32) -> bool,
    {
        let v0 = self.pop_value_f32();
        let v1 = self.pop_value_f32();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }

    fn apply_binop_cmp_f64<F>(&mut self, f: F)
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let v0 = self.pop_value_f64();
        let v1 = self.pop_value_f64();
        self.push_value(Val::I32(f(v1, v0) as i32));
    }
}

impl<V: VectorFactory> Debug for Executor<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("Executor").finish_non_exhaustive()
    }
}

// TODO: Activation(?)
#[derive(Debug, Default, Clone, Copy)]
pub struct Frame {
    pub arity: usize,
    pub locals_start: usize,
    pub values_start: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub arity: usize,
    pub values_start: usize,
}

#[cfg(test)]
mod tests {
    use crate::{Env, FuncInst, HostFunc, Module, Resolve, StdVectorFactory, Val};

    #[test]
    fn control_flow_br_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/br
        //
        // (module
        //   ;; import the browser console object, you'll need to pass this in from JavaScript
        //   (import "console" "log" (func $log (param i32)))
        //
        //   ;; create a global variable and initialize it to 0
        //   (global $i (mut i32) (i32.const 0))
        //
        //   (func
        //     (loop $my_loop
        //
        //       ;; add one to $i
        //       global.get $i
        //       i32.const 1
        //       i32.add
        //       global.set $i
        //
        //       ;; log the current value of $i
        //       global.get $i
        //       call $log
        //
        //       ;; if $i is less than 10 branch to loop
        //       global.get $i
        //       i32.const 10
        //       i32.lt_s
        //       br_if $my_loop
        //
        //     )
        //   )
        //
        //   (start 1) ;; run the first function automatically
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 6, 6, 1, 127, 1, 65, 0,
            11, 8, 1, 1, 10, 25, 1, 23, 0, 3, 64, 35, 0, 65, 1, 106, 36, 0, 35, 0, 16, 0, 35, 0,
            65, 10, 72, 13, 0, 11, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(10, host_func.messages.len());
        for (i, m) in host_func.messages.iter().enumerate() {
            assert_eq!(Val::I32((i + 1) as i32), *m);
        }
    }

    #[test]
    fn control_flow_drop_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/Drop
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (func $main
        //     ;; load two values onto the stack
        //     i32.const 10
        //     i32.const 20
        //
        //     ;; drop the top item from the stack (`20`)
        //     drop
        //
        //     call $log ;; log the top value on the stack (`10`)
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 11, 1, 9, 0,
            65, 10, 65, 20, 26, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(10)][..], &host_func.messages);
    }

    #[test]
    fn control_flow_select_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/Select
        //
        // (module
        //   (func (export "select_simple") (result i32)
        //     ;; load two values onto the stack
        //     i32.const 10
        //     i32.const 20
        //
        //     ;; change to `1` (true) to get the first value (`10`)
        //     i32.const 0
        //     select
        //   )
        //   ;; (func (export "select_externref") (param $value externref) (param $condition i32) (result externref)
        //   ;;  ;; this is "select t", the explicitly typed variant
        //   ;;  ref.null extern
        //   ;;  local.get $value
        //   ;;  local.get $condition
        //   ;;  select (result externref)
        //   ;; )
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 127, 3, 2, 1, 0, 7, 17, 1, 13, 115,
            101, 108, 101, 99, 116, 95, 115, 105, 109, 112, 108, 101, 0, 0, 10, 11, 1, 9, 0, 65,
            10, 65, 20, 65, 0, 27, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let mut instance = module.instantiate(Resolver).expect("instantiate");

        let val = instance
            .invoke("select_simple", &[])
            .expect("invoke")
            .expect("result");
        assert_eq!(Val::I32(10), val);
    }

    #[test]
    fn control_flow_block_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/block
        //
        // (module
        //   ;; import the browser console object, you'll need to pass this in from JavaScript
        //   (import "console" "log" (func $log (param i32)))
        //
        //   ;; create a function that takes in a number as a param,
        //   ;; and logs that number if it's not equal to 100.
        //   (func (export "log_if_not_100") (param $num i32)
        //     (block $my_block
        //
        //       ;; $num is equal to 100
        //       local.get $num
        //       i32.const 100
        //       i32.eq
        //
        //       (if
        //         (then
        //
        //           ;; branch to the end of the block
        //           br $my_block
        //
        //         )
        //       )
        //
        //       ;; not reachable when $num is 100
        //       local.get $num
        //       call $log
        //
        //     )
        //   )
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 1, 127, 0, 2, 15, 1, 7, 99, 111, 110, 115,
            111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 0, 7, 18, 1, 14, 108, 111, 103, 95,
            105, 102, 95, 110, 111, 116, 95, 49, 48, 48, 0, 1, 10, 22, 1, 20, 0, 2, 64, 32, 0, 65,
            228, 0, 70, 4, 64, 12, 1, 11, 32, 0, 16, 0, 11, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let mut instance = module.instantiate(Resolver).expect("instantiate");

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(99)])
            .expect("invoke")
            .is_none());

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(100)])
            .expect("invoke")
            .is_none());

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(101)])
            .expect("invoke")
            .is_none());

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(99), Val::I32(101)][..], &host_func.messages);
    }

    #[test]
    fn memory_size_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Memory/Size
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (memory 2)
        //   (func $main
        //
        //     memory.size ;; get the memory size
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 5, 3, 1, 0, 2, 8, 1, 1,
            10, 8, 1, 6, 0, 63, 0, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(2)][..], &host_func.messages);
    }

    #[test]
    fn memory_grow_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Memory/Grow
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (memory 1 2) ;; default memory with one page and max of 2 pages
        //
        //   (func $main
        //     ;; grow default memory by 1 page
        //     i32.const 1
        //     memory.grow
        //     call $log ;; log the result (previous no. pages = 1)
        //
        //     ;; grow default memory, using an S-function
        //     (memory.grow (i32.const 1))
        //     call $log ;; log the result (-1: max is 2 pages for default memory declared above!)
        //   )
        //   (start $main) ;; call immediately on loading
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 5, 4, 1, 1, 1, 2, 8, 1, 1,
            10, 16, 1, 14, 0, 65, 1, 64, 0, 16, 0, 65, 1, 64, 0, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(1), Val::I32(-1)][..], &host_func.messages);
    }

    #[test]
    fn consts_test() {
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (func $main

        //     i32.const 10
        //     call $log

        //     i32.const -3
        //     call $log
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 12, 1, 10, 0,
            65, 10, 16, 0, 65, 125, 16, 0, 11,
        ];

        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(10), Val::I32(-3)][..], &host_func.messages);
    }

    #[test]
    fn numeric_reinterpret_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Reinterpret
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (func $main
        //     ;; the value `10000000_00000000_00000000_00000000` in binary
        //     ;; maps to `-0` as a floating point and to `-2147483648` as an integer.
        //
        //     f32.const -0 ;; push an f32 onto the stack
        //
        //     i32.reinterpret_f32 ;; reinterpret the bytes of the f32 as i32
        //
        //     call $log ;; log the result
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 12, 1, 10, 0,
            67, 0, 0, 0, 128, 188, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(-2147483648)][..], &host_func.messages);
    }

    #[test]
    fn numeric_truncate_float_to_int_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Truncate_float_to_int
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (func $main
        //
        //     f32.const 10.5 ;; push an f32 onto the stack
        //
        //     i32.trunc_f32_s ;; convert from f32 to signed i32 rounding towards zero (.5 will be lost)
        //
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 12, 1, 10, 0,
            67, 0, 0, 40, 65, 168, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(10)][..], &host_func.messages);
    }

    #[test]
    fn numeric_convert_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Convert
        // (module
        //   (import "console" "log" (func $log (param f32)))
        //   (func $main
        //
        //     i32.const 10 ;; push an i32 onto the stack
        //
        //     f32.convert_i32_s ;; convert from signed i32 to f32
        //
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 125, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 9, 1, 7, 0,
            65, 10, 178, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::F32(10.0)][..], &host_func.messages);
    }

    #[test]
    fn numeric_demote_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Demote
        //
        // (module
        //   (import "console" "log" (func $log (param f32)))
        //   (func $main
        //
        //     f64.const 10.5 ;; push an f64 onto the stack
        //
        //     f32.demote_f64 ;; demote from f64 to f32
        //
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 125, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 16, 1, 14, 0,
            68, 0, 0, 0, 0, 0, 0, 37, 64, 182, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::F32(10.5)][..], &host_func.messages);
    }

    #[test]
    fn numeric_promote_test() {
        // Based on https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Promote
        //
        // (module
        //   (import "console" "log" (func $log (param f64)))
        //   (func $main
        //
        //     f32.const 10.5 ;; push an f32 onto the stack
        //
        //     f64.promote_f32 ;; promote from f32 to f64
        //
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 124, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 12, 1, 10, 0,
            67, 0, 0, 40, 65, 187, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::F64(10.5)][..], &host_func.messages);
    }

    #[test]
    fn numeric_wrap_test() {
        // Based on https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Wrap
        //
        // (module
        //   (import "console" "log" (func $log (param i32)))
        //   (func $main
        //
        //     i64.const 10 ;; push an i64 onto the stack
        //
        //     i32.wrap_i64 ;; wrap from i64 to i32
        //
        //     call $log ;; log the result
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 9, 1, 7, 0,
            66, 10, 167, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(10)][..], &host_func.messages);
    }

    #[test]
    fn numeric_extend_test() {
        // Based on https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/Extend
        //
        // (module
        //   (import "console" "log" (func $log (param i64)))
        //   (func $main
        //
        //     i32.const 10 ;; push an i32 onto the stack
        //
        //     i64.extend_i32_s ;; sign-extend from i32 to i64
        //
        //     call $log ;; log the result
        //
        //   )
        //   (start $main)
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 126, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 8, 1, 1, 10, 9, 1, 7, 0,
            65, 10, 172, 16, 0, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I64(10)][..], &host_func.messages);
    }

    #[derive(Debug)]
    struct Resolver;

    impl Resolve for Resolver {
        type HostFunc = Log;

        fn resolve_func(
            &self,
            module: &str,
            name: &str,
            params: &[crate::components::Valtype],
            result: crate::components::Resulttype,
        ) -> Option<Self::HostFunc> {
            if module == "console" && name == "log" && params.len() > 0 && result.len() == 0 {
                Some(Log::default())
            } else {
                None
            }
        }
    }

    #[derive(Debug, Default)]
    struct Log {
        messages: Vec<Val>,
    }

    impl HostFunc for Log {
        fn invoke(&mut self, args: &[Val], _env: &mut Env) -> Option<Val> {
            self.messages.push(args[0]);
            None
        }
    }
}
