use loxrsi::lexer::lex;
use loxrsi::parser::parse;
use std::env;

fn main() {
    let mut text = env::args().skip(1).collect::<Vec<_>>().join(" ");
    text.push_str("\n");
    println!("{}", text);

    //let text = "1+2* 4\n";
    let tokens = lex(&text).expect("!!");
    for token in &tokens {
        println!("{:?}", token);
    }
    //let mut stream = TokenStream::new(tokens.into_iter());
    let output = parse(tokens.into_iter()).expect("output!");

    println!("{}", output);
}
