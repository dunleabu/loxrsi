use std::fmt;

pub enum Expression {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
    Grouping(Grouping),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(x) => write!(f, "{}", x),
            Self::Unary(x) => write!(f, "{}", x),
            Self::Binary(x) => write!(f, "{}", x),
            Self::Grouping(x) => write!(f, "{}", x),
        }
    }
}

enum Literal {
    Number(f64),
    Str(String),
    True,
    False,
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

struct Grouping {
    e: Box<Expression>,
}

impl fmt::Display for Grouping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(group {})", self.e)
    }
}

enum Unary {
    Minus(Box<Expression>),
    Bang(Box<Expression>),
}

impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Minus(x) => write!(f, "(- {})", x),
            Self::Bang(x) => write!(f, "(! {})", x),
        }
    }
}

struct Binary {
    left: Box<Expression>,
    op: Operator,
    right: Box<Expression>,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.op, self.left, self.right)
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

fn binary(left: Expression, op: Operator, right: Expression) -> Expression {
    Expression::Binary(Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn num(n: f64) -> Expression {
    Expression::Literal(Literal::Number(n))
}

fn group(expr: Expression) -> Expression {
    Expression::Grouping(Grouping { e: Box::new(expr) })
}

fn add(left: Expression, right: Expression) -> Expression {
    binary(left, Operator::Add, right)
}

fn mul(left: Expression, right: Expression) -> Expression {
    binary(left, Operator::Mul, right)
}

fn negate(e: Expression) -> Expression {
    Expression::Unary(Unary::Minus(Box::new(e)))
}

pub fn demo() -> Expression {
    let n1 = num(56.0);
    let n2 = num(23.2);
    let n3 = num(23.2);
    let n4 = num(10.1);
    let x = add(n1, n2);
    let y = mul(group(x), mul(n3, n4));
    y
}
