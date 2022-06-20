#[no_mangle]
pub extern "C" fn print_int(x: u64) {
    println!("printing: {}", x);
}
