use crate::{
    components::{Exportdesc, Importdesc, Limits, Memtype, Resulttype, Typeidx, Valtype},
    execute::State,
    ExecuteError, Module, Val, Vector, VectorFactory, PAGE_SIZE,
};
use core::fmt::{Debug, Formatter};

pub trait HostFunc {
    fn invoke(&mut self, args: &[Val]) -> Option<Val>;
}

impl HostFunc for () {
    fn invoke(&mut self, _args: &[Val]) -> Option<Val> {
        panic!();
    }
}

pub trait Resolve {
    type HostFunc: HostFunc;

    #[allow(unused_variables)]
    fn resolve_mem(&self, module: &str, name: &str, ty: Memtype) -> Option<&[u8]> {
        None
    }

    #[allow(unused_variables)]
    fn resolve_table(
        &self,
        module: &str,
        name: &str,
        limits: Limits,
    ) -> Option<&[Option<Typeidx>]> {
        None
    }

    #[allow(unused_variables)]
    fn resolve_global(&self, module: &str, name: &str, ty: Valtype) -> Option<Val> {
        None
    }

    #[allow(unused_variables)]
    fn resolve_func(
        &self,
        module: &str,
        name: &str,
        params: &[Valtype],
        result: Resulttype,
    ) -> Option<Self::HostFunc> {
        None
    }
}

impl Resolve for () {
    type HostFunc = ();
}

pub struct ModuleInstance<V: VectorFactory, H> {
    pub module: Module<V>,
    pub state: State<V, H>,
}

impl<V: VectorFactory, H> ModuleInstance<V, H> {
    pub(crate) fn new<R>(module: Module<V>, resolver: R) -> Result<Self, ExecuteError>
    where
        R: Resolve<HostFunc = H>,
    {
        let mut imported_mem = None;
        let mut imported_globals = V::create_vector(None);
        for (index, import) in module.imports().iter().enumerate() {
            match &import.desc {
                Importdesc::Func(_) => todo!(),
                Importdesc::Table(_) => todo!(),
                Importdesc::Mem(ty) => {
                    let resolved = resolver
                        .resolve_mem(import.module.as_str(), import.name.as_str(), *ty)
                        .ok_or_else(|| ExecuteError::UnresolvedImport { index })?;
                    let resolved = V::clone_vector(resolved);
                    imported_mem = Some(resolved);
                }
                Importdesc::Global(ty) => {
                    let resolved = resolver
                        .resolve_global(import.module.as_str(), import.name.as_str(), ty.valtype())
                        .ok_or_else(|| ExecuteError::UnresolvedImport { index })?;
                    imported_globals.push(resolved);
                }
            }
        }

        let globals = Self::init_globals(&imported_globals, &module)?;
        let mem = Self::init_mem(&globals, imported_mem, &module)?;

        if module.start().is_some() {
            todo!()
        }

        let mut state = State::<V, H>::new(mem);
        state.globals = globals;

        Ok(Self { module, state })
    }

    fn init_globals(
        imported_globals: &[Val],
        module: &Module<V>,
    ) -> Result<V::Vector<Val>, ExecuteError> {
        let n = imported_globals.len() + module.globals().len();
        let mut globals = V::create_vector(Some(n));

        for global in imported_globals {
            globals.push(*global);
        }

        for (index, global) in module.globals().iter().enumerate() {
            let v = global
                .init(imported_globals)
                .ok_or_else(|| ExecuteError::InvalidGlobal { index })?;
            globals.push(v);
        }
        Ok(globals)
    }

    fn init_mem(
        globals: &[Val],
        mut mem: Option<V::Vector<u8>>,
        module: &Module<V>,
    ) -> Result<V::Vector<u8>, ExecuteError> {
        if let Some(ty) = module.mem() {
            if let Some(v) = &mem {
                if !ty.contains(v.len()) || v.len() % PAGE_SIZE != 0 {
                    return Err(ExecuteError::InvalidImportedMem);
                }
            } else {
                let mut m = V::create_vector(Some(ty.min_bytes()));
                for _ in 0..ty.min_bytes() {
                    m.push(0);
                }
                mem = Some(m);
            }
        } else if mem.is_some() {
            return Err(ExecuteError::InvalidImportedMem);
        }

        let mut mem = mem.unwrap_or_else(|| V::create_vector(None));
        for (index, data) in module.datas().iter().enumerate() {
            if module.mem().is_none() {
                return Err(ExecuteError::InvalidData { index });
            }
            let Some(offset) = data.offset.get(globals, module) else {
                return Err(ExecuteError::InvalidData { index });
            };
            if offset < 0 {
                return Err(ExecuteError::InvalidData { index });
            }

            let start = offset as usize;
            let end = start + data.init.len();
            if mem.len() < end {
                return Err(ExecuteError::InvalidData { index });
            }
            (&mut mem[start..end]).copy_from_slice(&data.init);
        }

        Ok(mem)
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

    pub fn globals(&self) -> &[Val] {
        &self.state.globals
    }

    pub fn globals_mut(&mut self) -> &mut [Val] {
        // TODO: check mutability
        &mut self.state.globals
    }

    // TODO: table
    // TODO: global

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
