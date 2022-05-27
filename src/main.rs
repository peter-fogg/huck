mod scanner;
mod parser;

use parser::HuckAst;

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
        Ok(ast) => println!("{:?}", eval(ast)),
        Err(err) => println!("Error: {:?}", err),
    }
}

fn eval(ast: HuckAst) -> Option<u64> {
    match ast {
        HuckAst::Num(n) => Some(n),
        HuckAst::Plus(l, r) => Some(eval(*l)? + eval(*r)?),
        HuckAst::Minus(l, r) => Some(eval(*l)? - eval(*r)?),
        HuckAst::Times(l, r) => Some(eval(*l)? * eval(*r)?),
        HuckAst::Div(l, r) => Some(eval(*l)? / eval(*r)?),
    }
}
