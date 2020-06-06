//use failure_derive::*;
use std::cmp::Ordering;
//use std::error::Error;
use std::fmt::Debug;
use thiserror::*;

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
    #[error("P = {}, V {}",.0,.1)]
    ObOn(&'static str, &'static str),
    #[error("\"{}\"",.0)]
    Str(&'static str),
    #[error("One of {:?}",.0)]
    OneOf(Vec<Expected>),
    #[error("but not {}",.0)]
    Except(Box<Expected>),
}

impl Expected {
    pub fn or(self, b: Self) -> Self {
        match self {
            Expected::OneOf(mut v) => {
                v.push(b);
                Expected::OneOf(v)
            }
            v => Expected::OneOf(vec![v, b]),
        }
    }

    pub fn except(a: Self) -> Self {
        Expected::Except(Box::new(a))
    }
    pub fn is_nil(&self) -> bool {
        match self {
            Expected::Unknown | Expected::WS => true,
            _ => false,
        }
    }

    pub fn first(a: Self, b: Self) -> Self {
        match a.is_nil() {
            true => b,
            false => a,
        }
    }
}

pub fn longer(mut a: ParseError, b: ParseError) -> ParseError {
    match a.partial_cmp(&b) {
        Some(Ordering::Greater) => a,
        Some(Ordering::Less) => b,
        _ => {
            a.exp = Expected::OneOf(vec![a.exp, b.exp]);
            a
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Parse Error {} at  {},{}: Expected {}",.is_brk, .line, .col, .exp)]
pub struct ParseError {
    pub exp: Expected,
    pub index: Option<usize>,
    pub line: usize,
    pub col: usize,
    pub is_brk: bool,
    pub child: Option<Box<ParseError>>,
}

impl ParseError {
    pub fn new(s: &'static str, index: Option<usize>, line: usize, col: usize) -> Self {
        ParseError {
            exp: Expected::Str(s),
            index,
            line,
            col,
            is_brk: false,
            child: None,
        }
    }
    pub fn expect(exp: Expected, index: Option<usize>, line: usize, col: usize) -> ParseError {
        ParseError {
            exp,
            index,
            line,
            col,
            is_brk: false,
            child: None,
        }
    }

    pub fn wrap(mut self, ne: Self) -> Self {
        match self.child {
            Some(c) => self.child = Some(Box::new(c.wrap(ne))),
            None => self.child = Some(Box::new(ne)),
        }
        self
    }

    pub fn new_exp(mut self, nexp: Expected) -> ParseError {
        self.exp = nexp;
        self
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

impl PartialOrd for ParseError {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        //None means the end of the string
        match (self.index, b.index) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Greater),
            (Some(_), None) => Some(Ordering::Less),
            (Some(av), Some(bv)) => av.partial_cmp(&bv),
        }
    }
}
