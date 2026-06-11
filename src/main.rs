use std::env;
use std::fs;
use std::str::CharIndices;

type Res<T> = Result<T, String>;

mod lex;

use lex::{Token, TokenContext};

#[derive(Debug)]
struct Args {
    pub path: String,
}

fn parse_args() -> Res<Args> {
    let mut args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        Ok(Args {
            path: args.pop().unwrap(),
        })
    } else {
        Err("usage: lox <path>".to_string())
    }
}

fn read_file(args: Args) -> Res<String> {
    match fs::read_to_string(args.path) {
        Ok(text) => Ok(text),
        Err(e) => Err(format!("{}", e)),
    }
}

fn print_errors(errors: &Vec<TokenContext>) -> String {
    let mut count = 0;
    println!("");
    for error in errors.iter() {
        match error {
            TokenContext {
                token: Token::Error(msg),
                line,
                pos,
            } => {
                count += 1;
                println!("line {}, pos {} : {}\n", line, pos, msg);
            }
            _ => {}
        }
    }
    format!("{} syntax errors", count)
}

fn print_tokens(tokens: &Vec<TokenContext>) {
    for tc in tokens.iter() {
        println!("{}, {} : {:?}", tc.line, tc.pos, tc.token)
    }

}

fn main() -> Res<()> {
    let args = parse_args()?;
    let text = read_file(args)?;
    match lex::lex(text) {
        Err(errors) => Err(print_errors(&errors)),
        Ok(tokens) => Ok(print_tokens(&tokens)),
    }
}
