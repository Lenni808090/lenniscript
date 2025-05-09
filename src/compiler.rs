use crate::ast::{ElseIfBranch, Expr, Property, Stmt};
use std::process::Command;
use std::fs;
use std::io::Write;

pub struct Compiler {
    output: String,
    indent_level: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn write(&mut self, str: &str) {
        self.output.push_str(str);
    }

    fn write_ch(&mut self, ch: char) {
        self.output.push(ch);
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn write_indent(&mut self) {
        self.write(&"    ".repeat(self.indent_level));
    }

    pub fn compile_program(&mut self, program: &Stmt) -> String {
        self.write("fn main() {\n");
        self.indent();

        if let Stmt::Program { body } = program {
            for stmt in body {
                self.compile_statement(stmt);
            }
        }

        self.dedent();
        self.write("}\n");
        
        let output = self.output.clone();
        if let Err(err) = fs::write("output.rs", &output) {
            eprintln!("Failed to write output file: {}", err);
        }
        
        output
    }

    fn compile_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration {
                constant,
                identifier,
                value,
            } => {
                self.write_indent();
                if *constant {
                    self.write("let ");
                } else {
                    self.write("let mut ");
                }
                self.write(identifier);

                if let Some(val) = value {
                    self.write(" = ");
                    self.compile_expression(val);
                }
                self.write(";\n");
            }

            Stmt::ReturnStatement { value } => {
                self.write_indent();
                self.write("return");
                if let Some(val) = value {
                    self.write_ch(' ');
                    self.compile_expression(val);
                }
                self.write(";\n");
            }

            Stmt::IfStatement { .. } => {
                self.compile_if_statement(stmt);
            }

            Stmt::WhileStatement { .. } => {
                self.compile_while_statement(stmt);
            }

            Stmt::Expression(expr) => {
                self.write_indent();
                self.compile_expression(expr);
                self.write(";\n");
            }

            _ => panic!("compile_statement: unimplemented statement type"),
        }
    }

    fn compile_while_statement(&mut self, stmt: &Stmt) {
        if let Stmt::WhileStatement { condition, body } = stmt {
            self.write_indent();
            self.write("while ");
            self.compile_expression(condition);
            self.write(" {\n");
            self.indent();
            for stmt in body {
                self.compile_statement(stmt);
            }
            self.dedent();
            self.write_indent();
            self.write("}\n");
        }
    }

    fn compile_if_statement(&mut self, stmt: &Stmt) {
        if let Stmt::IfStatement {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } = stmt
        {
            // "if" block
            self.write_indent();
            self.write("if ");
            self.compile_expression(condition);
            self.write(" {\n");

            self.indent();
            for stmt in then_branch {
                self.compile_statement(stmt);
            }
            self.dedent();

            self.write_indent();
            self.write_ch('}');

            // "else if" blocks
            if let Some(branches) = else_if_branches {
                for elseif in branches {
                    self.write(" else if ");
                    self.compile_expression(&elseif.condition);
                    self.write(" {\n");

                    self.indent();
                    for stmt in &elseif.body {
                        self.compile_statement(stmt);
                    }
                    self.dedent();

                    self.write_indent();
                    self.write_ch('}');
                }
            }

            // "else" block
            if let Some(stmts) = else_branch {
                self.write(" else {\n");

                self.indent();
                for stmt in stmts {
                    self.compile_statement(stmt);
                }
                self.dedent();

                self.write_indent();
                self.write_ch('}');
            }

            self.write_ch('\n');
        } else {
            panic!("compile_if_statement called with non-IfStatement");
        }
    }

    fn compile_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::NumericLiteral(num) => {
                self.write(&num.to_string());
            }
            Expr::StringLiteral(s) => {
                self.write_ch('"');
                self.write(s);
                self.write_ch('"');
            }
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                self.compile_expression(left);
                self.write(&format!(" {} ", operator));
                self.compile_expression(right);
            }
            Expr::Identifier(name) => {
                self.write(name);
            }
            Expr::Assignment { .. } => {
                self.compile_assignment_expression(expr);
            }
            _ => panic!("compile_expression: unimplemented expression type"),
        }
    }

    fn compile_assignment_expression(&mut self, expr: &Expr) {
        if let Expr::Assignment { assignee, value } = expr {
            if let Expr::Identifier(ref name) = **assignee {
                self.write(name.as_str());
                self.write(" = ");
                self.compile_expression(value);
            }
        }
    }

    pub fn compile_and_run(&mut self, program: &Stmt) -> Result<String, String> {
        // First compile the program
        let compiled_code = self.compile_program(program);
        
        // Write the compiled code to a temporary file
        let temp_file = "temp_program.rs";
        match fs::write(temp_file, &compiled_code) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to write temporary file: {}", e)),
        }

        // Compile the Rust code using rustc
        let compile_status = Command::new("rustc")
            .arg(temp_file)
            .status()
            .map_err(|e| format!("Failed to compile: {}", e))?;

        if !compile_status.success() {
            return Err("Compilation failed".to_string());
        }

        // Run the compiled program
        let output = Command::new("./temp_program")
            .output()
            .map_err(|e| format!("Failed to run program: {}", e))?;

        // Clean up temporary files
        let _ = fs::remove_file(temp_file);
        let _ = fs::remove_file("temp_program.exe"); // For Windows
        let _ = fs::remove_file("temp_program"); // For Unix-like systems

        // Return the program output
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
