#[derive(Debug)]
pub enum Stmt {
    Program {
        body: Vec<Stmt>,
    },
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<Expr>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
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
}

#[derive(Debug)]
pub enum Expr {
    BooleanLiteral(bool),

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
    },
    Identifier(String),
    NumericLiteral(f64),
    StringLiteral(String),
    ArrayLiteral(Vec<Expr>),
    ObjectLiteral(Vec<Property>),
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
