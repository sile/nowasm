use nowasm::{Env, HostFunc, Module, Resolve, StdVectorFactory, Val};

pub fn main() {
    let wasm_bytes = include_bytes!("../target/wasm32-unknown-unknown/debug/examples/hello.wasm");

    let module = Module::<StdVectorFactory>::decode(wasm_bytes).expect("Failed to decode module");

    let mut instance = module
        .instantiate(Resolver)
        .expect("Failed to instantiate module");

    instance
        .invoke("hello", &[])
        .expect("Failed to invoke function");
}

struct Resolver;

impl Resolve for Resolver {
    type HostFunc = Print;

    fn resolve_func(&self, module: &str, name: &str) -> Option<Self::HostFunc> {
        assert_eq!(module, "env");
        assert_eq!(name, "print");
        Some(Print)
    }
}

struct Print;

impl HostFunc for Print {
    fn invoke(&mut self, args: &[Val], env: &mut Env) -> Option<Val> {
        let ptr = args[0].as_i32().unwrap() as usize;
        let len = args[1].as_i32().unwrap() as usize;
        let string = std::str::from_utf8(&env.mem[ptr..ptr + len]).expect("Invalid utf8");
        print!("{string}");
        None
    }
}
