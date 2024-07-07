use clap::Parser;
use nowasm::{
    execution::{FrameRef, ModuleInstance, Stacks, Value},
    Allocator, Module, Vector,
};
use orfail::{Failure, OrFail};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
    func_name: String,
    func_args: Vec<i32>,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;

    let module = Module::<StdAllocator>::decode(&wasm_bytes)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let mem = StdVec(vec![0; 1024 * 1024]);
    let mut instance = ModuleInstance::new(module, ExampleStacks::default(), mem)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let func_args: Vec<_> = args.func_args.iter().copied().map(Value::I32).collect();
    let result = instance
        .invoke(&args.func_name, &func_args)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    println!("=> {:?}", result);

    Ok(())
}

#[derive(Debug, Default)]
pub struct ExampleStacks {
    frames: Vec<ExampleFrame>,
    values: Vec<Value>,
}

impl Stacks for ExampleStacks {
    fn push_frame(&mut self, locals: usize) {
        self.frames.push(ExampleFrame {
            locals: vec![Value::I32(0); locals],
            values_start: self.values.len(),
        });
    }

    fn pop_frame(&mut self) {
        let frame = self.frames.pop().expect("unreachable");
        self.values.truncate(frame.values_start);
    }

    fn current_frame(&mut self) -> FrameRef {
        let Some(last) = self.frames.last_mut() else {
            return FrameRef { locals: &mut [] };
        };
        FrameRef {
            locals: &mut last.locals,
        }
    }

    fn push_value(&mut self, value: Value) {
        self.values.push(value);
    }

    fn pop_value(&mut self) -> Value {
        self.values.pop().expect("unreachable")
    }
}

#[derive(Debug, Clone)]
struct ExampleFrame {
    locals: Vec<Value>,
    values_start: usize,
}

#[derive(Debug, Clone)]
pub struct StdAllocator;

impl Allocator for StdAllocator {
    type Vector<T: Clone + Debug> = StdVec<T>;

    fn allocate_vector<T: Clone + Debug>() -> Self::Vector<T> {
        StdVec(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct StdVec<T>(pub Vec<T>);

// TODO: remove Debug
impl<T: Debug + Clone> Vector<T> for StdVec<T> {
    fn push(&mut self, item: T) {
        self.0.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T> AsRef<[T]> for StdVec<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> AsMut<[T]> for StdVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}
