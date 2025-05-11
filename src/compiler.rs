use crate::ast::{Expr, Stmt};
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;
 // Changed from NativeFunctionImpl
use crate::native_function_impl::{NativeFunctionRegistry, execute_native_function, NativeFunctionType}; // Updated path
pub struct Compiler<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    native_functions: NativeFunctionRegistry,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Compiler {
            context,
            module: context.create_module("main"),
            builder: context.create_builder(),
            native_functions: NativeFunctionRegistry::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Stmt) {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        // Compile the program body and capture the result of the last expression
        let mut last_val: Option<BasicValueEnum<'ctx>> = None;

        if let Stmt::Program { body } = program {
            for stmt in body {
                last_val = Some(self.compile_statement(stmt));
            }
        }

        let return_val = last_val.unwrap_or_else(|| i64_type.const_int(0, false).into());
        self.builder.build_return(Some(&return_val)).unwrap();
    }

    fn compile_statement(&mut self, stmt: &Stmt) -> BasicValueEnum<'ctx> {
        match stmt {
            Stmt::Expression(expr) => self.compile_expression(expr),
            _ => panic!("Only expression statements are supported"),
        }
    }

    fn compile_expression(&mut self, expr: &Expr) -> BasicValueEnum<'ctx> {
        match expr {
            Expr::NumericLiteral(num) => {
                self.context.i64_type().const_int(*num as u64, false).into()
            },
            
            Expr::Binary { left, right, operator } => {
                let lhs = self.compile_expression(left).into_int_value();
                let rhs = self.compile_expression(right).into_int_value();

                match operator.as_str() {
                    "+" => self.builder.build_int_add(lhs, rhs, "add").unwrap().into(),
                    "-" => self.builder.build_int_sub(lhs, rhs, "sub").unwrap().into(),
                    "*" => self.builder.build_int_mul(lhs, rhs, "mul").unwrap().into(),
                    "/" => self.builder.build_int_signed_div(lhs, rhs, "div").unwrap().into(),
                    _ => panic!("Unsupported binary operator: {}", operator),
                }
            },
            
            Expr::Call { caller, args } => {
                if let Expr::Identifier(name) = &**caller {
                    // Check if it's a native function
                    if self.native_functions.contains(name) {
                        let mut compiled_args = Vec::new();
                        for arg in args {
                            compiled_args.push(self.compile_expression(arg));
                        }
                        
                        // Get the native function type and execute it
                        if let Some(func_type) = self.native_functions.get(name) {
                            return execute_native_function(
                                *func_type,
                                &compiled_args,
                                &self.builder,
                                self.context,
                                &self.module
                            );
                        }
                    }
                    
                    // If not a native function, it would be a user-defined function
                    panic!("User-defined function calls not yet implemented");
                } else {
                    panic!("Caller must be an identifier");
                }
            },
            
            Expr::Identifier(_) => {
                // This would handle variable lookup
                panic!("Variable lookup not yet implemented");
            },
            
            _ => panic!("Unsupported expression type: {:?}", expr),
        }
    }
    
    pub fn get_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}