use crate::ast;
use crate::ast::Expr::{Binary, CompoundAssignment};
use crate::ast::Stmt::WhileStatement;
use crate::ast::Type::{Boolean, Number};
use crate::ast::{caseBranch, ElseIfBranch, Expr, Property, Stmt, Type};
use crate::lexer;
use crate::lexer::TokenType::Not;
use crate::lexer::{tokenize, Token, TokenType};
use std::str::FromStr;

pub struct Parser {
    tokens: Vec<Token>,
}
/*
- Assignment
- Object
- Logic
- Comparison
- Additive
- Multiplicative
- Call/Member
- Unary
- Primary
*/

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
    fn get_type(&mut self) -> Type {
        let type_token_string = self
            .expect(
                TokenType::TypeAnnotation,
                "Type anotation after :/-> expected",
            )
            .value;
        let var_type = match type_token_string.as_str() {
            "array" => {
                self.expect(TokenType::LessThen, "Expected less then after array ");
                let type_array_string = self
                    .expect(
                        TokenType::TypeAnnotation,
                        "You need to specify the array type",
                    )
                    .value;
                match type_array_string.as_str() {
                    "bool" => {
                        self.expect(
                            TokenType::GreaterThen,
                            "greeater then after specifing array type needed",
                        );
                        Type::Array(Box::new(Boolean))
                    }
                    "num" => {
                        self.expect(
                            TokenType::GreaterThen,
                            "greeater then after specifing array type needed",
                        );
                        Type::Array(Box::new(Number))
                    }
                    "string" => {
                        self.expect(
                            TokenType::GreaterThen,
                            "greeater then after specifing array type needed",
                        );
                        Type::Array(Box::new(Type::String))
                    }

                    _ => {
                        panic!("Error in get type array");
                    }
                }
            }

            "bool" => Type::Boolean,
            "num" => Type::Number,
            "string" => Type::String,

            _ => {
                panic!("Error in get type");
            }
        };

        if self.at().token_type == TokenType::Question {
            self.eat();
            Type::Option(Box::new(var_type))
        } else {
            var_type
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

    pub fn produceAst(&mut self, source_code: &str) -> Stmt {
        self.tokens = tokenize(source_code);
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

            TokenType::Fn | TokenType::Async => self.parse_fn_declaration(),

            TokenType::Break => self.parse_break_stmt(),

            TokenType::Continue => self.parse_continue_stmt(),

            TokenType::Try => self.parse_try_catch_stmt(),

            TokenType::For => self.parse_for_statement(),

            TokenType::Switch => self.parse_switch_statement(),

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
            self.eat();

            if self.at().token_type == TokenType::If {
                self.eat();
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
                break;
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
        self.eat();
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
        self.eat();
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
                var_type: Type::Any,
                value: None,
            };
        }
        let mut var_type = Type::Any;
        if self.at().token_type == TokenType::Colon {
            self.eat();
            var_type = self.get_type();
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
            var_type,
            value: Some(value),
        }
    }

    fn parse_fn_declaration(&mut self) -> Stmt {
        let mut is_async = false;
        if self.at().token_type == TokenType::Async {
            self.eat();
            self.expect(TokenType::Fn, "fn expected after async keyword");
            is_async = true;
        } else {
            self.eat();
        }

        let name = self
            .expect(TokenType::Identifier, "name expected after fn keyword")
            .value;

        let (args, arg_types) = self.parse_args();
        let mut params: Vec<String> = Vec::new();

        for arg in args {
            if let Expr::Identifier(symbol) = arg {
                params.push(symbol);
            } else {
                println!("{:?}", arg);
                panic!("Inside function declaration expected parameters to be of type string.");
            }
        }
        let return_type: Type;
        if self.at().token_type == TokenType::Arrow {
            self.eat();
            return_type = self.get_type();
        } else {
            return_type = Type::Void;
        }

        self.expect(
            TokenType::OpenBrace,
            "Expected function body following declaration",
        );
        let mut body: Vec<Stmt> = Vec::new();

        while self.at().token_type != TokenType::CloseBrace {
            body.push(self.parse_stmt());
        }

        self.expect(
            TokenType::CloseBrace,
            "Closing brace expected inside function declarations",
        );

        Stmt::FunctionDeclaration {
            name,
            body,
            return_type,
            parameters: params,
            param_types: arg_types,
            is_async,
        }
    }

    fn parse_break_stmt(&mut self) -> Stmt {
        self.eat();
        self.expect(TokenType::Semicolon, "Erwarte Semikolon nach Ausdruck");
        Stmt::BreakStatement
    }

    fn parse_continue_stmt(&mut self) -> Stmt {
        self.eat();
        self.expect(TokenType::Semicolon, "Erwarte Semikolon nach Ausdruck");
        Stmt::ContinueStatement
    }

    fn parse_try_catch_stmt(&mut self) -> Stmt {
        self.eat();
        self.expect(TokenType::OpenBrace, "Open Brace Expected after try");

        let mut try_branch = Vec::new();
        while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
            try_branch.push(self.parse_stmt());
        }
        self.expect(
            TokenType::CloseBrace,
            "CLosing Brace after try body Expected",
        );

        self.expect(TokenType::Catch, "Catch stmt expected after try stmt");
        self.expect(TokenType::OpenBrace, "Open Brace Expected after catch");

        let mut catch_branch = Vec::new();
        while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
            catch_branch.push(self.parse_stmt());
        }
        self.expect(
            TokenType::CloseBrace,
            "Closing Brace after catch body Expected",
        );

        let mut finally_branch: Option<Vec<Stmt>> = None;

        if self.at().token_type == TokenType::Finally {
            self.eat();
            let mut finally_body = Vec::new();
            self.expect(TokenType::OpenBrace, "Open Brace Expected after finally");

            while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
                finally_body.push(self.parse_stmt());
            }
            self.expect(
                TokenType::CloseBrace,
                "CLosing Brace after finally body Expected",
            );

            finally_branch = Some(finally_body);
        }

        Stmt::TryCatchFinally {
            try_branch,
            catch_branch,
            finally_branch,
        }
    }

    fn parse_for_statement(&mut self) -> Stmt {
        self.eat();

        self.expect(TokenType::OpenParen, "Expected '(' after 'for'");

        let mut first_number: Option<String> = None;
        let mut second_number: Option<String> = None;
        let mut iterator_name: Option<String> = None;

        let mut initializer: Option<Box<Stmt>> = None;
        let mut iterable: Option<Expr> = None;
        let mut condition: Option<Expr> = None;
        let mut update: Option<Expr> = None;

        if self.at().token_type == TokenType::_Number {
            first_number = Some(self.eat().value);
            self.expect(
                TokenType::DotDot,
                "expected dot dot after for the iteration syntax type",
            );
            second_number = Some(
                self.expect(TokenType::_Number, "expected second number after dot dot")
                    .value,
            );

            if self.at().token_type == TokenType::As {
                self.eat();
                iterator_name = Some(
                    self.expect(TokenType::Identifier, "identifier expected after as")
                        .value,
                );
            }
        } else {
            initializer = if self.at().token_type != TokenType::Semicolon {
                if self.at().token_type == TokenType::Let
                    || self.at().token_type == TokenType::Const
                {
                    let constant = self.at().token_type == TokenType::Const;
                    self.eat();

                    let identifier = self
                        .expect(
                            TokenType::Identifier,
                            "Erwartete Bezeichner nach let/const Schlüsselwort",
                        )
                        .value;

                    if self.at().token_type == TokenType::In {
                        Some(Box::new(Stmt::VarDeclaration {
                            constant,
                            identifier,
                            var_type: Type::Any,
                            value: None,
                        }))
                    } else {
                        let mut var_type = Type::Any;
                        if self.at().token_type == TokenType::Colon {
                            self.eat();
                            var_type = self.get_type();
                        }
                        self.expect(TokenType::Equals, "Erwartete '=' nach Bezeichner");
                        let value = self.parse_expr();
                        self.expect(
                            TokenType::Semicolon,
                            "Erwartete Semikolon nach Initialisierung",
                        );
                        Some(Box::new(Stmt::VarDeclaration {
                            constant,
                            identifier,
                            var_type,
                            value: Some(value),
                        }))
                    }
                } else {
                    panic!("unexpected formating");
                }
            } else {
                None
            };

            if self.at().token_type == TokenType::In {
                self.eat();
                iterable = Some(self.parse_expr());
            } else {
                condition = if self.at().token_type != TokenType::Semicolon {
                    Some(self.parse_expr())
                } else {
                    None
                };

                self.expect(TokenType::Semicolon, "Expected ';' after loop condition");

                update = if self.at().token_type != TokenType::CloseParen {
                    Some(self.parse_expr())
                } else {
                    None
                };
            }
        }

        self.expect(TokenType::CloseParen, "Expected ')' after for clauses");

        let body = if self.at().token_type == TokenType::OpenBrace {
            self.eat();
            let mut statements = Vec::new();
            while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
                statements.push(self.parse_stmt());
            }
            self.expect(TokenType::CloseBrace, "Expected '}' after for loop body");
            statements
        } else {
            vec![self.parse_stmt()]
        };

        if iterable.is_some() {
            Stmt::ForInLoopStatement {
                iterator: initializer,
                iterable,
                body,
            }
        } else if first_number.is_some() && second_number.is_some() {
            Stmt::ForLoopIterated {
                first_number,
                second_number,
                iterator_name,
                body,
            }
        } else {
            Stmt::ForLoopStatement {
                initializer,
                condition,
                update,
                body,
            }
        }
    }

    fn parse_switch_statement(&mut self) -> Stmt {
        self.eat();
        self.expect(
            TokenType::OpenParen,
            "open paren after switch stmt expected",
        );
        let condition = self.parse_expr();
        self.expect(
            TokenType::CloseParen,
            "close paren after condition expected",
        );
        self.expect(
            TokenType::OpenBrace,
            "open brace expected after the condiotion",
        );

        let mut cases = Vec::new();
        let mut default_branch = None;
        loop {
            if self.at().token_type == TokenType::Case {
                self.eat();
                let case_condition = self.parse_expr();
                self.expect(
                    TokenType::SwitchArrow,
                    "Switch arrow expected after case condition",
                );
                self.expect(
                    TokenType::OpenBrace,
                    "open brace after case condition expected",
                );
                let mut case_body = Vec::new();
                while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
                    case_body.push(self.parse_stmt());
                }
                self.expect(TokenType::CloseBrace, "Closing brace after case body");

                cases.push(caseBranch {
                    condition: case_condition,
                    body: case_body,
                })
            } else if self.at().token_type == TokenType::Default {
                self.eat();
                self.expect(
                    TokenType::SwitchArrow,
                    "Switch arrow expected after default",
                );
                self.expect(
                    TokenType::OpenBrace,
                    "open brace after switch arrow expected",
                );
                let mut default_body = Vec::new();
                while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
                    default_body.push(self.parse_stmt());
                }
                self.expect(TokenType::CloseBrace, "Closing brace after case body");

                default_branch = Some(default_body);
            } else if self.at().token_type == TokenType::CloseBrace {
                self.eat();
                break;
            } else {
                panic!("Unexpected token");
            }
        }
        if cases.is_empty() {
            panic!("At least one case branch is required");
        }

        Stmt::SwitchStatement {
            condition,
            case_branches: cases,
            default_branch: default_branch.expect("default branch needed"),
        }
    }

    fn parse_assignment_expr(&mut self) -> Expr {
        let mut left = self.parse_object_expr();

        if (self.at().token_type == TokenType::Equals) {
            match &left {
                Expr::Identifier(_) | Expr::Member { .. } => {
                    self.eat();
                    let value = self.parse_assignment_expr();
                    left = Expr::Assignment {
                        assignee: Box::new(left),
                        value: Box::new(value),
                    }
                }
                _ => panic!("Invalid left-hand side in assignment expression"),
            }
        } else if self.at().token_type == TokenType::BinaryOperator {
            let operator = self.eat().value;
            if operator == "+="
                || operator == "-="
                || operator == "*="
                || operator == "%="
                || operator == "/="
            {
                match &left {
                    Expr::Identifier(_) | Expr::Member { .. } => {
                        let value = self.parse_expr();
                        left = CompoundAssignment {
                            assignee: Box::new(left),
                            value: Box::new(value),
                            operator,
                        }
                    }

                    _ => panic!("Invalid left-hand side in assignment expression"),
                }
            }
        }

        left
    }

    fn parse_object_expr(&mut self) -> Expr {
        if self.at().token_type != TokenType::OpenBrace {
            return self.parse_logic_expr();
        }

        self.eat();
        let mut properties: Vec<Property> = Vec::new();

        if self.at().token_type == TokenType::CloseBrace {
            self.eat();
            return Expr::ObjectLiteral(properties);
        }

        loop {
            if self.at().token_type == TokenType::CloseBrace {
                break;
            }

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

            if self.at().token_type == TokenType::Comma {
                self.eat();
            } else if self.at().token_type != TokenType::CloseBrace {
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
    fn parse_logic_expr(&mut self) -> Expr {
        let mut left = self.parse_comparison_expr();

        while self.at().value == "&&" || self.at().value == "||" {
            let operator = self.eat().value;
            let right = self.parse_comparison_expr();

            left = Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        left
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

    fn parse_call_member_expr(&mut self) -> Expr {
        let member = self.parse_member_expr();

        if self.at().token_type == TokenType::OpenParen {
            return self.parse_call_expr(member);
        }

        member
    }

    fn parse_call_expr(&mut self, caller: Expr) -> Expr {
        let (args, _) = self.parse_args();
        let mut call_expr = Expr::Call {
            caller: Box::new(caller),
            args,
        };

        if self.at().token_type == TokenType::OpenParen {
            call_expr = self.parse_call_expr(call_expr);
        }

        call_expr
    }

    fn parse_args(&mut self) -> (Vec<Expr>, Vec<Type>) {
        self.expect(TokenType::OpenParen, "Expected open parenthesis");
        let result = if self.at().token_type == TokenType::CloseParen {
            (Vec::new(), Vec::new())
        } else {
            self.parse_arguments_list()
        };

        self.expect(
            TokenType::CloseParen,
            "Missing closing parenthesis inside arguments list",
        );

        result
    }

    fn parse_arguments_list(&mut self) -> (Vec<Expr>, Vec<Type>) {
        let mut args: Vec<Expr> = vec![self.parse_assignment_expr()];
        let mut args_types: Vec<Type> = vec![Type::Any];

        if self.at().token_type == TokenType::Colon {
            self.eat();
            let param_type = self.get_type();

            args_types[0] = param_type;
        }

        while self.at().token_type == TokenType::Comma {
            self.eat();
            args.push(self.parse_assignment_expr());
            if self.at().token_type == TokenType::Colon {
                self.eat();
                let param_type = self.get_type();

                args_types.push(param_type);
            } else {
                args_types.push(Type::Any);
            }
        }

        (args, args_types)
    }

    fn parse_member_expr(&mut self) -> Expr {
        let mut object = self.parse_unary_expr();

        while self.at().token_type == TokenType::Dot
            || self.at().token_type == TokenType::OpenBracket
        {
            let operator = self.eat();
            let property: Expr;
            let computed: bool;
            if operator.token_type == TokenType::Dot {
                computed = false;
                property = self.parse_primary_expr();
                if let Expr::Identifier(value) = &property {
                    object = Expr::Member {
                        object: Box::new(object),
                        property: Box::new(property),
                        computed,
                    }
                } else {
                    panic!("Cannonot use dot operator without right hand side being a identifier")
                }
            } else {
                computed = true;
                property = self.parse_expr();
                self.expect(
                    TokenType::CloseBracket,
                    "Missing closing bracket in computed value.",
                );
                object = Expr::Member {
                    object: Box::new(object),
                    property: Box::new(property),
                    computed,
                }
            }
        }

        object
    }

    fn parse_unary_expr(&mut self) -> Expr {
        if self.at().token_type == Not {
            let operator = self.eat().value;
            let value = self.parse_unary_expr();
            return Expr::Unary {
                operator,
                value: Box::new(value),
            };
        }

        self.parse_primary_expr()
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
        let mut left = self.parse_call_member_expr();

        while ["*", "/", "%"].contains(&self.at().value.as_str()) {
            let operator = self.eat().value;
            let right = self.parse_call_member_expr();
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
            TokenType::True => {
                let token = self.eat();
                Expr::BooleanLiteral(true)
            }
            TokenType::False => {
                let token = self.eat();
                Expr::BooleanLiteral(false)
            }
            TokenType::Null => {
                let token = self.eat();
                Expr::NullLiteral
            }
            TokenType::Increment => {
                self.eat();
                let identifier = self
                    .expect(
                        TokenType::Identifier,
                        "Identifier after Increment exprected",
                    )
                    .value;

                Expr::Increment {
                    identifier: Box::new(Expr::Identifier(identifier)),
                    prefix: true,
                }
            }
            TokenType::Identifier => {
                let token = self.eat(); // Consumes 'i'
                let name = token.value;
                if self.at().token_type == TokenType::Increment {
                    self.eat();
                    Expr::Increment {
                        identifier: Box::new(Expr::Identifier(name)),
                        prefix: false,
                    }
                } else {
                    Expr::Identifier(name)
                }
            }
            TokenType::OpenBracket => {
                self.eat();
                let mut elements: Vec<Expr> = Vec::new();
                if self.at().token_type == TokenType::CloseBracket {
                    self.eat();
                    return Expr::ArrayLiteral(elements);
                }

                elements.push(self.parse_expr());

                while self.at().token_type == TokenType::Comma {
                    self.eat();
                    elements.push(self.parse_expr());
                }

                self.expect(
                    TokenType::CloseBracket,
                    "Expected closing bracket ']' for array literal",
                );
                Expr::ArrayLiteral(elements)
            }

            TokenType::Await => {
                self.eat();
                let value = self.parse_expr();
                Expr::AwaitExpression {
                    value: Box::new(value),
                }
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
