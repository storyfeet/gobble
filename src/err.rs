//use failure_derive::*;
use std::cmp::Ordering;
use thiserror::*;
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ECode {
    #[error("BREAK -- {}", .0)]
    BREAK(Box<ECode>),
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

impl ECode {
    pub fn brk(self) -> Self {
        ECode::BREAK(Box::new(self))
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Parse Error at {},{}: {}", self.line, self.col, self.code)]
pub struct ParseError {
    pub code: ECode,
    pub line: usize,
    pub col: usize,
}

impl ParseError {
    pub fn new(s: &'static str, line: usize, col: usize) -> ParseError {
        ParseError {
            code: ECode::SMess(s),
            line,
            col,
        }
    }
    pub fn code(code: ECode, line: usize, col: usize) -> ParseError {
        ParseError { code, line, col }
    }

    pub fn is_break(&self) -> bool {
        match self.code {
            ECode::BREAK(_) => true,
            _ => false,
        }
    }

    pub fn brk(self) -> Self {
        ParseError {
            code: self.code.brk(),
            line: self.line,
            col: self.col,
        }
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
