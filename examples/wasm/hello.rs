// $ cargo build --target wasm32-unknown-unknown --example hello
extern "C" {
    fn print(s: *const u8, len: i32);
}

#[no_mangle]
pub fn hello() {
    let msg = "Hello, world!\n";
    unsafe {
        print(msg.as_ptr(), msg.len() as i32);
    }
}