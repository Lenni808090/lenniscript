use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
use std::collections::HashMap;

// Define native function type as an enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NativeFunctionType {
    Print,
    // Add more native functions here
}

// A struct to hold function implementations
pub struct NativeFunctionRegistry {
    functions: HashMap<String, NativeFunctionType>,
}

impl NativeFunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        
        // Register default functions
        registry.register("print", NativeFunctionType::Print);
        
        registry
    }
    
    pub fn register(&mut self, name: &str, func_type: NativeFunctionType) {
        self.functions.insert(name.to_string(), func_type);
    }
    
    pub fn get(&self, name: &str) -> Option<&NativeFunctionType> {
        self.functions.get(name)
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

// Implementation of native functions
pub fn execute_native_function<'ctx>(
    func_type: NativeFunctionType,
    args: &[BasicValueEnum<'ctx>],
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> BasicValueEnum<'ctx> {
    match func_type {
        NativeFunctionType::Print => execute_print(args, builder, context, module),
    }
}

// Implementation of print function
fn execute_print<'ctx>(
    args: &[BasicValueEnum<'ctx>],
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> BasicValueEnum<'ctx> {
    // Get or create printf function
    let printf_fn = if let Some(func) = module.get_function("printf") {
        func
    } else {
        let i8_type = context.i8_type();
        // Use the correct AddressSpace value for your version of Inkwell
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
        let printf_type = context.i32_type().fn_type(&[i8_ptr_type.into()], true);
        module.add_function("printf", printf_type, None)
    };
    
    // For numeric values, convert to string for printing
    let i64_type = context.i64_type();
    
    // For simplicity, assume the first argument is an integer
    if !args.is_empty() {
        // Create a format string for display
        let format_string = builder.build_global_string_ptr("%lld\n", "format_string").unwrap();
        
        let value = match args[0] {
            BasicValueEnum::IntValue(val) => val,
            _ => i64_type.const_int(0, false), // Default if not an integer
        };
        
        let printf_args = &[format_string.as_pointer_value().into(), value.into()];
        builder.build_call(printf_fn, printf_args, "printf_call").unwrap();
        
        // Return the original value
        args[0]
    } else {
        // If no arguments, just print a newline
        let format_string = builder.build_global_string_ptr("\n", "newline").unwrap();
        let printf_args = &[format_string.as_pointer_value().into()];
        builder.build_call(printf_fn, printf_args, "printf_call").unwrap();
        
        // Return 0 as default
        i64_type.const_int(0, false).into()
    }
}