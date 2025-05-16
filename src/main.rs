mod ast;
mod compiler;
mod lexer;
mod parser;

use compiler::Compiler;
use std::fs::File;
use std::io::Write;

fn main() {
    let source = r#"
    let x = 5;
    
    
     while(true) {
        if(5 == 4) {
            x = 3;
        }else if(4 == 5){
            x = 2;
        }else {
            x = 4;
        }
     }   
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("Parsed Code {:?}", &ast);

    // Compiler initialisieren
    let mut compiler = Compiler {
        output: String::new(),
    };

    // AST kompilieren
    let compiled_output = compiler.compile_programm(&ast);

    // Ausgabe in eine Datei schreiben
    let mut output_file = File::create("output.js").expect("Konnte Ausgabedatei nicht erstellen");
    output_file
        .write_all(compiled_output.as_bytes())
        .expect("Konnte nicht in Ausgabedatei schreiben");

    println!("Kompilierung abgeschlossen. Ausgabe wurde in 'output.txt' gespeichert.");
}
