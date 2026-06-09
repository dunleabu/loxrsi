use std::env;
use std::fs;
use std::str::CharIndices;


type Res<T> = Result<T, String>;

mod lex;

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

fn main() -> Res<()> {
    let args = parse_args()?;
    let text = read_file(args)?;
    let tokens = lex::lex(text);
    println!(" -- {:?}", tokens);
    Ok(())
}
