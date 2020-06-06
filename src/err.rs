//use failure_derive::*;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Debug;
use thiserror::*;

pub trait Expectable: Debug + Clone + Error + PartialEq + Default {
    fn or(self, a: Self) -> Self;
    fn except(a: Self) -> Self;
    fn is_nil(&self) -> bool;

    fn first(a: Self, b: Self) -> Self {
        if a.is_nil() {
            return b;
        }
        a
    }
}

#[derive(Debug, PartialEq, Clone, Error)]
pub enum Expected {
    #[error("Unknown")]
    Unknown,
    #[error("{}",.0)]
    Char(char),
    #[error("WS")]
    WS,
    #[error("{}",.0)]
    CharIn(&'static str),
    #[error("\"{}\"",.0)]
    Str(&'static str),
    #[error("One of {:?}",.0)]
    OneOf(Vec<Expected>),
    #[error("but not {}",.0)]
    Except(Box<Expected>),
}
impl Default for Expected {
    fn default() -> Self {
        Expected::Unknown
    }
}

impl Expectable for Expected {
    fn or(self, b: Self) -> Self {
        match self {
            Expected::OneOf(mut v) => {
                v.push(b);
                Expected::OneOf(v)
            }
            v => Expected::OneOf(vec![v, b]),
        }
    }

    fn except(a: Self) -> Self {
        Expected::Except(Box::new(a))
    }
    fn is_nil(&self) -> bool {
        match self {
            Expected::Unknown | Expected::WS => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Parse Error {} at  {},{}: Expected {}",.is_brk, .line, .col, .exp)]
pub struct ParseError<E: Expectable> {
    pub exp: E,
    pub index: Option<usize>,
    pub line: usize,
    pub col: usize,
    pub is_brk: bool,
}

impl ParseError<Expected> {
    pub fn new(s: &'static str, index: Option<usize>, line: usize, col: usize) -> Self {
        ParseError {
            exp: Expected::Str(s),
            index,
            line,
            col,
            is_brk: false,
        }
    }
}

impl<E: Expectable> ParseError<E> {
    pub fn expect(exp: E, index: Option<usize>, line: usize, col: usize) -> ParseError<E> {
        ParseError {
            exp,
            index,
            line,
            col,
            is_brk: false,
        }
    }

    pub fn new_exp<NE: Expectable>(self, nexp: NE) -> ParseError<NE> {
        ParseError {
            exp: nexp,
            index: self.index,
            line: self.line,
            col: self.col,
            is_brk: self.is_brk,
        }
    }

    pub fn is_break(&self) -> bool {
        self.is_brk
    }

    pub fn brk(mut self) -> Self {
        self.is_brk = true;
        self
    }

    pub fn de_brk(mut self) -> Self {
        self.is_brk = false;
        self
    }
}

impl<E: Expectable> PartialOrd for ParseError<E> {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        if self.line == b.line {
            return self.col.partial_cmp(&b.col);
        }
        self.line.partial_cmp(&self.line)
    }
}
