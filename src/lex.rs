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
    pub fn new(data: &'a str) -> TextInput<'a> {
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

    pub fn mark(&mut self) -> usize {
        self.left = self.right;
        self.left
    }

    pub fn step(&mut self) -> Option<char> {
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

    pub fn slice(&self) -> &str {
        &self.data[self.left..self.right] //.to_string()
    }

    pub fn pr(&self) -> () {
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
}

enum State {
    Start,
    OnBang,
    //OnEqual,
    //OnGreat,
    //OnLess,
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
    EOF,
}

fn with_step(text: &mut TextInput, state: State, token: Token) -> (State, Option<Token>) {
    text.step();
    (state, Some(token))
}

fn to_start(text: &mut TextInput, token: Token) -> (State, Option<Token>) {
    with_step(text, State::Start, token)
}

fn from_start(text: &mut TextInput) -> (State, Option<Token>) {
    match text.current {
        Some('(') => to_start(text, Token::LeftParen),
        Some(')') => to_start(text, Token::RightParen),
        Some('{') => to_start(text, Token::LeftBrace),
        Some('}') => to_start(text, Token::RightBrace),
        Some(',') => to_start(text, Token::Comma),
        Some('.') => to_start(text, Token::Dot),
        Some('-') => to_start(text, Token::Minus),
        Some('+') => to_start(text, Token::Plus),
        Some(';') => to_start(text, Token::Semicolon),
        Some('/') => to_start(text, Token::Slash),
        Some('*') => to_start(text, Token::Star),
        Some('!') => with_step(text, State::OnBang, Token::Star),

        _ => {
            text.step();
            (State::Start, None)
        }
    }
}

fn from_bang(text: &mut TextInput) -> (State, Option<Token>) {
    match text.current {
        Some('=') => to_start(text, Token::BangEqual),
        _ => (State::Start, Some(Token::Bang)),
    }
}

fn step(state: State, text: &mut TextInput) -> (State, Option<Token>) {
    match state {
        State::Start => from_start(text),
        State::OnBang => from_bang(text),
    }
}

pub fn lex(s: String) -> () {
    let mut txt = TextInput::new(&s);
    txt.pr();
    let mut state = State::Start;
    while let Some(c) = txt.current {
        if c == 'o' {
            txt.mark();
        }
        if c == '\n' {
            txt.mark();
        }
        let (s, t) = step(state, &mut txt);
        println!(" {:?}", t);
        state = s;

        //txt.step();
        txt.pr();
    }
    ()
}
