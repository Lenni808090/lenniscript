#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl RuntimeValue {
    pub fn to_number(&self) -> f64 {
        match self {
            RuntimeValue::Number(n) => *n,
            RuntimeValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
            RuntimeValue::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            RuntimeValue::Null => 0.0,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            RuntimeValue::Number(n) => *n != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Null => false,
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Number(n) => write!(f, "{}", n),
            RuntimeValue::String(s) => write!(f, "{}", s),
            RuntimeValue::Boolean(b) => write!(f, "{}", b),
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}
