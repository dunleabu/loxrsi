use std::fmt;

pub struct InFix {
    left: Box<Expression>,
    op: Operator,
    right: Box<Expression>,
}

impl InFix {
    fn new(left: Expression, op: Operator, right: Expression) -> Self {
        Self {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

impl fmt::Display for InFix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.op, self.left, self.right)
    }
}

pub enum Expression {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
    Unary(UnaryOp, Box<Expression>),
    Binary(InFix),
    Grouping(Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
            Self::Unary(op, x) => write!(f, "({} {})", op, x),
            Self::Binary(x) => x.fmt(f),
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

pub enum Operator {
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

impl Operator {
    pub fn expr(self, left: Expression, right: Expression) -> Expression {
        Expression::Binary(InFix::new(left, self, right))
    }
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

impl Expression {
    pub fn number(n: f64) -> Expression {
        Expression::Number(n)
    }

    pub fn string(s: String) -> Expression {
        Expression::String(s)
    }

    pub fn grouping(expr: Expression) -> Expression {
        Expression::Grouping(Box::new(expr))
    }

    pub fn negate(expr: Expression) -> Expression {
        Expression::Unary(UnaryOp::Minus, Box::new(expr))
    }
    pub fn not(expr: Expression) -> Expression {
        Expression::Unary(UnaryOp::Bang, Box::new(expr))
    }

    pub fn binary(op: Operator, left: Expression, right: Expression) -> Expression {
        Expression::Binary(InFix::new(left, op, right))
    }
}

// functions for demonstrating pretty-printing of expression

fn mul(left: Expression, right: Expression) -> Expression {
    Operator::Mul.expr(left, right)
}

pub fn demo() -> Expression {
    let n1 = Expression::number(56.0);
    let n2 = Expression::number(23.2);
    let n3 = Expression::number(23.2);
    let n4 = Expression::number(10.1);
    let x = Operator::Add.expr(n1, n2);
    let y = mul(Expression::grouping(x), mul(n3, Expression::negate(n4)));
    y
}
