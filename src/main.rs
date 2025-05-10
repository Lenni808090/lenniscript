mod ast;
mod compiler;
mod lexer;
mod parser;

use compiler::Compiler;
use inkwell::context::Context;

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

    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    compiler.compile_program(&ast);
    
    let llvm_ir = compiler.get_llvm_ir();
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    std::fs::write("output.ll", llvm_ir)
        .expect("Failed to write LLVM IR to file");
}
