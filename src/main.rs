use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let data = fs::read_to_string(path);
    match data {
        Ok(text) => println!("{}", text),
        Err(err) => println!("Error reading source file: [{}]", err),
    }
}
