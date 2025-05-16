use crate::ast;
use crate::ast::Stmt::WhileStatement;
use crate::ast::{ElseIfBranch, Expr, Property, Stmt};
use crate::lexer;
use crate::lexer::{tokenize, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn not_eof(&self) -> bool {
        if let Some(token) = self.tokens.first() {
            token.token_type != TokenType::EoF
        } else {
            false
        }
    }

    fn at(&self) -> &Token {
        self.tokens.first().expect("Keine Tokens verfügbar")
    }

    fn eat(&mut self) -> Token {
        self.tokens.remove(0)
    }

    fn expect(&mut self, expected: TokenType, err: &str) -> Token {
        let token = self.eat();
        if token.token_type != expected {
            eprintln!(
                "Parser-Fehler: {}. Gefunden: {:?}, erwartet: {:?}",
                err, token, expected
            );
            panic!("Parsing abgebrochen {:?}", token);
        }
        token
    }

    pub fn produceAst(&mut self, sourceCode: &str) -> Stmt {
        self.tokens = tokenize(sourceCode);
        let mut body = Vec::new();
        while self.not_eof() {
            body.push(self.parse_stmt());
        }

        Stmt::Program { body }
    }

    fn parse_stmt(&mut self) -> Stmt {
        let tk = self.at().token_type;

        match tk {
            TokenType::Const | TokenType::Let => self.parse_var_declaration(),

            TokenType::Return => self.parse_return_statement(),

            TokenType::If => self.parse_if_statement(),

            TokenType::While => self.parse_while_statement(),

            _ => {
                let expr = self.parse_expr();
                self.expect(TokenType::Semicolon, "Erwarte Semikolon nach Ausdruck");
                Stmt::Expression(expr)
            }
        }
    }

    fn parse_return_statement(&mut self) -> Stmt {
        self.eat();
        let expr = self.parse_expr();
        self.expect(TokenType::Semicolon, "Erwarte Semikolon nach Ausdruck");
        Stmt::ReturnStatement { value: Some(expr) }
    }

    fn parse_if_statement(&mut self) -> Stmt {
        self.eat(); // Eat the `if`
        self.expect(TokenType::OpenParen, "Expect Open Paren after if");
        let condition = self.parse_expr();
        self.expect(
            TokenType::CloseParen,
            "Expected Closing Paren after condition",
        );
        self.expect(TokenType::OpenBrace, "Expected open brace after condition");

        let mut body: Vec<Stmt> = Vec::new();
        while self.at().token_type != TokenType::CloseBrace {
            let stmt = self.parse_stmt();
            body.push(stmt);
        }
        self.expect(TokenType::CloseBrace, "Expected Closing Brace after body");

        let mut else_if_branches: Vec<ElseIfBranch> = Vec::new();
        let mut else_branch: Option<Vec<Stmt>> = None;

        while self.at().token_type == TokenType::Else {
            self.eat(); // eat the `else`

            if self.at().token_type == TokenType::If {
                self.eat(); // eat the `if`
                self.expect(TokenType::OpenParen, "Expect Open Paren after else if");
                let else_if_condition = self.parse_expr();
                self.expect(
                    TokenType::CloseParen,
                    "Expected Closing Paren after condition",
                );
                self.expect(TokenType::OpenBrace, "Expected open brace after condition");

                let mut else_if_body: Vec<Stmt> = Vec::new();
                while self.at().token_type != TokenType::CloseBrace {
                    let stmt = self.parse_stmt();
                    else_if_body.push(stmt);
                }
                self.expect(
                    TokenType::CloseBrace,
                    "Expected Closing Brace after else-if body",
                );

                let else_if = ElseIfBranch {
                    condition: else_if_condition,
                    body: else_if_body,
                };
                else_if_branches.push(else_if);
            } else {
                self.expect(TokenType::OpenBrace, "Expected open brace after else");
                let mut else_body: Vec<Stmt> = Vec::new();
                while self.at().token_type != TokenType::CloseBrace {
                    let stmt = self.parse_stmt();
                    else_body.push(stmt);
                }
                self.expect(
                    TokenType::CloseBrace,
                    "Expected Closing Brace after else body",
                );
                else_branch = Some(else_body);
                break; // no more else or else-if allowed after `else`
            }
        }

        let else_if_branches = if else_if_branches.is_empty() {
            None
        } else {
            Some(else_if_branches)
        };

        Stmt::IfStatement {
            condition,
            then_branch: body,
            else_if_branches,
            else_branch,
        }
    }

    fn parse_while_statement(&mut self) -> Stmt {
        self.eat(); // Konsumiert 'while'
        self.expect(TokenType::OpenParen, "Open Paren expected after while");
        let condition = self.parse_expr();
        self.expect(
            TokenType::CloseParen,
            "Closing Paren expected after condition",
        );
        self.expect(
            TokenType::OpenBrace,
            "Expected Opening Brace after condition",
        );

        let mut body: Vec<Stmt> = Vec::new();
        while self.at().token_type != TokenType::CloseBrace {
            let stmt: Stmt = self.parse_stmt();
            body.push(stmt);
        }

        self.expect(
            TokenType::CloseBrace,
            "Expected Closing Brace after while body",
        );

        Stmt::WhileStatement { condition, body }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_assignment_expr()
    }

    fn parse_var_declaration(&mut self) -> Stmt {
        self.eat(); // Konsumiere das 'let' oder 'const' Token
        let constant = self.at().token_type == TokenType::Const;
        let identifier = self
            .expect(
                TokenType::Identifier,
                "Erwartete Bezeichner nach let/const Schlüsselwort",
            )
            .value;

        if self.at().token_type == TokenType::Semicolon {
            self.eat();

            if constant {
                panic!("Konstanten müssen initialisiert werden");
            }

            return Stmt::VarDeclaration {
                constant,
                identifier,
                value: None,
            };
        }

        self.expect(TokenType::Equals, "Erwartete '=' nach Bezeichner");
        let value = self.parse_expr();
        self.expect(
            TokenType::Semicolon,
            "Variablendeklaration muss mit Semikolon enden",
        );

        Stmt::VarDeclaration {
            constant,
            identifier,
            value: Some(value),
        }
    }

    fn parse_assignment_expr(&mut self) -> Expr {
        let mut left = self.parse_object_expr();

        if (self.at().token_type == TokenType::Equals) {
            self.eat();
            let value = self.parse_assignment_expr();
            left = Expr::Assignment {
                assignee: Box::new(left),
                value: Box::new(value),
            }
        }

        left
    }

    fn parse_object_expr(&mut self) -> Expr {
        if self.at().token_type != TokenType::OpenBrace {
            return self.parse_comparison_expr();
        }

        self.eat();
        let mut properties: Vec<Property> = Vec::new();

        while (self.not_eof() && self.at().token_type != TokenType::CloseBrace) {
            let key = self
                .expect(TokenType::Identifier, "Object literal key expected")
                .value;

            if self.at().token_type == TokenType::Comma {
                self.eat();
                properties.push(Property { key, value: None });
                continue;
            } else if self.at().token_type == TokenType::CloseBrace {
                properties.push(Property { key, value: None });
                continue;
            }

            self.expect(
                TokenType::Colon,
                "Missing colon following identifier in ObjectExpr",
            );
            let value = self.parse_expr();

            properties.push(Property {
                key,
                value: Some(value),
            });
            if self.at().token_type != TokenType::CloseBrace {
                self.expect(
                    TokenType::Comma,
                    "Expected comma or closing bracket following property",
                );
            }
        }

        self.expect(
            TokenType::CloseBrace,
            "Object literal missing closing brace.",
        );
        Expr::ObjectLiteral(properties)
    }

    fn parse_comparison_expr(&mut self) -> Expr {
        let mut left = self.parse_additive_expr();

        while matches!(
            self.at().token_type,
            TokenType::EqualsEquals
                | TokenType::GreaterThen
                | TokenType::GreaterThenEquals
                | TokenType::LessThen
                | TokenType::LessThenEquals
                | TokenType::NotEquals
        ) {
            let operator = self.eat().value;
            let right = self.parse_additive_expr();
            left = Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            };
        }

        left
    }
    fn parse_additive_expr(&mut self) -> Expr {
        let mut left = self.parse_multiplicative_expr();

        while ["+", "-"].contains(&self.at().value.as_str()) {
            let operator = self.eat().value;
            let right = self.parse_multiplicative_expr();
            left = Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> Expr {
        let mut left = self.parse_primary_expr();

        while ["*", "/", "%"].contains(&self.at().value.as_str()) {
            let operator = self.eat().value;
            let right = self.parse_primary_expr();
            left = Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        left
    }

    fn parse_primary_expr(&mut self) -> Expr {
        let tk = self.at().token_type;

        match tk {
            TokenType::_Number => {
                let token = self.eat();
                let value = token.value.parse::<f64>().unwrap();
                Expr::NumericLiteral(value)
            }
            TokenType::Identifier => {
                let token = self.eat();
                let name = token.value;
                Expr::Identifier(name)
            }
            TokenType::OpenParen => {
                self.eat();
                let expr = self.parse_expr();
                self.expect(TokenType::CloseParen, "Erwarte schließende Klammer");
                expr
            }
            TokenType::_String => {
                let token = self.eat();
                let value = token.value;
                Expr::StringLiteral(value)
            }
            _ => {
                panic!(
                    "Unerwarteter Token-Typ beim Parsen eines primären Ausdrucks: {:?}",
                    tk
                )
            }
        }
    }
}
