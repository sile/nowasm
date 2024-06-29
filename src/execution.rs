use crate::{Module, Vectors};

#[derive(Debug, Clone, Copy)]
pub enum ExecutionError {
    //
}

pub trait Stacks {
    //
}

pub trait Store {
    //
}

#[derive(Debug)]
pub struct ModuleInstance<V, G, S> {
    pub module: Module<V>,
    pub store: G,
    pub stacks: S,
}

impl<V, G, S> ModuleInstance<V, G, S>
where
    V: Vectors,
    G: Store,
    S: Stacks,
{
    pub fn new(module: Module<V>, store: G, stacks: S) -> Result<Self, ExecutionError> {
        Ok(Self {
            module,
            store,
            stacks,
        })
    }
}
