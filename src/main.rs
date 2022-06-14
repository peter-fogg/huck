mod scanner;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::path::Path;

use inkwell::OptimizationLevel;
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

            match codegen::Compiler::compile(
                &context,
                &builder,
                &module,
                &fpm,
                ast
            ) {
                Ok(function) => {
                    function.print_to_stderr();
                    let fname = "./huck";
                    match module.print_to_file(Path::new(fname).with_extension("ll")) {
                        Ok(_) => println!("LLVM IR written to {}", fname),
                        Err(_) => println!("Error writing IR!")
                    }

                    let ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

                    let maybe_fn = unsafe {
                        ee.get_function::<unsafe extern "C" fn() -> u64>("main")
                    };

                    let f = maybe_fn.expect("Error getting main function!");

                    unsafe {
                        println!("\n\n=> {}", f.call());
                    }
                }
                Err(err) => println!("Error compiling file: [{:?}]", err)
            }
        },
        Err(err) => println!("Error: {:?}", err),
    }
}
