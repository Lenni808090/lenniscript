mod ast;
mod compiler;
mod lexer;
mod parser;

mod typechecker;
use compiler::Compiler;
use std::fs::File;
use std::io::Write;

fn main() {
    let source = r#"
        fn test(num1: num, num2: num){
            let res: num = num1 + num2;
            return res;
        }
        let x: array<bool> = [true,false];
        x[1] = true;
        for(let wahr in x){
            let i = 0;
            x[i] = wahr;
            i = i + 1;
        }
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("{:?}", &ast);
    // Add type checking before compilation
    let mut type_checker = typechecker::TypeChecker::new();
    match type_checker.check_program(&ast) {
        Ok(_) => {
            let mut compiler = Compiler::new();
            match compiler.compile_programm(&ast) {
                Ok(compiled_output) => {
                    let mut output_file =
                        File::create("output.js").expect("Konnte Ausgabedatei nicht erstellen");
                    output_file
                        .write_all(compiled_output.as_bytes())
                        .expect("Konnte nicht in Ausgabedatei schreiben");
                    println!("Kompilierung erfolgreich abgeschlossen.");
                }
                Err(error) => {
                    eprintln!("Type Error: {:?}", error);
                }
            }
        }
        Err(error) => {
            eprintln!("Fehler beim Kompilieren: {:?}", error);
        }
    }
}
