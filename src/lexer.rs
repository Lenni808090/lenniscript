use std::str::Chars;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    _Number,
    _String,
    Identifier,
    TypeAnnotation,

    Let,
    Const,

    Fn,
    Arrow,

    True,
    False,

    If,
    Else,

    For,
    In,

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
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, line: u32) -> Self {
        Token {
            token_type,
            value,
            line,
        }
    }

    pub fn new_static(token_type: TokenType, value: &'static str, line: u32) -> Self {
        Self {
            token_type,
            value: value.to_string(),
            line,
        }
    }
}

pub fn tokenize(source_code: &str) -> Vec<Token> {
    Lexer::new(source_code).tokenize()
}

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
        let mut line: u32 = 1;
        while let Some(&c) = self.chars.peek() {
            match c {
                '(' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::OpenParen, "(", line));
                }
                ')' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::CloseParen, ")", line));
                }
                '{' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::OpenBrace, "{", line));
                }
                '}' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::CloseBrace, "}", line));
                }
                '[' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::OpenBracket, "[", line));
                }
                ']' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::CloseBracket, "]", line));
                }
                '/' | '*' | '%' | '-' | '+' => {
                    self.get_operatator(line);
                }
                '=' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::EqualsEquals, "==", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::Equals, "=", line));
                    }
                }
                '!' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::NotEquals, "!=", line));
                    } else {
                        panic!("Unbekanntes Zeichen: '!' ")
                    }
                }
                '<' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::LessThenEquals, "<=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::LessThen, "<", line));
                    }
                }
                '>' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens.push(Token::new_static(
                            TokenType::GreaterThenEquals,
                            ">=",
                            line,
                        ));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::GreaterThen, ">", line));
                    }
                }
                '"' => self.tokenize_string(line),
                ';' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::Semicolon, ";", line));
                }
                '.' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::Dot, ".", line));
                }
                ',' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::Comma, ",", line));
                }
                ':' => {
                    self.chars.next();
                    self.tokens
                        .push(Token::new_static(TokenType::Colon, ":", line));
                }
                _ => {
                    if c.is_ascii_digit() {
                        self.tokenize_number(line);
                    } else if c.is_alphabetic() {
                        self.tokenize_identifier(line);
                    } else if c.is_whitespace() {
                        if c == '\n' {
                            line += 1;
                        }
                        self.chars.next();
                    } else {
                        self.chars.next();
                    }
                }
            }
        }

        self.tokens
            .push(Token::new_static(TokenType::EoF, "EndOfFile", line));
        self.tokens
    }

    fn tokenize_string(&mut self, line: u32) {
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

        self.tokens
            .push(Token::new(TokenType::_String, string_literal, line));
    }

    fn tokenize_number(&mut self, line: u32) {
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

        self.tokens
            .push(Token::new(TokenType::_Number, number, line));
    }

    fn get_operatator(&mut self, line: u32) {
        if let Some(&c) = self.chars.peek() {
            match c {
                '-' => {
                    self.chars.next();
                    if let Some(&'>') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::Arrow, "->", line));
                    } else if let Some(&'=') = self.chars.peek() {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "-=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "-", line))
                    }
                }
                '+' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "+=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "+", line));
                    }
                }
                '/' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "/=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "/", line));
                    }
                }
                '%' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "%=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "%", line));
                    }
                }
                '*' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "*=", line));
                    } else {
                        self.tokens
                            .push(Token::new_static(TokenType::BinaryOperator, "*", line));
                    }
                }
                _ => panic!("Unknown typa beat"),
            }
        }
    }

    fn tokenize_identifier(&mut self, line: u32) {
        let mut identifier = String::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphabetic() || ch.is_ascii_digit() {
                identifier.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        let token_type = match identifier.as_str() {
            "let" => TokenType::Let,
            "return" => TokenType::Return,
            "const" => TokenType::Const,
            "while" => TokenType::While,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "fn" => TokenType::Fn,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "string" => TokenType::TypeAnnotation,
            "num" => TokenType::TypeAnnotation,
            "array" => TokenType::TypeAnnotation,
            "bool" => TokenType::TypeAnnotation,
            "for" => TokenType::For,
            "in" => TokenType::In,
            _ => TokenType::Identifier,
        };

        self.tokens.push(Token::new(token_type, identifier, line));
    }
}
