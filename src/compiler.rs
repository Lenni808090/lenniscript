use crate::ast::{Expr, Stmt};
use std::fmt::format;

pub struct Compiler {
    pub output: String,
}

impl Compiler {
    pub fn compile_programm(&mut self, program: &Stmt) -> String {
        if let Stmt::Program { body } = program {
            for stmt in body {
                let stmt = self.compile_stmt(&stmt);
                self.output.push_str(&stmt);
            }
        }

        self.output.clone()
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDeclaration { .. } => self.compile_var_declaration(stmt),
            Stmt::ReturnStatement { .. } => self.compile_return_stmt(stmt),
            Stmt::IfStatement { .. } => self.compile_if_stmt(stmt),
            Stmt::WhileStatement { .. } => self.compile_while_stmt(stmt),
            Stmt::Expression(expr) => self.compile_expr(expr),
            _ => {
                panic!("stmt type not unimplemented");
            }
        }
    }

    fn compile_var_declaration(&mut self, stmt: &Stmt) -> String {
        if let Stmt::VarDeclaration {
            constant,
            identifier,
            value,
        } = stmt
        {
            let mut vardecl = String::new();
            if *constant {
                vardecl.push_str("const ");
            } else {
                vardecl.push_str("let ");
            }

            vardecl.push_str(identifier);
            vardecl.push_str(" = ");

            let compiled_value =
                self.compile_expr(value.as_ref().expect("Fehlender Initialisierungswert"));
            vardecl.push_str(&compiled_value);

            vardecl.push_str(";");

            vardecl
        } else {
            panic!("expected var declaration")
        }
    }

    fn compile_return_stmt(&mut self, stmt: &Stmt) -> String {
        if let Stmt::ReturnStatement { value } = stmt {
            if let Some(expr) = value {
                let compiled_expr = self.compile_expr(expr);
                format!("return {};", compiled_expr)
            } else {
                "return;".to_string()
            }
        } else {
            panic!("Expected a return statement");
        }
    }

    fn compile_if_stmt(&mut self, stmt: &Stmt) -> String {
        if let Stmt::IfStatement {
            condition,
            else_if_branches,
            then_branch,
            else_branch,
        } = stmt
        {
            let mut compiled_if_stmt: String = String::new();
            let compiled_condition: String = self.compile_expr(condition);
            compiled_if_stmt.push_str(&format!("if ({})", compiled_condition));

            compiled_if_stmt.push_str(" {");

            for stmt in then_branch {
                let stmt = self.compile_stmt(stmt);
                compiled_if_stmt.push_str(&stmt);
            }

            compiled_if_stmt.push_str("}");

            if let Some(else_if_branches) = else_if_branches {
                for else_if_branch in else_if_branches {
                    let compiled_else_if_cond = self.compile_expr(&else_if_branch.condition);
                    compiled_if_stmt.push_str(&format!(" else if({}) ", compiled_else_if_cond));
                    compiled_if_stmt.push_str("{");
                    for stmt in &else_if_branch.body {
                        let stmt = self.compile_stmt(&stmt);
                        compiled_if_stmt.push_str(&stmt);
                    }
                    compiled_if_stmt.push_str("}");
                }
            }

            if let Some(else_branch) = else_branch {
                compiled_if_stmt.push_str(" else {");
                for stmt in else_branch {
                    let stmt = self.compile_stmt(&stmt);
                    compiled_if_stmt.push_str(&stmt);
                }
                compiled_if_stmt.push_str("}");
            }

            compiled_if_stmt
        } else {
            panic!("If statement expected")
        }
    }

    fn compile_while_stmt(&mut self, stmt: &Stmt) -> String {
        if let Stmt::WhileStatement { condition, body } = stmt {
            let mut compiled_while_stmt: String = String::new();
            let condition: String = self.compile_expr(&condition);
            compiled_while_stmt.push_str(&format!("while ({})", condition));
            compiled_while_stmt.push_str(" {");
            for stmt in body {
                let stmt = self.compile_stmt(&stmt);
                compiled_while_stmt.push_str(&stmt);
            }
            compiled_while_stmt.push_str("}");

            compiled_while_stmt
        } else {
            panic!("Expected a while stmt");
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary { .. } => self.compile_binary_expr(expr),
            Expr::NumericLiteral(val) => val.to_string(),
            Expr::BooleanLiteral(bool) => bool.to_string(),
            Expr::StringLiteral(string_literal) => format!("\"{}\"", string_literal),
            Expr::Identifier(ident) => ident.clone(),
            Expr::Assignment { .. } => self.compile_assignment_expr(expr),
            Expr::ObjectLiteral(..) => self.compile_object_literal(expr),
            _ => {
                panic!("expression not implemented");
            }
        }
    }

    fn compile_binary_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Binary {
            left,
            right,
            operator,
        } = expr
        {
            let left_value = self.compile_expr(left);
            let right_value = self.compile_expr(right);
            let op = match operator.as_str() {
                "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&"
                | "||" => operator.clone(),
                _ => panic!("unsupported operator: {}", operator),
            };

            format!("({} {} {})", left_value, op, right_value)
        } else {
            panic!("expected binary expression")
        }
    }

    fn compile_assignment_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Assignment { assignee, value } = expr {
            let mut compiled_assignment: String = String::new();
            let assigne = self.compile_expr(assignee);
            let value = self.compile_expr(value);

            compiled_assignment.push_str(&format!("{} = {};", assigne, value));

            compiled_assignment
        } else {
            panic!("Expected Assignment Expresseion");
        }
    }

    fn compile_object_literal(&mut self, expr: &Expr) -> String {
        if let Expr::ObjectLiteral(properties) = expr {
            let mut compiled_object: String = String::new();
            compiled_object.push_str("{");
            for (i, property) in properties.iter().enumerate() {
                if let Some(value) = &property.value {
                    let property_compiled = self.compile_expr(value);
                    compiled_object.push_str(&format!("{}: {}", property.key, property_compiled));
                } else {
                    compiled_object.push_str(&format!("{}: null", property.key));
                }

                if i < properties.len() - 1 {
                    compiled_object.push_str(", ");
                }
            }
            compiled_object.push_str("}");
            compiled_object
        } else {
            panic!("Object literal erwartet")
        }
    }
}
