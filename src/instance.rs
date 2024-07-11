use crate::{
    components::{Exportdesc, Funcidx, Importdesc, Limits, Memtype, Resulttype, Valtype},
    execute::State,
    ExecuteError, Module, Vector, VectorFactory, PAGE_SIZE,
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

#[derive(Debug)]
pub enum FuncInst<H> {
    Imported { imports_index: usize, host_func: H },
    Module { funcs_index: usize },
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
    ) -> Option<&[Option<Funcidx>]> {
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
        let mut imported_table = None;
        let mut imported_globals = V::create_vector(None);
        let mut imported_funcs = V::create_vector(None);
        for (index, import) in module.imports().iter().enumerate() {
            match &import.desc {
                Importdesc::Func(typeidx) => {
                    let ty = module
                        .types()
                        .get(typeidx.get())
                        .ok_or_else(|| ExecuteError::UnresolvedImport { index })?;
                    let host_func = resolver
                        .resolve_func(
                            import.module.as_str(),
                            import.name.as_str(),
                            &ty.params,
                            ty.result,
                        )
                        .ok_or_else(|| ExecuteError::UnresolvedImport { index })?;
                    imported_funcs.push(FuncInst::Imported {
                        imports_index: index,
                        host_func,
                    });
                }
                Importdesc::Table(ty) => {
                    let resolved = resolver
                        .resolve_table(import.module.as_str(), import.name.as_str(), ty.limits)
                        .ok_or_else(|| ExecuteError::UnresolvedImport { index })?;
                    let resolved = V::clone_vector(resolved);
                    imported_table = Some(resolved);
                }
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
                    imported_globals.push(GlobalVal::new(ty.is_const(), resolved));
                }
            }
        }

        let mut funcs = imported_funcs;
        for i in 0..module.funcs().len() {
            funcs.push(FuncInst::Module { funcs_index: i });
        }

        let globals = Self::init_globals(&imported_globals, &module)?;
        let mem = Self::init_mem(&globals, imported_mem, &module)?;
        let table = Self::init_table(&globals, &funcs, imported_table, &module)?;

        let state = State::<V, H>::new(mem, table, globals, funcs);
        let mut this = Self { module, state };

        if let Some(funcidx) = this.module.start() {
            // TODO: check function type (in decoding phase?)
            this.state.call_function(funcidx, &this.module)?;
        }

        Ok(this)
    }

    fn init_globals(
        imported_globals: &[GlobalVal],
        module: &Module<V>,
    ) -> Result<V::Vector<GlobalVal>, ExecuteError> {
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
        globals: &[GlobalVal],
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
            let Some(offset) = data.offset.get(globals) else {
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

    fn init_table(
        globals: &[GlobalVal],
        funcs: &[FuncInst<H>],
        mut table: Option<V::Vector<Option<Funcidx>>>,
        module: &Module<V>,
    ) -> Result<V::Vector<Option<Funcidx>>, ExecuteError> {
        if let Some(ty) = module.table() {
            if let Some(v) = &table {
                if !ty.contains(v.len()) {
                    return Err(ExecuteError::InvalidImportedTable);
                }
            } else {
                let mut vs = V::create_vector(Some(ty.limits.min as usize));
                for _ in 0..ty.limits.min {
                    vs.push(None);
                }
                table = Some(vs);
            }
        } else if table.is_some() {
            return Err(ExecuteError::InvalidImportedTable);
        }

        let mut table = table.unwrap_or_else(|| V::create_vector(None));
        for (index, elem) in module.elems().iter().enumerate() {
            if module.table().is_none() {
                return Err(ExecuteError::InvalidElem { index });
            }
            let Some(offset) = elem.offset.get(globals) else {
                return Err(ExecuteError::InvalidElem { index });
            };
            if offset < 0 {
                return Err(ExecuteError::InvalidElem { index });
            }

            let start = offset as usize;
            let end = start + elem.init.len();
            if table.len() < end {
                return Err(ExecuteError::InvalidElem { index });
            }
            for (i, funcidx) in (start..).zip(elem.init.iter().copied()) {
                table[i] = Some(funcidx);
            }
        }

        if table
            .iter()
            .filter_map(|i| *i)
            .any(|i| funcs.len() <= i.get())
        {
            return Err(ExecuteError::InvalidFuncidx);
        }

        Ok(table)
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

    pub fn globals(&self) -> &[GlobalVal] {
        &self.state.globals
    }

    pub fn globals_mut(&mut self) -> &mut [GlobalVal] {
        &mut self.state.globals
    }

    pub fn table(&self) -> &[Option<Funcidx>] {
        &self.state.table
    }

    pub fn table_mut(&mut self) -> &mut [Option<Funcidx>] {
        &mut self.state.table
    }

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalVal {
    is_const: bool,
    val: Val,
}

impl GlobalVal {
    pub(crate) const fn new(is_const: bool, val: Val) -> Self {
        Self { is_const, val }
    }

    pub const fn is_const(self) -> bool {
        self.is_const
    }

    pub const fn get(self) -> Val {
        self.val
    }

    pub fn set(&mut self, val: Val) -> bool {
        if !self.is_const {
            self.val = val;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Val {
    pub const fn ty(self) -> Valtype {
        match self {
            Self::I32(_) => Valtype::I32,
            Self::I64(_) => Valtype::I64,
            Self::F32(_) => Valtype::F32,
            Self::F64(_) => Valtype::F64,
        }
    }

    pub const fn as_i32(self) -> Option<i32> {
        if let Self::I32(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn as_i64(self) -> Option<i64> {
        if let Self::I64(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn as_f32(self) -> Option<f32> {
        if let Self::F32(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn as_f64(self) -> Option<f64> {
        if let Self::F64(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn zero(ty: Valtype) -> Self {
        match ty {
            Valtype::I32 => Self::I32(0),
            Valtype::I64 => Self::I64(0),
            Valtype::F32 => Self::F32(0.0),
            Valtype::F64 => Self::F64(0.0),
        }
    }

    pub(crate) fn byte_size(self) -> usize {
        match self {
            Self::I32(_) => 4,
            Self::I64(_) => 8,
            Self::F32(_) => 4,
            Self::F64(_) => 8,
        }
    }

    pub(crate) fn copy_to(self, mem: &mut [u8]) {
        match self {
            Self::I32(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Self::I64(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Self::F32(v) => mem.copy_from_slice(&v.to_le_bytes()),
            Self::F64(v) => mem.copy_from_slice(&v.to_le_bytes()),
        }
    }
}
