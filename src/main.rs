mod scanner;
mod parser;
mod codegen;
mod typecheck;

use std::env;
use std::fs;
use std::io::stdout;

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

    let ast = p.parse().expect("Error parsing AST!");

    let mut checker = typecheck::Checker::new();
    let checked_ast = checker.check(&ast).expect("Typechecking error!");

    codegen::compile(checked_ast, &mut stdout());
}
