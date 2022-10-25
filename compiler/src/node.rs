use crate::token::Token;
use crate::token::TokenType;
use crate::Punctuation;
use std::iter::Peekable;

#[derive(Debug)]
pub enum NError {
    ExpectedNumber,
    Eof,
}

pub enum Node {
    Add { lhs: Box<Node>, rhs: Box<Node> },
    Sub { lhs: Box<Node>, rhs: Box<Node> },
    Number(i32),
}

pub fn expect_number<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token<'a>>>,
) -> Result<i32, NError> {
    if let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Number(n) => {
                tokens.next();
                Ok(n)
            }
            _ => Err(NError::ExpectedNumber),
        }
    } else {
        Err(NError::Eof)
    }
}

pub fn consume<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token<'a>>>,
    expect: Punctuation,
) -> bool {
    if let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Punctuation(ref p) if p == &expect => {
                tokens.next();
                return true;
            }
            _ => false,
        }
    } else {
        false
    }
}

pub fn expr<'a>(
    stream: &mut Peekable<impl Iterator<Item = &'a Token<'a>>>,
) -> Result<Box<Node>, String> {
    let mut node = number(stream)?;
    loop {
        if consume(stream, Punctuation::Plus) {
            node = Box::new(Node::Add {
                lhs: node,
                rhs: number(stream)?,
            });
        } else if consume(stream, Punctuation::Minus) {
            node = Box::new(Node::Sub {
                lhs: node,
                rhs: number(stream)?,
            });
        } else {
            return Ok(node);
        }
    }
}

pub fn number<'a>(
    stream: &mut Peekable<impl Iterator<Item = &'a Token<'a>>>,
) -> Result<Box<Node>, String> {
    if let Ok(n) = expect_number(stream) {
        return Ok(Box::new(Node::Number(n)));
    }

    Err("expect number".into())
}
