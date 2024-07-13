use crate::{
    components::{Blocktype, Funcidx, Functype, Importdesc, Localidx},
    instance::FuncInst,
    instructions::Instr,
    Env, GlobalVal, HostFunc, Module, Val, Vector, VectorFactory,
};
use core::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum ExecuteError {
    NotExportedFunction,
    UnresolvedImport { index: usize },
    InvalidImportedMem,
    InvalidImportedTable,
    InvalidData { index: usize },
    InvalidElem { index: usize },
    InvalidGlobal { index: usize },
    InvalidMemidx,
    InvalidFuncidx,
    InvalidTypeidx,
    InvalidFuncArgs,
    Trapped,
}

impl Display for ExecuteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotExportedFunction => write!(f, "Not exported function"),
            Self::UnresolvedImport { index } => write!(f, "Unresolved import: {}", index),
            Self::InvalidImportedMem => write!(f, "Invalid imported memory"),
            Self::InvalidImportedTable => write!(f, "Invalid imported table"),
            Self::InvalidData { index } => write!(f, "Invalid data: {}", index),
            Self::InvalidElem { index } => write!(f, "Invalid elem: {}", index),
            Self::InvalidGlobal { index } => write!(f, "Invalid global: {}", index),
            Self::InvalidMemidx => write!(f, "Invalid memidx"),
            Self::InvalidFuncidx => write!(f, "Invalid funcidx"),
            Self::InvalidTypeidx => write!(f, "Invalid typeidx"),
            Self::InvalidFuncArgs => write!(f, "Invalid function arguments"),
            Self::Trapped => write!(f, "Trapped"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecuteError {}

pub struct Executor<V: VectorFactory> {
    pub mem: V::Vector<u8>,
    pub table: V::Vector<Option<Funcidx>>,
    pub globals: V::Vector<GlobalVal>,
    pub locals: V::Vector<Val>,
    pub values: V::Vector<Val>,
    pub current_frame: Frame,
    pub current_block: Block,
}

impl<V: VectorFactory> Executor<V> {
    pub fn new(
        mem: V::Vector<u8>,
        table: V::Vector<Option<Funcidx>>,
        globals: V::Vector<GlobalVal>,
    ) -> Self {
        Self {
            mem,
            table,
            globals,
            locals: V::create_vector(None),
            values: V::create_vector(None),
            current_frame: Frame::default(),
            current_block: Block::default(),
        }
    }

    pub fn enter_frame(&mut self, ty: &Functype<V>, level: usize) -> Frame {
        let locals_start = self.locals.len();
        for _ in 0..ty.params.len() {
            let v = self.pop_value();
            self.locals.push(v);
        }
        self.locals[locals_start..].reverse();
        let values_start = self.values.len();

        let prev = self.current_frame;
        self.current_frame = Frame {
            level,
            arity: ty.result.len(),
            locals_start,
            values_start,
        };
        prev
    }

    // TODO: delete unused parameter
    pub fn exit_frame(&mut self, _ty: &Functype<V>, prev: Frame) {
        let frame = self.current_frame;

        assert!(frame.locals_start <= self.locals.len());
        self.locals.truncate(frame.locals_start);

        self.values
            .remove_range(frame.values_start..self.values.len() - frame.arity);

        self.current_frame = prev;
    }

    pub fn enter_block(&mut self, ty: Blocktype) -> Block {
        assert!(matches!(ty, Blocktype::Empty)); // TODO

        let prev = self.current_block;
        self.current_block = Block {
            arity: ty.arity(),
            values_start: self.values.len(),
        };
        prev
    }

    // TODO: rename skipped
    pub fn exit_block(&mut self, ty: Blocktype, skipped: bool, prev: Block) {
        assert!(matches!(ty, Blocktype::Empty)); // TODO

        let block = self.current_block;

        if !skipped {
            self.values
                .remove_range(block.values_start..self.values.len() - block.arity);
        }

        self.current_block = prev;
    }

    pub fn set_local(&mut self, i: Localidx, v: Val) {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i] = v;
    }

    pub fn get_local(&self, i: Localidx) -> Val {
        let i = self.current_frame.locals_start + i.get() as usize;
        self.locals[i]
    }

    pub fn push_value(&mut self, v: Val) {
        self.values.push(v);
    }

    pub fn pop_value(&mut self) -> Val {
        self.values.pop().expect("unreachable")
    }

    pub fn pop_value_i32(&mut self) -> i32 {
        let Some(Val::I32(v)) = self.values.pop() else {
            unreachable!();
        };
        v
    }

    pub fn call_function<H: HostFunc>(
        &mut self,
        func_idx: Funcidx,
        level: usize,
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<usize, ExecuteError> {
        // TODO: Add validation phase
        let func = funcs
            .get_mut(func_idx.get())
            .ok_or(ExecuteError::InvalidFuncidx)?;
        let func_type = func.get_type(module).ok_or(ExecuteError::InvalidFuncidx)?; // TODO: change reason

        let prev_frame = self.enter_frame(func_type, level);
        match func {
            FuncInst::Imported {
                imports_index,
                host_func,
            } => {
                let Importdesc::Func(typeidx) = module.imports()[*imports_index].desc else {
                    unreachable!()
                };
                let func_type = &module.types()[typeidx.get()];
                let args_end = self.locals.len();
                let args_start = args_end - func_type.params.len();
                let args = &self.locals[args_start..args_end];

                let mut env = Env {
                    mem: &mut self.mem,
                    globals: &mut self.globals,
                };
                let value = host_func.invoke(args, &mut env);

                // TODO: check return value type
                if let Some(v) = value {
                    self.values.push(v);
                }
            }
            FuncInst::Module { funcs_index } => {
                let func = module
                    .funcs()
                    .get(*funcs_index)
                    .ok_or(ExecuteError::InvalidFuncidx)?;
                for v in func.locals.iter().copied().map(Val::zero) {
                    self.locals.push(v);
                }
                self.execute_instrs(func.body.instrs(), level + 1, funcs, module)?;
            }
        };
        self.exit_frame(func_type, prev_frame);
        Ok(level)
    }

    pub fn execute_instrs<H: HostFunc>(
        &mut self,
        instrs: &[Instr<V>],
        level: usize, // TODO: label
        funcs: &mut [FuncInst<H>],
        module: &Module<V>,
    ) -> Result<Option<usize>, ExecuteError> {
        dbg!(level);
        for instr in instrs {
            dbg!(instr);
            match instr {
                Instr::Nop => {}
                Instr::Unreachable => return Err(ExecuteError::Trapped),
                Instr::GlobalSet(idx) => {
                    let v = self.pop_value();
                    self.globals[idx.get()].set(v);
                }
                Instr::GlobalGet(idx) => {
                    let v = self.globals[idx.get()].get();
                    self.push_value(v);
                }
                Instr::LocalSet(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                }
                Instr::LocalGet(idx) => {
                    let v = self.get_local(*idx);
                    self.push_value(v);
                }
                Instr::LocalTee(idx) => {
                    let v = self.pop_value();
                    self.set_local(*idx, v);
                    self.push_value(v);
                }
                Instr::I32Const(v) => self.push_value(Val::I32(*v)),
                Instr::I64Const(v) => self.push_value(Val::I64(*v)),
                Instr::F32Const(v) => self.push_value(Val::F32(*v)),
                Instr::F64Const(v) => self.push_value(Val::F64(*v)),
                Instr::I32Sub => self.apply_binop_i32(|v0, v1| v0 - v1),
                Instr::I32Add => self.apply_binop_i32(|v0, v1| v0 + v1),
                Instr::I32Xor => self.apply_binop_i32(|v0, v1| v0 ^ v1),
                Instr::I32And => self.apply_binop_i32(|v0, v1| v0 & v1),
                Instr::I32LtS => self.apply_binop_i32(|v0, v1| if v0 < v1 { 1 } else { 0 }),
                Instr::I32LeS => self.apply_binop_i32(|v0, v1| if v0 <= v1 { 1 } else { 0 }),
                Instr::I32GtS => self.apply_binop_i32(|v0, v1| if v0 > v1 { 1 } else { 0 }),
                Instr::I32GeS => self.apply_binop_i32(|v0, v1| if v0 >= v1 { 1 } else { 0 }),
                Instr::I32Load8U(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = self.mem[i] as i32;
                    self.values.push(Val::I32(v));
                }
                Instr::I32Load(arg) => {
                    // TODO: handle alignment
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 4;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = i32::from_le_bytes(self.mem[start..end].try_into().unwrap()); // TODO
                    self.values.push(Val::I32(v));
                }
                Instr::I32Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::I32Store8(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let i = (i + arg.offset as i32) as usize;
                    if self.mem.len() < i {
                        return Err(ExecuteError::Trapped);
                    }
                    let v = v.as_i32().ok_or(ExecuteError::Trapped)? as u8; // TODO:
                    self.mem[i] = v;
                }
                Instr::I32Store16(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + 2;
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }

                    let v = v.as_i32().ok_or(ExecuteError::Trapped)? as i16; // TODO:
                    (&mut self.mem[start..end]).copy_from_slice(&v.to_le_bytes());
                }
                Instr::I64Store(arg) => {
                    // TODO: handle alignment
                    let v = self.pop_value();
                    let i = self.pop_value_i32();
                    let start = (i + arg.offset as i32) as usize;
                    let end = start + v.byte_size();
                    if self.mem.len() < end {
                        return Err(ExecuteError::Trapped);
                    }
                    v.copy_to(&mut self.mem[start..end]);
                }
                Instr::Block(block) => {
                    let prev_block = self.enter_block(block.blocktype);
                    let return_level =
                        self.execute_instrs(&block.instrs, level + 1, funcs, module)?;
                    let skipped = return_level.map_or(false, |return_level| return_level <= level);
                    self.exit_block(block.blocktype, skipped, prev_block);
                    if skipped {
                        return Ok(return_level);
                    }
                }
                Instr::Loop(block) => {
                    let current_level = level + 1;
                    let blocktype = Blocktype::Empty;
                    let prev_block = self.enter_block(blocktype);
                    loop {
                        let return_level =
                            self.execute_instrs(&block.instrs, current_level, funcs, module)?;
                        if return_level == Some(current_level) {
                            continue;
                        }
                        let skipped =
                            return_level.map_or(false, |return_level| return_level <= level);
                        self.exit_block(blocktype, skipped, prev_block);
                        if skipped {
                            return Ok(return_level);
                        }
                        break;
                    }
                }
                Instr::Br(label) => {
                    return Ok(Some(level - label.get()));
                }
                Instr::BrIf(label) => {
                    let c = self.pop_value_i32();
                    dbg!(c);
                    if c != 0 {
                        dbg!(level, label.get());
                        return Ok(Some(level - label.get()));
                    }
                }
                Instr::BrTable(table) => {
                    let i = self.pop_value_i32() as usize;
                    let label = table
                        .labels
                        .get(i)
                        .unwrap_or_else(|| table.labels.last().expect("unreachable"));
                    return Ok(Some(level - label.get()));
                }
                Instr::Return => {
                    return Ok(Some(self.current_frame.level));
                }
                Instr::Call(funcidx) => {
                    self.call_function(*funcidx, level + 1, funcs, module)?;
                }
                _ => todo!("{instr:?}"),
            }
        }
        Ok(None)
    }

    fn apply_binop_i32<F>(&mut self, f: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let v0 = self.pop_value_i32();
        let v1 = self.pop_value_i32();
        self.push_value(Val::I32(f(v1, v0)));
    }
}

impl<V: VectorFactory> Debug for Executor<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("Executor").finish_non_exhaustive()
    }
}

// TODO: Activation(?)
#[derive(Debug, Default, Clone, Copy)]
pub struct Frame {
    pub level: usize,
    pub arity: usize,
    pub locals_start: usize,
    pub values_start: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub arity: usize,
    pub values_start: usize,
}

#[cfg(test)]
mod tests {
    use crate::{Env, FuncInst, HostFunc, Module, Resolve, StdVectorFactory, Val};

    #[test]
    fn control_flow_br_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/br
        //
        // (module
        //   ;; import the browser console object, you'll need to pass this in from JavaScript
        //   (import "console" "log" (func $log (param i32)))
        //
        //   ;; create a global variable and initialize it to 0
        //   (global $i (mut i32) (i32.const 0))
        //
        //   (func
        //     (loop $my_loop
        //
        //       ;; add one to $i
        //       global.get $i
        //       i32.const 1
        //       i32.add
        //       global.set $i
        //
        //       ;; log the current value of $i
        //       global.get $i
        //       call $log
        //
        //       ;; if $i is less than 10 branch to loop
        //       global.get $i
        //       i32.const 10
        //       i32.lt_s
        //       br_if $my_loop
        //
        //     )
        //   )
        //
        //   (start 1) ;; run the first function automatically
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 8, 2, 96, 1, 127, 0, 96, 0, 0, 2, 15, 1, 7, 99, 111,
            110, 115, 111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 1, 6, 6, 1, 127, 1, 65, 0,
            11, 8, 1, 1, 10, 25, 1, 23, 0, 3, 64, 35, 0, 65, 1, 106, 36, 0, 35, 0, 16, 0, 35, 0,
            65, 10, 72, 13, 0, 11, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let instance = module.instantiate(Resolver).expect("instantiate");

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(10, host_func.messages.len());
        for (i, m) in host_func.messages.iter().enumerate() {
            assert_eq!(Val::I32((i + 1) as i32), *m);
        }
    }

    #[test]
    fn control_flow_block_test() {
        // From: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/block
        //
        // (module
        //   ;; import the browser console object, you'll need to pass this in from JavaScript
        //   (import "console" "log" (func $log (param i32)))
        //
        //   ;; create a function that takes in a number as a param,
        //   ;; and logs that number if it's not equal to 100.
        //   (func (export "log_if_not_100") (param $num i32)
        //     (block $my_block
        //
        //       ;; $num is equal to 100
        //       local.get $num
        //       i32.const 100
        //       i32.eq
        //
        //       (if
        //         (then
        //
        //           ;; branch to the end of the block
        //           br $my_block
        //
        //         )
        //       )
        //
        //       ;; not reachable when $num is 100
        //       local.get $num
        //       call $log
        //
        //     )
        //   )
        // )
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 1, 127, 0, 2, 15, 1, 7, 99, 111, 110, 115,
            111, 108, 101, 3, 108, 111, 103, 0, 0, 3, 2, 1, 0, 7, 18, 1, 14, 108, 111, 103, 95,
            105, 102, 95, 110, 111, 116, 95, 49, 48, 48, 0, 1, 10, 22, 1, 20, 0, 2, 64, 32, 0, 65,
            228, 0, 70, 4, 64, 12, 1, 11, 32, 0, 16, 0, 11, 11,
        ];
        let module = Module::<StdVectorFactory>::decode(&input).expect("decode");
        let mut instance = module.instantiate(Resolver).expect("instantiate");

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(99)])
            .expect("invoke")
            .is_none());

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(100)])
            .expect("invoke")
            .is_none());

        assert!(instance
            .invoke("log_if_not_100", &[Val::I32(101)])
            .expect("invoke")
            .is_none());

        let FuncInst::Imported { host_func, .. } = &instance.funcs()[0] else {
            panic!()
        };
        assert_eq!(&[Val::I32(99), Val::I32(101)][..], &host_func.messages);
    }

    #[derive(Debug)]
    struct Resolver;

    impl Resolve for Resolver {
        type HostFunc = Log;

        fn resolve_func(
            &self,
            module: &str,
            name: &str,
            params: &[crate::components::Valtype],
            result: crate::components::Resulttype,
        ) -> Option<Self::HostFunc> {
            if module == "console" && name == "log" && params.len() > 0 && result.len() == 0 {
                Some(Log::default())
            } else {
                None
            }
        }
    }

    #[derive(Debug, Default)]
    struct Log {
        messages: Vec<Val>,
    }

    impl HostFunc for Log {
        fn invoke(&mut self, args: &[Val], _env: &mut Env) -> Option<Val> {
            self.messages.push(args[0]);
            None
        }
    }
}
