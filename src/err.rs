//use failure_derive::*;
use std::cmp::Ordering;
use thiserror::*;
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ECode {
    #[error("End of Input")]
    EOF,
    #[error("This Error Should Never Happen: {}", .0)]
    Never(&'static str),
    #[error("{}", .0)]
    SMess(&'static str),
    #[error("{}", .0)]
    Mess(String),
    #[error("{}::{}", .0, .1)]
    Wrap(&'static str, Box<ParseError>),
    #[error("Error {} or {}", .0, .1)]
    Or(Box<ParseError>, Box<ParseError>),
    #[error("Expected {}, got {:?}", .0, .1)]
    Char(char, Option<char>),
    #[error("Expected a char in {:?}, got {:?}", .0, .1)]
    CharInStr(&'static str, char),
    #[error("Expected {:?}", .0)]
    Tag(&'static str),
    #[error("Require {} repeats, got only {} -- {}", .0, .1, .2)]
    Count(usize, usize, Box<ParseError>),
    #[error("Unexpected {}", .0)]
    UnexpectedChar(char),
    #[error("Char Expected {:?} - got{:?}", .0, .1)]
    CharExpected(crate::chars::Expected, Option<char>),
    #[error("Failon {:?}", .0)]
    FailOn(String),
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum BreakMode {
    #[error("")]
    None,
    #[error("Break")]
    Total,
    #[error("Break Depth {}",.0)]
    Depth(usize),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Parse Error {} at  {},{}: {}",.brk, .line, .col, .code)]
pub struct ParseError {
    pub code: ECode,
    pub line: usize,
    pub col: usize,
    pub brk: bool,
}

impl ParseError {
    pub fn new(s: &'static str, line: usize, col: usize) -> ParseError {
        ParseError {
            code: ECode::SMess(s),
            line,
            col,
            brk: false,
        }
    }
    pub fn code(code: ECode, line: usize, col: usize) -> ParseError {
        ParseError {
            code,
            line,
            col,
            brk: false,
        }
    }

    pub fn is_break(&self) -> bool {
        self.brk
    }

    pub fn brk(mut self) -> Self {
        self.brk = true;
        self
    }

    pub fn de_brk(mut self) -> Self {
        self.brk = false;
        self
    }
}

impl PartialOrd for ParseError {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        if self.line == b.line {
            return self.col.partial_cmp(&b.col);
        }
        self.line.partial_cmp(&self.line)
    }
}
