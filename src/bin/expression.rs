use loxrsi::expression::demo;
use loxrsi::lexer::lex;
use loxrsi::parser::{TokenStream, expression};

fn main() {
    println!("{}", demo());
    let text = "1+2\n";
    let tokens = lex(text).expect("!!");
    let mut stream = TokenStream::new(tokens.into_iter());
    let output = expression(&mut stream).expect("output!");

    println!("{}", output);
}
