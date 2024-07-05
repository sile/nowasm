use crate::{
    symbols::{Code, ExportDesc, GlobalIdx, ValType},
    Instr, Module, Vectors,
};

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    NotExportedFunction,
    InvalidFuncIdx,
    InvalidFuncArgs,
    InvalidGlobalInitializer,
}

// TODO: s/Stacks/Stack/
pub trait Stacks {
    // TODO: Return Result
    fn push_frame(&mut self, locals: usize);
    fn pop_frame(&mut self);
    fn current_frame(&mut self) -> Frame;

    fn push_value(&mut self, value: Value);
    fn pop_value(&mut self) -> Value;
}

pub trait Store {
    fn push_global(&mut self, value: Value);
    fn set_global(&mut self, i: GlobalIdx, value: Value);
    fn get_global(&self, i: GlobalIdx) -> Value;
}

pub trait ImportObject {}

#[derive(Debug)]
pub struct ModuleInstance<V, G, S, I> {
    pub module: Module<V>,
    pub store: G,
    pub stacks: S,
    pub import_object: I,
}

impl<V, G, S, I> ModuleInstance<V, G, S, I>
where
    V: Vectors,
    G: Store,
    S: Stacks,
    I: ImportObject,
{
    pub fn new(
        module: Module<V>,
        mut store: G,
        stacks: S,
        import_object: I,
    ) -> Result<Self, ExecutionError> {
        if module.start_section().start.is_some() {
            todo!()
        }

        // TODO: check import_object

        for global in module.global_section().globals.iter(module.vectors()) {
            store.push_global(global.init(&module)?);
        }

        Ok(Self {
            module,
            store,
            stacks,
            import_object,
        })
    }

    pub fn invoke(
        &mut self,
        function_name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, ExecutionError> {
        let Some(export) = self.module.exports().find(|export| {
            matches!(export.desc, ExportDesc::Func(_))
                && Some(function_name) == self.module.get_name(export.name)
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

        let code = func_idx
            .get_code(&self.module)
            .ok_or(ExecutionError::InvalidFuncIdx)?;

        let locals = args.len() + code.locals(&self.module).count();
        self.stacks.push_frame(locals);
        let result = self.call(code, args);
        self.stacks.pop_frame();
        result
    }

    fn call(&mut self, code: Code, args: &[Value]) -> Result<Option<Value>, ExecutionError> {
        let frame = self.stacks.current_frame();
        for (i, arg) in args
            .iter()
            .copied()
            .chain(code.locals(&self.module).map(Value::zero))
            .enumerate()
        {
            frame.locals[i] = arg;
        }

        for instr in code.instrs(&self.module) {
            match instr {
                Instr::GlobalGet(idx) => {
                    let v = self.store.get_global(idx);
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
                    self.stacks.push_value(Value::I32(v));
                }
                Instr::I64Const(v) => {
                    self.stacks.push_value(Value::I64(v));
                }
                Instr::F32Const(v) => {
                    self.stacks.push_value(Value::F32(v));
                }
                Instr::F64Const(v) => {
                    self.stacks.push_value(Value::F64(v));
                }
                _ => {
                    dbg!(instr);
                    todo!();
                }
            }
        }

        todo!()
    }
}

#[derive(Debug)]
pub struct Frame<'a> {
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
}
