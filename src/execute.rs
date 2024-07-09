use core::fmt::{Debug, Formatter};

use crate::{
    components::{Blocktype, Exportdesc, Funcidx, Functype, Localidx, Valtype},
    instructions::Instr,
    Module, Vector, VectorFactory,
};

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    NotExportedFunction,
    InvalidFuncidx,
    InvalidTypeidx,
    InvalidFuncArgs,
    InvalidGlobalInitializer,
    Trapped, // TODO: Add reason
}

// TODO: Rename
#[derive(Debug)]
pub struct State<V: VectorFactory> {
    pub mem: V::Vector<u8>,
    pub globals: V::Vector<Value>,
    pub locals: V::Vector<Value>,
    pub values: V::Vector<Value>,
    pub current_frame: Frame,
    pub current_block: Block,
}

impl<V: VectorFactory> State<V> {
    pub fn new(mem: V::Vector<u8>) -> Self {
        Self {
            mem,
            globals: V::create_vector(None),
            locals: V::create_vector(None),
            values: V::create_vector(None),
            current_frame: Frame::root(),
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

    pub fn set_local(&mut self, i: Localidx, v: Value) {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i] = v;
    }

    pub fn get_local(&self, i: Localidx) -> Value {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i]
    }

    pub fn push_value(&mut self, v: Value) {
        self.values.push(v);
    }

    pub fn pop_value(&mut self) -> Value {
        self.values.pop().expect("unreachable")
    }

    pub fn pop_value_i32(&mut self) -> i32 {
        let Some(Value::I32(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn call_function(
        &mut self,
        func_idx: Funcidx,
        module: &Module<V>,
    ) -> Result<usize, ExecutionError> {
        // TODO: check trapped flag

        // TODO: Add validation phase
        let func = module
            .funcs()
            .get(func_idx.get())
            .ok_or(ExecutionError::InvalidFuncidx)?;
        let func_type = module
            .types()
            .get(func.ty.get())
            .ok_or(ExecutionError::InvalidFuncidx)?; // TODO: change reason

        let prev_frame = self.enter_frame(func_type, 0);
        for v in func.locals.iter().copied().map(Value::zero) {
            self.locals.push(v);
        }
        let value = self.execute_instrs(func.body.instrs(), 0, module)?; // TODO: set trapped flag if needed
        self.exit_frame(func_type, prev_frame);
        Ok(value)
    }

    pub fn execute_instrs(
        &mut self,
        instrs: &[Instr<V>],
        level: usize,
        module: &Module<V>,
    ) -> Result<usize, ExecutionError> {
        for instr in instrs {
            match instr {
                Instr::Nop => {}
                Instr::Unreachable => return Err(ExecutionError::Trapped),
                Instr::GlobalSet(idx) => {
                    let v = self.pop_value();
                    self.globals[idx.get()] = v;
                }
                Instr::GlobalGet(idx) => {
                    let v = self.globals[idx.get()];
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
                Instr::I32Const(v) => self.push_value(Value::I32(*v)),
                Instr::I64Const(v) => self.push_value(Value::I64(*v)),
                Instr::F32Const(v) => self.push_value(Value::F32(*v)),
                Instr::F64Const(v) => self.push_value(Value::F64(*v)),
                Instr::I32Sub => self.apply_binop_i32(|v0, v1| v0 - v1),
                Instr::I32Add => self.apply_binop_i32(|v0, v1| v0 + v1),
                Instr::I32Xor => self.apply_binop_i32(|v0, v1| v0 ^ v1),
                Instr::I32And => self.apply_binop_i32(|v0, v1| v0 & v1),
                Instr::I32LtS => self.apply_binop_i32(|v0, v1| if v0 < v1 { 1 } else { 0 }),
                Instr::I32Store(arg) => {
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecutionError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::Block(block) => {
                    let prev_block = self.enter_block(block.blocktype);
                    let return_level = self.execute_instrs(&block.instrs, level + 1, module)?;
                    self.exit_block(block.blocktype, return_level <= level, prev_block);
                    if return_level <= level {
                        return Ok(return_level);
                    }
                }
                Instr::BrIf(label) => {
                    let c = self.pop_value_i32();
                    if c != 0 {
                        dbg!(label);
                        todo!();
                    }
                }
                Instr::Return => {
                    return Ok(self.current_frame.level);
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
        self.push_value(Value::I32(f(v1, v0)));
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

impl Frame {
    pub fn root() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub arity: usize,
    pub values_start: usize,
}

pub struct ModuleInstanceOptions<V: VectorFactory> {
    pub mem: Option<V::Vector<u8>>,
}

impl<V: VectorFactory> Default for ModuleInstanceOptions<V> {
    fn default() -> Self {
        Self { mem: None }
    }
}

impl<V: VectorFactory> Clone for ModuleInstanceOptions<V> {
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.as_ref().map(V::clone_vector),
        }
    }
}

impl<V: VectorFactory> Debug for ModuleInstanceOptions<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleInstanceOptions")
            .field("mem", &self.mem.as_ref().map(|v| v.as_ref()))
            .finish()
    }
}

// TODO: #[derive(Debug)]
pub struct ModuleInstance<V: VectorFactory> {
    pub module: Module<V>,
    pub state: State<V>,
}

impl<V: VectorFactory> ModuleInstance<V> {
    // TODO:
    pub(crate) fn new(
        module: Module<V>,

        // TODO: Use builder
        mem: V::Vector<u8>,
    ) -> Result<Self, ExecutionError> {
        if module.start().is_some() {
            todo!()
        }

        // TODO: check mem (min, max, pagesize)
        let mut state = State::<V>::new(mem);

        for global in module.globals().iter() {
            state.globals.push(global.init()?);
        }

        Ok(Self { module, state })
    }

    pub fn invoke(
        &mut self,
        function_name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, ExecutionError> {
        let Some(export) = self.module.exports().iter().find(|export| {
            matches!(export.desc, Exportdesc::Func(_)) && function_name == export.name.as_str()
        }) else {
            return Err(ExecutionError::NotExportedFunction);
        };
        let Exportdesc::Func(func_idx) = export.desc else {
            unreachable!();
        };

        let func = self
            .module
            .funcs()
            .get(func_idx.get())
            .ok_or(ExecutionError::InvalidFuncidx)?;
        let func_type = self
            .module
            .types()
            .get(func.ty.get())
            .ok_or(ExecutionError::InvalidTypeidx)?;
        func_type.validate_args(args, &self.module)?;

        for v in args.iter().copied() {
            self.state.push_value(v);
        }

        self.state.call_function(func_idx, &self.module)?;

        // TODO: validate return value type
        match func_type.result.len() {
            0 => Ok(None),
            1 => Ok(Some(self.state.pop_value())),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    pub fn ty(self) -> Valtype {
        match self {
            Value::I32(_) => Valtype::I32,
            Value::I64(_) => Valtype::I64,
            Value::F32(_) => Valtype::F32,
            Value::F64(_) => Valtype::F64,
        }
    }

    pub fn zero(ty: Valtype) -> Self {
        match ty {
            Valtype::I32 => Self::I32(0),
            Valtype::I64 => Self::I64(0),
            Valtype::F32 => Self::F32(0.0),
            Valtype::F64 => Self::F64(0.0),
        }
    }

    pub fn byte_size(self) -> usize {
        match self {
            Value::I32(_) => 4,
            Value::I64(_) => 8,
            Value::F32(_) => 4,
            Value::F64(_) => 8,
        }
    }

    pub fn copy_to(self, mem: &mut [u8]) {
        match self {
            Value::I32(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Value::I64(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Value::F32(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Value::F64(v) => mem.copy_from_slice(&v.to_le_bytes()),
        }
    }
}