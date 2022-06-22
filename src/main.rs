mod scanner;
mod parser;
mod codegen;
mod typecheck;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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

    let ast = p.parse().expect("Error parsing AST!");

    let mut checker = typecheck::Checker::new();
    let checked_ast = checker.check(&ast).expect("Typechecking error!");

    let context = Context::create();
    let module = context.create_module("huck_main");
    let builder = context.create_builder();

    let fpm = PassManager::create(&module);

    // This code really needs some cleaning up but I just want
    // it to work for now
    let function = codegen::Compiler::compile(
        &context,
        &builder,
        &module,
        &fpm,
        checked_ast
    ).unwrap();
    function.print_to_stderr();
    let fname = "./huck";
    let path = Path::new(fname).with_extension("ll");
    // god what a hack
    let stdlib_path = "/home/pfogg/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/libstd-05b39ac0cb4c5688.so";
    module.print_to_file(&path).expect("Error writing IR!");
    println!("LLVM IR written to {}", fname);

    let output = Command::new("clang")
        .arg(&path)
        .arg("runtime.o")
        .arg("-o")
        .arg("hucktest")
        .arg(stdlib_path)
        .output()
        .unwrap();
    let status = output.status;
    if status.success() {
        println!("Successfully linked!");
    } else {
        println!("Linker error: {}", std::str::from_utf8(&output.stderr).unwrap());
    }
}
