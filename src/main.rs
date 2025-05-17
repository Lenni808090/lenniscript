mod ast;
mod compiler;
mod lexer;
mod parser;

use compiler::Compiler;
use std::fs::File;
use std::io::Write;

fn main() {
    let source = r#"
    let x = {
        foo: "bar",
        hi,
    };
    
    let i = 0; 
     while(i < 5) {
        if(5 == 4) {
            x.foo = ["3", "bar"];
        }else if(4 == 5){
            x.foo = "2";
        }else {
            x.foo = "4";
        }
        i = i + 1;
     }
     console.log(x.foo[1]);
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("Parsed Code {:?}", &ast);

    // Compiler initialisieren
    let mut compiler = Compiler {
        output: String::new(),
        indent_level: 0,
    };

    // AST kompilieren
    let compiled_output = compiler.compile_programm(&ast);

    // Ausgabe in eine Datei schreiben
    let mut output_file = File::create("output.js").expect("Konnte Ausgabedatei nicht erstellen");
    output_file
        .write_all(compiled_output.as_bytes())
        .expect("Konnte nicht in Ausgabedatei schreiben");

    println!("Kompilierung abgeschlossen. Ausgabe wurde in 'output.js' gespeichert.");
}