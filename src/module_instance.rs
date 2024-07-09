use crate::{
    components::{Exportdesc, Limits, Resulttype, Valtype},
    execute::State,
    ExecuteError, Module, Val, Vector, VectorFactory, PAGE_SIZE,
};
use core::fmt::{Debug, Formatter};

pub trait Invoke {
    fn invoke(&mut self, args: &[Val]) -> Option<Val>;
}

impl Invoke for () {
    fn invoke(&mut self, _args: &[Val]) -> Option<Val> {
        panic!();
    }
}

// TODO: rename
pub trait Import {
    type HostFunc: Invoke;

    fn import(
        &mut self,
        module: &str,
        name: &str,
        spec: &ImportSpec,
    ) -> Option<Imported<Self::HostFunc>>;
}

impl Import for () {
    type HostFunc = ();

    fn import(
        &mut self,
        _module: &str,
        _name: &str,
        _spec: &ImportSpec,
    ) -> Option<Imported<Self::HostFunc>> {
        None
    }
}

#[derive(Debug)]
pub enum ImportSpec<'a> {
    Mem {
        limits: Limits,
    },
    Table {
        limits: Limits,
    },
    Global {
        ty: Valtype,
    },
    Func {
        params: &'a [Valtype],
        result: Resulttype,
    },
}

#[derive(Debug)]
pub enum Imported<F> {
    Func(F),
    Mem(Mem),
    Table(Table),
    Global(Val),
}

#[derive(Debug)]
pub struct Mem;

#[derive(Debug)]
pub struct Table;

pub struct ModuleInstance<V: VectorFactory, H> {
    pub module: Module<V>,
    pub state: State<V, H>,
}

impl<V: VectorFactory, H> ModuleInstance<V, H> {
    pub(crate) fn new<I>(module: Module<V>, _importer: I) -> Result<Self, ExecuteError>
    where
        I: Import<HostFunc = H>,
    {
        // TODO: let mem = env.mem.unwrap_or_else(|| V::create_vector(None));
        let mut mem = V::create_vector(None);
        if let Some(m) = module.mem() {
            for _ in 0..m.limits.min as usize * PAGE_SIZE {
                mem.push(0);
            }
        }

        if module.start().is_some() {
            todo!()
        }

        // TODO: check mem (min, max, pagesize)
        let mut state = State::<V, H>::new(mem);

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

impl<V: VectorFactory, H> Debug for ModuleInstance<V, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleInstance")
            .field("module", &self.module)
            // TODO: .field("state", &self.state)
            .finish()
    }
}
