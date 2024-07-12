// $ cargo build --target wasm32-unknown-unknown --example fib

#[no_mangle]
pub fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
