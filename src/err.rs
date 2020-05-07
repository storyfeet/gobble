use failure_derive::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Fail)]
pub enum ECode {
    #[fail(display = "End of Input")]
    EOF,
    #[fail(display = "This Error Should Never Happen: {}", 0)]
    Never(&'static str),
    #[fail(display = "{}", 0)]
    SMess(&'static str),
    #[fail(display = "{}", 0)]
    Mess(String),
    #[fail(display = "{}::{}", 0, 1)]
    Wrap(&'static str, Box<ParseError>),
    #[fail(display = "Error {} or {}", 0, 1)]
    Or(Box<ParseError>, Box<ParseError>),
    #[fail(display = "Expected {:?}", 0)]
    Tag(&'static str),
    #[fail(display = "Require {} repeats, got only {} -- {}", 0, 1, 2)]
    Count(usize, usize, Box<ParseError>),
    #[fail(display = "Unexpected {}", 0)]
    UnexpectedChar(char),
}

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Parse Error at {},{}: {}", line, col, code)]
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
}

impl PartialOrd for ParseError {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        if self.line == b.line {
            return self.col.partial_cmp(&b.col);
        }
        self.line.partial_cmp(&self.line)
    }
}
