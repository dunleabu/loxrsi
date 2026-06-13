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
            None => self.current = None,
        }
        self.current
    }

    fn slice(&self) -> &str {
        &self.data[self.left..self.right] //.to_string()
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
}

#[derive(Debug)]
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

#[derive(Debug)]
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
            return (
                State::Start,
                Some(text.add_context(Token::Error(format!(
                    "unterminated number: {}",
                    text.slice()
                )))),
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
                '!' => with_step(text, State::OnBang, None),
                '=' => with_step(text, State::OnEqual, None),
                '>' => with_step(text, State::OnGreater, None),
                '<' => with_step(text, State::OnLess, None),
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
    }
}

pub fn lex(s: &String) -> Result<Vec<TokenContext>, Vec<TokenContext>> {
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
    if has_error { Err(errors) } else { Ok(tokens) }
}
