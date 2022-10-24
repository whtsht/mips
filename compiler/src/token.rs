use std::fmt::Write;
use std::iter::Peekable;

pub type Result<'a, T> = std::result::Result<(&'a str, T), (&'a str, String)>;

pub trait Parser<'a, T>: Fn(&'a str) -> Result<'a, T> {}
impl<'a, T, F: Fn(&'a str) -> Result<'a, T>> Parser<'a, T> for F {}

pub fn integer(i: &str) -> Result<i32> {
    let end = i.find(|c: char| !c.is_ascii_digit()).unwrap_or(i.len());
    match i[..end].parse() {
        Ok(value) => Ok((&i[end..], value)),
        Err(_) => Err((i, "expected number".into())),
    }
}

pub fn char<'a>(c: char) -> impl Parser<'a, char> {
    move |i: &'a str| {
        let mut chars = i.chars();
        if chars.next() == Some(c) {
            Ok((chars.as_str(), c))
        } else {
            Err((i, format!("expected '{}'", c)))
        }
    }
}

pub fn trim<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, T> {
    move |s: &'a str| parser(s.trim_start())
}

pub fn map<'a, A, B>(parser: impl Parser<'a, A>, f: impl Fn(A) -> B + 'a) -> impl Parser<'a, B> {
    move |s: &'a str| parser(s).and_then(|(s, a)| Ok((s, f(a))))
}

pub fn alt<'a, T>(parser1: impl Parser<'a, T>, parser2: impl Parser<'a, T>) -> impl Parser<'a, T> {
    move |s: &'a str| parser1(s).or_else(|_| parser2(s))
}

pub fn convert_error(input: &str, stop: &str, message: &str) -> String {
    let mut output = String::new();
    writeln!(output, "{}", input).unwrap();
    writeln!(
        output,
        "{}^ {}",
        " ".repeat(input.len() - stop.len()),
        message
    )
    .unwrap();

    output
}

#[test]
fn test_error_message() {
    let input = "a456abc";
    let error = match integer(input) {
        Err((s, m)) => convert_error(input, s, &m),
        _ => String::new(),
    };

    assert_eq!(error, "a456abc\n^ expected number\n");
}

#[derive(Debug)]
pub enum TokenType {
    Number(i32),
    Punctuation(Punctuation),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Punctuation {
    Plus,
    Minus,
}

#[derive(Debug)]
pub struct Token<'a> {
    input: &'a str,
    token_type: TokenType,
}

impl<'a> Token<'a> {
    pub fn new(input: &str, token_type: TokenType) -> Token {
        Token { input, token_type }
    }
}

#[macro_export]
macro_rules! alt {
    ($parser0:expr $(,$parser:expr)*) => {{
        let p = $parser0;
        $(
            let p = $crate::token::alt(p, $parser);
        )*
        p
    }};
}

pub fn tokenize(mut i: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    let plus = map(trim(char('+')), |_| {
        TokenType::Punctuation(Punctuation::Plus)
    });
    let minus = map(trim(char('-')), |_| {
        TokenType::Punctuation(Punctuation::Minus)
    });
    let integer = map(trim(integer), |n| TokenType::Number(n));
    let parser = alt!(plus, minus, integer);

    while i.chars().filter(|c| !c.is_whitespace()).count() > 0 {
        match parser(i) {
            Ok((rest, token_type)) => {
                tokens.push(Token::new(rest, token_type));
                i = rest;
            }
            Err((stop, message)) => return Err((stop, message)),
        }
    }

    Ok((i, tokens))
}

pub fn expect_number<'a>(tokens: &mut impl Iterator<Item = &'a Token<'a>>) -> Result<i32> {
    match tokens.next() {
        Some(Token::Number(n)) => Ok(*n),
        _ => Err(anyhow!("expected number")),
    }
}

pub fn consume<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token<'a>>>,
    expect: Punctuation,
) -> bool {
    if let Some(Token::Punctuation(peek)) = tokens.peek() {
        if peek == &expect {
            tokens.next();
            return true;
        }
    }
    false
}
