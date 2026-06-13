use std::fmt;

pub enum Expression {
    Number(f64),
    Str(String),
    True,
    False,
    Nil,
    Unary(UnaryOp, Box<Expression>),
    Binary {
        left: Box<Expression>,
        op: Operator,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
            Self::Unary(op, x) => write!(f, "({} {})", op, x),
            Self::Binary { left, op, right } => write!(f, "({} {} {})", op, left, right),
            Self::Grouping(e) => write!(f, "(group {})", e),
        }
    }
}

enum UnaryOp {
    Minus,
    Bang,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Minus => write!(f, "-"),
            Self::Bang => write!(f, "!"),
        }
    }
}

enum Operator {
    IsEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IsEqual => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

// functions for demonstrating pretty-printing of expression

fn binary(left: Expression, op: Operator, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

fn num(n: f64) -> Expression {
    Expression::Number(n)
}

fn group(expr: Expression) -> Expression {
    Expression::Grouping(Box::new(expr))
}

fn add(left: Expression, right: Expression) -> Expression {
    binary(left, Operator::Add, right)
}

fn mul(left: Expression, right: Expression) -> Expression {
    binary(left, Operator::Mul, right)
}

fn negate(e: Expression) -> Expression {
    Expression::Unary(UnaryOp::Minus, Box::new(e))
}

pub fn demo() -> Expression {
    let n1 = num(56.0);
    let n2 = num(23.2);
    let n3 = num(23.2);
    let n4 = num(10.1);
    let x = add(n1, n2);
    let y = mul(group(x), mul(n3, negate(n4)));
    y
}
