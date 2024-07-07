use crate::{
    symbols::{Code, ExportDesc, ValType},
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

// TODO: s/Stacks/Stack/
pub trait Stacks {
    // TODO: Return Result
    fn push_frame(&mut self, locals: usize);
    fn pop_frame(&mut self);
    fn current_frame(&mut self) -> FrameRef;

    fn push_value(&mut self, value: Value);
    fn pop_value(&mut self) -> Value;

    fn pop_value_i32(&mut self) -> i32 {
        let Value::I32(v) = self.pop_value() else {
            // TODO: Implement validation phases
            unreachable!();
        };
        v
    }
}

// TODO: Add trap_handler()

// TODO: Rename
#[derive(Debug)]
pub struct State<A: Allocator> {
    _allocator: PhantomData<A>,
    pub mem: A::Vector<u8>,
    pub globals: A::Vector<Value>,
    pub locals: A::Vector<Value>,
    pub frames: A::Vector<Frame>,
    pub values: A::Vector<Value>,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub locals_start: usize,
    pub values_start: usize,
    pub labels_start: usize,
}

// TODO: #[derive(Debug)]
pub struct ModuleInstance<S, A: Allocator> {
    pub module: Module<A>,
    pub state: State<A>,

    pub stacks: S,
}

impl<S, A> ModuleInstance<S, A>
where
    S: Stacks,
    A: Allocator,
{
    pub fn new(
        module: Module<A>,
        stacks: S,

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

        Ok(Self {
            module,
            state,
            stacks,
        })
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

        let locals = args.len() + code.locals().count();
        self.stacks.push_frame(locals);
        let result = match self.call(&code, args, returns) {
            Err(e) => Err(e),
            Ok(()) => {
                // TODO: validate result type
                if returns == 0 {
                    Ok(None)
                } else {
                    Ok(Some(self.stacks.pop_value()))
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
        let frame = self.stacks.current_frame();
        for (i, arg) in args
            .iter()
            .copied()
            .chain(code.locals().map(Value::zero))
            .enumerate()
        {
            frame.locals[i] = arg;
        }

        for instr in code.instrs() {
            match instr {
                Instr::GlobalSet(idx) => {
                    let v = self.stacks.pop_value();
                    self.state.globals.as_mut()[idx.as_usize()] = v;
                }
                Instr::GlobalGet(idx) => {
                    let v = self.state.globals.as_ref()[idx.as_usize()];
                    self.stacks.push_value(v);
                }
                Instr::LocalSet(idx) => {
                    let v = self.stacks.pop_value();
                    self.stacks.current_frame().locals[idx.as_usize()] = v;
                }
                Instr::LocalGet(idx) => {
                    let v = self.stacks.current_frame().locals[idx.as_usize()];
                    self.stacks.push_value(v);
                }
                Instr::I32Const(v) => {
                    self.stacks.push_value(Value::I32(*v));
                }
                Instr::I64Const(v) => {
                    self.stacks.push_value(Value::I64(*v));
                }
                Instr::F32Const(v) => {
                    self.stacks.push_value(Value::F32(*v));
                }
                Instr::F64Const(v) => {
                    self.stacks.push_value(Value::F64(*v));
                }
                Instr::I32Add => {
                    let v0 = self.stacks.pop_value_i32();
                    let v1 = self.stacks.pop_value_i32();
                    self.stacks.push_value(Value::I32(v1 + v0));
                }
                Instr::I32Sub => {
                    let v0 = self.stacks.pop_value_i32();
                    let v1 = self.stacks.pop_value_i32();
                    self.stacks.push_value(Value::I32(v1 - v0));
                }
                Instr::I32Xor => {
                    let v0 = self.stacks.pop_value_i32();
                    let v1 = self.stacks.pop_value_i32();
                    self.stacks.push_value(Value::I32(v1 ^ v0));
                }
                Instr::I32And => {
                    let v0 = self.stacks.pop_value_i32();
                    let v1 = self.stacks.pop_value_i32();
                    self.stacks.push_value(Value::I32(v1 & v0));
                }
                Instr::I32LtS => {
                    let v0 = self.stacks.pop_value_i32();
                    let v1 = self.stacks.pop_value_i32();
                    let r = if v1 < v0 { 1 } else { 0 };
                    self.stacks.push_value(Value::I32(r));
                }
                Instr::I32Store(arg) => {
                    let v = self.stacks.pop_value();
                    let i = self.stacks.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    let mem = self.state.mem.as_mut();
                    if mem.len() < end {
                        return Err(ExecutionError::Trapped);
                    }
                    v.copy_to(&mut mem[start..end]);
                }
                Instr::BrIf(label) => {
                    let c = self.stacks.pop_value_i32();
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
            self.stacks.pop_frame();
        } else {
            assert_eq!(return_values, 1);
            let v = self.stacks.pop_value();
            self.stacks.pop_frame();
            self.stacks.push_value(v);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FrameRef<'a> {
    pub locals: &'a mut [Value],
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
