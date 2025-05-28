mod ast;
mod compiler;
mod js_stdlib;
mod lexer;
mod parser;
mod typechecker;
use compiler::Compiler;
use std::fs::File;
use std::io::Write;

fn main() {
    let source = r#"
        async fn test(num1: num, num2: num) -> num {
            try {
                num1 += 1;
                let res = num1 + num2;
                return res;
                for(let i = 0; i < 10; ++i) {
                    console.log("hi");
                    if (i == 6) {
                        break;
                    }
                }
                let zahl: num = await 5+5;
            }
            catch {
                console.log("ERROR");
            }
            finally {
                console.log("clean up");
            }
        }
         let test: string? = "hihih";
         test = "hi";
         test = null;
         let x = 2;
         let y = 3;
         
         switch (y) {
            case 3 => {
                console.log("3");
            }

            default => {
                console.log("unknown");
            }
         }

         let unaryTest: bool = false;

         if(!unaryTest) {
            console.log("unaryTest ist falsch");
         }
         
         if(x == y || x != y){
            console.log(x,y);
         }

        for(0..9 as lol){
            console.log("hi");
        }

        let result: num = test(2, 4);
        let stri = {
            str: "hi"
        };
        let array1: array<num> = [1,1,2,2];
        result = array1[2];
    "#;

    let mut parser = parser::Parser::new();
    let ast = parser.produceAst(source);
    println!("{:?}", &ast);
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
