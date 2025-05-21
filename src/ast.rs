use std::collections::HashMap;

#[derive(Debug)]
pub enum Stmt {
    Program {
        body: Vec<Stmt>,
    },
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<Expr>,
        var_type: Type,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        param_types: Vec<Type>,
        return_type: Type,
        body: Vec<Stmt>,
    },
    Expression(Expr),
    IfStatement {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_if_branches: Option<Vec<ElseIfBranch>>,
        else_branch: Option<Vec<Stmt>>,
    },
    WhileStatement {
        condition: Expr,
        body: Vec<Stmt>,
    },
    ReturnStatement {
        value: Option<Expr>,
    },

    ForLoopStatement {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },

    ForInLoopStatement {
        iterator: Option<Box<Stmt>>,
        iterable: Option<Expr>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug)]
pub enum Expr {
    BooleanLiteral(bool),
    CompoundAssignment {
        assignee: Box<Expr>,
        value: Box<Expr>,
        operator: String,
    },

    Assignment {
        assignee: Box<Expr>,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: String,
    },
    Call {
        caller: Box<Expr>,
        args: Vec<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool,
    },
    Identifier(String),
    NumericLiteral(f64),
    StringLiteral(String),
    ArrayLiteral(Vec<Expr>),
    ObjectLiteral(Vec<Property>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Array(Box<Type>),
    Object(HashMap<String, Type>),
    Any,
    Void,
}
#[derive(Debug)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Property {
    pub key: String,
    pub value: Option<Expr>,
}
