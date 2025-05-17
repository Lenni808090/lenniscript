use crate::ast::{Expr, Stmt, Type};
use std::collections::HashMap;

pub struct TypeChecker {
    scope_stack: Vec<HashMap<String, Type>>,
    function_signatures: HashMap<String, Vec<Type>>,
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

    pub fn new() -> Self {
        let mut scope_stack = Vec::new();
        scope_stack.push(HashMap::new()); // Globaler Scope
        Self {
            scope_stack,
            function_signatures: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, programm: &Stmt) -> Result<(), TypeError> {
        if let Stmt::Program { body } = programm {
            for stmt in body {
                self.check_statement(stmt)?;
            }
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::VarDeclaration {
                var_type,
                value: Some(value),
                identifier,
                ..
            } => {
                let expr_type = self.infer_type(value)?;
                if *var_type != expr_type && *var_type != Type::Any {
                    return Err(TypeError {
                        message: format!(
                            "Type conflict: expected {:?}, got {:?}",
                            var_type, expr_type
                        ),
                    });
                }

                let final_type = if *var_type == Type::Any {
                    expr_type.clone()
                } else {
                    var_type.clone()
                };
                self.declare_variable(identifier.clone(), final_type);
                Ok(())
            }
            Stmt::FunctionDeclaration { .. } => self.check_fn_declaration(stmt),
            Stmt::IfStatement { .. } => self.check_if_stmt(stmt),
            Stmt::WhileStatement { .. } => self.check_while_declaration(stmt),
            Stmt::Expression(expr) => {
                self.infer_type(expr)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_fn_declaration(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::FunctionDeclaration {
            name,
            parameters,
            param_types,
            body,
        } = stmt
        {
            self.function_signatures
                .insert(name.clone(), param_types.clone());

            self.enter_scope();

            for (param, param_type) in parameters.iter().zip(param_types.iter()) {
                self.declare_variable(param.clone(), param_type.clone());
            }

            for stmt in body {
                self.check_statement(stmt)?;
            }

            self.exit_scope();
            Ok(())
        } else {
            Err(TypeError {
                message: "Expected function declaration".to_string(),
            })
        }
    }

    fn check_while_declaration(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        if let Stmt::WhileStatement { condition, body } = stmt {
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

    fn infer_type(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::NumericLiteral(_) => Ok(Type::Number),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::BooleanLiteral(_) => Ok(Type::Boolean),
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

            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if left_type == Type::Number && right_type == Type::Number {
                            Ok(Type::Number)
                        } else {
                            Err(TypeError {
                                message: "Arithmetic operations require number types".to_string(),
                            })
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => Ok(Type::Boolean),
                    _ => Ok(Type::Any),
                }
            }

            _ => Ok(Type::Any),
        }
    }

    fn check_assignment(&mut self, expr: &Expr) -> Result<(), TypeError> {
        if let Expr::Assignment { assignee, value } = expr {
            if let Expr::Member {
                object,
                computed,
                property,
            } = assignee.as_ref()
            {
                let object_type = self.infer_type(object)?;
                match object_type {
                    Type::Array(element_type) => {
                        if *computed {
                            let index_type = self.infer_type(property)?;
                            if index_type != Type::Number {
                                return Err(TypeError {
                                    message: "Array index must be a number".to_string(),
                                });
                            }
                            let value_type = self.infer_type(value)?;
                            if value_type != *element_type && *element_type != Type::Any {
                                return Err(TypeError {
                                    message: format!(
                                        "Cannot assign value of type {:?} to array of type {:?}",
                                        value_type, element_type
                                    ),
                                });
                            }
                            return Ok(());
                        }
                    }
                    Type::Object(properties) => {
                        if let Expr::Identifier(prop_name) = property.as_ref() {
                            if let Some(expected_type) = properties.get(prop_name) {
                                let value_type = self.infer_type(value)?;

                                if *expected_type != value_type && *expected_type != Type::Any {
                                    return Err(TypeError {
                                        message: format!(
                                            "Property '{}' expected type {:?}, but got {:?}",
                                            prop_name, expected_type, value_type
                                        ),
                                    });
                                }
                                return Ok(());
                            } else {
                                return Err(TypeError {
                                    message: format!(
                                        "Property '{}' does not exist on this object",
                                        prop_name
                                    ),
                                });
                            }
                        } else {
                            return Err(TypeError {
                                message: format!(
                                    "Cannot access property on non-object type "
                                ),
                            });
                        }
                    }
                    _ => {
                        return Err(TypeError {
                            message: format!(
                                "Cannot assign to non-object type: {:?}",
                                &object_type
                            ),
                        });
                    }
                }
            } else {
                let target_type = self.infer_type(assignee)?;
                let value_type = self.infer_type(value)?;

                if target_type != value_type && target_type != Type::Any {
                    return Err(TypeError {
                        message: format!(
                            "Ungültige Zuweisung Erwartet {:?}, aber erhielt {:?}",
                            target_type, value_type
                        ),
                    });
                }
            }
        } else {
            panic!("assignment expected");
        }

        Ok(())
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
                            first_type,
                            value_type
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
                            Ok(Type::Any)
                        }
                        None => Err(TypeError {
                            message: format!("Undefinierte Funktion: {}", fn_name),
                        }),
                    }
                }
                Expr::Member {
                    object, property, ..
                } => {
                    let obj_type = self.infer_type(object)?;

                    if let Expr::Identifier(prop_name) = property.as_ref() {
                        match obj_type {
                            Type::Object(_) => Ok(Type::Any),
                            _ => Err(TypeError {
                                message: format!(
                                    "Cannot call method '{}' on type {:?}",
                                    prop_name, obj_type
                                ),
                            }),
                        }
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
            let object_type = self.infer_type(object)?;

            match object_type {
                Type::Array(element_type) => {
                    if *computed {
                        if let Ok(Type::Number) = self.infer_type(property) {
                            return Ok(*element_type);
                        }
                        return Err(TypeError {
                            message: "Array index must be a number".to_string(),
                        });
                    }
                    Err(TypeError {
                        message: "Array access must use computed property syntax []".to_string(),
                    })
                },
                Type::Object(prop_types) => {
                    if let Expr::Identifier(prop_name) = property.as_ref() {
                        if let Some(prop_type) = prop_types.get(prop_name) {
                            return Ok(prop_type.clone());
                        } else {
                            return Err(TypeError {
                                message: format!(
                                    "Property '{}' does not exist on this object",
                                    prop_name
                                ),
                            });
                        }
                    } else {
                        return Err(TypeError {
                            message: "Property must be an identifier".to_string(),
                        });
                    }
                }
                _ => Err(TypeError {
                    message: format!(
                        "Cannot access property on non-object type: {:?}",
                        object_type
                    ),
                }),
            }
        } else {
            panic!("Expected Member expression");
        }
    }
}
