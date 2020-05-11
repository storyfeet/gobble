use failure_derive::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Fail)]
pub enum ECode {
    #[fail(display = "BREAK -- {}", 0)]
    BREAK(Box<ECode>),
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
    #[fail(display = "Expected {}, got {:?}", 0, 1)]
    Char(char, Option<char>),
    #[fail(display = "Expected a char in {:?}, got {:?}", 0, 1)]
    CharInStr(&'static str, char),
    #[fail(display = "Expected {:?}", 0)]
    Tag(&'static str),
    #[fail(display = "Require {} repeats, got only {} -- {}", 0, 1, 2)]
    Count(usize, usize, Box<ParseError>),
    #[fail(display = "Unexpected {}", 0)]
    UnexpectedChar(char),
    #[fail(display = "Char Expected {:?} - got{:?}", 0, 1)]
    CharExpected(crate::chars::Expected, Option<char>),
}

impl ECode {
    pub fn brk(self) -> Self {
        ECode::BREAK(Box::new(self))
    }
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
