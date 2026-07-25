use std::str::CharIndices;

#[derive(Debug)]
struct TextInput<'a> {
    iter: CharIndices<'a>,
    data: &'a str,
    left: usize,
    pub current: Option<char>,
    right: usize,
    char_pos: usize,
    line_num: usize,
}

impl<'a> TextInput<'_> {
    fn new(data: &'a str) -> TextInput<'a> {
        let mut iter = data.char_indices();
        match iter.next() {
            Some((right, current)) => TextInput {
                iter,
                data,
                left: 0,
                current: Some(current),
                right,
                char_pos: 1,
                line_num: 1,
            },
            None => TextInput {
                iter,
                data,
                left: 0,
                current: None,
                right: 0,
                char_pos: 1,
                line_num: 1,
            },
        }
    }

    fn mark(&mut self) -> usize {
        self.left = self.right;
        self.left
    }

    fn step(&mut self) -> Option<char> {
        match self.iter.next() {
            Some((right, current)) => {
                self.current = Some(current);
                self.right = right;
                if current == '\n' {
                    self.char_pos = 0;
                    self.line_num += 1;
                } else {
                    self.char_pos += 1;
                }
            }
            None => {
                self.current = None;
                self.right = self.data.len();
            }
        }
        self.current
    }

    fn slice(&self) -> &str {
        &self.data[self.left..self.right]
    }

    fn pr(&self) -> () {
        println!(
            "  {} {:?} {} ({},{}): {}",
            self.left,
            self.current,
            self.right,
            self.char_pos,
            self.line_num,
            self.slice()
        );
    }

    fn add_context(&self, token: Token) -> TokenContext {
        TokenContext {
            token,
            context: Context {
                line: self.line_num,
                pos: self.char_pos,
            },
        }
    }
}

enum State {
    Start,
    OnBang,
    OnEqual,
    OnGreater,
    OnLess,
    InComment,
    AddDot, // special state that should always add a dot next turn
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    // single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // one-or-two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals
    Identifier(String),
    String(String),
    Number(f64),
    // keywords
    Keyword(Keyword),
    //EOF,
    Error(String),
}

#[derive(Debug)]
pub struct Context {
    pub line: usize,
    pub pos: usize,
}
#[derive(Debug)]
pub struct TokenContext {
    pub token: Token,
    pub context: Context,
}

type StepOut = (State, Option<TokenContext>);

fn with_step(text: &mut TextInput, state: State, token: Option<TokenContext>) -> StepOut {
    text.step();
    (state, token)
}

fn emit_if_end(text: &mut TextInput, state: State, token: Token) -> StepOut {
    text.step();
    match text.current {
        None => (State::Start, Some(text.add_context(token))),
        Some(_) => (state, None),
    }
}

fn to_start(text: &mut TextInput, token: Token) -> StepOut {
    with_step(text, State::Start, Some(text.add_context(token)))
}

fn maybe_comment(text: &mut TextInput) -> StepOut {
    text.step();
    if let Some('/') = text.current {
        (State::InComment, None)
    } else {
        (State::Start, Some(text.add_context(Token::Slash)))
    }
}

fn step_digits(text: &mut TextInput) -> usize {
    let mut counter = 0;
    loop {
        match text.current {
            Some(c) if c.is_ascii_digit() => {
                counter += 1;
                text.step();
            }
            _ => break,
        }
    }
    counter
}

fn to_number(text: &mut TextInput) -> StepOut {
    text.mark();
    text.step();
    step_digits(text);
    if let Some(c) = text.current
        && c == '.'
    {
        text.step();
        if step_digits(text) == 0 {
            let number = text
                .slice()
                .strip_suffix('.')
                .unwrap()
                .parse::<f64>()
                .expect("failed number conversion!");
            return (
                State::AddDot,
                Some(text.add_context(Token::Number(number))), //Some(text.add_context(Token::Error(format!(
                                                               //    "unterminated number: {}",
                                                               //    text.slice()
                                                               //)))),
            );
        }
    }
    let number = text
        .slice()
        .parse::<f64>()
        .expect("failed number conversion");
    (State::Start, Some(text.add_context(Token::Number(number))))
}

fn to_string(text: &mut TextInput) -> StepOut {
    text.step();
    text.mark();
    loop {
        match text.current {
            None => {
                return (
                    State::Start,
                    Some(text.add_context(Token::Error("unterminated string".to_string()))),
                );
            }
            Some('"') => break,
            _ => {
                text.step();
            }
        }
    }
    let string = text.slice().to_owned();
    let token = text.add_context(Token::String(string));
    text.step();
    (State::Start, Some(token))
}

fn whitespace(text: &mut TextInput) -> StepOut {
    with_step(text, State::Start, None)
}

fn okay_for_id(x: Option<char>) -> bool {
    if let Some(c) = x {
        c.is_alphanumeric() || c == '_'
    } else {
        false
    }
}

fn to_identifier(text: &mut TextInput) -> StepOut {
    text.mark();
    text.step();
    loop {
        if okay_for_id(text.current) {
            text.step();
        } else {
            break;
        }
    }
    let token = match text.slice() {
        "and" => Token::Keyword(Keyword::And),
        "class" => Token::Keyword(Keyword::Class),
        "else" => Token::Keyword(Keyword::Else),
        "false" => Token::Keyword(Keyword::False),
        "fun" => Token::Keyword(Keyword::Fun),
        "for" => Token::Keyword(Keyword::For),
        "if" => Token::Keyword(Keyword::If),
        "nil" => Token::Keyword(Keyword::Nil),
        "or" => Token::Keyword(Keyword::Or),
        "print" => Token::Keyword(Keyword::Print),
        "return" => Token::Keyword(Keyword::Return),
        "super" => Token::Keyword(Keyword::Super),
        "this" => Token::Keyword(Keyword::This),
        "true" => Token::Keyword(Keyword::True),
        "var" => Token::Keyword(Keyword::Var),
        "while" => Token::Keyword(Keyword::While),
        x => Token::Identifier(x.to_owned()),
    };
    (State::Start, Some(text.add_context(token)))
}

fn from_start(text: &mut TextInput) -> StepOut {
    match text.current {
        Some(c) => {
            match c {
                // whitespace
                ' ' => whitespace(text),
                '\t' => whitespace(text),
                '\n' => whitespace(text),
                // single-char
                '(' => to_start(text, Token::LeftParen),
                ')' => to_start(text, Token::RightParen),
                '{' => to_start(text, Token::LeftBrace),
                '}' => to_start(text, Token::RightBrace),
                ',' => to_start(text, Token::Comma),
                '.' => to_start(text, Token::Dot),
                '-' => to_start(text, Token::Minus),
                '+' => to_start(text, Token::Plus),
                ';' => to_start(text, Token::Semicolon),
                //'/' => to_start(text, Token::Slash),
                '*' => to_start(text, Token::Star),
                // possible double-char
                '!' => emit_if_end(text, State::OnBang, Token::Bang),
                '=' => emit_if_end(text, State::OnEqual, Token::Equal),
                '>' => emit_if_end(text, State::OnGreater, Token::Greater),
                '<' => emit_if_end(text, State::OnLess, Token::Less),
                // multi-character
                '/' => maybe_comment(text),
                '"' => to_string(text),
                '_' => to_identifier(text),
                _ if c.is_ascii_digit() => to_number(text),
                _ if c.is_alphanumeric() => to_identifier(text),

                _ => {
                    text.step();
                    (
                        State::Start,
                        Some(text.add_context(Token::Error(format!("bad character: '{}'", c)))),
                    )
                }
            }
        }
        None => (State::Start, None),
    }
}

fn if_equal(text: &mut TextInput, equal_token: Token, other_token: Token) -> StepOut {
    match text.current {
        Some('=') => to_start(text, equal_token),
        _ => (State::Start, Some(text.add_context(other_token))),
    }
}

fn from_comment(text: &mut TextInput) -> StepOut {
    text.step();
    if let Some('\n') = text.current {
        (State::Start, None)
    } else {
        (State::InComment, None)
    }
}

fn step(state: State, text: &mut TextInput) -> StepOut {
    match state {
        State::Start => from_start(text),
        State::OnBang => if_equal(text, Token::BangEqual, Token::Bang),
        State::OnEqual => if_equal(text, Token::EqualEqual, Token::Equal),
        State::OnGreater => if_equal(text, Token::GreaterEqual, Token::Greater),
        State::OnLess => if_equal(text, Token::LessEqual, Token::Less),
        State::InComment => from_comment(text),
        State::AddDot => (State::Start, Some(text.add_context(Token::Dot))),
    }
}

pub fn lex(s: &str) -> Result<Vec<TokenContext>, Vec<TokenContext>> {
    let mut txt = TextInput::new(s);
    //txt.pr();
    let mut state = State::Start;
    let mut has_error = false;
    let mut tokens: Vec<TokenContext> = Vec::new();
    let mut errors: Vec<TokenContext> = Vec::new();
    while let Some(c) = txt.current {
        if c == 'o' {
            txt.mark();
        }
        if c == '\n' {
            txt.mark();
        }
        let (s, t) = step(state, &mut txt);
        match t {
            Some(
                tc @ TokenContext {
                    token: Token::Error(_),
                    ..
                },
            ) => {
                //println!("ERROR on line {} pos {}: {:?}", line, pos, &msg);
                has_error = true;
                errors.push(tc);
            }

            Some(token) => {
                //println!("{} -> {:?}", c, token);
                tokens.push(token);
            }
            None => {
                //println!("{} -> ...", c);
            }
        }
        state = s;

        //txt.step();
        //txt.pr();
    }
    if matches!(state, State::AddDot) {
        tokens.push(txt.add_context(Token::Dot))
    }
    if has_error { Err(errors) } else { Ok(tokens) }
}

#[cfg(test)]
mod tests {
    //! Expected tokens below were worked out by hand from the Lox language
    //! definition, independent of the lexer implementation.
    use super::*;

    /// Lex `input`, asserting success, and return just the tokens.
    fn lex_ok(input: &str) -> Vec<Token> {
        lex(input)
            .expect("expected a successful lex")
            .into_iter()
            .map(|tc| tc.token)
            .collect()
    }

    /// Lex `input`, asserting it produced errors, and return the error tokens.
    fn lex_err(input: &str) -> Vec<Token> {
        lex(input)
            .expect_err("expected a lex error")
            .into_iter()
            .map(|tc| tc.token)
            .collect()
    }

    fn id(s: &str) -> Token {
        Token::Identifier(s.to_string())
    }
    fn string(s: &str) -> Token {
        Token::String(s.to_string())
    }
    fn num(n: f64) -> Token {
        Token::Number(n)
    }
    fn kw(k: Keyword) -> Token {
        Token::Keyword(k)
    }

    // ---- empty / whitespace ----

    #[test]
    fn empty_input_yields_no_tokens() {
        assert_eq!(lex_ok(""), vec![]);
    }

    #[test]
    fn whitespace_only_yields_no_tokens() {
        assert_eq!(lex_ok("   \t\n  \n\t "), vec![]);
    }

    // ---- single-character tokens ----

    #[test]
    fn all_single_character_tokens() {
        assert_eq!(
            lex_ok("(){},.-+;*/"),
            vec![
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBrace,
                Token::RightBrace,
                Token::Comma,
                Token::Dot,
                Token::Minus,
                Token::Plus,
                Token::Semicolon,
                Token::Star,
                Token::Slash,
            ]
        );
    }

    #[test]
    fn lone_slash_is_division() {
        assert_eq!(lex_ok("/"), vec![Token::Slash]);
        assert_eq!(lex_ok("a / b"), vec![id("a"), Token::Slash, id("b")]);
    }

    // ---- comments ----

    #[test]
    fn comment_line_produces_no_tokens() {
        assert_eq!(lex_ok("// this is a comment"), vec![]);
    }

    #[test]
    fn comment_without_trailing_newline() {
        assert_eq!(lex_ok("// no newline at eof"), vec![]);
    }

    #[test]
    fn code_after_comment_on_next_line() {
        assert_eq!(lex_ok("// comment\n+"), vec![Token::Plus]);
    }

    #[test]
    fn code_before_comment_on_same_line() {
        assert_eq!(
            lex_ok("+ // trailing comment\n-"),
            vec![Token::Plus, Token::Minus]
        );
    }

    // ---- one-or-two character operators ----

    #[test]
    fn bang_and_bang_equal() {
        assert_eq!(lex_ok("! "), vec![Token::Bang]);
        assert_eq!(lex_ok("!="), vec![Token::BangEqual]);
    }

    #[test]
    fn equal_and_equal_equal() {
        assert_eq!(lex_ok("= "), vec![Token::Equal]);
        assert_eq!(lex_ok("=="), vec![Token::EqualEqual]);
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(lex_ok("< "), vec![Token::Less]);
        assert_eq!(lex_ok("<="), vec![Token::LessEqual]);
        assert_eq!(lex_ok("> "), vec![Token::Greater]);
        assert_eq!(lex_ok(">="), vec![Token::GreaterEqual]);
    }

    #[test]
    fn operators_run_together() {
        // "!=" "<=" ">=" "=="
        assert_eq!(
            lex_ok("!=<=>==="),
            vec![
                Token::BangEqual,
                Token::LessEqual,
                Token::GreaterEqual,
                Token::EqualEqual,
            ]
        );
    }

    // Corner case: an operator needing lookahead sitting at EOF.
    #[test]
    fn bare_bang_at_eof() {
        assert_eq!(lex_ok("!"), vec![Token::Bang]);
    }

    #[test]
    fn bare_greater_at_eof() {
        assert_eq!(lex_ok(">"), vec![Token::Greater]);
    }

    #[test]
    fn bare_equal_at_eof() {
        assert_eq!(lex_ok("="), vec![Token::Equal]);
    }

    // ---- strings ----

    #[test]
    fn simple_string() {
        assert_eq!(lex_ok("\"hello\""), vec![string("hello")]);
    }

    #[test]
    fn empty_string() {
        assert_eq!(lex_ok("\"\""), vec![string("")]);
    }

    #[test]
    fn string_with_spaces_and_punctuation() {
        assert_eq!(lex_ok("\"a b, c! 123\""), vec![string("a b, c! 123")]);
    }

    #[test]
    fn string_keeps_slashes_and_keywords_literal() {
        assert_eq!(
            lex_ok("\"and // not a comment\""),
            vec![string("and // not a comment")]
        );
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(lex("\"no closing quote").is_err());
    }

    // ---- numbers ----

    #[test]
    fn integer_literal() {
        assert_eq!(lex_ok("123"), vec![num(123.0)]);
    }

    #[test]
    fn integer_followed_by_semicolon() {
        assert_eq!(lex_ok("123;"), vec![num(123.0), Token::Semicolon]);
    }

    #[test]
    fn zero_literal() {
        assert_eq!(lex_ok("0"), vec![num(0.0)]);
    }

    #[test]
    fn decimal_literal() {
        assert_eq!(lex_ok("3.14"), vec![num(3.14)]);
    }

    #[test]
    fn decimal_followed_by_semicolon() {
        assert_eq!(lex_ok("3.14;"), vec![num(3.14), Token::Semicolon]);
    }

    // In Lox a trailing dot is not part of the number: it lexes as the
    // number followed by a `.` token (methods can be called on numbers).
    #[test]
    fn number_with_trailing_dot() {
        assert_eq!(lex_ok("123."), vec![num(123.0), Token::Dot]);
    }

    // A leading dot is likewise not part of the number.
    #[test]
    fn leading_dot_before_digits() {
        assert_eq!(lex_ok(".5"), vec![Token::Dot, num(5.0)]);
    }

    #[test]
    fn trailing_dot_before_more_input() {
        assert_eq!(lex_ok("123.;"), vec![num(123.0), Token::Dot, Token::Semicolon]);
        assert_eq!(lex_ok("123. "), vec![num(123.0), Token::Dot]);
        assert_eq!(lex_ok("0."), vec![num(0.0), Token::Dot]);
    }

    // The re-emitted dot must not swallow or duplicate a following dot.
    #[test]
    fn number_with_two_trailing_dots() {
        assert_eq!(lex_ok("123.."), vec![num(123.0), Token::Dot, Token::Dot]);
    }

    // Only the first dot joins the number; the rest are separate tokens.
    #[test]
    fn dotted_access_on_number() {
        assert_eq!(lex_ok("1.2.3"), vec![num(1.2), Token::Dot, num(3.0)]);
        assert_eq!(lex_ok("1.a"), vec![num(1.0), Token::Dot, id("a")]);
    }

    // ---- non-ascii ----

    // `okay_for_id` accepts any alphanumeric, so identifiers may be non-ascii.
    // Multi-byte chars at end of input exercise the slice boundary.
    #[test]
    fn unicode_identifier() {
        assert_eq!(lex_ok("café"), vec![id("café")]);
        assert_eq!(lex_ok("é"), vec![id("é")]);
        assert_eq!(lex_ok("café;"), vec![id("café"), Token::Semicolon]);
    }

    #[test]
    fn unicode_string_contents() {
        assert_eq!(lex_ok("\"héllo\""), vec![string("héllo")]);
    }

    #[test]
    fn unicode_after_slash_at_eof() {
        assert_eq!(lex_ok("/é"), vec![Token::Slash, id("é")]);
    }

    #[test]
    fn non_alphanumeric_unicode_is_error() {
        assert!(lex("€").is_err());
    }

    // ---- identifiers ----

    #[test]
    fn simple_identifier() {
        assert_eq!(lex_ok("foo"), vec![id("foo")]);
    }

    #[test]
    fn identifier_followed_by_semicolon() {
        assert_eq!(lex_ok("foo;"), vec![id("foo"), Token::Semicolon]);
    }

    #[test]
    fn identifier_with_underscore_and_digits() {
        assert_eq!(lex_ok("_foo_bar2"), vec![id("_foo_bar2")]);
    }

    #[test]
    fn leading_underscore_identifier() {
        assert_eq!(lex_ok("_"), vec![id("_")]);
    }

    // ---- keywords ----

    #[test]
    fn every_keyword() {
        assert_eq!(
            lex_ok("and class else false fun for if nil or print return super this true var while"),
            vec![
                kw(Keyword::And),
                kw(Keyword::Class),
                kw(Keyword::Else),
                kw(Keyword::False),
                kw(Keyword::Fun),
                kw(Keyword::For),
                kw(Keyword::If),
                kw(Keyword::Nil),
                kw(Keyword::Or),
                kw(Keyword::Print),
                kw(Keyword::Return),
                kw(Keyword::Super),
                kw(Keyword::This),
                kw(Keyword::True),
                kw(Keyword::Var),
                kw(Keyword::While),
            ]
        );
    }

    // Words that merely start with a keyword are identifiers.
    #[test]
    fn keyword_prefixes_are_identifiers() {
        assert_eq!(lex_ok("orchid"), vec![id("orchid")]);
        assert_eq!(lex_ok("android"), vec![id("android")]);
        assert_eq!(lex_ok("classy"), vec![id("classy")]);
        assert_eq!(lex_ok("iffy"), vec![id("iffy")]);
    }

    // ---- fuller snippets ----

    #[test]
    fn var_declaration_with_spaces() {
        assert_eq!(
            lex_ok("var x = 10;"),
            vec![
                kw(Keyword::Var),
                id("x"),
                Token::Equal,
                num(10.0),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn var_declaration_without_spaces() {
        assert_eq!(
            lex_ok("var x=10;"),
            vec![
                kw(Keyword::Var),
                id("x"),
                Token::Equal,
                num(10.0),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn print_statement() {
        assert_eq!(
            lex_ok("print \"hi\";"),
            vec![kw(Keyword::Print), string("hi"), Token::Semicolon]
        );
    }

    #[test]
    fn function_definition() {
        assert_eq!(
            lex_ok("fun add(a, b) { return a + b; }"),
            vec![
                kw(Keyword::Fun),
                id("add"),
                Token::LeftParen,
                id("a"),
                Token::Comma,
                id("b"),
                Token::RightParen,
                Token::LeftBrace,
                kw(Keyword::Return),
                id("a"),
                Token::Plus,
                id("b"),
                Token::Semicolon,
                Token::RightBrace,
            ]
        );
    }

    #[test]
    fn multiline_program() {
        let src = "var a = 1;\nif (a >= 1) {\n  print a;\n}\n";
        assert_eq!(
            lex_ok(src),
            vec![
                kw(Keyword::Var),
                id("a"),
                Token::Equal,
                num(1.0),
                Token::Semicolon,
                kw(Keyword::If),
                Token::LeftParen,
                id("a"),
                Token::GreaterEqual,
                num(1.0),
                Token::RightParen,
                Token::LeftBrace,
                kw(Keyword::Print),
                id("a"),
                Token::Semicolon,
                Token::RightBrace,
            ]
        );
    }

    // ---- errors ----

    #[test]
    fn bad_character_is_error() {
        assert!(lex("@").is_err());
        // The good tokens around it should not turn it into a success.
        assert!(lex("a @ b").is_err());
    }

    #[test]
    fn bad_character_reports_error_token() {
        let errs = lex_err("@");
        assert_eq!(errs.len(), 1);
        assert!(matches!(errs[0], Token::Error(_)));
    }
}
