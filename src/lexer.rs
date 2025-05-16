use std::str::Chars;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    _Number,
    _String,
    Identifier,

    Let,
    Const,

    Fn,

    If,
    Else,

    While,

    GreaterThen,
    LessThen,
    GreaterThenEquals,
    LessThenEquals,
    EqualsEquals,
    NotEquals,

    BinaryOperator,
    Equals,

    Comma,
    Dot,
    Colon,
    Semicolon,

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,

    EoF,

    Return,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Self { token_type, value }
    }
    
    // This creates tokens for single characters without allocating a new String
    pub fn new_single_char(token_type: TokenType, c: char) -> Self {
        let mut s = String::with_capacity(1);
        s.push(c);
        Self { token_type, value: s }
    }
    
    // This creates tokens for keywords and other predefined values
    pub fn new_static(token_type: TokenType, value: &'static str) -> Self {
        Self { token_type, value: value.to_string() }
    }
}

// Using a struct with iterator instead of nested match statements
pub struct Lexer<'a> {
    chars: std::iter::Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: Vec::new(),
        }
    }
    
    pub fn tokenize(mut self) -> Vec<Token> {
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::OpenParen, "("));
                }
                ')' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::CloseParen, ")"));
                }
                '{' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::OpenBrace, "{"));
                }
                '}' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::CloseBrace, "}"));
                }
                '[' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::OpenBracket, "["));
                }
                ']' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::CloseBracket, "]"));
                }
                '+' | '-' | '/' | '*' | '%' => {
                    let op = self.chars.next().unwrap();
                    self.tokens.push(Token::new_single_char(TokenType::BinaryOperator, op));
                }
                '=' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens.push(Token::new_static(TokenType::EqualsEquals, "=="));
                    } else {
                        self.tokens.push(Token::new_static(TokenType::Equals, "="));
                    }
                }
                '!' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens.push(Token::new_static(TokenType::NotEquals, "!="));
                    } else {
                        panic!("Unbekanntes Zeichen: '!' ")
                    }
                }
                '<' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens.push(Token::new_static(TokenType::LessThenEquals, "<="));
                    } else {
                        self.tokens.push(Token::new_static(TokenType::LessThen, "<"));
                    }
                }
                '>' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens.push(Token::new_static(TokenType::GreaterThenEquals, ">="));
                    } else {
                        self.tokens.push(Token::new_static(TokenType::GreaterThen, ">"));
                    }
                }
                '"' => self.tokenize_string(),
                ';' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::Semicolon, ";"));
                }
                '.' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::Dot, "."));
                }
                ',' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::Comma, ","));
                }
                ':' => {
                    self.chars.next();
                    self.tokens.push(Token::new_static(TokenType::Colon, ":"));
                }
                _ => {
                    if c.is_ascii_digit() {
                        self.tokenize_number();
                    } else if c.is_alphabetic() {
                        self.tokenize_identifier();
                    } else if c.is_whitespace() {
                        self.chars.next();
                    } else {
                        self.chars.next(); // Skip unknown characters
                    }
                }
            }
        }
        
        self.tokens.push(Token::new_static(TokenType::EoF, "EndOfFile"));
        self.tokens
    }
    
    fn tokenize_string(&mut self) {
        self.chars.next(); // Skip the opening quote
        let mut string_literal = String::new();
        
        while let Some(&ch) = self.chars.peek() {
            if ch == '"' {
                break;
            }
            string_literal.push(ch);
            self.chars.next();
        }
        
        if self.chars.next() != Some('"') {
            panic!("Unbeendeter String-Literal");
        }
        
        self.tokens.push(Token::new(TokenType::_String, string_literal));
    }
    
    fn tokenize_number(&mut self) {
        let mut number = String::new();
        let mut has_dot = false;
        
        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.chars.next();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                number.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        
        if number.ends_with('.') {
            panic!("Ungültiger Float-Wert: {}", number);
        }
        
        self.tokens.push(Token::new(TokenType::_Number, number));
    }
    
    fn tokenize_identifier(&mut self) {
        let mut identifier = String::new();
        
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphabetic() || ch.is_ascii_digit() {
                identifier.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        
        // Use a match for keywords instead of repeated comparisons
        let token_type = match identifier.as_str() {
            "let" => TokenType::Let,
            "return" => TokenType::Return,
            "const" => TokenType::Const,
            "while" => TokenType::While,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "fn" => TokenType::Fn,
            _ => TokenType::Identifier,
        };
        
        self.tokens.push(Token::new(token_type, identifier));
    }
}

// Keep the original function signature for compatibility
pub fn tokenize(source_code: &str) -> Vec<Token> {
    Lexer::new(source_code).tokenize()
}