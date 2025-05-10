use crate::ast::{Expr, Stmt};
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use std::fs;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Stmt) {
        // Create main function
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        // Compile program body
        if let Stmt::Program { body } = program {
            for stmt in body {
                self.compile_statement(stmt);
            }
        }

        // Return 0 if no explicit return
        let return_value = i64_type.const_int(0, false);
        self.builder.build_return(Some(&return_value));
    }

    fn compile_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { identifier, value, .. } => {
                let i64_type = self.context.i64_type();
                let alloca = self.builder.build_alloca(i64_type, identifier);
                
                if let Some(init) = value {
                    let init_val = self.compile_expression(init);
                    self.builder.build_store(alloca, init_val);
                }
                
                self.variables.insert(identifier.clone(), alloca);
            }
            
            Stmt::ReturnStatement { value } => {
                if let Some(val) = value {
                    let return_value = self.compile_expression(val);
                    self.builder.build_return(Some(&return_value));
                }
            }
            
            Stmt::Expression(expr) => {
                self.compile_expression(expr);
            }
            
            Stmt::WhileStatement { condition, body } => {
                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let loop_bb = self.context.append_basic_block(function, "loop");
                let body_bb = self.context.append_basic_block(function, "body");
                let after_bb = self.context.append_basic_block(function, "afterloop");

                self.builder.build_unconditional_branch(loop_bb);
                self.builder.position_at_end(loop_bb);
                
                let cond_value = self.compile_expression(condition);
                self.builder.build_conditional_branch(cond_value.into_int_value(), body_bb, after_bb);
                
                self.builder.position_at_end(body_bb);
                for stmt in body {
                    self.compile_statement(stmt);
                }
                self.builder.build_unconditional_branch(loop_bb);
                
                self.builder.position_at_end(after_bb);
            }
            
            Stmt::IfStatement { condition, then_branch, else_if_branches, else_branch } => {
                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "merge");

                let cond_value = self.compile_expression(condition);
                self.builder.build_conditional_branch(cond_value.into_int_value(), then_bb, else_bb);
                
                self.builder.position_at_end(then_bb);
                for stmt in then_branch {
                    self.compile_statement(stmt);
                }
                self.builder.build_unconditional_branch(merge_bb);
                
                self.builder.position_at_end(else_bb);
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.compile_statement(stmt);
                    }
                }
                self.builder.build_unconditional_branch(merge_bb);
                
                self.builder.position_at_end(merge_bb);
            }
            
            _ => panic!("Unimplemented statement: {:?}", stmt),
        }
    }

    fn compile_expression(&mut self, expr: &Expr) -> BasicValueEnum<'ctx> {
        match expr {
            Expr::NumericLiteral(num) => {
                let i64_type = self.context.i64_type();
                i64_type.const_int(*num as u64, false).into()
            }
            
            Expr::Identifier(name) => {
                let ptr = self.variables.get(name).expect("Unknown variable");
                self.builder.build_load(*ptr, name).into()
            }
            
            Expr::Binary { left, right, operator } => {
                let lhs = self.compile_expression(left).into_int_value();
                let rhs = self.compile_expression(right).into_int_value();
                
                match operator.as_str() {
                    "+" => self.builder.build_int_add(lhs, rhs, "addtmp").into(),
                    "-" => self.builder.build_int_sub(lhs, rhs, "subtmp").into(),
                    "*" => self.builder.build_int_mul(lhs, rhs, "multmp").into(),
                    "/" => self.builder.build_int_signed_div(lhs, rhs, "divtmp").into(),
                    "<" => self.builder.build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "cmptmp").into(),
                    ">" => self.builder.build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "cmptmp").into(),
                    "<=" => self.builder.build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "cmptmp").into(),
                    ">=" => self.builder.build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "cmptmp").into(),
                    "==" => self.builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "cmptmp").into(),
                    _ => panic!("Unknown binary operator: {}", operator),
                }
            }
            
            Expr::Assignment { assignee, value } => {
                if let Expr::Identifier(name) = &**assignee {
                    let val = self.compile_expression(value);
                    let ptr = self.variables.get(name).expect("Unknown variable");
                    self.builder.build_store(*ptr, val);
                    val
                } else {
                    panic!("Invalid assignment target");
                }
            }
            
            _ => panic!("Unimplemented expression: {:?}", expr),
        }
    }

    pub fn get_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

