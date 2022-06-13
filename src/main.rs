mod scanner;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::path::Path;

use inkwell::context::Context;
use inkwell::passes::PassManager;

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
        Ok(ast) => {
            let context = Context::create();
            let module = context.create_module("huck_main");
            let builder = context.create_builder();

            let fpm = PassManager::create(&module);

            codegen::Compiler::compile(
                &context,
                &builder,
                &module,
                &fpm,
                ast
            ).expect("Compilation error!");

            let fname = "./huck.out";

            match module.print_to_file(Path::new(fname)) {
                Ok(_) => println!("LLVM IR written to {}", fname),
                Err(_) => println!("Error writing IR!")
            }
        },
        Err(err) => println!("Error: {:?}", err),
    }
}
