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

struct TokenStream {
    iter: IntoIter<TokenContext>,
    current: Option<TokenContext>,
}

impl TokenStream {
    fn advance(&mut self) -> Option<TokenContext> {
        replace(&mut self.current, self.iter.next())
    }

    fn peek(&self) -> &Option<TokenContext> {
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
                None => Err(t),
                x => Ok(x),
            },
            None => Ok(None),
        }
    }
}

// Token filtering functions

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

fn expression(stream: &mut TokenStream) -> Option<Expression> {
    todo!("expression!")
}

fn equality(stream: &mut TokenStream) -> Option<Expression> {
    todo!("equality!")
}

fn comparison(stream: &mut TokenStream) -> Option<Expression> {
    let left = term(stream)?;

    match stream.expect(inequality) {
        Ok(None) => None,
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
    let left = factor(stream)?;

    match stream.expect(plus_or_minus) {
        Ok(None) => None,
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
    let left = unary(stream)?;

    match stream.expect(star_or_slash) {
        Ok(None) => None,
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
    match stream.peek() {
        Some(TokenContext {
            token: t,
            context: _c,
        }) => match t {
            Token::Minus => {
                stream.drop();
                let expr = unary(stream);
                Some(Expression::negate(expr?))
            }
            Token::Bang => {
                stream.drop();
                let expr = unary(stream);
                Some(Expression::not(expr?))
            }
            _ => primary(stream),
        },
        None => None,
    }
}

fn primary(stream: &mut TokenStream) -> Option<Expression> {
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
