use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    file_path: String,
}

fn main() {
    let cli = Cli::parse();
    let file_path = Path::new(&cli.file_path);

    // Überprüfen, ob die Datei existiert und eine .ls-Erweiterung hat
    if !file_path.exists() {
        eprintln!(
            "Fehler: Die angegebene Datei existiert nicht: {}",
            cli.file_path
        );
        std::process::exit(1);
    }

    if file_path.extension().unwrap_or_default() != "ls" {
        eprintln!("Fehler: Die angegebene Datei ist keine .ls-Datei");
        std::process::exit(1);
    }

    // Lesen des Inhalts der .ls-Datei
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Fehler beim Lesen der Datei: {}", e);
            std::process::exit(1);
        }
    };

    // Importieren und Ausführen der benötigten Module aus Ihrem Projekt
    let mut parser = rust_rewritr_programming_language::parser::Parser::new();
    let ast = parser.produceAst(&source);
    println!("Geparster Code: {:?}", &ast);

    // Compiler initialisieren
    let mut compiler = rust_rewritr_programming_language::compiler::Compiler {
        output: String::new(),
        indent_level: 0,
    };

    // AST kompilieren
    let compiled_output = compiler.compile_programm(&ast);

    // Speichern der kompilierten Ausgabe in eine JavaScript-Datei
    let js_output_path = "temp_output.js";
    let mut output_file =
        fs::File::create(js_output_path).expect("Konnte temporäre Ausgabedatei nicht erstellen");
    output_file
        .write_all(compiled_output.as_bytes())
        .expect("Konnte nicht in temporäre Ausgabedatei schreiben");

    // Ausführen des generierten JavaScript-Codes mit Node.js - wichtige Änderung hier:
    println!("\nAusgabe des Programms:");

    // Diese Methode führt den Befehl mit direkter Weiterleitung der Ein-/Ausgabe aus
    let status = Command::new("node")
        .arg(js_output_path)
        .spawn()
        .expect("Fehler beim Starten von Node.js")
        .wait()
        .expect("Fehler beim Ausführen des JavaScript-Codes");

    // Löschen der temporären Datei
    fs::remove_file(js_output_path).ok();

    if !status.success() {
        eprintln!(
            "\nProgramm wurde mit Fehlercode beendet: {}",
            status.code().unwrap_or(-1)
        );
    }
}
