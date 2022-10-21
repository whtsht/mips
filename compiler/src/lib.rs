use anyhow::{anyhow, Result};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum Token {
    Number(i32),
    Punctuation(Punctuation),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Punctuation {
    Plus,
    Minus,
}

pub trait Parser<'a, T>: Fn(&'a str) -> Result<(&'a str, T)> {}
impl<'a, T, F: Fn(&'a str) -> Result<(&'a str, T)>> Parser<'a, T> for F {}

pub fn integer(s: &str) -> Result<(&str, i32)> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    match s[..end].parse() {
        Ok(value) => Ok((&s[end..], value)),
        Err(_) => Err(anyhow!("expected number")),
    }
}

pub fn char<'a>(c: char) -> impl Parser<'a, char> {
    move |i: &'a str| {
        let mut chars = i.chars();
        if chars.next() == Some(c) {
            Ok((chars.as_str(), c))
        } else {
            Err(anyhow!("expected '{}'", c))
        }
    }
}

pub fn trim<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, T> {
    move |s: &'a str| parser(s.trim_start())
}

pub fn map_res<'a, A, B>(
    parser: impl Parser<'a, A>,
    f: impl Fn(A) -> Result<B> + 'a,
) -> impl Parser<'a, B> {
    move |s: &'a str| parser(s).and_then(|(s, a)| f(a).map(|value| (s, value)))
}

pub fn map<'a, A, B>(parser: impl Parser<'a, A>, f: impl Fn(A) -> B + 'a) -> impl Parser<'a, B> {
    move |s: &'a str| parser(s).and_then(|(s, a)| Ok((s, f(a))))
}

pub fn alt<'a, T>(parser1: impl Parser<'a, T>, parser2: impl Parser<'a, T>) -> impl Parser<'a, T> {
    move |s: &'a str| parser1(s).or_else(|_| parser2(s))
}

#[macro_export]
macro_rules! alt {
    ($parser0:expr $(,$parser:expr)*) => {{
        let p = $parser0;
        $(
            let p = $crate::alt(p, $parser);
        )*
        p
    }};
}

pub fn tokenize(mut i: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let plus = map(trim(char('+')), |_| Token::Punctuation(Punctuation::Plus));
    let minus = map(trim(char('-')), |_| Token::Punctuation(Punctuation::Minus));
    let integer = map(trim(integer), |n| Token::Number(n));
    let parser = alt!(plus, minus, integer);

    while let Ok((rest, token)) = parser(i) {
        tokens.push(token);
        i = rest;
    }
    tokens
}

pub fn expect_number<'a>(tokens: &mut impl Iterator<Item = &'a Token>) -> Result<i32> {
    match tokens.next() {
        Some(Token::Number(n)) => Ok(*n),
        _ => Err(anyhow!("expected number")),
    }
}

use std::iter::Peekable;

pub fn consume<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token>>,
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

pub fn gen_code(i: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(output, ".text")?;
    writeln!(output, ".globl main")?;
    writeln!(output, "main:")?;
    let tokens = tokenize(i);
    let mut tokens = tokens.iter().peekable();

    writeln!(output, "  ori $a0, $zero, {}", expect_number(&mut tokens)?)?;

    while tokens.len() > 0 {
        if consume(&mut tokens, Punctuation::Plus) {
            writeln!(output, "  addi $a0, $a0, {}", expect_number(&mut tokens)?)?;
            continue;
        }

        if consume(&mut tokens, Punctuation::Minus) {
            writeln!(output, "  addi $a0, $a0, -{}", expect_number(&mut tokens)?)?;
            continue;
        }

        return Err(anyhow!("unexpected string"));
    }

    writeln!(output, "  ori $v0, $zero, 1")?;
    writeln!(output, "  syscall")?;
    writeln!(output, "  jr $ra")?;

    Ok(output)
}

pub fn compile_from_path<P: AsRef<Path> + std::fmt::Debug>(input: P) -> Result<String> {
    let mut source = String::new();
    let mut input = File::open(input)?;

    input.read_to_string(&mut source)?;
    source = source.replace("\n", "");

    gen_code(&source)
}
