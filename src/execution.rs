use crate::{
    symbols::{ExportDesc, FuncIdx, LocalIdx, ValType},
    Allocator, FuncType, Instr, Module, Vector,
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
    pub values: A::Vector<Value>,
    pub current_frame: Frame,
    pub current_block: Block,
}

impl<A: Allocator> State<A> {
    pub fn new(mem: A::Vector<u8>) -> Self {
        Self {
            _allocator: PhantomData,
            mem,
            globals: A::allocate_vector(),
            locals: A::allocate_vector(),
            values: A::allocate_vector(),
            current_frame: Frame::root(),
            current_block: Block::default(),
        }
    }

    pub fn enter_frame(&mut self, ty: &FuncType<A>) -> Frame {
        let locals_start = self.locals.len();
        for _ in 0..ty.args_len() {
            let v = self.pop_value();
            self.locals.push(v);
        }
        let values_start = self.values.len();

        let prev = self.current_frame;
        self.current_frame = Frame {
            locals_start,
            values_start,
        };
        prev
    }

    pub fn exit_frame(&mut self, ty: &FuncType<A>, succeeded: bool, prev: Frame) {
        let frame = self.current_frame;

        let return_value = match ty.return_values_len() {
            0 => None,
            1 => Some(self.pop_value()),
            _ => unreachable!(),
        };

        assert!(frame.locals_start <= self.locals.len());
        self.locals.truncate(frame.locals_start);

        assert!(frame.values_start <= self.values.len());
        self.values.truncate(frame.values_start);

        self.current_frame = prev;

        if succeeded {
            if let Some(v) = return_value {
                self.push_value(v);
            }
        }
    }

    pub fn enter_block(&mut self) -> Block {
        let prev = self.current_block;
        self.current_block = Block {
            values_start: self.values.len(),
        };
        prev
    }

    pub fn exit_block(&mut self, prev: Block) {
        let block = self.current_block;

        assert!(block.values_start <= self.values.len());
        self.values.truncate(block.values_start);

        self.current_block = prev;

        // TODO: return value handling
    }

    pub fn set_local(&mut self, i: LocalIdx, v: Value) {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals.as_mut()[i] = v;
    }

    pub fn get_local(&self, i: LocalIdx) -> Value {
        let i = self.current_frame.locals_start + i.get() as usize;
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

    pub fn call_function(
        &mut self,
        func_idx: FuncIdx,
        module: &Module<A>,
    ) -> Result<(), ExecutionError> {
        // TODO: Add validation phase
        let func_type = func_idx
            .get_type(module)
            .ok_or(ExecutionError::InvalidFuncIdx)?;
        let code = func_idx
            .get_code(module)
            .ok_or(ExecutionError::InvalidFuncIdx)?;

        let prev_frame = self.enter_frame(func_type);
        for v in code.locals().map(Value::zero) {
            self.locals.push(v);
        }
        let result = self.execute_instrs(code.instrs(), module);
        self.exit_frame(func_type, result.is_ok(), prev_frame);
        result
    }

    pub fn execute_instrs(
        &mut self,
        instrs: &[Instr<A>],
        module: &Module<A>,
    ) -> Result<(), ExecutionError> {
        for instr in instrs {
            match instr {
                _ => todo!("{instr:?}"),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub locals_start: usize,
    pub values_start: usize,
}

impl Frame {
    pub fn root() -> Self {
        Self {
            locals_start: 0,
            values_start: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub values_start: usize,
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

        let func_type = func_idx
            .get_type(&self.module)
            .ok_or(ExecutionError::InvalidFuncIdx)?;
        func_type.validate_args(args, &self.module)?;

        for v in args.iter().rev().copied() {
            self.state.push_value(v);
        }

        self.state.call_function(func_idx, &self.module)?;

        // TODO: validate return value type
        match func_type.return_values_len() {
            0 => Ok(None),
            1 => Ok(Some(self.state.pop_value())),
            _ => unreachable!(),
        }
    }

    // fn call(
    //     &mut self,
    //     code: &Code<A>,
    //     args: &[Value],
    //     _return_values: usize,
    // ) -> Result<(), ExecutionError> {
    //     for v in args.iter().copied().chain(code.locals().map(Value::zero)) {
    //         self.state.locals.push(v);
    //     }

    //     for instr in code.body_iter() {
    //         match instr {
    //             Instr::Nop => {}
    //             Instr::GlobalSet(idx) => {
    //                 let v = self.state.pop_value();
    //                 self.state.globals.as_mut()[idx.as_usize()] = v;
    //             }
    //             Instr::GlobalGet(idx) => {
    //                 let v = self.state.globals.as_ref()[idx.as_usize()];
    //                 self.state.push_value(v);
    //             }
    //             Instr::LocalSet(idx) => {
    //                 let v = self.state.pop_value();
    //                 self.state.set_local(*idx, v);
    //             }
    //             Instr::LocalGet(idx) => {
    //                 let v = self.state.get_local(*idx);
    //                 self.state.push_value(v);
    //             }
    //             Instr::I32Const(v) => {
    //                 self.state.push_value(Value::I32(*v));
    //             }
    //             Instr::I64Const(v) => {
    //                 self.state.push_value(Value::I64(*v));
    //             }
    //             Instr::F32Const(v) => {
    //                 self.state.push_value(Value::F32(*v));
    //             }
    //             Instr::F64Const(v) => {
    //                 self.state.push_value(Value::F64(*v));
    //             }
    //             Instr::I32Add => {
    //                 let v0 = self.state.pop_value_i32();
    //                 let v1 = self.state.pop_value_i32();
    //                 self.state.push_value(Value::I32(v1 + v0));
    //             }
    //             Instr::I32Sub => {
    //                 let v0 = self.state.pop_value_i32();
    //                 let v1 = self.state.pop_value_i32();
    //                 self.state.push_value(Value::I32(v1 - v0));
    //             }
    //             Instr::I32Xor => {
    //                 let v0 = self.state.pop_value_i32();
    //                 let v1 = self.state.pop_value_i32();
    //                 self.state.push_value(Value::I32(v1 ^ v0));
    //             }
    //             Instr::I32And => {
    //                 let v0 = self.state.pop_value_i32();
    //                 let v1 = self.state.pop_value_i32();
    //                 self.state.push_value(Value::I32(v1 & v0));
    //             }
    //             Instr::I32LtS => {
    //                 let v0 = self.state.pop_value_i32();
    //                 let v1 = self.state.pop_value_i32();
    //                 let r = if v1 < v0 { 1 } else { 0 };
    //                 self.state.push_value(Value::I32(r));
    //             }
    //             Instr::I32Store(arg) => {
    //                 let v = self.state.pop_value();
    //                 let i = self.state.pop_value_i32();
    //                 let start = (i + arg.offset as i32) as usize;
    //                 let end = start + v.byte_size();
    //                 let mem = self.state.mem.as_mut();
    //                 if mem.len() < end {
    //                     return Err(ExecutionError::Trapped);
    //                 }
    //                 v.copy_to(&mut mem[start..end]);
    //             }
    //             Instr::BrIf(label) => {
    //                 let c = self.state.pop_value_i32();
    //                 if c != 0 {
    //                     dbg!(label);
    //                     todo!();
    //                 }
    //             }
    //             Instr::Return => {
    //                 break;
    //             }
    //             Instr::Call(idx) => {
    //                 dbg!(idx);
    //                 dbg!(&self.module.code_section().codes.as_ref()[idx.get() as usize]);
    //                 todo!();
    //             }
    //             Instr::Unreachable => {
    //                 return Err(ExecutionError::Trapped);
    //             }
    //             Instr::Block(block) => {
    //                 // TODO: Add block_type handling
    //                 assert!(matches!(
    //                     block.block_type,
    //                     BlockType::Empty | BlockType::Val(_)
    //                 ));
    //                 // push label
    //                 dbg!(block);

    //                 todo!();
    //             }
    //             _ => {
    //                 dbg!(instr);
    //                 todo!();
    //             }
    //         }
    //     }

    //     // if return_values == 0 {
    //     //     self.state.pop_frame();
    //     // } else {
    //     //     assert_eq!(return_values, 1);
    //     //     let v = self.state.pop_value();
    //     //     self.state.pop_frame();
    //     //     self.state.push_value(v);
    //     // }
    //     Ok(())
    // }
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
