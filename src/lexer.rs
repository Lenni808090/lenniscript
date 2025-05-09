use std::string::String;
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
}

fn isalpha(c: char) -> bool {
    c.is_alphabetic()
}

fn isnum(n: char) -> bool {
    n.is_numeric()
}

fn isskippable(w: char) -> bool {
    w.is_whitespace()
}

pub fn tokenize(source_code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source_code.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '(' => {
                chars.next();
                tokens.push(Token::new(TokenType::OpenParen, "(".to_string()));
            }
            ')' => {
                chars.next();
                tokens.push(Token::new(TokenType::CloseParen, ")".to_string()));
            }
            '{' => {
                chars.next();
                tokens.push(Token::new(TokenType::OpenBrace, "{".to_string()));
            }
            '}' => {
                chars.next();
                tokens.push(Token::new(TokenType::CloseBrace, "}".to_string()));
            }
            '[' => {
                chars.next();
                tokens.push(Token::new(TokenType::OpenBracket, "[".to_string()));
            }
            ']' => {
                chars.next();
                tokens.push(Token::new(TokenType::CloseBracket, "]".to_string()));
            }
            '+' | '-' | '/' | '*' | '%' => {
                let op = chars.next().unwrap();
                tokens.push(Token::new(TokenType::BinaryOperator, op.to_string()));
            }
            '=' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::EqualsEquals, "==".to_string()));
                } else {
                    tokens.push(Token::new(TokenType::Equals, "=".to_string()));
                }
            }
            '!' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::NotEquals, "!=".to_string()))
                } else {
                    panic!("Unbekantes Zeichen: '!' ")
                }
            }
            '<' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::LessThenEquals, "<=".to_string()));
                } else {
                    tokens.push(Token::new(TokenType::LessThen, "<".to_string()));
                }
            }
            '>' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(TokenType::GreaterThenEquals, ">=".to_string()));
                } else {
                    tokens.push(Token::new(TokenType::GreaterThen, ">".to_string()));
                }
            }
            '"' => {
                chars.next();
                let mut string_literal = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        break;
                    }
                    string_literal.push(ch);
                    chars.next();
                }
                if chars.next() != Some('"') {
                    panic!("Unbeendeter String-Literal");
                }
                tokens.push(Token::new(TokenType::_String, string_literal));
            }
            ';' => {
                chars.next();
                tokens.push(Token::new(TokenType::Semicolon, ";".to_string()));
            }
            '.' => {
                chars.next();
                tokens.push(Token::new(TokenType::Dot, ".".to_string()));
            }
            ',' => {
                chars.next();
                tokens.push(Token::new(TokenType::Comma, ",".to_string()));
            }
            ':' => {
                chars.next();
                tokens.push(Token::new(TokenType::Colon, ":".to_string()));
            }
            _ => {
                if isnum(c) {
                    let mut number = String::new();
                    let mut has_dot: bool = false;
                    while let Some(&ch) = chars.peek() {
                        if isnum(ch) {
                            number.push(ch);
                            chars.next();
                        } else if ch == '.' && !has_dot {
                            has_dot = true;
                            number.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if number.ends_with('.') {
                        panic!("Ungültiger Float-Wert: {}", number);
                    }
                    tokens.push(Token::new(TokenType::_Number, number));
                } else if isalpha(c) {
                    let mut identifier = String::new();
                    while let Some(&ch) = chars.peek() {
                        if isalpha(ch) || isnum(ch) {
                            identifier.push(ch);
                            chars.next();
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
                        _ => TokenType::Identifier,
                    };

                    tokens.push(Token::new(token_type, identifier));
                } else if isskippable(c) {
                    chars.next();
                }
            }
        }
    }
    tokens.push(Token::new(TokenType::EoF, "EndOfFile".to_string()));
    tokens
}
