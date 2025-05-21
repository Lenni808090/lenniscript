use crate::ast::{Expr, Stmt};
use crate::typechecker::TypeChecker;

pub struct Compiler {
    pub output: String,
    pub indent_level: usize,
}

impl Compiler {
    fn get_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn new() -> Self {
        Compiler {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn compile_programm(&mut self, program: &Stmt) -> Result<String, String> {
        if let Stmt::Program { body } = program {
            for stmt in body {
                let stmt = self.compile_stmt(stmt);
                self.output.push_str(&stmt);
                self.output.push('\n');
            }
        }

        Ok(self.output.clone())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDeclaration { .. } => self.compile_var_declaration(stmt),
            Stmt::ReturnStatement { .. } => self.compile_return_stmt(stmt),
            Stmt::IfStatement { .. } => self.compile_if_stmt(stmt),
            Stmt::WhileStatement { .. } => self.compile_while_stmt(stmt),
            Stmt::FunctionDeclaration { .. } => self.compile_fun_declaration(stmt),
            Stmt::ForLoopStatement { .. } => self.compile_for_loop(stmt),
            Stmt::ForInLoopStatement { .. } => self.compile_for_in_loop(stmt),
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
            ..
        } = stmt
        {
            let mut vardecl = String::new();
            if *constant {
                vardecl.push_str("const ");
            } else {
                vardecl.push_str("let ");
            }

            vardecl.push_str(identifier);

            let compiled_value = if let Some(val) = value {
                vardecl.push_str(" = ");
                self.compile_expr(val)
            } else {
                "".to_string()
            };

            vardecl.push_str(&compiled_value);

            vardecl
        } else {
            panic!("expected var declaration")
        }
    }

    fn compile_return_stmt(&mut self, stmt: &Stmt) -> String {
        if let Stmt::ReturnStatement { value } = stmt {
            if let Some(expr) = value {
                let compiled_expr = self.compile_expr(expr);
                format!("return {}", compiled_expr)
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

            compiled_if_stmt.push_str(" {\n");
            self.increase_indent();

            for stmt in then_branch {
                let stmt = self.compile_stmt(stmt);
                compiled_if_stmt.push_str(&format!("{}{}\n", self.get_indent(), stmt));
            }

            self.decrease_indent();
            compiled_if_stmt.push_str(&format!("{}}}", self.get_indent()));

            if let Some(else_if_branches) = else_if_branches {
                for else_if_branch in else_if_branches {
                    let compiled_else_if_cond = self.compile_expr(&else_if_branch.condition);
                    compiled_if_stmt.push_str(&format!(" else if({}) {{\n", compiled_else_if_cond));

                    self.increase_indent();
                    for stmt in &else_if_branch.body {
                        let stmt = self.compile_stmt(&stmt);
                        compiled_if_stmt.push_str(&format!("{}{}\n", self.get_indent(), stmt));
                    }
                    self.decrease_indent();

                    compiled_if_stmt.push_str(&format!("{}}}", self.get_indent()));
                }
            }

            if let Some(else_branch) = else_branch {
                compiled_if_stmt.push_str(&format!(" else {{\n"));

                self.increase_indent();
                for stmt in else_branch {
                    let stmt = self.compile_stmt(&stmt);
                    compiled_if_stmt.push_str(&format!("{}{}\n", self.get_indent(), stmt));
                }
                self.decrease_indent();

                compiled_if_stmt.push_str(&format!("{}}}", self.get_indent()));
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
            compiled_while_stmt.push_str(&format!("while ({})", condition)); // kein get_indent() hier
            compiled_while_stmt.push_str(" {\n");

            self.increase_indent();
            for stmt in body {
                let stmt = self.compile_stmt(&stmt);
                compiled_while_stmt.push_str(&format!("{}{}\n", self.get_indent(), stmt));
            }
            self.decrease_indent();

            compiled_while_stmt.push_str(&format!("{}}}", self.get_indent()));

            compiled_while_stmt
        } else {
            panic!("Expected a while stmt");
        }
    }

    fn compile_fun_declaration(&mut self, stmt: &Stmt) -> String {
        if let Stmt::FunctionDeclaration {
            name,
            parameters,
            body,
            ..
        } = stmt
        {
            let mut compiled_function: String = String::new();
            compiled_function.push_str(&format!("function {}(", name));

            if !parameters.is_empty() {
                compiled_function.push_str(&parameters.join(", "));
            }

            compiled_function.push_str(") {\n");
            self.increase_indent();

            for stmt in body {
                let compiled_stmt = self.compile_stmt(stmt);
                compiled_function.push_str(&format!("{}{}\n", self.get_indent(), compiled_stmt));
            }

            self.decrease_indent();
            compiled_function.push_str(&format!("{}}}", self.get_indent()));
            compiled_function
        } else {
            panic!("Function declaration expected")
        }
    }

    fn compile_for_loop(&mut self, stmt: &Stmt) -> String {
        if let Stmt::ForLoopStatement {
            initializer,
            condition,
            update,
            body,
        } = stmt
        {
            let mut compiled_for_loop = String::new();
            let mut comp_initializer = String::new();
            if let Some(init) = initializer {
                comp_initializer = self.compile_var_declaration(init);
            }
            let mut comp_condition = String::new();
            if let Some(cond) = condition {
                comp_condition = self.compile_binary_expr(cond);
            } else {
                panic!("No condition");
            }
            let mut comp_update = String::new();
            if let Some(up) = update {
                comp_update = self.compile_expr(up);
            } else {
                panic!("No update");
            }

            compiled_for_loop.push_str(&format!(
                "for ({}; {}; {})",
                comp_initializer, comp_condition, comp_update
            ));
            compiled_for_loop.push_str(" {\n");

            self.increase_indent();
            for stmt in body {
                let compiled_stmt = self.compile_stmt(stmt);
                compiled_for_loop.push_str(&format!("{}{}\n", self.get_indent(), compiled_stmt));
            }
            self.decrease_indent();

            compiled_for_loop.push_str(&format!("{}}}", self.get_indent()));
            compiled_for_loop
        } else {
            panic!("Expected for loop")
        }
    }

    fn compile_for_in_loop(&mut self, stmt: &Stmt) -> String {
        if let Stmt::ForInLoopStatement {
            iterator,
            iterable,
            body,
        } = stmt
        {
            let mut compiled_for_in = String::new();
            let mut comp_iterator = String::new();
            let mut comp_iterable = String::new();

            if let Some(iter) = iterator {
                comp_iterator = self.compile_stmt(iter);
            } else {
                panic!("iterator expected");
            }

            if let Some(itera) = iterable {
                comp_iterable = self.compile_expr(itera);
            } else {
                panic!("iterable expected");
            }

            compiled_for_in.push_str(&format!("for ({} in {})", comp_iterator, comp_iterable));
            compiled_for_in.push_str(" {\n");

            self.increase_indent();
            for stmt in body {
                let comp_stmt = self.compile_stmt(stmt);
                compiled_for_in.push_str(&format!("{}{}\n", self.get_indent(), comp_stmt));
            }
            self.decrease_indent();

            compiled_for_in.push_str(&format!("{}}}", self.get_indent()));
            compiled_for_in
        } else {
            panic!("Expected for in expression");
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary { .. } => self.compile_binary_expr(expr),
            Expr::NumericLiteral(val) => val.to_string(),
            Expr::BooleanLiteral(bool) => bool.to_string(),
            Expr::StringLiteral(string_literal) => format!("\"{}\"", string_literal),
            Expr::ArrayLiteral { .. } => self.compile_array_literal(expr),
            Expr::Identifier(ident) => ident.clone(),
            Expr::Assignment { .. } => self.compile_assignment_expr(expr),
            Expr::CompoundAssignment { .. } => self.compile_compound_expr(expr),
            Expr::ObjectLiteral(..) => self.compile_object_literal(expr),
            Expr::Member { .. } => self.compile_member_expr(expr),
            Expr::Call { .. } => self.compile_call_expr(expr),
            Expr::Increment { .. } => self.compile_increment_expr(expr),
            _ => {
                panic!("expression not implemented {:?}", expr);
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

    fn compile_array_literal(&mut self, expr: &Expr) -> String {
        if let Expr::ArrayLiteral(values) = expr {
            let mut compiled_array: String = String::new();
            compiled_array.push('[');
            for expr in values {
                let expr = self.compile_expr(expr);
                compiled_array.push_str(&format!("{}, ", expr));
            }
            compiled_array.push(']');
            compiled_array
        } else {
            panic!("array literal expected")
        }
    }

    fn compile_assignment_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Assignment { assignee, value } = expr {
            let mut compiled_assignment: String = String::new();
            let assigne = self.compile_expr(assignee);
            let value = self.compile_expr(value);

            compiled_assignment.push_str(&format!("{} = {}", assigne, value));

            compiled_assignment
        } else {
            panic!("Expected Assignment Expresseion");
        }
    }

    fn compile_compound_expr(&mut self, expr: &Expr) -> String {
        if let Expr::CompoundAssignment {
            assignee,
            value,
            operator,
        } = expr
        {
            let compiled_assignee = self.compile_expr(assignee);
            let compile_value = self.compile_expr(value);

            format!("{} {} {}", compiled_assignee, operator, compile_value)
        } else {
            panic!("Compound Statement expected");
        }
    }

    fn compile_object_literal(&mut self, expr: &Expr) -> String {
        if let Expr::ObjectLiteral(properties) = expr {
            let mut compiled_object: String = String::new();
            compiled_object.push('{');
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
            compiled_object.push('}');
            compiled_object
        } else {
            panic!("Object literal erwartet")
        }
    }

    fn compile_member_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Member {
            object,
            property,
            computed,
        } = expr
        {
            let mut compiled_member: String = String::new();
            let object_compiled = self.compile_expr(object);
            let property_compiled = self.compile_expr(property);
            if *computed {
                compiled_member.push_str(&format!("{}[{}]", object_compiled, property_compiled));
            } else {
                compiled_member.push_str(&format!("{}.{}", object_compiled, property_compiled));
            }
            compiled_member
        } else {
            panic!("Expected Member Expression");
        }
    }

    fn compile_call_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Call { caller, args } = expr {
            let mut compiled_call: String = String::new();
            let caller_compiled = self.compile_expr(caller);
            compiled_call.push_str(&format!("{}(", caller_compiled));
            for (i, arg) in args.iter().enumerate() {
                let compiled_arg = self.compile_expr(arg);
                compiled_call.push_str(&compiled_arg);

                if i < args.len() - 1 {
                    compiled_call.push_str(", ");
                }
            }

            compiled_call.push(')');

            compiled_call
        } else {
            panic!("Expected Call Expr")
        }
    }

    fn compile_increment_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Increment { identifier, prefix } = expr {
            let compiled_identifier = self.compile_expr(identifier);
            if *prefix {
                format!("++{}", compiled_identifier)
            } else {
                format!("{}++", compiled_identifier)
            }
        }else { 
            panic!("Expected increment expression")
        }
    }
}
