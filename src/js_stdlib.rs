use crate::ast::Type;
use std::collections::HashMap;

pub struct JsStdLib {
    pub objects: HashMap<String, HashMap<String, Type>>,
}

impl JsStdLib {
    pub fn new() -> Self {
        let mut objects = HashMap::new();

        // Array methods
        let mut array_methods = HashMap::new();
        array_methods.insert("push".to_string(), Type::Number);
        array_methods.insert("pop".to_string(), Type::Any);
        array_methods.insert("shift".to_string(), Type::Any);
        array_methods.insert("unshift".to_string(), Type::Number);
        array_methods.insert("slice".to_string(), Type::Array(Box::new(Type::Any)));
        array_methods.insert("splice".to_string(), Type::Array(Box::new(Type::Any)));
        array_methods.insert("join".to_string(), Type::String);
        array_methods.insert("length".to_string(), Type::Number);
        array_methods.insert("forEach".to_string(), Type::Void);
        array_methods.insert("map".to_string(), Type::Array(Box::new(Type::Any)));
        array_methods.insert("filter".to_string(), Type::Array(Box::new(Type::Any)));
        array_methods.insert("reduce".to_string(), Type::Any);
        array_methods.insert("some".to_string(), Type::Boolean);
        array_methods.insert("every".to_string(), Type::Boolean);
        array_methods.insert("indexOf".to_string(), Type::Number);
        array_methods.insert("includes".to_string(), Type::Boolean);
        objects.insert("Array".to_string(), array_methods);

        // String methods
        let mut string_methods = HashMap::new();
        string_methods.insert("charAt".to_string(), Type::String);
        string_methods.insert("concat".to_string(), Type::String);
        string_methods.insert("includes".to_string(), Type::Boolean);
        string_methods.insert("indexOf".to_string(), Type::Number);
        string_methods.insert("replace".to_string(), Type::String);
        string_methods.insert("slice".to_string(), Type::String);
        string_methods.insert("split".to_string(), Type::Array(Box::new(Type::String)));
        string_methods.insert("substring".to_string(), Type::String);
        string_methods.insert("toLowerCase".to_string(), Type::String);
        string_methods.insert("toUpperCase".to_string(), Type::String);
        string_methods.insert("trim".to_string(), Type::String);
        string_methods.insert("length".to_string(), Type::Number);
        string_methods.insert("startsWith".to_string(), Type::Boolean);
        string_methods.insert("endsWith".to_string(), Type::Boolean);
        string_methods.insert("match".to_string(), Type::Array(Box::new(Type::String)));
        string_methods.insert("padStart".to_string(), Type::String);
        string_methods.insert("padEnd".to_string(), Type::String);
        objects.insert("String".to_string(), string_methods);

        // Math object
        let mut math_methods = HashMap::new();
        math_methods.insert("abs".to_string(), Type::Number);
        math_methods.insert("ceil".to_string(), Type::Number);
        math_methods.insert("floor".to_string(), Type::Number);
        math_methods.insert("max".to_string(), Type::Number);
        math_methods.insert("min".to_string(), Type::Number);
        math_methods.insert("pow".to_string(), Type::Number);
        math_methods.insert("random".to_string(), Type::Number);
        math_methods.insert("round".to_string(), Type::Number);
        math_methods.insert("sqrt".to_string(), Type::Number);
        math_methods.insert("PI".to_string(), Type::Number);
        math_methods.insert("E".to_string(), Type::Number);
        objects.insert("Math".to_string(), math_methods);

        // Number methods
        let mut number_methods = HashMap::new();
        number_methods.insert("toFixed".to_string(), Type::String);
        number_methods.insert("toPrecision".to_string(), Type::String);
        number_methods.insert("toString".to_string(), Type::String);
        number_methods.insert("valueOf".to_string(), Type::Number);
        number_methods.insert("MAX_VALUE".to_string(), Type::Number);
        number_methods.insert("MIN_VALUE".to_string(), Type::Number);
        objects.insert("Number".to_string(), number_methods);

        // Object methods
        let mut object_methods = HashMap::new();
        object_methods.insert("keys".to_string(), Type::Array(Box::new(Type::String)));
        object_methods.insert("values".to_string(), Type::Array(Box::new(Type::Any)));
        object_methods.insert(
            "entries".to_string(),
            Type::Array(Box::new(Type::Array(Box::new(Type::Any)))),
        );
        object_methods.insert("hasOwnProperty".to_string(), Type::Boolean);
        object_methods.insert("toString".to_string(), Type::String);
        objects.insert("Object".to_string(), object_methods);

        // Console methods
        let mut console_methods = HashMap::new();
        console_methods.insert("log".to_string(), Type::Void);
        console_methods.insert("error".to_string(), Type::Void);
        console_methods.insert("warn".to_string(), Type::Void);
        console_methods.insert("info".to_string(), Type::Void);
        console_methods.insert("debug".to_string(), Type::Void);
        console_methods.insert("table".to_string(), Type::Void);
        objects.insert("console".to_string(), console_methods);

        // JSON methods
        let mut json_methods = HashMap::new();
        json_methods.insert("parse".to_string(), Type::Any);
        json_methods.insert("stringify".to_string(), Type::String);
        objects.insert("JSON".to_string(), json_methods);

        Self { objects }
    }

    pub fn get_method_type(&self, object_name: &str, method_name: &str) -> Option<Type> {
        if let Some(methods) = self.objects.get(object_name) {
            if let Some(method_type) = methods.get(method_name) {
                return Some(method_type.clone());
            }
        }
        None
    }

    pub fn get_primitive_method_type(
        &self,
        primitive_type: &Type,
        method_name: &str,
    ) -> Option<Type> {
        match primitive_type {
            Type::String => self.get_method_type("String", method_name),
            Type::Number => self.get_method_type("Number", method_name),
            Type::Array(_) => self.get_method_type("Array", method_name),
            Type::Object(_) => self.get_method_type("Object", method_name),
            _ => None,
        }
    }
}

impl Default for JsStdLib {
    fn default() -> Self {
        Self::new()
    }
}
