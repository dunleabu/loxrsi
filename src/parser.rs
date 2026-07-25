use std::mem::replace;
use std::vec::IntoIter;

use crate::expression::Expression;
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
}

fn expression(stream: &mut TokenStream) -> Option<Expression> {
    panic!("EXPRESSION!")
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
                match stream.peek() {
                    Some(TokenContext {
                        token: Token::RightParen,
                        ..
                    }) => {
                        stream.drop();
                        Expression::grouping(expr?)
                    }
                    x => panic!("no ) in grouping: {:?}", x),
                }
            }
            x => panic!("not supported as primary! {:?}", x),
        }),
        None => None,
    }
}

/// peek: if +/- parse rest as 1ary else parse all as 1ary
fn unary(stream: &mut TokenStream) -> Option<Expression> {
    match stream.peek() {
        Some(TokenContext {
            token: t,
            context: _c,
        }) => match t {
            Token::Minus => {
                stream.drop();
                let expr = unary(stream);
                Some(Expression::unary_neg(expr?))
            }
            Token::Bang => {
                stream.drop();
                let expr = unary(stream);
                Some(Expression::unary_not(expr?))
            }
            _ => primary(stream),
        },
        None => None,
    }
}
