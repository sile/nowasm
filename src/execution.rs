use crate::{Module, Vectors};

pub trait Stacks {}

#[derive(Debug)]
pub struct Store {}

// #[derive(Debug)]
// pub struct ModuleInstanceBuilder {}

#[derive(Debug)]
pub struct ModuleInstance<V> {
    pub module: Module<V>,
}

impl<V: Vectors> ModuleInstance<V> {
    pub fn new(module: Module<V>) -> Self {
        ModuleInstance { module }
    }
}
