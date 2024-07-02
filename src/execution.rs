use crate::{
    symbols::{ExportDesc, ValType},
    Module, Vectors,
};

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    NotExportedFunction,
    InvalidFuncIdx,
    InvalidFuncArgs,
}

pub trait Stacks {
    //
}

pub trait Store {
    //
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
        store: G,
        stacks: S,
        import_object: I,
    ) -> Result<Self, ExecutionError> {
        if module.start_section().start.is_some() {
            todo!()
        }

        // TODO: check import_object

        Ok(Self {
            module,
            store,
            stacks,
            import_object,
        })
    }

    pub fn invoke(
        &self,
        function_name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, ExecutionError> {
        for export in self.module.exports() {
            let ExportDesc::Func(func_idx) = export.desc else {
                continue;
            };
            if Some(function_name) != self.module.get_name(export.name) {
                continue;
            }

            let fun_type = func_idx
                .get_type(&self.module)
                .ok_or(ExecutionError::InvalidFuncIdx)?;
            fun_type.validate_args(args, &self.module)?;
            dbg!(fun_type);

            let code = func_idx
                .get_code(&self.module)
                .ok_or(ExecutionError::InvalidFuncIdx)?;
            dbg!(code);
        }
        Err(ExecutionError::NotExportedFunction)
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
}
