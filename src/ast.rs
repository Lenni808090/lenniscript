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
        is_async: bool,
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

    TryCatchFinally {
        try_branch: Vec<Stmt>,
        catch_branch: Vec<Stmt>,
        finally_branch: Option<Vec<Stmt>>,
    },

    SwitchStatement {
        condition: Expr,
        case_branches: Vec<caseBranch>,
        default_branch: Vec<Stmt>,
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
    BreakStatement,
    ContinueStatement,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BooleanLiteral(bool),
    NullLiteral,
    CompoundAssignment {
        assignee: Box<Expr>,
        value: Box<Expr>,
        operator: String,
    },

    Unary {
        operator: String,
        value: Box<Expr>,
    },

    AwaitExpression {
        value: Box<Expr>,
    },

    Increment {
        identifier: Box<Expr>,
        prefix: bool,
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
    Option(Box<Type>),
    Null,
    Any,
    Void,
}
#[derive(Debug)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}
#[derive(Debug)]
pub struct caseBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub key: String,
    pub value: Option<Expr>,
}
