use crate::ast::{Expr, Stmt, Type};
use crate::js_stdlib::JsStdLib;
use std::collections::HashMap;
use std::fmt::format;
use std::ptr::null;

pub struct TypeChecker {
    scope_stack: Vec<HashMap<String, Type>>,
    function_signatures: HashMap<String, Vec<Type>>,
    function_return_type: HashMap<String, Type>,
    current_return_type: Option<Type>,
    currently_async: bool,
    currently_loop: bool,
    js_stdlib: JsStdLib,
}

#[derive(Debug)]
pub struct TypeError {
    message: String,
}

impl TypeChecker {
    fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    fn declare_variable(&mut self, name: String, var_type: Type) {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.insert(name, var_type);
        }
    }

    fn lookup_variable(&self, name: &str) -> Option<&Type> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Some(var_type);
            }
        }
        None
    }

    fn matching_types(&self, target_type: &Type, value_type: &Type) -> bool {
        if target_type == value_type || *target_type == Type::Any {
            return true;
        }

        if let Type::Option(inner_type) = &target_type {
            if *value_type == **inner_type || *value_type == Type::Null {
                return true;
            }
        }

        false
    }

    fn get_method_return_type(
        &mut self,
        object: &Expr,
        method_name: &str,
    ) -> Result<Type, TypeError> {
        let obj_type = self.infer_type(object)?;

        if let Expr::Identifier(obj_name) = object {
            if let Some(return_type) = self.js_stdlib.get_method_type(obj_name, method_name) {
                return Ok(return_type);
            }
        }

        if let Some(return_type) = self
            .js_stdlib
            .get_primitive_method_type(&obj_type, method_name)
        {
            return Ok(return_type);
        }

        match obj_type {
            Type::Object(_) => Ok(Type::Any),
            _ => Err(TypeError {
                message: format!(
                    "Cannot call method '{}' on type {:?}",
                    method_name, obj_type
                ),
            }),
        }
    }

    pub fn print_current_scope(&mut self) {
        println!("\nCurrent Scope Variables:");
        println!("-----------------------");
        if let Some(current_scope) = self.scope_stack.last() {
            for (name, var_type) in current_scope {
                println!("{}: {:?}", name, var_type);
            }
        }
    }

    pub fn new() -> Self {
        let mut scope_stack = Vec::new();
        let mut global_scope = HashMap::new();

        let js_stdlib = JsStdLib::new();
        for (obj_name, methods) in &js_stdlib.objects {
            global_scope.insert(obj_name.clone(), Type::Object(methods.clone()));
        }

        scope_stack.push(global_scope);
        Self {
            scope_stack,
            function_signatures: HashMap::new(),
            function_return_type: HashMap::new(),
            current_return_type: None,
            currently_async: false,
            currently_loop: false,
            js_stdlib,
        }
    }

    pub fn check_program(&mut self, programm: &Stmt) -> Result<(), TypeError> {
        if let Stmt::Program { body } = programm {
            for stmt in body {
                self.check_statement(stmt)?;
            }

            println!("\nVariable Types:");
            println!("---------------");
            if let Some(global_scope) = self.scope_stack.first() {
                for (name, var_type) in global_scope {
                    println!("{}: {:?}", name, var_type);
                }
            }
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::VarDeclaration { .. } => self.check_var_declaration(stmt),
            Stmt::FunctionDeclaration { .. } => self.check_fn_declaration(stmt),
            Stmt::IfStatement { .. } => self.check_if_stmt(stmt),
            Stmt::WhileStatement { .. } => self.check_while_declaration(stmt),
            Stmt::ForLoopStatement { .. } => self.check_for_loop(stmt),
            Stmt::ForInLoopStatement { .. } => self.check_for_in_loop(stmt),
            Stmt::ForLoopIterated { .. } => self.check_for_iter_loop(stmt),
            Stmt::ReturnStatement { .. } => self.check_return_stmt(stmt),
            Stmt::TryCatchFinally { .. } => self.check_try_catch_stmt(stmt),
            Stmt::SwitchStatement { .. } => self.check_switch_stmt(stmt),
            Stmt::ContinueStatement | Stmt::BreakStatement => self.check_loop_control_stmt(stmt),
            Stmt::Expression(expr) => {
                self.infer_type(expr)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }
    fn check_var_declaration(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::VarDeclaration {
            identifier,
            value: Some(value),
            var_type,
            ..
        } = stmt
        {
            let expr_type = self.infer_type(value)?;

            let final_type = if *var_type == Type::Any {
                expr_type.clone()
            } else {
                var_type.clone()
            };

            if self.matching_types(var_type, &expr_type) {
                self.declare_variable(identifier.clone(), final_type);
                Ok(())
            } else {
                return Err(TypeError {
                    message: format!("Expected {:?} got {:?}", var_type, &expr_type),
                });
            }
        } else {
            panic!("Var declaration expected")
        }
    }
    fn check_fn_declaration(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::FunctionDeclaration {
            name,
            parameters,
            param_types,
            body,
            return_type,
            is_async,
        } = stmt
        {
            if *is_async {
                self.currently_async = true;
            }
            self.function_signatures
                .insert(name.clone(), param_types.clone());

            self.current_return_type = Some(return_type.clone());
            self.function_return_type
                .insert(name.clone(), return_type.clone());
            self.enter_scope();

            for (param, param_type) in parameters.iter().zip(param_types.iter()) {
                self.declare_variable(param.clone(), param_type.clone());
            }

            for stmt in body {
                self.check_statement(stmt)?;
            }

            self.exit_scope();

            self.current_return_type = None;

            if *is_async {
                self.currently_async = false;
            }

            Ok(())
        } else {
            panic!("function declaration expected")
        }
    }

    fn check_while_declaration(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::WhileStatement { condition, body } = stmt {
            self.currently_loop = true;
            let cond_type = self.infer_type(condition)?;

            if cond_type != Type::Boolean {
                return Err(TypeError {
                    message: format!("While condition must be boolean, got {:?}", cond_type),
                });
            }

            self.enter_scope();
            for stmt in body {
                self.check_statement(stmt)?;
            }
            self.exit_scope();
            self.currently_loop = false;
            Ok(())
        } else {
            Err(TypeError {
                message: "Expected while statement".to_string(),
            })
        }
    }

    fn check_if_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::IfStatement {
            condition,
            else_branch,
            else_if_branches,
            then_branch,
        } = stmt
        {
            let cond_type = self.infer_type(condition)?;

            if cond_type != Type::Boolean {
                return Err(TypeError {
                    message: format!("If condition must be boolean, got {:?}", cond_type),
                });
            }

            self.enter_scope();
            for stmt in then_branch {
                self.check_statement(stmt)?;
            }
            self.exit_scope();

            if let Some(else_if) = else_if_branches {
                for branch in else_if {
                    let branch_cond_type = self.infer_type(&branch.condition)?;

                    if branch_cond_type != Type::Boolean {
                        return Err(TypeError {
                            message: format!(
                                "Else-if condition must be boolean, got {:?}",
                                branch_cond_type
                            ),
                        });
                    }

                    self.enter_scope();
                    for stmt in &branch.body {
                        self.check_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }

            if let Some(else_stmts) = else_branch {
                self.enter_scope();
                for stmt in else_stmts {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            Ok(())
        } else {
            Err(TypeError {
                message: "Expected if statement".to_string(),
            })
        }
    }

    fn check_for_loop(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::ForLoopStatement {
            initializer,
            condition,
            update,
            body,
        } = stmt
        {
            self.currently_loop = true;
            self.enter_scope();

            if let Some(init) = initializer {
                self.check_statement(init)?;
            }
            if let Some(cond) = condition {
                let cond_type = self.infer_type(cond)?;
                if cond_type != Type::Boolean {
                    return Err(TypeError {
                        message: format!("For loop condition must be boolean, got {:?}", cond),
                    });
                }
            }

            if let Some(update_expr) = update {
                self.infer_type(update_expr)?;
            }

            for stmt in body {
                self.check_statement(stmt)?;
            }

            self.exit_scope();
            self.currently_loop = false;
            Ok(())
        } else {
            Err(TypeError {
                message: "Expected for loop statement".to_string(),
            })
        }
    }

    fn check_for_in_loop(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::ForInLoopStatement {
            iterable,
            iterator,
            body,
        } = stmt
        {
            self.currently_loop = true;
            let element_type = if let Some(itera) = iterable {
                let iter_type = self.infer_type(itera)?;
                match iter_type {
                    Type::Array(element_type) => Ok(*element_type),
                    _ => {
                        return Err(TypeError {
                            message: "iterable must be of type array".to_string(),
                        })
                    }
                }?
            } else {
                panic!("iteratable needed")
            };

            if let Some(iter) = iterator {
                if let Stmt::VarDeclaration { identifier, .. } = iter.as_ref() {
                    self.declare_variable(identifier.clone(), element_type.clone());
                }
            } else {
                panic!("iterator needed");
            }

            self.enter_scope();
            for stmt in body {
                self.check_statement(stmt)?;
            }
            self.exit_scope();
            self.currently_loop = false;
            Ok(())
        } else {
            Err(TypeError {
                message: "Expected for-in loop statement".to_string(),
            })
        }
    }

    fn check_for_iter_loop(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::ForLoopIterated {
            body,
            iterator_name,
            first_number,
            ..
        } = stmt{
            if let Some(iter) = iterator_name {
                self.check_statement(&Stmt::VarDeclaration {
                    constant: false,
                    identifier: iter.clone(),
                    value: first_number.clone(),
                    var_type: Type::Number,
                })?;
            }
            
            for stmt in body {
                self.check_statement(stmt)?;
            }
        
            Ok(())
        } else {
            panic!("For iter loop expected ")
        }
    }

    fn check_return_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::ReturnStatement { value } = stmt {
            if let Some(expected_value_type) = self.current_return_type.clone() {
                match value {
                    Some(expr) => {
                        let actual_return_type = self.infer_type(expr)?;

                        if !self.matching_types(&expected_value_type, &actual_return_type) {
                            return Err(TypeError {
                                message: format!(
                                    "Return type mismatch: expected {:?}, but got {:?}",
                                    expected_value_type, actual_return_type
                                ),
                            });
                        }
                    }
                    None => {
                        if expected_value_type != Type::Void {
                            return Err(TypeError {
                                message: format!(
                                    "Return type mismatch: expected {:?}, but function returns nothing (Void)",
                                    expected_value_type
                                ),
                            });
                        }
                    }
                }
            } else {
                return Err(TypeError {
                    message: "Return statement outside of function".to_string(),
                });
            }

            Ok(())
        } else {
            panic!("Return statement expected")
        }
    }

    fn check_try_catch_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::TryCatchFinally {
            try_branch,
            catch_branch,
            finally_branch,
        } = stmt
        {
            for stmt in try_branch {
                self.check_statement(stmt)?;
            }
            for stmt in catch_branch {
                self.check_statement(stmt)?;
            }
            if let Some(finally_branch) = finally_branch {
                for stmt in finally_branch {
                    self.check_statement(stmt)?;
                }
            }

            Ok(())
        } else {
            panic!("Try catch stmt expected")
        }
    }

    fn check_switch_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::SwitchStatement {
            case_branches,
            default_branch,
            ..
        } = stmt
        {
            for case_branch in case_branches {
                for stmt in &case_branch.body {
                    self.check_statement(stmt)?;
                }
            }
            for stmt in default_branch {
                self.check_statement(stmt)?
            }

            Ok(())
        } else {
            panic!("Expected swiotcvh stmt");
        }
    }

    fn check_loop_control_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if !self.currently_loop {
            return Err(TypeError {
                message: "Loop control cant be used outside loops".to_string(),
            });
        }
        self.currently_loop = false;
        Ok(())
    }

    fn infer_type(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::NumericLiteral(_) => Ok(Type::Number),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::BooleanLiteral(_) => Ok(Type::Boolean),
            Expr::NullLiteral => Ok(Type::Null),
            Expr::Identifier(name) => {
                if let Some(var_type) = self.lookup_variable(name).cloned() {
                    Ok(var_type)
                } else {
                    Err(TypeError {
                        message: format!("Nicht deklarierte Variable: {}", name),
                    })
                }
            }

            Expr::ArrayLiteral(..) => self.check_array_literal(expr),
            Expr::ObjectLiteral(..) => self.check_object_literal(expr),

            Expr::Member { .. } => self.check_member_expr(expr),

            Expr::Call { .. } => self.check_call_expr(expr),

            Expr::Assignment { .. } => {
                self.check_assignment(expr)?;
                Ok(Type::Any)
            }

            Expr::Unary { .. } => self.check_unary_expr(expr),

            Expr::AwaitExpression { .. } => self.check_await_expression(expr),

            Expr::Increment { .. } => self.check_increment(expr),

            Expr::CompoundAssignment { .. } => self.check_compund_assignment(expr),

            Expr::Binary { .. } => self.check_binary_expr(expr),

            _ => Ok(Type::Any),
        }
    }

    fn check_assignment(&mut self, expr: &Expr) -> Result<(), TypeError> {
        if let Expr::Assignment { assignee, value } = expr {
            let target_type = self.infer_type(assignee)?;
            let value_type = self.infer_type(value)?;

            if self.matching_types(&target_type, &value_type) {
                Ok(())
            } else {
                Err(TypeError {
                    message: format!("Expected {:?} got {:?}", target_type, value_type),
                })
            }
        } else {
            panic!("assignment expected");
        }
    }

    fn check_unary_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::Unary { value, .. } = expr {
            let value_type = self.infer_type(value)?;
            if !self.matching_types(&Type::Boolean, &value_type) {
                return Err(TypeError {
                    message: "Unary operator ! can only be applied to booleans".to_string(),
                });
            }

            Ok(Type::Boolean)
        } else {
            panic!("unary expression expected");
        }
    }

    fn check_await_expression(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::AwaitExpression { value } = expr {
            let value_type = self.infer_type(value)?;
            if !self.currently_async {
                return Err(TypeError {
                    message: "Await can only be called in async functions".to_string(),
                });
            }

            Ok(value_type)
        } else {
            panic!("await expression expected");
        }
    }

    fn check_increment(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::Increment { identifier, prefix } = expr {
            let identifier_type = self.infer_type(identifier)?;
            if identifier_type != Type::Number {
                return Err(TypeError {
                    message: "can call increwment only on type number".to_string(),
                });
            }
            Ok(Type::Number)
        } else {
            panic!("Expected Increment expr")
        }
    }

    fn check_compund_assignment(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::CompoundAssignment {
            assignee,
            value,
            operator,
        } = expr
        {
            let assignee_type = self.infer_type(assignee)?;
            let value_type = self.infer_type(value)?;

            if operator != "+=" && (assignee_type == Type::String || value_type == Type::String) {
                return Err(TypeError {
                    message: format!(
                        "Cannot use operator '{}' with a string value/assignee. Only '+=' is allowed for string concatenation.",
                        operator
                    ),
                });
            }

            let binary_expr = Expr::Binary {
                left: Box::new((**assignee).clone()),
                right: Box::new((**value).clone()),
                operator: operator.clone().replace("=", ""),
            };

            let new_expr = Expr::Assignment {
                assignee: Box::new((**assignee).clone()),
                value: Box::new(binary_expr),
            };

            self.check_assignment(&new_expr)?;
            Ok(Type::Any)
        } else {
            panic!("Expected compound assignment")
        }
    }

    fn check_binary_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::Binary {
            left,
            right,
            operator,
        } = expr
        {
            let left_type = self.infer_type(left)?;
            let right_type = self.infer_type(right)?;

            let both_numbers = left_type == Type::Number && right_type == Type::Number;
            let left_is_string = left_type == Type::String;
            let right_is_string = right_type == Type::String;

            match operator.as_str() {
                "+" => {
                    if both_numbers {
                        Ok(Type::Number)
                    } else if left_is_string || right_is_string {
                        Ok(Type::String)
                    } else {
                        Err(TypeError {
                            message: format!(
                                "Operator '+' kann nicht auf die Typen {:?} und {:?} angewendet werden. Erlaubt für Zahl + Zahl oder String + (String/Zahl).",
                                left_type, right_type
                            ),
                        })
                    }
                }

                "-" | "*" | "/" => {
                    if both_numbers {
                        Ok(Type::Number)
                    } else {
                        Err(TypeError {
                            message: format!(
                                "Arithmetischer Operator '{}' erfordert Zahlentypen, aber {:?} und {:?} wurden empfangen.",
                                operator, left_type, right_type
                            ),
                        })
                    }
                }

                "==" | "!=" | "<" | ">" | "<=" | ">=" | "||" | "&&" => Ok(Type::Boolean),

                _ => Err(TypeError {
                    message: format!("Unbekannter Operator '{}'", operator),
                }),
            }
        } else {
            panic!("Binary Expression exprected");
        }
    }

    fn check_array_literal(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::ArrayLiteral(values) = expr {
            if values.is_empty() {
                return Ok(Type::Array(Box::new(Type::Any)));
            }

            let first_type = self.infer_type(&values[0])?;

            for value in values.iter().skip(1) {
                let value_type = self.infer_type(value)?;
                if value_type != first_type && first_type != Type::Any && value_type != Type::Any {
                    return Err(TypeError {
                        message: format!(
                            "Array elements must have consistent types. Expected {:?}, got {:?}",
                            first_type, value_type
                        ),
                    });
                }
            }

            Ok(Type::Array(Box::new(first_type)))
        } else {
            panic!("array literal expected");
        }
    }

    fn check_call_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::Call { caller, args } = expr {
            match &**caller {
                Expr::Identifier(fn_name) => {
                    let param_types_opt = self.function_signatures.get(fn_name).cloned();
                    match param_types_opt {
                        Some(param_types) => {
                            if args.len() != param_types.len() {
                                return Err(TypeError {
                                    message: format!(
                                        "Funktion '{}' erwartet {} Argumente, aber {} wurden übergeben",
                                        fn_name,
                                        param_types.len(),
                                        args.len()
                                    ),
                                });
                            }

                            for (idx, (arg, expected_type)) in
                                args.iter().zip(param_types.iter()).enumerate()
                            {
                                let actual_type = self.infer_type(arg)?;

                                if actual_type != *expected_type && *expected_type != Type::Any {
                                    return Err(TypeError {
                                        message: format!(
                                            "Typfehler bei Funktionsaufruf '{}': Parameter {} sollte vom Typ {:?} sein, ist aber {:?}",
                                            fn_name,
                                            idx + 1,
                                            expected_type,
                                            actual_type
                                        ),
                                    });
                                }
                            }
                            if let Some(return_type) = self.function_return_type.get(fn_name) {
                                Ok(return_type.clone())
                            } else {
                                Ok(Type::Any)
                            }
                        }
                        None => Err(TypeError {
                            message: format!("Undefinierte Funktion: {}", fn_name),
                        }),
                    }
                }
                Expr::Member {
                    object, property, ..
                } => {
                    if let Expr::Identifier(prop_name) = property.as_ref() {
                        self.get_method_return_type(object, prop_name)
                    } else {
                        Err(TypeError {
                            message: "Property must be an identifier".to_string(),
                        })
                    }
                }
                _ => Ok(Type::Any),
            }
        } else {
            Err(TypeError {
                message: "Expected function call expression".to_string(),
            })
        }
    }

    fn check_object_literal(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::ObjectLiteral(properties) = expr {
            let mut property_types = HashMap::new();
            for property in properties {
                if let Some(value) = &property.value {
                    let property_type = self.infer_type(value)?;

                    property_types.insert(property.key.clone(), property_type);
                } else {
                    property_types.insert(property.key.clone(), Type::Any);
                }
            }
            Ok(Type::Object(property_types))
        } else {
            panic!("Object literal expected");
        }
    }

    fn check_member_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        if let Expr::Member {
            object,
            property,
            computed,
        } = expr
        {
            let obj_type = self.infer_type(object)?;

            match obj_type {
                Type::Array(element_type) => {
                    if *computed {
                        let index_type = self.infer_type(property)?;
                        if index_type != Type::Number {
                            return Err(TypeError {
                                message: "Array index must be a number".to_string(),
                            });
                        }
                        return Ok(*element_type.clone());
                    }
                }
                Type::Object(properties) => {
                    if let Expr::Identifier(prop_name) = property.as_ref() {
                        if let Some(prop_type) = properties.get(prop_name) {
                            return Ok(prop_type.clone());
                        } else {
                            return Err(TypeError {
                                message: format!(
                                    "Property '{}' does not exist on this object",
                                    prop_name
                                ),
                            });
                        }
                    } else if *computed {
                        let _prop_type = self.infer_type(property)?;
                        return Ok(Type::Any);
                    }
                }
                _ => {
                    return Err(TypeError {
                        message: format!("Cannot access property on type {:?}", obj_type),
                    });
                }
            }

            Ok(Type::Any)
        } else {
            panic!("Member expression expected");
        }
    }
}
