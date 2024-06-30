use crate::{Module, Vectors};

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    NotExportedFunction,
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
        self.module.export_section().exports;
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}
