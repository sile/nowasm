use crate::{
    components::{Blocktype, Funcidx, Functype, Importdesc, Localidx},
    instance::FuncInst,
    instructions::Instr,
    Env, GlobalVal, HostFunc, Module, Val, Vector, VectorFactory,
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

    pub fn enter_frame(&mut self, ty: &Functype<V>, level: usize) -> Frame {
        let locals_start = self.locals.len();
        for _ in 0..ty.params.len() {
            let v = self.pop_value();
            self.locals.push(v);
        }
        self.locals[locals_start..].reverse();
        let values_start = self.values.len();

        let prev = self.current_frame;
        self.current_frame = Frame {
            level,
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

    pub fn call_function<H: HostFunc>(
        &mut self,
        func_idx: Funcidx,
        level: usize,
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<usize, ExecuteError> {
        // TODO: Add validation phase
        let func = funcs
            .get_mut(func_idx.get())
            .ok_or(ExecuteError::InvalidFuncidx)?;
        let func_type = func.get_type(module).ok_or(ExecuteError::InvalidFuncidx)?; // TODO: change reason

        let prev_frame = self.enter_frame(func_type, level);
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
                self.execute_instrs(func.body.instrs(), level + 1, funcs, module)?;
            }
        };
        self.exit_frame(func_type, prev_frame);
        Ok(level)
    }

    pub fn execute_instrs<H: HostFunc>(
        &mut self,
        instrs: &[Instr<V>],
        level: usize,
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<usize, ExecuteError> {
        for instr in instrs {
            match instr {
                Instr::Nop => {}
                Instr::Unreachable => return Err(ExecuteError::Trapped),
                Instr::GlobalSet(idx) => {
                    let v = self.pop_value();
                    self.globals[idx.get()].set(v);
                }
                Instr::GlobalGet(idx) => {
                    let v = self.globals[idx.get()].get();
                    self.push_value(v);
                }
                Instr::LocalSet(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                }
                Instr::LocalGet(idx) => {
                    let v = self.get_local(*idx);
                    self.push_value(v);
                }
                Instr::LocalTee(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                    self.push_value(v);
                }
                Instr::I32Const(v) => self.push_value(Val::I32(*v)),
                Instr::I64Const(v) => self.push_value(Val::I64(*v)),
                Instr::F32Const(v) => self.push_value(Val::F32(*v)),
                Instr::F64Const(v) => self.push_value(Val::F64(*v)),
                Instr::I32Sub => self.apply_binop_i32(|v0, v1| v0 - v1),
                Instr::I32Add => self.apply_binop_i32(|v0, v1| v0 + v1),
                Instr::I32Xor => self.apply_binop_i32(|v0, v1| v0 ^ v1),
                Instr::I32And => self.apply_binop_i32(|v0, v1| v0 & v1),
                Instr::I32LtS => self.apply_binop_i32(|v0, v1| if v0 < v1 { 1 } else { 0 }),
                Instr::I32LeS => self.apply_binop_i32(|v0, v1| if v0 <= v1 { 1 } else { 0 }),
                Instr::I32GtS => self.apply_binop_i32(|v0, v1| if v0 > v1 { 1 } else { 0 }),
                Instr::I32GeS => self.apply_binop_i32(|v0, v1| if v0 >= v1 { 1 } else { 0 }),
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
                Instr::Block(block) => {
                    let prev_block = self.enter_block(block.blocktype);
                    let return_level =
                        self.execute_instrs(&block.instrs, level + 1, funcs, module)?;
                    self.exit_block(block.blocktype, return_level <= level, prev_block);
                    if return_level <= level {
                        return Ok(return_level);
                    }
                }
                Instr::Br(label) => {
                    return Ok(level - label.get());
                }
                Instr::BrIf(label) => {
                    let c = self.pop_value_i32();
                    if c != 0 {
                        return Ok(level - label.get());
                    }
                }
                Instr::BrTable(table) => {
                    let i = self.pop_value_i32() as usize;
                    let label = table
                        .labels
                        .get(i)
                        .unwrap_or_else(|| table.labels.last().expect("unreachable"));
                    return Ok(level - label.get());
                }
                Instr::Return => {
                    return Ok(self.current_frame.level);
                }
                Instr::Call(funcidx) => {
                    self.call_function(*funcidx, level + 1, funcs, module)?;
                }
                _ => todo!("{instr:?}"),
            }
        }
        Ok(level)
    }

    fn apply_binop_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let v0 = self.pop_value_i32();
        let v1 = self.pop_value_i32();
        self.push_value(Val::I32(f(v1, v0)));
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
    pub level: usize,
    pub arity: usize,
    pub locals_start: usize,
    pub values_start: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub arity: usize,
    pub values_start: usize,
}
