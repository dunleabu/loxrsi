use std::env;
use std::fs;
use std::io;
use std::str::CharIndices;

type Res<T> = Result<T, String>;

mod lex;

use lex::{Token, TokenContext};

#[derive(Debug)]
struct FileArgs {
    pub path: Option<String>,
}

struct ReplArgs {}

enum Args {
    File(FileArgs),
    Repl(ReplArgs),
}

fn parse_args() -> Res<Args> {
    let mut args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        Ok(Args::File(FileArgs {
            path: Some(args.pop().unwrap()),
        }))
    } else if args.len() == 1 {
        Ok(Args::Repl(ReplArgs {}))
    } else {
        Err("usage: lox <path>".to_string())
    }
}

fn read_file(args: FileArgs) -> Res<String> {
    match fs::read_to_string(args.path.unwrap()) {
        Ok(text) => Ok(text),
        Err(e) => Err(format!("{}", e)),
    }
}

fn print_errors(errors: &Vec<TokenContext>) -> String {
    let mut count = 0;
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

    println!("");
}

fn run_file(args: FileArgs) -> Res<()> {
    let text = read_file(args)?;
    match lex::lex(text) {
        Err(errors) => Err(print_errors(&errors)),
        Ok(tokens) => Ok(print_tokens(&tokens)),
    }
}

fn run_repl(args: ReplArgs) -> Res<()> {
    panic!("no repl");
    let mut input = String::new();
    loop {
        //std::io
    }
}

fn main() -> Res<()> {
    println!("");
    match parse_args()? {
        Args::File(args) => run_file(args),
        Args::Repl(args) => run_repl(args),
    }
}
