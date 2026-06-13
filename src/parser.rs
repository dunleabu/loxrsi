use std::mem::replace;
use std::vec::IntoIter;

use crate::expression::Expression;
use crate::lexer::{Context, Token, TokenContext};

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
}

fn primary(stream: &mut TokenStream) -> Option<Expression> {
    match stream.advance() {
        Some(TokenContext {
            token: t,
            context: c,
        }) => Some(match t {
            Token::Number(n) => Expression::number(n),
            Token::String(s) => Expression::string(s),
            x => panic!("not supported! {:?}", x),
        }),
        None => None,
    }
}
