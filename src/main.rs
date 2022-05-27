mod scanner;
mod parser;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let data = fs::read_to_string(path);
    match data {
        Ok(text) => parse_file(text),
        Err(err) => println!("Error reading source file: [{}]", err),
    }
}

fn parse_file(text: String) {
    let tokens = scanner::Scanner::new(&text).peekable();

    let mut p = parser::Parser::new(tokens);

    match p.parse() {
        Ok(ast) => println!("{:?}", ast),
        Err(err) => println!("Error: {:?}", err),
    }
}
