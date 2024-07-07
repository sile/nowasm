use crate::{
    symbols::{BlockType, Code, ExportDesc, LocalIdx, ValType},
    Allocator, Instr, Module, Vector,
};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    NotExportedFunction,
    InvalidFuncIdx,
    InvalidFuncArgs,
    InvalidGlobalInitializer,
    Trapped, // TODO: Add reason
}

// TODO: Rename
#[derive(Debug)]
pub struct State<A: Allocator> {
    _allocator: PhantomData<A>,
    pub mem: A::Vector<u8>,
    pub globals: A::Vector<Value>,
    pub locals: A::Vector<Value>,
    pub frames: A::Vector<Frame>,
    pub values: A::Vector<Value>,
    pub labels: A::Vector<Label>,
}

impl<A: Allocator> State<A> {
    pub fn new(mem: A::Vector<u8>) -> Self {
        Self {
            _allocator: PhantomData,
            mem,
            globals: A::allocate_vector(),
            locals: A::allocate_vector(),
            frames: A::allocate_vector(),
            values: A::allocate_vector(),
            labels: A::allocate_vector(),
        }
    }

    pub fn push_frame(&mut self) {
        let frame = Frame {
            locals_start: self.locals.len(),
            values_start: self.values.len(),
            labels_start: self.labels.len(),
        };
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) {
        let Some(frame) = self.frames.pop() else {
            unreachable!();
        };

        assert!(frame.locals_start <= self.locals.len());
        self.locals.truncate(frame.locals_start);

        assert!(frame.values_start <= self.values.len());
        self.values.truncate(frame.values_start);

        assert!(frame.labels_start <= self.labels.len());
        self.labels.truncate(frame.labels_start);
    }

    fn current_frame(&self) -> Frame {
        self.frames.as_ref().last().copied().expect("unreachable")
    }

    pub fn set_local(&mut self, i: LocalIdx, v: Value) {
        let i = self.current_frame().locals_start + i.get() as usize;
        self.locals.as_mut()[i] = v;
    }

    pub fn get_local(&self, i: LocalIdx) -> Value {
        let i = self.current_frame().locals_start + i.get() as usize;
        self.locals.as_ref()[i]
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
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub locals_start: usize,
    pub values_start: usize,
    pub labels_start: usize,
}

// TODO: #[derive(Debug)]
pub struct ModuleInstance<A: Allocator> {
    pub module: Module<A>,
    pub state: State<A>,
}

impl<A: Allocator> ModuleInstance<A> {
    pub fn new(
        module: Module<A>,

        // TODO: Use builder
        mem: A::Vector<u8>,
    ) -> Result<Self, ExecutionError> {
        if module.start_section().start.is_some() {
            todo!()
        }

        // TODO: check mem (min, max, pagesize)
        let mut state = State::<A>::new(mem);

        for global in module.global_section().globals.as_ref().iter() {
            state.globals.push(global.init()?);
        }

        Ok(Self { module, state })
    }

    pub fn invoke(
        &mut self,
        function_name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, ExecutionError> {
        let Some(export) = self.module.exports().find(|export| {
            matches!(export.desc, ExportDesc::Func(_)) && function_name == export.name.as_str()
        }) else {
            return Err(ExecutionError::NotExportedFunction);
        };
        let ExportDesc::Func(func_idx) = export.desc else {
            unreachable!();
        };

        let fun_type = func_idx
            .get_type(&self.module)
            .ok_or(ExecutionError::InvalidFuncIdx)?;
        fun_type.validate_args(args, &self.module)?;
        let returns = fun_type.rt2.len();

        let code = func_idx
            .get_code(&self.module)
            .ok_or(ExecutionError::InvalidFuncIdx)?
            .clone(); // TODO: remove clone

        // TODO: delete let locals = args.len() + code.locals().count();
        self.state.push_frame();
        let result = match self.call(&code, args, returns) {
            Err(e) => Err(e),
            Ok(()) => {
                // TODO: validate result type
                if returns == 0 {
                    Ok(None)
                } else {
                    Ok(Some(self.state.pop_value()))
                }
            }
        };

        // TODO: Clear all stacks when error
        result
    }

    fn call(
        &mut self,
        code: &Code<A>,
        args: &[Value],
        return_values: usize,
    ) -> Result<(), ExecutionError> {
        for v in args.iter().copied().chain(code.locals().map(Value::zero)) {
            self.state.locals.push(v);
        }

        for instr in code.instrs() {
            match instr {
                Instr::Nop => {}
                Instr::GlobalSet(idx) => {
                    let v = self.state.pop_value();
                    self.state.globals.as_mut()[idx.as_usize()] = v;
                }
                Instr::GlobalGet(idx) => {
                    let v = self.state.globals.as_ref()[idx.as_usize()];
                    self.state.push_value(v);
                }
                Instr::LocalSet(idx) => {
                    let v = self.state.pop_value();
                    self.state.set_local(*idx, v);
                }
                Instr::LocalGet(idx) => {
                    let v = self.state.get_local(*idx);
                    self.state.push_value(v);
                }
                Instr::I32Const(v) => {
                    self.state.push_value(Value::I32(*v));
                }
                Instr::I64Const(v) => {
                    self.state.push_value(Value::I64(*v));
                }
                Instr::F32Const(v) => {
                    self.state.push_value(Value::F32(*v));
                }
                Instr::F64Const(v) => {
                    self.state.push_value(Value::F64(*v));
                }
                Instr::I32Add => {
                    let v0 = self.state.pop_value_i32();
                    let v1 = self.state.pop_value_i32();
                    self.state.push_value(Value::I32(v1 + v0));
                }
                Instr::I32Sub => {
                    let v0 = self.state.pop_value_i32();
                    let v1 = self.state.pop_value_i32();
                    self.state.push_value(Value::I32(v1 - v0));
                }
                Instr::I32Xor => {
                    let v0 = self.state.pop_value_i32();
                    let v1 = self.state.pop_value_i32();
                    self.state.push_value(Value::I32(v1 ^ v0));
                }
                Instr::I32And => {
                    let v0 = self.state.pop_value_i32();
                    let v1 = self.state.pop_value_i32();
                    self.state.push_value(Value::I32(v1 & v0));
                }
                Instr::I32LtS => {
                    let v0 = self.state.pop_value_i32();
                    let v1 = self.state.pop_value_i32();
                    let r = if v1 < v0 { 1 } else { 0 };
                    self.state.push_value(Value::I32(r));
                }
                Instr::I32Store(arg) => {
                    let v = self.state.pop_value();
                    let i = self.state.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    let mem = self.state.mem.as_mut();
                    if mem.len() < end {
                        return Err(ExecutionError::Trapped);
                    }
                    v.copy_to(&mut mem[start..end]);
                }
                Instr::BrIf(label) => {
                    let c = self.state.pop_value_i32();
                    if c != 0 {
                        dbg!(label);
                        todo!();
                    }
                }
                Instr::Return => {
                    break;
                }
                Instr::Call(idx) => {
                    dbg!(idx);
                    dbg!(&self.module.code_section().codes.as_ref()[idx.get() as usize]);
                    todo!();
                }
                Instr::Unreachable => {
                    return Err(ExecutionError::Trapped);
                }
                Instr::Block(block) => {
                    // TODO: Add block_type handling
                    assert!(matches!(
                        block.block_type,
                        BlockType::Empty | BlockType::Val(_)
                    ));
                    // push label
                    dbg!(block);

                    todo!();
                }
                _ => {
                    dbg!(instr);
                    todo!();
                }
            }
        }

        if return_values == 0 {
            self.state.pop_frame();
        } else {
            assert_eq!(return_values, 1);
            let v = self.state.pop_value();
            self.state.pop_frame();
            self.state.push_value(v);
        }
        Ok(())
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
    pub fn ty(self) -> ValType {
        match self {
            Value::I32(_) => ValType::I32,
            Value::I64(_) => ValType::I64,
            Value::F32(_) => ValType::F32,
            Value::F64(_) => ValType::F64,
        }
    }

    pub fn zero(ty: ValType) -> Self {
        match ty {
            ValType::I32 => Self::I32(0),
            ValType::I64 => Self::I64(0),
            ValType::F32 => Self::F32(0.0),
            ValType::F64 => Self::F64(0.0),
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

#[derive(Debug, Clone, Copy)]
pub struct Label {
    pub instr_position: usize,
}

impl Label {
    pub fn new(instr_position: usize) -> Self {
        Self { instr_position }
    }
}
