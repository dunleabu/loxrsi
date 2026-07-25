use std::mem::replace;
use std::vec::IntoIter;

use crate::expression::{Expression, Operator};
use crate::lexer::{Context, Keyword, Token, TokenContext};

/*
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
*/

pub struct TokenStream {
    iter: IntoIter<TokenContext>,
    current: Option<TokenContext>,
}

impl TokenStream {

    pub fn new(iter: IntoIter<TokenContext>) -> Self {
        let mut stream = Self{iter, current: None};
        stream.advance();
        stream
    }

    fn advance(&mut self) -> Option<TokenContext> {
        let x = replace(&mut self.current, self.iter.next());
        println!("stream advance: {:?}", x);
        x
    }

    fn peek(&self) -> &Option<TokenContext> {
        println!("stream peek: {:?}", self.current);
        &self.current
    }

    fn drop(&mut self) {
        let _ = self.advance();
    }

    fn expect(&mut self, f: fn(&Token) -> Option<Operator>) -> Result<Option<Operator>, Token> {
        match self.advance() {
            Some(TokenContext {
                token: t,
                context: _c,
            }) => match f(&t) {
                None => {
                    println!("stream expect rejects {:?}", t);
                    Err(t)
                },
                x => {
                    println!("stream expect accepts {:?} => {:?}", t, x);
                    Ok(x)
                },
            },
            None => Ok(None),
        }
    }
}

// Token filtering functions

fn eq_or_not_eq(x: &Token) -> Option<Operator> {
    match x {
        Token::EqualEqual => Some(Operator::IsEqual),
        Token::BangEqual => Some(Operator::NotEqual),
        _ => None,
    }
}

fn inequality(x: &Token) -> Option<Operator> {
    match x {
        Token::Greater => Some(Operator::GreaterThan),
        Token::GreaterEqual => Some(Operator::GreaterThanOrEqual),
        Token::Less => Some(Operator::LessThan),
        Token::LessEqual => Some(Operator::LessThanOrEqual),
        _ => None,
    }
}

fn plus_or_minus(x: &Token) -> Option<Operator> {
    match x {
        Token::Plus => Some(Operator::Add),
        Token::Minus => Some(Operator::Sub),
        _ => None,
    }
}

fn star_or_slash(x: &Token) -> Option<Operator> {
    match x {
        Token::Star => Some(Operator::Mul),
        Token::Slash => Some(Operator::Div),
        _ => None,
    }
}

// Parsing functions

pub fn expression(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter expression");
    equality(stream)
}

fn equality(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter equality");
    let left = comparison(stream)?;

    match stream.expect(eq_or_not_eq) {
        Ok(None) => Some(left),
        Ok(Some(op)) => {
            let right = equality(stream).expect("unterminated equality expression");
            Some(op.expr(left, right))
        }
        Err(x) => {
            panic!("unexpected token after comparison expression: {:?}", x);
        }
    }
}

fn comparison(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter comparison");
    let left = term(stream)?;

    match stream.expect(inequality) {
        Ok(None) => Some(left),
        Ok(Some(op)) => {
            let right = comparison(stream).expect("unterminated comparison expression");
            Some(op.expr(left, right))
        }
        Err(x) => {
            panic!("unexpected token after term expression: {:?}", x);
        }
    }
}

fn term(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter term");
    let left = factor(stream)?;

    match stream.expect(plus_or_minus) {
        Ok(None) => Some(left),
        Ok(Some(op)) => {
            let right = term(stream).expect("unterminated term expression");
            Some(op.expr(left, right))
        }
        Err(x) => {
            panic!("unexpected token after factor expression: {:?}", x);
        }
    }
}

fn factor(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter factor");
    let left = unary(stream)?;

    match stream.expect(star_or_slash) {
        Ok(None) => Some(left),
        Ok(Some(op)) => {
            let right = factor(stream).expect("unterminated factor expression");
            Some(op.expr(left, right))
        }
        Err(x) => {
            panic!("unexpected token after unary expression: {:?}", x);
        }
    }
}

fn unary(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter unary");
    match stream.peek() {
        Some(TokenContext {
            token: t,
            context: _c,
        }) => match t {
            Token::Minus => {
                println!("unary: found minus");
                stream.drop();
                let expr = unary(stream);
                Some(Expression::negate(expr?))
            }
            Token::Bang => {
                println!("unary: found plus");
                stream.drop();
                let expr = unary(stream);
                Some(Expression::not(expr?))
            }
            _ => {
                println!("unary: not unary");
                primary(stream)
            },
        },
        None => None,
    }
}

fn primary(stream: &mut TokenStream) -> Option<Expression> {
    let x = _primary(stream);
    match &x {
        Some(y) => println!("primary: returned {}", y),
        None => println!("primary: returned None"),
        };
    x
}

fn _primary(stream: &mut TokenStream) -> Option<Expression> {
    println!("enter primary");
    match stream.advance() {
        Some(TokenContext {
            token: t,
            context: _c,
        }) => Some(match t {
            Token::Number(n) => Expression::number(n),
            Token::String(s) => Expression::string(s),
            Token::Keyword(Keyword::True) => Expression::True,
            Token::Keyword(Keyword::False) => Expression::False,
            Token::Keyword(Keyword::Nil) => Expression::Nil,
            Token::LeftParen => {
                let expr = expression(stream);
                match stream.advance() {
                    Some(TokenContext {
                        token: Token::RightParen,
                        ..
                    }) => Expression::grouping(expr?),
                    x => panic!("no ) in grouping: {:?}", x),
                }
            }
            x => panic!("not supported as primary! {:?}", x),
        }),
        None => None,
    }
}
