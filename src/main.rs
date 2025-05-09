mod ast;
mod compiler;
mod lexer;
mod parser;
mod runtime;

use compiler::Compiler;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let source = r#"
        let x = 2;
        while(x < 2){
            x = x + 1;
            return x;
        }
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("Parsed Code {:?}", &ast);

    let mut compiler = Compiler::new();
    match compiler.compile_and_run(&ast) {
        Ok(output) => {
            println!("Program output:\n{}", output);
            println!("Compiled code has been saved to 'output.rs'");
        }
        Err(err) => {
            eprintln!("Error running program: {}", err);
        }
    }
}
