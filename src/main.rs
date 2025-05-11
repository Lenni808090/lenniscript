mod ast;
mod lexer;
mod parser;
mod compiler;

mod native_function_impl;

use inkwell::context::Context;
use crate::compiler::Compiler;

fn main() {
    // Example with a print function call
    let source = r#"
        print(5 + 3 * 2);
        print(10 - 4 / 2);
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("Parsed AST: {:?}", &ast);

    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    
    // Compile the program
    compiler.compile_program(&ast);
    
    // Get the LLVM IR
    let llvm_ir = compiler.get_llvm_ir();
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    std::fs::write("output.ll", &llvm_ir)
        .expect("Failed to write LLVM IR to file");
        
    println!("LLVM IR written to output.ll");
}