use crate::{
    components::{Exportdesc, Funcidx, Resulttype, Valtype},
    execute::State,
    ExecuteError, Module, Val, Vector, VectorFactory,
};
use core::fmt::{Debug, Formatter};

pub trait HostFuncs {
    type HostFunc;

    fn invoke(&mut self, func: &mut Self::HostFunc, args: &[Val]) -> Option<Val>;

    fn resolve(
        &mut self,
        module_name: &str,
        func_name: &str,
        params: &[Valtype],
        result: Resulttype,
    ) -> Option<Self::HostFunc>;
}

// Mem, Table, Globals
pub struct Env<V: VectorFactory> {
    pub mem: Option<V::Vector<u8>>,
    pub table: Option<V::Vector<Option<Funcidx>>>,
}

impl<V: VectorFactory> Default for Env<V> {
    fn default() -> Self {
        Self {
            mem: None,
            table: None,
        }
    }
}

impl<V: VectorFactory> Clone for Env<V> {
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.as_ref().map(V::clone_vector),
            table: self.table.as_ref().map(V::clone_vector),
        }
    }
}

impl<V: VectorFactory> Debug for Env<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Env")
            .field("mem", &self.mem.as_ref().map(|v| v.as_ref()))
            .field("table", &self.table.as_ref().map(|v| v.as_ref()))
            .finish()
    }
}

pub struct ModuleInstance<V: VectorFactory> {
    pub module: Module<V>,
    pub state: State<V>,
}

impl<V: VectorFactory> ModuleInstance<V> {
    pub(crate) fn new(module: Module<V>, env: Env<V>) -> Result<Self, ExecuteError> {
        let mem = env.mem.unwrap_or_else(|| V::create_vector(None));
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

    pub fn module(&self) -> &Module<V> {
        &self.module
    }

    pub fn mem(&self) -> &[u8] {
        &self.state.mem
    }

    pub fn mem_mut(&mut self) -> &mut [u8] {
        &mut self.state.mem
    }

    // TODO: table

    pub fn invoke(
        &mut self,
        function_name: &str,
        args: &[Val],
    ) -> Result<Option<Val>, ExecuteError> {
        let Some(export) = self.module.exports().iter().find(|export| {
            matches!(export.desc, Exportdesc::Func(_)) && function_name == export.name.as_str()
        }) else {
            return Err(ExecuteError::NotExportedFunction);
        };
        let Exportdesc::Func(func_idx) = export.desc else {
            unreachable!();
        };

        let func = self
            .module
            .funcs()
            .get(func_idx.get())
            .ok_or(ExecuteError::InvalidFuncidx)?;
        let func_type = self
            .module
            .types()
            .get(func.ty.get())
            .ok_or(ExecuteError::InvalidTypeidx)?;
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

impl<V: VectorFactory> Debug for ModuleInstance<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleInstance")
            .field("module", &self.module)
            // TODO: .field("state", &self.state)
            .finish()
    }
}
